use crate::display::DisplayView;
use crate::history_sidebar::HistorySidebar;
use crate::keypad::{KeyAction, KeypadView};
use crate::keys::install_calculator_keys;
use crate::{prefs, settings_dialog, theme};
use calculator_core::{CalcError, FormatOptions, MemoryRegister, evaluate_str, format_decimal, preview_str};
use calculator_storage::CalculatorDb;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use nixe_ui::chrome;
use rust_decimal::Decimal;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

pub(crate) struct Ui {
    window: gtk::ApplicationWindow,
    overlay: gtk::Overlay,
    display: DisplayView,
    keypad: KeypadView,
    sidebar: Rc<RefCell<HistorySidebar>>,
    sidebar_container: gtk::Box,
    db: CalculatorDb,
    memory: MemoryRegister,
    toast: gtk::Label,
    toast_timer: Option<glib::SourceId>,

    // Calculation state
    input_buffer: String,
    tape_text: String,
    is_result_shown: bool,
    error_state: Option<String>,
}

thread_local! {
    static UI: RefCell<Option<Rc<RefCell<Ui>>>> = const { RefCell::new(None) };
}

pub fn ui() -> Option<Rc<RefCell<Ui>>> {
    UI.with(|u| u.borrow().clone())
}

pub fn window() -> Option<gtk::ApplicationWindow> {
    ui().map(|u| u.borrow().window.clone())
}

pub fn overlay() -> Option<gtk::Overlay> {
    ui().map(|u| u.borrow().overlay.clone())
}

pub fn present(app: &gtk::Application, _section: Option<&str>) {
    if let Some(ui) = ui() {
        ui.borrow().window.present();
        return;
    }
    theme::install();
    build(app);

    // Headless snapshot test check
    if let Some(out) = std::env::var_os("CALCULATOR_SNAPSHOT") {
        snapshot_and_quit(app, std::path::PathBuf::from(out));
        return;
    }

    if let Some(ui) = ui() {
        ui.borrow().window.present();
    }
}

fn build(app: &gtk::Application) {
    let window =
        gtk::ApplicationWindow::builder().application(app).title("Calculator").default_width(360).default_height(540).build();
    window.add_css_class("calculator-window");

    // No client-side titlebar: Hyprland manages the window frame
    window.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    chrome::set_title(&window, "Calculator", "Standard");

    // Overlay for toast and settings card
    let overlay = gtk::Overlay::new();

    // Main horizontal container (left: calc pane, right: history sidebar)
    let main_h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    // Left pane: Calculator view (Top bar, Display, Keypad)
    let calc_v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    calc_v_box.set_hexpand(true);

    // Top bar
    let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    topbar.add_css_class("calc-topbar");

    // Mode dropdown button
    let mode_btn = gtk::Button::with_label("Standard ▾");
    mode_btn.add_css_class("mode-selector");
    mode_btn.set_tooltip_text(Some("Calculator mode (Standard)"));
    topbar.append(&mode_btn);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    topbar.append(&spacer);

    // History toggle button
    let hist_btn = gtk::Button::builder()
        .icon_name("document-open-recent-symbolic")
        .tooltip_text("Calculation history (Ctrl+H)")
        .has_frame(false)
        .build();
    hist_btn.add_css_class("bar-button");
    topbar.append(&hist_btn);

    // Settings button
    let settings_btn =
        gtk::Button::builder().icon_name("emblem-system-symbolic").tooltip_text("Settings (Ctrl+,)").has_frame(false).build();
    settings_btn.add_css_class("bar-button");
    settings_btn.connect_clicked(|_| settings_dialog::open());
    topbar.append(&settings_btn);

    calc_v_box.append(&topbar);

    // Display
    let display = DisplayView::new();
    calc_v_box.append(&display.root);

    // Keypad
    let keypad = KeypadView::new(|action| {
        handle_action(action);
    });
    calc_v_box.append(&keypad.root);

    main_h_box.append(&calc_v_box);

    // Database & History sidebar
    let db = CalculatorDb::open_default().unwrap_or_else(|_| CalculatorDb::open_in_memory().unwrap());
    let sidebar = HistorySidebar::new(
        db.clone(),
        |formula| {
            restore_calculation(&formula);
        },
        |msg| {
            toast(msg);
        },
    );

    let sidebar_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    sidebar_container.append(&sidebar.borrow().root);
    let initial_open = prefs::get().history_open;
    sidebar_container.set_visible(initial_open);
    main_h_box.append(&sidebar_container);

    overlay.set_child(Some(&main_h_box));

    // Toast overlay label
    let toast_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    toast_box.add_css_class("calc-toast-box");
    toast_box.set_halign(gtk::Align::Center);
    toast_box.set_valign(gtk::Align::End);
    toast_box.set_visible(false);

    let toast_lbl = gtk::Label::new(None);
    toast_lbl.add_css_class("calc-toast");
    toast_box.append(&toast_lbl);
    overlay.add_overlay(&toast_box);

    window.set_child(Some(&overlay));

    // Common keys (settings, shortcuts, quit)
    nixe_ui::keys::install(
        &window,
        nixe_ui::keys::Common {
            settings: Some(settings_dialog::open),
            shortcuts: Some(show_shortcuts),
            close: true,
            palette: None,
        },
    );

    // Calculator typing keys
    install_calculator_keys(&window, handle_action, copy_result, paste_to_expression, toggle_history);

    // History button click
    hist_btn.connect_clicked(|_| toggle_history());

    let state = Rc::new(RefCell::new(Ui {
        window,
        overlay,
        display,
        keypad,
        sidebar,
        sidebar_container,
        db,
        memory: MemoryRegister::new(),
        toast: toast_lbl,
        toast_timer: None,
        input_buffer: String::new(),
        tape_text: String::new(),
        is_result_shown: false,
        error_state: None,
    }));

    UI.with(|u| *u.borrow_mut() = Some(state));
}

pub fn handle_action(action: KeyAction) {
    let Some(ui_rc) = ui() else { return };
    let mut ui = ui_rc.borrow_mut();

    // Clear previous error on any new action
    if ui.error_state.is_some() && action != KeyAction::Clear {
        ui.error_state = None;
        ui.display.set_error(None);
    }

    match action {
        KeyAction::Digit(c) => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            }
            ui.input_buffer.push(c);
            update_readout(&mut ui);
        }
        KeyAction::Dot => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            }
            if ui.input_buffer.is_empty() {
                ui.input_buffer.push_str("0.");
            } else {
                // Check if current number segment already has a dot
                let last_segment = ui
                    .input_buffer
                    .rsplit(|ch| ['+', '-', '*', '/', '^', '%', '(', ' ', '×', '÷', '−'].contains(&ch))
                    .next()
                    .unwrap_or("");
                if !last_segment.contains('.') {
                    ui.input_buffer.push('.');
                }
            }
            update_readout(&mut ui);
        }
        KeyAction::OpAdd => append_binary_op(&mut ui, " + "),
        KeyAction::OpSub => append_binary_op(&mut ui, " − "),
        KeyAction::OpMul => append_binary_op(&mut ui, " × "),
        KeyAction::OpDiv => append_binary_op(&mut ui, " ÷ "),
        KeyAction::OpPow => append_binary_op(&mut ui, " ^ "),
        KeyAction::Percent => {
            if !ui.input_buffer.is_empty() {
                ui.input_buffer.push('%');
                ui.is_result_shown = false;
                update_readout(&mut ui);
            }
        }
        KeyAction::LParen => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            }
            ui.input_buffer.push('(');
            update_readout(&mut ui);
        }
        KeyAction::RParen => {
            ui.input_buffer.push(')');
            ui.is_result_shown = false;
            update_readout(&mut ui);
        }
        KeyAction::Negate => {
            if let Ok(val) = evaluate_str(&ui.input_buffer) {
                let neg = -val;
                ui.input_buffer = neg.to_string();
                update_readout(&mut ui);
            }
        }
        KeyAction::Square => {
            wrap_current_or_evaluate(&mut ui, "sqr");
        }
        KeyAction::Sqrt => {
            wrap_current_or_evaluate(&mut ui, "sqrt");
        }
        KeyAction::Reciprocal => {
            wrap_current_or_evaluate(&mut ui, "recip");
        }
        KeyAction::Backspace => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            } else {
                // Pop character or trimmed whitespace
                let trimmed = ui.input_buffer.trim_end();
                if trimmed.len() < ui.input_buffer.len() {
                    ui.input_buffer = trimmed.to_string();
                } else {
                    ui.input_buffer.pop();
                }
            }
            update_readout(&mut ui);
        }
        KeyAction::Clear => {
            ui.input_buffer.clear();
            ui.tape_text.clear();
            ui.is_result_shown = false;
            ui.error_state = None;
            ui.display.set_tape("");
            ui.display.set_preview(None);
            ui.display.set_readout("0");
            ui.display.set_error(None);
        }
        KeyAction::ClearEntry => {
            // Clears last typed operand
            if let Some(pos) = ui.input_buffer.rfind(|ch| ['+', '-', '*', '/', '^', '×', '÷', '−'].contains(&ch)) {
                ui.input_buffer.truncate(pos + 1);
            } else {
                ui.input_buffer.clear();
            }
            ui.is_result_shown = false;
            update_readout(&mut ui);
        }
        KeyAction::Equals => {
            execute_calculation(&mut ui);
        }
        KeyAction::MemClear => {
            ui.memory.clear();
            ui.display.set_memory_active(false);
            ui.keypad.set_memory_enabled(false);
            toast("Memory cleared");
        }
        KeyAction::MemRecall => {
            if ui.memory.is_active() {
                let val = ui.memory.recall();
                if ui.is_result_shown {
                    ui.input_buffer.clear();
                    ui.is_result_shown = false;
                }
                ui.input_buffer.push_str(&val.to_string());
                update_readout(&mut ui);
            }
        }
        KeyAction::MemStore => {
            if let Ok(val) = get_current_number(&ui) {
                ui.memory.store(val);
                ui.display.set_memory_active(true);
                ui.keypad.set_memory_enabled(true);
                toast(&format!("Memory set: {val}"));
            }
        }
        KeyAction::MemAdd => {
            if let Ok(val) = get_current_number(&ui)
                && ui.memory.add(val).is_ok()
            {
                ui.display.set_memory_active(true);
                ui.keypad.set_memory_enabled(true);
                toast(&format!("Memory: {}", ui.memory.recall()));
            }
        }
        KeyAction::MemSub => {
            if let Ok(val) = get_current_number(&ui)
                && ui.memory.sub(val).is_ok()
            {
                ui.display.set_memory_active(true);
                ui.keypad.set_memory_enabled(true);
                toast(&format!("Memory: {}", ui.memory.recall()));
            }
        }
    }
}

fn append_binary_op(ui: &mut Ui, op: &str) {
    if ui.input_buffer.is_empty() {
        ui.input_buffer.push('0');
    }
    ui.is_result_shown = false;

    // Replace operator if buffer already ends with one
    let trimmed = ui.input_buffer.trim_end();
    if trimmed.ends_with(['+', '-', '*', '/', '^', '×', '÷', '−']) {
        let mut s = trimmed.to_string();
        s.pop();
        ui.input_buffer = s;
    }
    ui.input_buffer.push_str(op);
    update_readout(ui);
}

fn wrap_current_or_evaluate(ui: &mut Ui, func_name: &str) {
    if ui.input_buffer.is_empty() {
        ui.input_buffer.push('0');
    }
    ui.input_buffer = format!("{func_name}({})", ui.input_buffer);
    execute_calculation(ui);
}

fn update_readout(ui: &mut Ui) {
    let opts = get_format_options();
    if ui.input_buffer.is_empty() {
        ui.display.set_readout("0");
        ui.display.set_preview(None);
        return;
    }

    // Try evaluating live preview
    if let Some(preview_val) = preview_str(&ui.input_buffer) {
        let formatted_preview = format_decimal(preview_val, &opts);
        ui.display.set_preview(Some(&formatted_preview));
    } else {
        ui.display.set_preview(None);
    }

    // Display formatted input buffer
    ui.display.set_readout(&ui.input_buffer);
}

fn execute_calculation(ui: &mut Ui) {
    if ui.input_buffer.trim().is_empty() {
        return;
    }
    let opts = get_format_options();
    match evaluate_str(&ui.input_buffer) {
        Ok(res) => {
            let formatted = format_decimal(res, &opts);
            let raw_res = res.to_string();
            let expr = ui.input_buffer.clone();

            ui.tape_text = format!("{expr} =");
            ui.display.set_tape(&ui.tape_text);
            ui.display.set_readout(&formatted);
            ui.display.set_preview(None);
            ui.display.set_error(None);

            if prefs::get().history_enabled {
                let _ = ui.db.save_calculation(&expr, &formatted, &raw_res, "standard");
                ui.sidebar.borrow().refresh();
            }

            ui.input_buffer = raw_res;
            ui.is_result_shown = true;
        }
        Err(err) => {
            let msg = match err {
                CalcError::DivisionByZero => "Cannot divide by zero".to_string(),
                CalcError::InvalidDomain(m) => m,
                CalcError::Overflow => "Numerical overflow".to_string(),
                CalcError::Underflow => "Numerical underflow".to_string(),
                CalcError::UnbalancedParentheses => "Unbalanced parentheses".to_string(),
                CalcError::SyntaxError(m) => m,
                CalcError::EmptyExpression => "Empty expression".to_string(),
            };
            ui.error_state = Some(msg.clone());
            ui.display.set_error(Some(&msg));
        }
    }
}

fn get_current_number(ui: &Ui) -> Result<Decimal, CalcError> {
    if let Ok(res) = evaluate_str(&ui.input_buffer) {
        Ok(res)
    } else {
        Decimal::from_str(&ui.input_buffer).map_err(|_| CalcError::SyntaxError("Invalid number".into()))
    }
}

pub fn restore_calculation(val: &str) {
    let Some(ui_rc) = ui() else { return };
    let mut ui = ui_rc.borrow_mut();
    ui.input_buffer = val.to_string();
    ui.is_result_shown = false;
    update_readout(&mut ui);
}

pub fn toggle_history() {
    let Some(ui_rc) = ui() else { return };
    let ui = ui_rc.borrow();
    let current_vis = ui.sidebar_container.is_visible();
    let new_vis = !current_vis;
    ui.sidebar_container.set_visible(new_vis);

    let (w, h) = ui.window.default_size();
    if new_vis {
        ui.window.set_default_size(w + 280, h);
    } else {
        ui.window.set_default_size((w - 280).max(360), h);
    }

    prefs::update(|p| p.history_open = new_vis);
}

pub fn copy_result() {
    let Some(ui_rc) = ui() else { return };
    let ui = ui_rc.borrow();
    let text = ui.display.readout_label.text().to_string();
    if let Some(display) = gdk::Display::default() {
        display.clipboard().set_text(&text);
        toast("Copied to clipboard");
    }
}

pub fn paste_to_expression() {
    let Some(display) = gdk::Display::default() else { return };
    display.clipboard().read_text_async(None::<&gio::Cancellable>, move |res| {
        if let Ok(Some(text)) = res {
            let clean = text.trim();
            if !clean.is_empty()
                && let Some(ui_rc) = ui()
            {
                let mut ui = ui_rc.borrow_mut();
                if ui.is_result_shown {
                    ui.input_buffer.clear();
                    ui.is_result_shown = false;
                }
                ui.input_buffer.push_str(clean);
                update_readout(&mut ui);
            }
        }
    });
}

pub fn refresh_display() {
    let Some(ui_rc) = ui() else { return };
    let mut ui = ui_rc.borrow_mut();
    update_readout(&mut ui);
}

pub fn toast(msg: &str) {
    let Some(ui_rc) = ui() else { return };
    let mut ui = ui_rc.borrow_mut();
    if let Some(t) = ui.toast_timer.take() {
        t.remove();
    }
    ui.toast.set_text(msg);
    if let Some(parent) = ui.toast.parent() {
        parent.set_visible(true);
    }
    let weak = Rc::downgrade(&ui_rc);
    ui.toast_timer = Some(glib::timeout_add_local_once(std::time::Duration::from_millis(2500), move || {
        if let Some(u) = weak.upgrade() {
            let mut u = u.borrow_mut();
            u.toast_timer = None;
            if let Some(parent) = u.toast.parent() {
                parent.set_visible(false);
            }
        }
    }));
}

fn get_format_options() -> FormatOptions {
    let p = prefs::get();
    FormatOptions {
        thousands_separator: p.thousands_separator,
        precision: if p.precision == 0 { None } else { Some(p.precision) },
    }
}

pub fn show_shortcuts() {
    let (dialog, card) = nixe_ui::widgets::dialog("Keyboard Shortcuts", 460);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    list.add_css_class("group-list");

    let shortcuts: &[(&[&str], &str)] = &[
        (&["0–9", "."], "Enter numbers"),
        (&["+", "−", "×", "÷"], "Basic operators"),
        (&["Enter", "="], "Calculate result"),
        (&["Backspace"], "Delete last character"),
        (&["Esc", "C"], "Clear all"),
        (&["Ctrl", "H"], "Toggle calculation history"),
        (&["Ctrl", "C"], "Copy displayed result"),
        (&["Ctrl", "V"], "Paste formula"),
        (&["Ctrl", "M"], "Memory store (MS)"),
        (&["Ctrl", "R"], "Memory recall (MR)"),
        (&["Ctrl", "P"], "Memory add (M+)"),
        (&["Ctrl", "Q"], "Memory subtract (M−)"),
        (&["Ctrl", "L"], "Memory clear (MC)"),
        (&["Ctrl", ","], "Open settings"),
        (&["F1"], "Show shortcuts"),
    ];

    for (keys, what) in shortcuts {
        list.append(&nixe_ui::widgets::row(what, "", Some(nixe_ui::widgets::key_caps(keys).upcast_ref())));
    }

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::External)
        .child(&list)
        .max_content_height(400)
        .propagate_natural_height(true)
        .build();
    card.append(&scroll);

    let close = gtk::Button::with_label("Close");
    close.set_halign(gtk::Align::End);
    let d = dialog.clone();
    close.connect_clicked(move |_| d.close());
    card.append(&close);
    dialog.present();
}

fn snapshot_and_quit(app: &gtk::Application, out: std::path::PathBuf) {
    let Some(ui) = ui() else { return };
    let window = ui.borrow().window.clone();
    window.set_opacity(0.01);
    window.present();

    let app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        if let Some(child) = window.child() {
            let paintable = gtk::WidgetPaintable::new(Some(&child));
            let (w, h) = (child.width(), child.height());
            let snapshot = gtk::Snapshot::new();
            snapshot.append_color(&gdk::RGBA::BLACK, &gtk::graphene::Rect::new(0.0, 0.0, w as f32, h as f32));
            paintable.snapshot(&snapshot, w as f64, h as f64);
            if let (Some(node), Some(renderer)) = (snapshot.to_node(), window.renderer()) {
                let texture = renderer.render_texture(node, None);
                if texture.save_to_png(&out).is_ok() {
                    println!("Rendered headless snapshot: {}", out.display());
                }
            }
        }
        app.quit();
    });
}
