pub mod programmer;
pub mod scientific;
pub mod standard;

pub use programmer::ProgrammerKeypad;
pub use scientific::ScientificKeypad;
pub use standard::StandardKeypad;

use calculator_core::programmer::Radix;
use calculator_core::scientific::SciFunc;
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
    OpMod,
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
    // Scientific
    Sci(SciFunc),
    ConstPi,
    ConstE,
    ConstTau,
    ToggleAngleMode,
    ExpNotation,
    // Programmer
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    ToggleBit(usize),
    SelectRadix(Radix),
    CycleWordWidth,
    ToggleSignedness,
}

pub fn make_btn(label: &str, extra_class: &str, action: KeyAction, cb: Rc<dyn Fn(KeyAction)>) -> gtk::Button {
    let btn = gtk::Button::with_label(label);
    btn.add_css_class("calc-btn");
    if !extra_class.is_empty() {
        btn.add_css_class(extra_class);
    }
    btn.set_focusable(false);
    btn.connect_clicked(move |_| cb(action));
    btn
}

pub fn make_mem_btn(label: &str, tooltip: &str, action: KeyAction, cb: Rc<dyn Fn(KeyAction)>) -> gtk::Button {
    let btn = gtk::Button::with_label(label);
    btn.add_css_class("mem-btn");
    btn.set_tooltip_text(Some(tooltip));
    btn.set_focusable(false);
    btn.connect_clicked(move |_| cb(action));
    btn
}
