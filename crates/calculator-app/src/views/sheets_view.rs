use calculator_core::natural::{NaturalSession, SheetLineKind};
use calculator_storage::{CalculatorDb, SheetRecord};
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SheetsView {
    pub root: gtk::Box,
    #[allow(dead_code)]
    state: Rc<RefCell<SheetsState>>,
}

struct SheetsState {
    db: CalculatorDb,
    current_sheet_id: Option<i64>,
    sheets: Vec<SheetRecord>,
    sheet_dropdown: gtk::DropDown,
    title_entry: gtk::Entry,
    text_view: gtk::TextView,
    results_box: gtk::Box,
    session: NaturalSession,
    save_timer: Option<glib::SourceId>,
    on_toast: Rc<dyn Fn(&str)>,
    is_updating: bool,
}

impl SheetsView {
    pub fn new(db: CalculatorDb, on_toast: impl Fn(&str) + 'static) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
        root.add_css_class("sheets-view");
        root.set_margin_start(12);
        root.set_margin_end(12);
        root.set_margin_top(8);
        root.set_margin_bottom(12);

        let toast_rc: Rc<dyn Fn(&str)> = Rc::new(on_toast);

        // 1. Top toolbar
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        toolbar.add_css_class("sheets-toolbar");

        let sheet_strings = gtk::StringList::new(&[]);
        let sheet_dropdown = gtk::DropDown::new(Some(sheet_strings), None::<gtk::Expression>);
        sheet_dropdown.add_css_class("converter-dropdown");
        sheet_dropdown.set_hexpand(false);
        sheet_dropdown.set_size_request(160, -1);
        toolbar.append(&sheet_dropdown);

        let title_entry = gtk::Entry::new();
        title_entry.set_placeholder_text(Some("Sheet Title"));
        title_entry.add_css_class("sheet-title-input");
        title_entry.set_hexpand(true);
        toolbar.append(&title_entry);

        let new_btn = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Create new calculation sheet")
            .has_frame(false)
            .build();
        new_btn.add_css_class("bar-button");
        toolbar.append(&new_btn);

        let copy_all_btn = gtk::Button::builder()
            .icon_name("edit-copy-symbolic")
            .tooltip_text("Copy all expressions and results to clipboard")
            .has_frame(false)
            .build();
        copy_all_btn.add_css_class("bar-button");
        toolbar.append(&copy_all_btn);

        let delete_btn =
            gtk::Button::builder().icon_name("user-trash-symbolic").tooltip_text("Delete current sheet").has_frame(false).build();
        delete_btn.add_css_class("bar-button");
        toolbar.append(&delete_btn);

        root.append(&toolbar);

        // 2. Editor & Gutter Pane
        let editor_pane = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        editor_pane.add_css_class("sheets-editor-pane");
        editor_pane.set_vexpand(true);

        // Left: Multi-line text view in scrolled window
        let scrolled_editor = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::External)
            .vscrollbar_policy(gtk::PolicyType::External)
            .hexpand(true)
            .vexpand(true)
            .build();
        scrolled_editor.add_css_class("sheets-scrolled-editor");

        let text_view = gtk::TextView::new();
        text_view.add_css_class("sheets-text-view");
        text_view.set_monospace(true);
        text_view.set_wrap_mode(gtk::WrapMode::WordChar);
        text_view.set_left_margin(8);
        text_view.set_right_margin(8);
        text_view.set_top_margin(8);
        text_view.set_bottom_margin(8);
        scrolled_editor.set_child(Some(&text_view));
        editor_pane.append(&scrolled_editor);

        // Right: Results gutter in scrolled window
        let scrolled_results = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::External)
            .vscrollbar_policy(gtk::PolicyType::External)
            .hexpand(false)
            .vexpand(true)
            .build();
        scrolled_results.set_size_request(180, -1);
        scrolled_results.add_css_class("sheets-scrolled-results");

        let results_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        results_box.add_css_class("sheets-results-box");
        results_box.set_margin_top(8);
        results_box.set_margin_bottom(8);
        results_box.set_margin_start(4);
        results_box.set_margin_end(8);
        scrolled_results.set_child(Some(&results_box));
        editor_pane.append(&scrolled_results);

        // Synchronize scrolling between editor and gutter
        let v_adj_editor = scrolled_editor.vadjustment();
        let v_adj_results = scrolled_results.vadjustment();
        v_adj_editor.bind_property("value", &v_adj_results, "value").bidirectional().build();

        root.append(&editor_pane);

        let state = Rc::new(RefCell::new(SheetsState {
            db,
            current_sheet_id: None,
            sheets: Vec::new(),
            sheet_dropdown: sheet_dropdown.clone(),
            title_entry: title_entry.clone(),
            text_view: text_view.clone(),
            results_box: results_box.clone(),
            session: NaturalSession::new(),
            save_timer: None,
            on_toast: toast_rc.clone(),
            is_updating: false,
        }));

        // Load existing sheets from DB or initialize default
        reload_sheets_from_db(&state);

        // Text change listener
        let buffer = text_view.buffer();
        let st_buf = state.clone();
        buffer.connect_changed(move |_| {
            if st_buf.borrow().is_updating {
                return;
            }
            re_evaluate_and_schedule_save(&st_buf);
        });

        // Title change listener
        let st_title = state.clone();
        title_entry.connect_changed(move |_| {
            if st_title.borrow().is_updating {
                return;
            }
            schedule_save(&st_title);
        });

        // Sheet dropdown switch listener
        let st_drop = state.clone();
        sheet_dropdown.connect_selected_notify(move |dd| {
            if st_drop.borrow().is_updating {
                return;
            }
            let idx = dd.selected() as usize;
            let (target_id, cur_id) = {
                let st = st_drop.borrow();
                let tid = st.sheets.get(idx).map(|s| s.id);
                (tid, st.current_sheet_id)
            };
            if let Some(tid) = target_id
                && Some(tid) != cur_id
            {
                load_sheet_by_id(&st_drop, tid);
            }
        });

        // "+ New" sheet button
        let st_new = state.clone();
        new_btn.connect_clicked(move |_| {
            create_new_sheet(&st_new, "Untitled Sheet", "");
        });

        // Copy All button
        let st_copy = state.clone();
        copy_all_btn.connect_clicked(move |_| {
            copy_sheet_summary(&st_copy);
        });

        // Delete button
        let st_del = state.clone();
        delete_btn.connect_clicked(move |_| {
            delete_current_sheet(&st_del);
        });

        Self { root, state }
    }
}

fn reload_sheets_from_db(state: &Rc<RefCell<SheetsState>>) {
    let list = state.borrow().db.list_sheets().unwrap_or_default();
    if list.is_empty() {
        // Create initial default welcome sheet
        let default_content = r#"# Nyx Calculation Sheet
// Natural language math, units & variables

price = 100
tax = 8%
total = price + tax

$250 split between 4 people
5 miles in kilometers
32 fahrenheit to celsius
2 hours 30 minutes in seconds"#;

        if let Ok(id) = state.borrow().db.create_sheet("Default Sheet", default_content) {
            let list = state.borrow().db.list_sheets().unwrap_or_default();
            populate_sheet_dropdown(state, list, Some(id));
            load_sheet_by_id(state, id);
            return;
        }
    }

    let first_id = list.first().map(|s| s.id);
    populate_sheet_dropdown(state, list, first_id);
    if let Some(id) = first_id {
        load_sheet_by_id(state, id);
    }
}

fn populate_sheet_dropdown(state: &Rc<RefCell<SheetsState>>, sheets: Vec<SheetRecord>, sel_id: Option<i64>) {
    let titles: Vec<String> = sheets.iter().map(|s| s.title.clone()).collect();
    let title_refs: Vec<&str> = titles.iter().map(|s| s.as_str()).collect();
    let model = gtk::StringList::new(&title_refs);

    let sel_idx = if let Some(id) = sel_id { sheets.iter().position(|s| s.id == id).unwrap_or(0) } else { 0 };

    let dd = {
        let mut st = state.borrow_mut();
        st.is_updating = true;
        st.sheets = sheets;
        st.sheet_dropdown.clone()
    };

    dd.set_model(Some(&model));
    dd.set_selected(sel_idx as u32);

    state.borrow_mut().is_updating = false;
}

fn load_sheet_by_id(state: &Rc<RefCell<SheetsState>>, id: i64) {
    let maybe_record = state.borrow().db.get_sheet(id).ok().flatten();
    if let Some(record) = maybe_record {
        let (title_entry, text_view) = {
            let mut st = state.borrow_mut();
            st.current_sheet_id = Some(id);
            st.is_updating = true;
            (st.title_entry.clone(), st.text_view.clone())
        };

        title_entry.set_text(&record.title);
        text_view.buffer().set_text(&record.content);

        state.borrow_mut().is_updating = false;

        re_evaluate(state);
    }
}

fn create_new_sheet(state: &Rc<RefCell<SheetsState>>, title: &str, content: &str) {
    if let Ok(id) = state.borrow().db.create_sheet(title, content) {
        let list = state.borrow().db.list_sheets().unwrap_or_default();
        populate_sheet_dropdown(state, list, Some(id));
        load_sheet_by_id(state, id);
        (state.borrow().on_toast)(&format!("Created new sheet: {title}"));
    }
}

fn delete_current_sheet(state: &Rc<RefCell<SheetsState>>) {
    let cur_id = state.borrow().current_sheet_id;
    if let Some(id) = cur_id {
        let _ = state.borrow().db.delete_sheet(id);
        reload_sheets_from_db(state);
        (state.borrow().on_toast)("Deleted sheet");
    }
}

fn re_evaluate_and_schedule_save(state: &Rc<RefCell<SheetsState>>) {
    re_evaluate(state);
    schedule_save(state);
}

fn re_evaluate(state: &Rc<RefCell<SheetsState>>) {
    let text = {
        let buf = state.borrow().text_view.buffer();
        let (start, end) = buf.bounds();
        buf.text(&start, &end, false).to_string()
    };

    let outputs = state.borrow_mut().session.evaluate_sheet(&text);

    let results_box = state.borrow().results_box.clone();
    while let Some(child) = results_box.first_child() {
        results_box.remove(&child);
    }

    let on_toast = state.borrow().on_toast.clone();

    for out in outputs {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        row.set_halign(gtk::Align::End);
        row.add_css_class("sheet-result-row");

        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::End);
        label.add_css_class("sheet-result-label");

        match out.kind {
            SheetLineKind::Empty | SheetLineKind::Comment(_) => {
                label.set_text("");
            }
            SheetLineKind::Heading(_) => {
                label.set_text("");
            }
            SheetLineKind::Result(res) => {
                label.set_text(&res.formatted);
                label.add_css_class("sheet-result-val");

                // Clickable button to copy result
                let click = gtk::GestureClick::new();
                let text_to_copy = res.formatted.clone();
                let toast_cb = on_toast.clone();
                click.connect_released(move |_, _, _, _| {
                    if let Some(display) = gdk::Display::default() {
                        display.clipboard().set_text(&text_to_copy);
                        toast_cb(&format!("Copied: {text_to_copy}"));
                    }
                });
                row.add_controller(click);
            }
            SheetLineKind::Error(err) => {
                label.set_text(&err);
                label.add_css_class("sheet-result-error");
            }
        }

        row.append(&label);
        results_box.append(&row);
    }
}

fn schedule_save(state: &Rc<RefCell<SheetsState>>) {
    if let Some(id) = state.borrow_mut().save_timer.take() {
        id.remove();
    }

    let st = state.clone();
    let source_id = glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
        let (cur_id, title, content) = {
            let s = st.borrow();
            let buf = s.text_view.buffer();
            let (start, end) = buf.bounds();
            (s.current_sheet_id, s.title_entry.text().to_string(), buf.text(&start, &end, false).to_string())
        };

        if let Some(id) = cur_id {
            let final_title = if title.trim().is_empty() { "Untitled Sheet" } else { title.trim() };
            let _ = st.borrow().db.update_sheet(id, final_title, &content);
        }
    });

    state.borrow_mut().save_timer = Some(source_id);
}

fn copy_sheet_summary(state: &Rc<RefCell<SheetsState>>) {
    let (title, content) = {
        let s = state.borrow();
        let buf = s.text_view.buffer();
        let (start, end) = buf.bounds();
        (s.title_entry.text().to_string(), buf.text(&start, &end, false).to_string())
    };

    let mut session = NaturalSession::new();
    let outputs = session.evaluate_sheet(&content);

    let mut full_text = format!("# {title}\n\n");
    for out in outputs {
        match out.kind {
            SheetLineKind::Empty => full_text.push('\n'),
            SheetLineKind::Heading(h) => full_text.push_str(&format!("# {h}\n")),
            SheetLineKind::Comment(c) => full_text.push_str(&format!("{c}\n")),
            SheetLineKind::Result(res) => {
                full_text.push_str(&format!("{:<30} => {}\n", out.original_text, res.formatted));
            }
            SheetLineKind::Error(err) => {
                full_text.push_str(&format!("{:<30} => [Error: {}]\n", out.original_text, err));
            }
        }
    }

    if let Some(display) = gdk::Display::default() {
        display.clipboard().set_text(&full_text);
        (state.borrow().on_toast)("Copied full sheet to clipboard");
    }
}
