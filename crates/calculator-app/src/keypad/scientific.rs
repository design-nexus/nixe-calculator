use super::{KeyAction, make_btn, make_mem_btn};
use calculator_core::scientific::{AngleMode, SciFunc};
use gtk::prelude::*;
use std::rc::Rc;

pub struct ScientificKeypad {
    pub root: gtk::Box,
    pub deg_rad_btn: gtk::Button,
    pub mem_clear_btn: gtk::Button,
    pub mem_recall_btn: gtk::Button,
}

impl ScientificKeypad {
    pub fn new(on_action: impl Fn(KeyAction) + 'static) -> Self {
        let on_action = Rc::new(on_action);
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Top bar: DEG/RAD toggle + Memory buttons
        let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        topbar.add_css_class("calc-membar");

        let deg_rad = gtk::Button::with_label("DEG");
        deg_rad.add_css_class("mode-selector");
        deg_rad.set_tooltip_text(Some("Toggle angle mode (Degrees / Radians)"));
        deg_rad.set_focusable(false);
        let cb_deg = on_action.clone();
        deg_rad.connect_clicked(move |_| cb_deg(KeyAction::ToggleAngleMode));
        topbar.append(&deg_rad);

        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        topbar.append(&spacer);

        let mc = make_mem_btn("MC", "Clear memory", KeyAction::MemClear, on_action.clone());
        let mr = make_mem_btn("MR", "Recall memory", KeyAction::MemRecall, on_action.clone());
        let m_plus = make_mem_btn("M+", "Add to memory", KeyAction::MemAdd, on_action.clone());
        let m_minus = make_mem_btn("M−", "Subtract from memory", KeyAction::MemSub, on_action.clone());
        let ms = make_mem_btn("MS", "Store in memory", KeyAction::MemStore, on_action.clone());

        topbar.append(&mc);
        topbar.append(&mr);
        topbar.append(&m_plus);
        topbar.append(&m_minus);
        topbar.append(&ms);
        root.append(&topbar);

        // 6-column Grid for Scientific Functions + Standard Keypad
        let grid = gtk::Grid::builder().column_spacing(6).row_spacing(6).column_homogeneous(true).row_homogeneous(true).build();
        grid.add_css_class("calc-keypad");

        // Row 0: sin, cos, tan, π, C, ⌫
        grid.attach(&make_btn("sin", "calc-btn-op", KeyAction::Sci(SciFunc::Sin), on_action.clone()), 0, 0, 1, 1);
        grid.attach(&make_btn("cos", "calc-btn-op", KeyAction::Sci(SciFunc::Cos), on_action.clone()), 1, 0, 1, 1);
        grid.attach(&make_btn("tan", "calc-btn-op", KeyAction::Sci(SciFunc::Tan), on_action.clone()), 2, 0, 1, 1);
        grid.attach(&make_btn("π", "calc-btn-op", KeyAction::ConstPi, on_action.clone()), 3, 0, 1, 1);
        grid.attach(&make_btn("C", "calc-btn-clear", KeyAction::Clear, on_action.clone()), 4, 0, 1, 1);
        grid.attach(&make_btn("⌫", "calc-btn-clear", KeyAction::Backspace, on_action.clone()), 5, 0, 1, 1);

        // Row 1: asin, acos, atan, e, (, )
        grid.attach(&make_btn("asin", "calc-btn-op", KeyAction::Sci(SciFunc::Asin), on_action.clone()), 0, 1, 1, 1);
        grid.attach(&make_btn("acos", "calc-btn-op", KeyAction::Sci(SciFunc::Acos), on_action.clone()), 1, 1, 1, 1);
        grid.attach(&make_btn("atan", "calc-btn-op", KeyAction::Sci(SciFunc::Atan), on_action.clone()), 2, 1, 1, 1);
        grid.attach(&make_btn("e", "calc-btn-op", KeyAction::ConstE, on_action.clone()), 3, 1, 1, 1);
        grid.attach(&make_btn("(", "calc-btn-op", KeyAction::LParen, on_action.clone()), 4, 1, 1, 1);
        grid.attach(&make_btn(")", "calc-btn-op", KeyAction::RParen, on_action.clone()), 5, 1, 1, 1);

        // Row 2: sinh, cosh, tanh, τ, xʸ, ÷
        grid.attach(&make_btn("sinh", "calc-btn-op", KeyAction::Sci(SciFunc::Sinh), on_action.clone()), 0, 2, 1, 1);
        grid.attach(&make_btn("cosh", "calc-btn-op", KeyAction::Sci(SciFunc::Cosh), on_action.clone()), 1, 2, 1, 1);
        grid.attach(&make_btn("tanh", "calc-btn-op", KeyAction::Sci(SciFunc::Tanh), on_action.clone()), 2, 2, 1, 1);
        grid.attach(&make_btn("τ", "calc-btn-op", KeyAction::ConstTau, on_action.clone()), 3, 2, 1, 1);
        grid.attach(&make_btn("xʸ", "calc-btn-op", KeyAction::OpPow, on_action.clone()), 4, 2, 1, 1);
        grid.attach(&make_btn("÷", "calc-btn-op", KeyAction::OpDiv, on_action.clone()), 5, 2, 1, 1);

        // Row 3: ln, log, 7, 8, 9, ×
        grid.attach(&make_btn("ln", "calc-btn-op", KeyAction::Sci(SciFunc::Ln), on_action.clone()), 0, 3, 1, 1);
        grid.attach(&make_btn("log", "calc-btn-op", KeyAction::Sci(SciFunc::Log10), on_action.clone()), 1, 3, 1, 1);
        grid.attach(&make_btn("7", "calc-btn-digit", KeyAction::Digit('7'), on_action.clone()), 2, 3, 1, 1);
        grid.attach(&make_btn("8", "calc-btn-digit", KeyAction::Digit('8'), on_action.clone()), 3, 3, 1, 1);
        grid.attach(&make_btn("9", "calc-btn-digit", KeyAction::Digit('9'), on_action.clone()), 4, 3, 1, 1);
        grid.attach(&make_btn("×", "calc-btn-op", KeyAction::OpMul, on_action.clone()), 5, 3, 1, 1);

        // Row 4: √x, ³√x, 4, 5, 6, −
        grid.attach(&make_btn("√x", "calc-btn-op", KeyAction::Sqrt, on_action.clone()), 0, 4, 1, 1);
        grid.attach(&make_btn("³√x", "calc-btn-op", KeyAction::Sci(SciFunc::Cbrt), on_action.clone()), 1, 4, 1, 1);
        grid.attach(&make_btn("4", "calc-btn-digit", KeyAction::Digit('4'), on_action.clone()), 2, 4, 1, 1);
        grid.attach(&make_btn("5", "calc-btn-digit", KeyAction::Digit('5'), on_action.clone()), 3, 4, 1, 1);
        grid.attach(&make_btn("6", "calc-btn-digit", KeyAction::Digit('6'), on_action.clone()), 4, 4, 1, 1);
        grid.attach(&make_btn("−", "calc-btn-op", KeyAction::OpSub, on_action.clone()), 5, 4, 1, 1);

        // Row 5: n!, 1/x, 1, 2, 3, +
        grid.attach(&make_btn("n!", "calc-btn-op", KeyAction::Sci(SciFunc::Fact), on_action.clone()), 0, 5, 1, 1);
        grid.attach(&make_btn("1/x", "calc-btn-op", KeyAction::Reciprocal, on_action.clone()), 1, 5, 1, 1);
        grid.attach(&make_btn("1", "calc-btn-digit", KeyAction::Digit('1'), on_action.clone()), 2, 5, 1, 1);
        grid.attach(&make_btn("2", "calc-btn-digit", KeyAction::Digit('2'), on_action.clone()), 3, 5, 1, 1);
        grid.attach(&make_btn("3", "calc-btn-digit", KeyAction::Digit('3'), on_action.clone()), 4, 5, 1, 1);
        grid.attach(&make_btn("+", "calc-btn-op", KeyAction::OpAdd, on_action.clone()), 5, 5, 1, 1);

        // Row 6: abs, EXP, ±, 0, ., =
        grid.attach(&make_btn("|x|", "calc-btn-op", KeyAction::Sci(SciFunc::Abs), on_action.clone()), 0, 6, 1, 1);
        grid.attach(&make_btn("EXP", "calc-btn-op", KeyAction::ExpNotation, on_action.clone()), 1, 6, 1, 1);
        grid.attach(&make_btn("±", "calc-btn-digit", KeyAction::Negate, on_action.clone()), 2, 6, 1, 1);
        grid.attach(&make_btn("0", "calc-btn-digit", KeyAction::Digit('0'), on_action.clone()), 3, 6, 1, 1);
        grid.attach(&make_btn(".", "calc-btn-digit", KeyAction::Dot, on_action.clone()), 4, 6, 1, 1);
        grid.attach(&make_btn("=", "calc-btn-equals", KeyAction::Equals, on_action), 5, 6, 1, 1);

        root.append(&grid);

        Self { root, deg_rad_btn: deg_rad, mem_clear_btn: mc, mem_recall_btn: mr }
    }

    pub fn set_angle_mode(&self, mode: AngleMode) {
        match mode {
            AngleMode::Degrees => self.deg_rad_btn.set_label("DEG"),
            AngleMode::Radians => self.deg_rad_btn.set_label("RAD"),
        }
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
