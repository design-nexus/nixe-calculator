use crate::display::DisplayView;
use crate::history_sidebar::HistorySidebar;
use crate::keypad::{KeyAction, ProgrammerKeypad, ScientificKeypad, StandardKeypad};
use crate::keys::install_calculator_keys;
use crate::views::{ConverterView, GraphView, SheetsView};
use crate::{prefs, settings_dialog, theme};
use calculator_core::programmer::{ProgrammerState, Signedness, WordWidth};
use calculator_core::scientific::AngleMode;
use calculator_core::{CalcError, FormatOptions, MemoryRegister, evaluate_str_with_mode, format_decimal, preview_str_with_mode};
use calculator_storage::CalculatorDb;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use nixe_ui::chrome;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

pub(crate) struct Ui {
    window: gtk::ApplicationWindow,
    overlay: gtk::Overlay,
    content_stack: gtk::Stack,
    display: DisplayView,
    keypad_stack: gtk::Stack,
    standard_keypad: StandardKeypad,
    scientific_keypad: ScientificKeypad,
    programmer_keypad: ProgrammerKeypad,
    #[allow(dead_code)]
    converter_view: ConverterView,
    #[allow(dead_code)]
    sheets_view: SheetsView,
    #[allow(dead_code)]
    graph_view: GraphView,
    sidebar: Rc<RefCell<HistorySidebar>>,
    sidebar_container: gtk::Box,
    mode_btn: gtk::Button,
    db: CalculatorDb,
    memory: MemoryRegister,
    toast: gtk::Label,
    toast_timer: Option<glib::SourceId>,

    // Calculation state
    mode: String,
    angle_mode: AngleMode,
    programmer: ProgrammerState,
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

    window.set_titlebar(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
    chrome::set_title(&window, "Calculator", "Standard");

    let overlay = gtk::Overlay::new();
    let main_h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let calc_v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    calc_v_box.set_hexpand(true);

    // Top bar
    let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    topbar.add_css_class("calc-topbar");

    // Mode dropdown button & Popover
    let mode_btn = gtk::Button::with_label("Standard ▾");
    mode_btn.add_css_class("mode-selector");
    mode_btn.set_tooltip_text(Some("Switch mode (Alt+1 to Alt+5)"));

    let popover = gtk::Popover::new();
    let pop_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    pop_box.set_margin_start(6);
    pop_box.set_margin_end(6);
    pop_box.set_margin_top(6);
    pop_box.set_margin_bottom(6);

    let btn_m_std = gtk::Button::with_label("Standard");
    btn_m_std.add_css_class("flat");
    let btn_m_sci = gtk::Button::with_label("Scientific");
    btn_m_sci.add_css_class("flat");
    let btn_m_prg = gtk::Button::with_label("Programmer");
    btn_m_prg.add_css_class("flat");
    let btn_m_conv = gtk::Button::with_label("Unit Converter");
    btn_m_conv.add_css_class("flat");
    let btn_m_sheets = gtk::Button::with_label("Calculation Sheets");
    btn_m_sheets.add_css_class("flat");
    let btn_m_graph = gtk::Button::with_label("Graphing Calculator");
    btn_m_graph.add_css_class("flat");

    pop_box.append(&btn_m_std);
    pop_box.append(&btn_m_sci);
    pop_box.append(&btn_m_prg);
    pop_box.append(&btn_m_conv);
    pop_box.append(&btn_m_sheets);
    pop_box.append(&btn_m_graph);
    popover.set_child(Some(&pop_box));
    popover.set_parent(&mode_btn);

    let p_clone = popover.clone();
    mode_btn.connect_clicked(move |_| p_clone.popup());

    let p_s1 = popover.clone();
    btn_m_std.connect_clicked(move |_| {
        p_s1.popdown();
        switch_mode("standard");
    });
    let p_s2 = popover.clone();
    btn_m_sci.connect_clicked(move |_| {
        p_s2.popdown();
        switch_mode("scientific");
    });
    let p_s3 = popover.clone();
    btn_m_prg.connect_clicked(move |_| {
        p_s3.popdown();
        switch_mode("programmer");
    });
    let p_s4 = popover.clone();
    btn_m_conv.connect_clicked(move |_| {
        p_s4.popdown();
        switch_mode("converter");
    });
    let p_s5 = popover.clone();
    btn_m_sheets.connect_clicked(move |_| {
        p_s5.popdown();
        switch_mode("sheets");
    });
    let p_s6 = popover.clone();
    btn_m_graph.connect_clicked(move |_| {
        p_s6.popdown();
        switch_mode("graph");
    });

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

    // Database
    let db = CalculatorDb::open_default().unwrap_or_else(|_| CalculatorDb::open_in_memory().unwrap());

    // Calc View (Display + Keypad Stack)
    let calc_view = gtk::Box::new(gtk::Orientation::Vertical, 0);
    calc_view.set_vexpand(true);

    let display = DisplayView::new();
    calc_view.append(&display.root);

    let keypad_stack = gtk::Stack::new();
    keypad_stack.set_transition_type(gtk::StackTransitionType::Crossfade);

    let standard_keypad = StandardKeypad::new(handle_action);
    keypad_stack.add_named(&standard_keypad.root, Some("standard"));

    let scientific_keypad = ScientificKeypad::new(handle_action);
    keypad_stack.add_named(&scientific_keypad.root, Some("scientific"));

    let programmer_keypad = ProgrammerKeypad::new(handle_action);
    keypad_stack.add_named(&programmer_keypad.root, Some("programmer"));

    calc_view.append(&keypad_stack);

    // Content Stack
    let content_stack = gtk::Stack::new();
    content_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    content_stack.set_vexpand(true);
    content_stack.add_named(&calc_view, Some("calc"));

    let converter_view = ConverterView::new(toast);
    content_stack.add_named(&converter_view.root, Some("converter"));

    let sheets_view = SheetsView::new(db.clone(), toast);
    content_stack.add_named(&sheets_view.root, Some("sheets"));

    let graph_view = GraphView::new(toast);
    content_stack.add_named(&graph_view.root, Some("graph"));

    calc_v_box.append(&content_stack);
    main_h_box.append(&calc_v_box);

    // History sidebar
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

    // Common keys
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
    install_calculator_keys(&window, handle_action, copy_result, paste_to_expression, toggle_history, switch_mode);

    hist_btn.connect_clicked(|_| toggle_history());

    let state = Rc::new(RefCell::new(Ui {
        window,
        overlay,
        content_stack,
        display,
        keypad_stack,
        standard_keypad,
        scientific_keypad,
        programmer_keypad,
        converter_view,
        sheets_view,
        graph_view,
        sidebar,
        sidebar_container,
        mode_btn,
        db,
        memory: MemoryRegister::new(),
        toast: toast_lbl,
        toast_timer: None,
        mode: "standard".into(),
        angle_mode: AngleMode::Degrees,
        programmer: ProgrammerState::new(),
        input_buffer: String::new(),
        tape_text: String::new(),
        is_result_shown: false,
        error_state: None,
    }));

    UI.with(|u| *u.borrow_mut() = Some(state));

    let initial_mode = std::env::var("CALCULATOR_MODE").unwrap_or_else(|_| prefs::get().calculator_mode);
    if initial_mode != "standard" {
        switch_mode(&initial_mode);
    }
}

pub fn switch_mode(mode: &str) {
    let Some(ui_rc) = ui() else { return };
    let mut ui = ui_rc.borrow_mut();
    ui.mode = mode.to_string();

    let (label, title, stack_view, keypad_view, w, h) = match mode {
        "scientific" => ("Scientific ▾", "Scientific", "calc", "scientific", 500, 580),
        "programmer" => ("Programmer ▾", "Programmer", "calc", "programmer", 540, 660),
        "converter" => ("Converter ▾", "Unit Converter", "converter", "standard", 400, 580),
        "sheets" => ("Sheets ▾", "Calculation Sheets", "sheets", "standard", 680, 580),
        "graph" => ("Graph ▾", "Graphing Calculator", "graph", "standard", 780, 600),
        _ => ("Standard ▾", "Standard", "calc", "standard", 360, 540),
    };

    ui.content_stack.set_visible_child_name(stack_view);
    ui.keypad_stack.set_visible_child_name(keypad_view);
    ui.mode_btn.set_label(label);
    chrome::set_title(&ui.window, "Calculator", title);

    let is_hist_open = ui.sidebar_container.is_visible();
    let final_w = if is_hist_open { w + 280 } else { w };
    ui.window.set_default_size(final_w, h);

    if mode == "programmer" {
        // Sync programmer values
        if let Ok(num) = get_current_number(&ui)
            && let Some(u) = num.to_u64()
        {
            ui.programmer.set_value(u);
        }
        ui.programmer_keypad.update_state(&ui.programmer);
    }

    prefs::update(|p| p.calculator_mode = mode.to_string());
}

pub fn handle_action(action: KeyAction) {
    let Some(ui_rc) = ui() else { return };
    let mut ui = ui_rc.borrow_mut();

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
        KeyAction::OpMod => append_binary_op(&mut ui, " % "),
        KeyAction::BitAnd => append_binary_op(&mut ui, " & "),
        KeyAction::BitOr => append_binary_op(&mut ui, " | "),
        KeyAction::BitXor => append_binary_op(&mut ui, " ^ "),
        KeyAction::Shl => append_binary_op(&mut ui, " << "),
        KeyAction::Shr => append_binary_op(&mut ui, " >> "),
        KeyAction::BitNot => {
            if ui.input_buffer.is_empty() {
                ui.input_buffer.push_str("~0");
            } else {
                ui.input_buffer = format!("~({})", ui.input_buffer);
            }
            execute_calculation(&mut ui);
        }
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
            if let Ok(val) = evaluate_str_with_mode(&ui.input_buffer, ui.angle_mode) {
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
        KeyAction::Sci(func) => {
            let name = match func {
                calculator_core::scientific::SciFunc::Sin => "sin",
                calculator_core::scientific::SciFunc::Cos => "cos",
                calculator_core::scientific::SciFunc::Tan => "tan",
                calculator_core::scientific::SciFunc::Asin => "asin",
                calculator_core::scientific::SciFunc::Acos => "acos",
                calculator_core::scientific::SciFunc::Atan => "atan",
                calculator_core::scientific::SciFunc::Sinh => "sinh",
                calculator_core::scientific::SciFunc::Cosh => "cosh",
                calculator_core::scientific::SciFunc::Tanh => "tanh",
                calculator_core::scientific::SciFunc::Ln => "ln",
                calculator_core::scientific::SciFunc::Log10 => "log",
                calculator_core::scientific::SciFunc::Log2 => "log2",
                calculator_core::scientific::SciFunc::Exp => "exp",
                calculator_core::scientific::SciFunc::Sqrt => "sqrt",
                calculator_core::scientific::SciFunc::Cbrt => "cbrt",
                calculator_core::scientific::SciFunc::Sqr => "sqr",
                calculator_core::scientific::SciFunc::Fact => "fact",
                calculator_core::scientific::SciFunc::Abs => "abs",
            };
            wrap_current_or_evaluate(&mut ui, name);
        }
        KeyAction::ConstPi => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            }
            ui.input_buffer.push('π');
            update_readout(&mut ui);
        }
        KeyAction::ConstE => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            }
            ui.input_buffer.push('e');
            update_readout(&mut ui);
        }
        KeyAction::ConstTau => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            }
            ui.input_buffer.push('τ');
            update_readout(&mut ui);
        }
        KeyAction::ToggleAngleMode => {
            ui.angle_mode = match ui.angle_mode {
                AngleMode::Degrees => AngleMode::Radians,
                AngleMode::Radians => AngleMode::Degrees,
            };
            ui.scientific_keypad.set_angle_mode(ui.angle_mode);
            let mode_name = match ui.angle_mode {
                AngleMode::Degrees => "Degrees (DEG)",
                AngleMode::Radians => "Radians (RAD)",
            };
            toast(&format!("Angle mode: {mode_name}"));
            update_readout(&mut ui);
        }
        KeyAction::ExpNotation => {
            ui.input_buffer.push('e');
            ui.is_result_shown = false;
            update_readout(&mut ui);
        }
        // Programmer Mode Actions
        KeyAction::ToggleBit(bit_idx) => {
            ui.programmer.toggle_bit(bit_idx);
            ui.input_buffer = ui.programmer.value.to_string();
            ui.programmer_keypad.update_state(&ui.programmer);
            update_readout(&mut ui);
        }
        KeyAction::SelectRadix(radix) => {
            ui.programmer.active_radix = radix;
            ui.programmer_keypad.update_state(&ui.programmer);
        }
        KeyAction::CycleWordWidth => {
            let next_width = match ui.programmer.width {
                WordWidth::Qword => WordWidth::Dword,
                WordWidth::Dword => WordWidth::Word,
                WordWidth::Word => WordWidth::Byte,
                WordWidth::Byte => WordWidth::Qword,
            };
            ui.programmer.set_width(next_width);
            ui.programmer_keypad.update_state(&ui.programmer);
            update_readout(&mut ui);
        }
        KeyAction::ToggleSignedness => {
            ui.programmer.signedness = match ui.programmer.signedness {
                Signedness::Signed => Signedness::Unsigned,
                Signedness::Unsigned => Signedness::Signed,
            };
            ui.programmer_keypad.update_state(&ui.programmer);
        }
        KeyAction::Backspace => {
            if ui.is_result_shown {
                ui.input_buffer.clear();
                ui.is_result_shown = false;
            } else {
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
            ui.programmer.set_value(0);
            ui.programmer_keypad.update_state(&ui.programmer);
            ui.display.set_tape("");
            ui.display.set_preview(None);
            ui.display.set_readout("0");
            ui.display.set_error(None);
        }
        KeyAction::ClearEntry => {
            if let Some(pos) = ui.input_buffer.rfind(|ch| ['+', '-', '*', '/', '^', '×', '÷', '−', '&', '|'].contains(&ch)) {
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
            ui.standard_keypad.set_memory_enabled(false);
            ui.scientific_keypad.set_memory_enabled(false);
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
                ui.standard_keypad.set_memory_enabled(true);
                ui.scientific_keypad.set_memory_enabled(true);
                toast(&format!("Memory set: {val}"));
            }
        }
        KeyAction::MemAdd => {
            if let Ok(val) = get_current_number(&ui)
                && ui.memory.add(val).is_ok()
            {
                ui.display.set_memory_active(true);
                ui.standard_keypad.set_memory_enabled(true);
                ui.scientific_keypad.set_memory_enabled(true);
                toast(&format!("Memory: {}", ui.memory.recall()));
            }
        }
        KeyAction::MemSub => {
            if let Ok(val) = get_current_number(&ui)
                && ui.memory.sub(val).is_ok()
            {
                ui.display.set_memory_active(true);
                ui.standard_keypad.set_memory_enabled(true);
                ui.scientific_keypad.set_memory_enabled(true);
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

    let trimmed = ui.input_buffer.trim_end();
    if trimmed.ends_with(['+', '-', '*', '/', '^', '×', '÷', '−', '&', '|']) {
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
        if ui.mode == "programmer" {
            ui.programmer.set_value(0);
            ui.programmer_keypad.update_state(&ui.programmer);
        }
        return;
    }

    if let Some(preview_val) = preview_str_with_mode(&ui.input_buffer, ui.angle_mode) {
        let formatted_preview = format_decimal(preview_val, &opts);
        ui.display.set_preview(Some(&formatted_preview));
        if ui.mode == "programmer"
            && let Some(u) = preview_val.to_u64()
        {
            ui.programmer.set_value(u);
            ui.programmer_keypad.update_state(&ui.programmer);
        }
    } else {
        ui.display.set_preview(None);
    }

    ui.display.set_readout(&ui.input_buffer);
}

fn execute_calculation(ui: &mut Ui) {
    if ui.input_buffer.trim().is_empty() {
        return;
    }
    let opts = get_format_options();
    match evaluate_str_with_mode(&ui.input_buffer, ui.angle_mode) {
        Ok(res) => {
            let formatted = format_decimal(res, &opts);
            let raw_res = res.to_string();
            let expr = ui.input_buffer.clone();

            ui.tape_text = format!("{expr} =");
            ui.display.set_tape(&ui.tape_text);
            ui.display.set_readout(&formatted);
            ui.display.set_preview(None);
            ui.display.set_error(None);

            if ui.mode == "programmer"
                && let Some(u) = res.to_u64()
            {
                ui.programmer.set_value(u);
                ui.programmer_keypad.update_state(&ui.programmer);
            }

            if prefs::get().history_enabled {
                let _ = ui.db.save_calculation(&expr, &formatted, &raw_res, &ui.mode);
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
    if let Ok(res) = evaluate_str_with_mode(&ui.input_buffer, ui.angle_mode) {
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
    let (dialog, card) = nixe_ui::widgets::dialog("Keyboard Shortcuts", 480);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    list.add_css_class("group-list");

    let shortcuts: &[(&[&str], &str)] = &[
        (&["Alt", "1"], "Standard mode"),
        (&["Alt", "2"], "Scientific mode"),
        (&["Alt", "3"], "Programmer mode"),
        (&["Alt", "4"], "Unit Converter"),
        (&["Alt", "5"], "Calculation Sheets"),
        (&["Alt", "6"], "Graphing Calculator"),
        (&["0–9", "A–F", "."], "Enter numbers / hex digits"),
        (&["+", "−", "×", "÷"], "Basic arithmetic"),
        (&["&", "|", "~", "^"], "Bitwise AND, OR, NOT, XOR"),
        (&["Enter", "="], "Calculate result"),
        (&["Backspace"], "Delete character"),
        (&["Esc", "C"], "Clear expression"),
        (&["Ctrl", "H"], "Toggle history"),
        (&["Ctrl", "C"], "Copy result"),
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
        .max_content_height(420)
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
    glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
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
