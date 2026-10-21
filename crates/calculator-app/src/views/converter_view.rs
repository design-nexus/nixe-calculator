use calculator_core::units::{UnitCategory, UnitDefinition, convert, units_for_category};
use gtk::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

pub struct ConverterView {
    pub root: gtk::Box,
    #[allow(dead_code)]
    state: Rc<RefCell<ConverterState>>,
}

struct ConverterState {
    category: UnitCategory,
    from_units: Vec<&'static UnitDefinition>,
    from_idx: usize,
    to_idx: usize,
    #[allow(dead_code)]
    category_dropdown: gtk::DropDown,
    from_dropdown: gtk::DropDown,
    to_dropdown: gtk::DropDown,
    from_entry: gtk::Entry,
    to_entry: gtk::Entry,
    formula_label: gtk::Label,
    chips_box: gtk::Box,
}

impl ConverterView {
    pub fn new(on_toast: impl Fn(&str) + 'static) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.add_css_class("converter-view");
        root.set_margin_start(16);
        root.set_margin_end(16);
        root.set_margin_top(12);
        root.set_margin_bottom(16);

        // 1. Category selector
        let cat_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let cat_label = gtk::Label::new(Some("Category:"));
        cat_label.add_css_class("converter-cat-label");
        cat_box.append(&cat_label);

        let cat_names: Vec<&str> = UnitCategory::ALL.iter().map(|c| c.display_name()).collect();
        let cat_strings = gtk::StringList::new(&cat_names);
        let cat_dropdown = gtk::DropDown::new(Some(cat_strings), None::<gtk::Expression>);
        cat_dropdown.add_css_class("converter-dropdown");
        cat_dropdown.set_hexpand(true);
        cat_box.append(&cat_dropdown);
        root.append(&cat_box);

        // 2. Conversion Cards container
        let cards_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        cards_box.add_css_class("converter-cards");

        // From Card
        let from_card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        from_card.add_css_class("converter-card");

        let from_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let from_title = gtk::Label::new(Some("From"));
        from_title.add_css_class("converter-card-title");
        from_header.append(&from_title);

        let from_strings = gtk::StringList::new(&[]);
        let from_dropdown = gtk::DropDown::new(Some(from_strings), None::<gtk::Expression>);
        from_dropdown.add_css_class("converter-dropdown");
        from_dropdown.set_hexpand(true);
        from_header.append(&from_dropdown);
        from_card.append(&from_header);

        let from_entry = gtk::Entry::new();
        from_entry.set_text("1");
        from_entry.add_css_class("converter-input");
        from_card.append(&from_entry);
        cards_box.append(&from_card);

        // Swap button row
        let swap_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        swap_row.set_halign(gtk::Align::Center);
        let swap_btn = gtk::Button::builder()
            .icon_name("object-flip-horizontal-symbolic")
            .tooltip_text("Swap source and destination units (⇄)")
            .has_frame(false)
            .build();
        swap_btn.add_css_class("converter-swap-btn");
        swap_row.append(&swap_btn);
        cards_box.append(&swap_row);

        // To Card
        let to_card = gtk::Box::new(gtk::Orientation::Vertical, 6);
        to_card.add_css_class("converter-card");

        let to_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let to_title = gtk::Label::new(Some("To"));
        to_title.add_css_class("converter-card-title");
        to_header.append(&to_title);

        let to_strings = gtk::StringList::new(&[]);
        let to_dropdown = gtk::DropDown::new(Some(to_strings), None::<gtk::Expression>);
        to_dropdown.add_css_class("converter-dropdown");
        to_dropdown.set_hexpand(true);
        to_header.append(&to_dropdown);
        to_card.append(&to_header);

        let to_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let to_entry = gtk::Entry::new();
        to_entry.set_editable(false);
        to_entry.set_can_focus(true);
        to_entry.add_css_class("converter-output");
        to_entry.set_hexpand(true);
        to_row.append(&to_entry);

        let copy_btn =
            gtk::Button::builder().icon_name("edit-copy-symbolic").tooltip_text("Copy converted result").has_frame(false).build();
        copy_btn.add_css_class("bar-button");
        to_row.append(&copy_btn);
        to_card.append(&to_row);

        cards_box.append(&to_card);
        root.append(&cards_box);

        // Formula / explanation label
        let formula_label = gtk::Label::new(None);
        formula_label.add_css_class("converter-formula");
        formula_label.set_halign(gtk::Align::Center);
        root.append(&formula_label);

        // Quick Favorites / Common chips
        let chips_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        chips_box.add_css_class("converter-chips");
        chips_box.set_halign(gtk::Align::Center);
        root.append(&chips_box);

        let state = Rc::new(RefCell::new(ConverterState {
            category: UnitCategory::Length,
            from_units: Vec::new(),
            from_idx: 0,
            to_idx: 1,
            category_dropdown: cat_dropdown.clone(),
            from_dropdown: from_dropdown.clone(),
            to_dropdown: to_dropdown.clone(),
            from_entry: from_entry.clone(),
            to_entry: to_entry.clone(),
            formula_label: formula_label.clone(),
            chips_box: chips_box.clone(),
        }));

        // Populate initial category units
        update_category_units(&state);
        recalculate(&state);

        // Connect category dropdown
        let st_cat = state.clone();
        cat_dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if idx < UnitCategory::ALL.len() {
                st_cat.borrow_mut().category = UnitCategory::ALL[idx];
                st_cat.borrow_mut().from_idx = 0;
                st_cat.borrow_mut().to_idx = 1;
                update_category_units(&st_cat);
                recalculate(&st_cat);
            }
        });

        // Connect From unit dropdown
        let st_from = state.clone();
        from_dropdown.connect_selected_notify(move |dd| {
            st_from.borrow_mut().from_idx = dd.selected() as usize;
            recalculate(&st_from);
        });

        // Connect To unit dropdown
        let st_to = state.clone();
        to_dropdown.connect_selected_notify(move |dd| {
            st_to.borrow_mut().to_idx = dd.selected() as usize;
            recalculate(&st_to);
        });

        // Connect Input entry text change
        let st_entry = state.clone();
        from_entry.connect_changed(move |_| {
            recalculate(&st_entry);
        });

        // Connect Swap button
        let st_swap = state.clone();
        swap_btn.connect_clicked(move |_| {
            let (f_idx, t_idx) = {
                let st = st_swap.borrow();
                (st.from_idx, st.to_idx)
            };
            st_swap.borrow().from_dropdown.set_selected(t_idx as u32);
            st_swap.borrow().to_dropdown.set_selected(f_idx as u32);
            st_swap.borrow_mut().from_idx = t_idx;
            st_swap.borrow_mut().to_idx = f_idx;
            recalculate(&st_swap);
        });

        // Connect Copy button
        let st_copy = state.clone();
        let toast_fn = Rc::new(on_toast);
        copy_btn.connect_clicked(move |_| {
            let text = st_copy.borrow().to_entry.text().to_string();
            if !text.is_empty()
                && let Some(display) = gdk::Display::default()
            {
                display.clipboard().set_text(&text);
                toast_fn(&format!("Copied: {text}"));
            }
        });

        Self { root, state }
    }
}

fn update_category_units(state: &Rc<RefCell<ConverterState>>) {
    let units = {
        let st = state.borrow();
        units_for_category(st.category)
    };

    let labels: Vec<String> = units.iter().map(|u| format!("{} ({})", u.name, u.symbol)).collect();
    let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

    let from_list = gtk::StringList::new(&label_refs);
    let to_list = gtk::StringList::new(&label_refs);

    let (from_sel, to_sel) = {
        let mut st = state.borrow_mut();
        st.from_units = units;
        if st.from_idx >= st.from_units.len() {
            st.from_idx = 0;
        }
        if st.to_idx >= st.from_units.len() {
            st.to_idx = if st.from_units.len() > 1 { 1 } else { 0 };
        }
        st.from_dropdown.set_model(Some(&from_list));
        st.to_dropdown.set_model(Some(&to_list));
        (st.from_idx, st.to_idx)
    };

    state.borrow().from_dropdown.set_selected(from_sel as u32);
    state.borrow().to_dropdown.set_selected(to_sel as u32);

    update_quick_chips(state);
}

fn update_quick_chips(state: &Rc<RefCell<ConverterState>>) {
    let chips_box = state.borrow().chips_box.clone();
    while let Some(child) = chips_box.first_child() {
        chips_box.remove(&child);
    }

    let category = state.borrow().category;
    let favorites: &[(&str, &str, usize, usize)] = match category {
        UnitCategory::Length => &[("km ⇄ mi", "km", 3, 7), ("m ⇄ ft", "m", 2, 5), ("cm ⇄ in", "cm", 1, 4)],
        UnitCategory::Mass => &[("kg ⇄ lb", "kg", 2, 5), ("g ⇄ oz", "g", 1, 4)],
        UnitCategory::Temperature => &[("°C ⇄ °F", "c", 0, 1), ("°C ⇄ K", "c", 0, 2)],
        UnitCategory::Data => &[("MB ⇄ KB", "MB", 3, 2), ("GB ⇄ MB", "GB", 4, 3), ("GiB ⇄ MiB", "GiB", 9, 8)],
        UnitCategory::Time => &[("h ⇄ min", "h", 5, 4), ("d ⇄ h", "d", 6, 5), ("min ⇄ s", "min", 4, 3)],
        UnitCategory::Speed => &[("km/h ⇄ mph", "km/h", 1, 2), ("m/s ⇄ km/h", "m/s", 0, 1)],
        UnitCategory::Volume => &[("L ⇄ gal", "L", 1, 9), ("mL ⇄ fl oz", "mL", 0, 5)],
        _ => &[],
    };

    for &(label, _, from_i, to_i) in favorites {
        let btn = gtk::Button::with_label(label);
        btn.add_css_class("converter-chip");
        let st = state.clone();
        btn.connect_clicked(move |_| {
            st.borrow().from_dropdown.set_selected(from_i as u32);
            st.borrow().to_dropdown.set_selected(to_i as u32);
            st.borrow_mut().from_idx = from_i;
            st.borrow_mut().to_idx = to_i;
            recalculate(&st);
        });
        chips_box.append(&btn);
    }
}

fn recalculate(state: &Rc<RefCell<ConverterState>>) {
    let (input_text, from_idx, to_idx, units) = {
        let st = state.borrow();
        (st.from_entry.text().to_string(), st.from_idx, st.to_idx, st.from_units.clone())
    };

    if from_idx >= units.len() || to_idx >= units.len() {
        return;
    }

    let from_unit = units[from_idx];
    let to_unit = units[to_idx];

    let val = match Decimal::from_str(input_text.trim()) {
        Ok(d) => d,
        Err(_) => {
            state.borrow().to_entry.set_text("");
            state.borrow().formula_label.set_text("Enter a valid number");
            return;
        }
    };

    match convert(val, from_unit.id, to_unit.id) {
        Ok(result) => {
            state.borrow().to_entry.set_text(&format!("{result} {}", to_unit.symbol));

            // Compute unit ratio explanation: 1 [from] = ? [to]
            if let Ok(unit_ratio) = convert(dec!(1), from_unit.id, to_unit.id) {
                state.borrow().formula_label.set_text(&format!("1 {} = {} {}", from_unit.symbol, unit_ratio, to_unit.symbol));
            } else {
                state.borrow().formula_label.set_text("");
            }
        }
        Err(e) => {
            state.borrow().to_entry.set_text("Error");
            state.borrow().formula_label.set_text(&e.to_string());
        }
    }
}
