use gtk::prelude::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Digit(char),
    Dot,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpPow,
    Percent,
    Negate,
    Reciprocal,
    Square,
    Sqrt,
    LParen,
    RParen,
    Clear,
    ClearEntry,
    Backspace,
    Equals,
    MemClear,
    MemRecall,
    MemAdd,
    MemSub,
    MemStore,
}

pub struct KeypadView {
    pub root: gtk::Box,
    pub mem_clear_btn: gtk::Button,
    pub mem_recall_btn: gtk::Button,
}

impl KeypadView {
    pub fn new(on_action: impl Fn(KeyAction) + 'static) -> Self {
        let on_action = Rc::new(on_action);
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Memory bar
        let membar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        membar.add_css_class("calc-membar");
        membar.set_homogeneous(true);

        let mc = make_mem_btn("MC", "Clear memory", KeyAction::MemClear, on_action.clone());
        let mr = make_mem_btn("MR", "Recall memory", KeyAction::MemRecall, on_action.clone());
        let m_plus = make_mem_btn("M+", "Add to memory", KeyAction::MemAdd, on_action.clone());
        let m_minus = make_mem_btn("M−", "Subtract from memory", KeyAction::MemSub, on_action.clone());
        let ms = make_mem_btn("MS", "Store in memory", KeyAction::MemStore, on_action.clone());

        membar.append(&mc);
        membar.append(&mr);
        membar.append(&m_plus);
        membar.append(&m_minus);
        membar.append(&ms);
        root.append(&membar);

        // 4-column Grid for Keypad
        let grid = gtk::Grid::builder().column_spacing(8).row_spacing(8).column_homogeneous(true).row_homogeneous(true).build();
        grid.add_css_class("calc-keypad");

        // Row 0: %, CE, C, ⌫
        grid.attach(&make_btn("%", "calc-btn-op", KeyAction::Percent, on_action.clone()), 0, 0, 1, 1);
        grid.attach(&make_btn("CE", "calc-btn-clear", KeyAction::ClearEntry, on_action.clone()), 1, 0, 1, 1);
        grid.attach(&make_btn("C", "calc-btn-clear", KeyAction::Clear, on_action.clone()), 2, 0, 1, 1);
        grid.attach(&make_btn("⌫", "calc-btn-clear", KeyAction::Backspace, on_action.clone()), 3, 0, 1, 1);

        // Row 1: 1/x, x², √x, ÷
        grid.attach(&make_btn("1/x", "calc-btn-op", KeyAction::Reciprocal, on_action.clone()), 0, 1, 1, 1);
        grid.attach(&make_btn("x²", "calc-btn-op", KeyAction::Square, on_action.clone()), 1, 1, 1, 1);
        grid.attach(&make_btn("√x", "calc-btn-op", KeyAction::Sqrt, on_action.clone()), 2, 1, 1, 1);
        grid.attach(&make_btn("÷", "calc-btn-op", KeyAction::OpDiv, on_action.clone()), 3, 1, 1, 1);

        // Row 2: 7, 8, 9, ×
        grid.attach(&make_btn("7", "calc-btn-digit", KeyAction::Digit('7'), on_action.clone()), 0, 2, 1, 1);
        grid.attach(&make_btn("8", "calc-btn-digit", KeyAction::Digit('8'), on_action.clone()), 1, 2, 1, 1);
        grid.attach(&make_btn("9", "calc-btn-digit", KeyAction::Digit('9'), on_action.clone()), 2, 2, 1, 1);
        grid.attach(&make_btn("×", "calc-btn-op", KeyAction::OpMul, on_action.clone()), 3, 2, 1, 1);

        // Row 3: 4, 5, 6, −
        grid.attach(&make_btn("4", "calc-btn-digit", KeyAction::Digit('4'), on_action.clone()), 0, 3, 1, 1);
        grid.attach(&make_btn("5", "calc-btn-digit", KeyAction::Digit('5'), on_action.clone()), 1, 3, 1, 1);
        grid.attach(&make_btn("6", "calc-btn-digit", KeyAction::Digit('6'), on_action.clone()), 2, 3, 1, 1);
        grid.attach(&make_btn("−", "calc-btn-op", KeyAction::OpSub, on_action.clone()), 3, 3, 1, 1);

        // Row 4: 1, 2, 3, +
        grid.attach(&make_btn("1", "calc-btn-digit", KeyAction::Digit('1'), on_action.clone()), 0, 4, 1, 1);
        grid.attach(&make_btn("2", "calc-btn-digit", KeyAction::Digit('2'), on_action.clone()), 1, 4, 1, 1);
        grid.attach(&make_btn("3", "calc-btn-digit", KeyAction::Digit('3'), on_action.clone()), 2, 4, 1, 1);
        grid.attach(&make_btn("+", "calc-btn-op", KeyAction::OpAdd, on_action.clone()), 3, 4, 1, 1);

        // Row 5: ±, 0, ., =
        grid.attach(&make_btn("±", "calc-btn-digit", KeyAction::Negate, on_action.clone()), 0, 5, 1, 1);
        grid.attach(&make_btn("0", "calc-btn-digit", KeyAction::Digit('0'), on_action.clone()), 1, 5, 1, 1);
        grid.attach(&make_btn(".", "calc-btn-digit", KeyAction::Dot, on_action.clone()), 2, 5, 1, 1);
        grid.attach(&make_btn("=", "calc-btn-equals", KeyAction::Equals, on_action), 3, 5, 1, 1);

        root.append(&grid);

        Self { root, mem_clear_btn: mc, mem_recall_btn: mr }
    }

    pub fn set_memory_enabled(&self, enabled: bool) {
        self.mem_clear_btn.set_sensitive(enabled);
        self.mem_recall_btn.set_sensitive(enabled);
        if enabled {
            self.mem_recall_btn.add_css_class("active");
        } else {
            self.mem_recall_btn.remove_css_class("active");
        }
    }
}

fn make_btn(label: &str, extra_class: &str, action: KeyAction, cb: Rc<dyn Fn(KeyAction)>) -> gtk::Button {
    let btn = gtk::Button::with_label(label);
    btn.add_css_class("calc-btn");
    if !extra_class.is_empty() {
        btn.add_css_class(extra_class);
    }
    btn.set_focusable(false); // Don't steal keyboard focus from window
    btn.connect_clicked(move |_| cb(action));
    btn
}

fn make_mem_btn(label: &str, tooltip: &str, action: KeyAction, cb: Rc<dyn Fn(KeyAction)>) -> gtk::Button {
    let btn = gtk::Button::with_label(label);
    btn.add_css_class("mem-btn");
    btn.set_tooltip_text(Some(tooltip));
    btn.set_focusable(false);
    btn.connect_clicked(move |_| cb(action));
    btn
}
