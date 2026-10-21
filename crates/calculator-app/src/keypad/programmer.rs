use super::{KeyAction, make_btn};
use calculator_core::programmer::{ProgrammerState, Radix, Signedness, WordWidth};
use gtk::prelude::*;
use std::rc::Rc;

pub struct ProgrammerKeypad {
    pub root: gtk::Box,
    pub hex_label: gtk::Label,
    pub dec_label: gtk::Label,
    pub oct_label: gtk::Label,
    pub bin_label: gtk::Label,
    pub hex_row: gtk::Box,
    pub dec_row: gtk::Box,
    pub oct_row: gtk::Box,
    pub bin_row: gtk::Box,
    pub word_btn: gtk::Button,
    pub sign_btn: gtk::Button,
    pub bit_buttons: Vec<gtk::Button>,
    pub digit_buttons: Vec<(char, gtk::Button)>,
}

impl ProgrammerKeypad {
    pub fn new(on_action: impl Fn(KeyAction) + 'static) -> Self {
        let on_action = Rc::new(on_action);
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);

        // 1. Radix Display Rows
        let radix_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        radix_box.add_css_class("calc-membar");

        let (hex_row, hex_label) = make_radix_row("HEX", Radix::Hex, on_action.clone());
        let (dec_row, dec_label) = make_radix_row("DEC", Radix::Dec, on_action.clone());
        let (oct_row, oct_label) = make_radix_row("OCT", Radix::Oct, on_action.clone());
        let (bin_row, bin_label) = make_radix_row("BIN", Radix::Bin, on_action.clone());

        radix_box.append(&hex_row);
        radix_box.append(&dec_row);
        radix_box.append(&oct_row);
        radix_box.append(&bin_row);
        root.append(&radix_box);

        // 2. Control Row: Word Width & Signedness
        let ctrl_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        ctrl_row.set_margin_start(12);
        ctrl_row.set_margin_end(12);
        ctrl_row.set_margin_top(4);
        ctrl_row.set_margin_bottom(4);

        let word_btn = gtk::Button::with_label("QWORD");
        word_btn.add_css_class("mode-selector");
        word_btn.set_tooltip_text(Some("Word width: QWORD (64), DWORD (32), WORD (16), BYTE (8)"));
        word_btn.set_focusable(false);
        let cb_word = on_action.clone();
        word_btn.connect_clicked(move |_| cb_word(KeyAction::CycleWordWidth));
        ctrl_row.append(&word_btn);

        let sign_btn = gtk::Button::with_label("SIGNED");
        sign_btn.add_css_class("mode-selector");
        sign_btn.set_tooltip_text(Some("Toggle Signed / Unsigned representation"));
        sign_btn.set_focusable(false);
        let cb_sign = on_action.clone();
        sign_btn.connect_clicked(move |_| cb_sign(KeyAction::ToggleSignedness));
        ctrl_row.append(&sign_btn);

        root.append(&ctrl_row);

        // 3. Interactive 64-bit Grid
        // 4 rows of 16 bits: 63..48, 47..32, 31..16, 15..0
        let bit_grid = gtk::Grid::builder().column_spacing(2).row_spacing(2).column_homogeneous(true).build();
        bit_grid.set_margin_start(10);
        bit_grid.set_margin_end(10);
        bit_grid.set_margin_top(4);
        bit_grid.set_margin_bottom(6);

        let mut bit_buttons = Vec::with_capacity(64);
        for row in 0..4 {
            for col in 0..16 {
                let bit_idx = 63 - (row * 16 + col);
                let btn = gtk::Button::with_label("0");
                btn.add_css_class("bit-btn");
                btn.set_tooltip_text(Some(&format!("Bit {bit_idx}")));
                btn.set_focusable(false);
                let cb_bit = on_action.clone();
                btn.connect_clicked(move |_| cb_bit(KeyAction::ToggleBit(bit_idx)));
                bit_grid.attach(&btn, col as i32, row as i32, 1, 1);
                bit_buttons.push(btn);
            }
        }
        root.append(&bit_grid);

        // 4. Programmer Keypad Grid (6 columns)
        let grid = gtk::Grid::builder().column_spacing(6).row_spacing(6).column_homogeneous(true).row_homogeneous(true).build();
        grid.add_css_class("calc-keypad");
        grid.add_css_class("calc-keypad-prog");

        let mut digit_buttons = Vec::new();

        // Row 0: A, B, AND, OR, NOT, C
        let btn_a = make_digit_btn('A', on_action.clone());
        grid.attach(&btn_a, 0, 0, 1, 1);
        digit_buttons.push(('A', btn_a));

        let btn_b = make_digit_btn('B', on_action.clone());
        grid.attach(&btn_b, 1, 0, 1, 1);
        digit_buttons.push(('B', btn_b));

        grid.attach(&make_btn("AND", "calc-btn-op", KeyAction::BitAnd, on_action.clone()), 2, 0, 1, 1);
        grid.attach(&make_btn("OR", "calc-btn-op", KeyAction::BitOr, on_action.clone()), 3, 0, 1, 1);
        grid.attach(&make_btn("NOT", "calc-btn-op", KeyAction::BitNot, on_action.clone()), 4, 0, 1, 1);
        grid.attach(&make_btn("C", "calc-btn-clear", KeyAction::Clear, on_action.clone()), 5, 0, 1, 1);

        // Row 1: C, D, XOR, LSH, RSH, ⌫
        let btn_c = make_digit_btn('C', on_action.clone());
        grid.attach(&btn_c, 0, 1, 1, 1);
        digit_buttons.push(('C', btn_c));

        let btn_d = make_digit_btn('D', on_action.clone());
        grid.attach(&btn_d, 1, 1, 1, 1);
        digit_buttons.push(('D', btn_d));

        grid.attach(&make_btn("XOR", "calc-btn-op", KeyAction::BitXor, on_action.clone()), 2, 1, 1, 1);
        grid.attach(&make_btn("LSH", "calc-btn-op", KeyAction::Shl, on_action.clone()), 3, 1, 1, 1);
        grid.attach(&make_btn("RSH", "calc-btn-op", KeyAction::Shr, on_action.clone()), 4, 1, 1, 1);
        grid.attach(&make_btn("⌫", "calc-btn-clear", KeyAction::Backspace, on_action.clone()), 5, 1, 1, 1);

        // Row 2: E, F, mod, (, ), ÷
        let btn_e = make_digit_btn('E', on_action.clone());
        grid.attach(&btn_e, 0, 2, 1, 1);
        digit_buttons.push(('E', btn_e));

        let btn_f = make_digit_btn('F', on_action.clone());
        grid.attach(&btn_f, 1, 2, 1, 1);
        digit_buttons.push(('F', btn_f));

        grid.attach(&make_btn("mod", "calc-btn-op", KeyAction::OpMod, on_action.clone()), 2, 2, 1, 1);
        grid.attach(&make_btn("(", "calc-btn-op", KeyAction::LParen, on_action.clone()), 3, 2, 1, 1);
        grid.attach(&make_btn(")", "calc-btn-op", KeyAction::RParen, on_action.clone()), 4, 2, 1, 1);
        grid.attach(&make_btn("÷", "calc-btn-op", KeyAction::OpDiv, on_action.clone()), 5, 2, 1, 1);

        // Row 3: 7, 8, 9, ×, ±, CE
        let btn_7 = make_digit_btn('7', on_action.clone());
        grid.attach(&btn_7, 0, 3, 1, 1);
        digit_buttons.push(('7', btn_7));

        let btn_8 = make_digit_btn('8', on_action.clone());
        grid.attach(&btn_8, 1, 3, 1, 1);
        digit_buttons.push(('8', btn_8));

        let btn_9 = make_digit_btn('9', on_action.clone());
        grid.attach(&btn_9, 2, 3, 1, 1);
        digit_buttons.push(('9', btn_9));

        grid.attach(&make_btn("×", "calc-btn-op", KeyAction::OpMul, on_action.clone()), 3, 3, 1, 1);
        grid.attach(&make_btn("±", "calc-btn-digit", KeyAction::Negate, on_action.clone()), 4, 3, 1, 1);
        grid.attach(&make_btn("CE", "calc-btn-clear", KeyAction::ClearEntry, on_action.clone()), 5, 3, 1, 1);

        // Row 4: 4, 5, 6, −, +, =
        let btn_4 = make_digit_btn('4', on_action.clone());
        grid.attach(&btn_4, 0, 4, 1, 1);
        digit_buttons.push(('4', btn_4));

        let btn_5 = make_digit_btn('5', on_action.clone());
        grid.attach(&btn_5, 1, 4, 1, 1);
        digit_buttons.push(('5', btn_5));

        let btn_6 = make_digit_btn('6', on_action.clone());
        grid.attach(&btn_6, 2, 4, 1, 1);
        digit_buttons.push(('6', btn_6));

        grid.attach(&make_btn("−", "calc-btn-op", KeyAction::OpSub, on_action.clone()), 3, 4, 1, 1);
        grid.attach(&make_btn("+", "calc-btn-op", KeyAction::OpAdd, on_action.clone()), 4, 4, 1, 1);
        grid.attach(&make_btn("=", "calc-btn-equals", KeyAction::Equals, on_action.clone()), 5, 4, 1, 2);

        // Row 5: 1, 2, 3, 0, .
        let btn_1 = make_digit_btn('1', on_action.clone());
        grid.attach(&btn_1, 0, 5, 1, 1);
        digit_buttons.push(('1', btn_1));

        let btn_2 = make_digit_btn('2', on_action.clone());
        grid.attach(&btn_2, 1, 5, 1, 1);
        digit_buttons.push(('2', btn_2));

        let btn_3 = make_digit_btn('3', on_action.clone());
        grid.attach(&btn_3, 2, 5, 1, 1);
        digit_buttons.push(('3', btn_3));

        let btn_0 = make_digit_btn('0', on_action);
        grid.attach(&btn_0, 3, 5, 2, 1);
        digit_buttons.push(('0', btn_0));

        root.append(&grid);

        Self {
            root,
            hex_label,
            dec_label,
            oct_label,
            bin_label,
            hex_row,
            dec_row,
            oct_row,
            bin_row,
            word_btn,
            sign_btn,
            bit_buttons,
            digit_buttons,
        }
    }

    pub fn update_state(&self, state: &ProgrammerState) {
        // Update labels
        self.hex_label.set_text(&state.format_hex());
        self.dec_label.set_text(&state.format_dec());
        self.oct_label.set_text(&state.format_oct());
        self.bin_label.set_text(&state.format_bin());

        // Update active radix styling
        self.hex_row.remove_css_class("active-radix");
        self.dec_row.remove_css_class("active-radix");
        self.oct_row.remove_css_class("active-radix");
        self.bin_row.remove_css_class("active-radix");

        match state.active_radix {
            Radix::Hex => self.hex_row.add_css_class("active-radix"),
            Radix::Dec => self.dec_row.add_css_class("active-radix"),
            Radix::Oct => self.oct_row.add_css_class("active-radix"),
            Radix::Bin => self.bin_row.add_css_class("active-radix"),
        }

        // Update word width and signedness labels
        let w_label = match state.width {
            WordWidth::Qword => "QWORD (64)",
            WordWidth::Dword => "DWORD (32)",
            WordWidth::Word => "WORD (16)",
            WordWidth::Byte => "BYTE (8)",
        };
        self.word_btn.set_label(w_label);

        let s_label = match state.signedness {
            Signedness::Signed => "SIGNED",
            Signedness::Unsigned => "UNSIGNED",
        };
        self.sign_btn.set_label(s_label);

        // Update 64 bits display and sensitivity
        let max_bits = state.width.bits();
        for (i, btn) in self.bit_buttons.iter().enumerate() {
            let bit_idx = 63 - i;
            let bit_val = state.get_bit(bit_idx);
            btn.set_label(if bit_val { "1" } else { "0" });
            if bit_idx < max_bits {
                btn.set_sensitive(true);
                if bit_val {
                    btn.add_css_class("active");
                } else {
                    btn.remove_css_class("active");
                }
            } else {
                btn.set_sensitive(false);
                btn.remove_css_class("active");
            }
        }

        // Update digit buttons sensitivity based on active radix
        for (ch, btn) in &self.digit_buttons {
            btn.set_sensitive(state.active_radix.is_digit_valid(*ch));
        }
    }
}

fn make_radix_row(name: &str, radix: Radix, cb: Rc<dyn Fn(KeyAction)>) -> (gtk::Box, gtk::Label) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("calc-radix-row");

    let tag = gtk::Label::new(Some(name));
    tag.add_css_class("calc-radix-tag");
    row.append(&tag);

    let val = gtk::Label::new(Some("0"));
    val.add_css_class("calc-radix-val");
    val.set_hexpand(true);
    val.set_xalign(1.0);
    val.set_ellipsize(gtk::pango::EllipsizeMode::Start);
    val.set_selectable(true);
    row.append(&val);

    let gesture = gtk::GestureClick::new();
    gesture.connect_pressed(move |_, _, _, _| {
        cb(KeyAction::SelectRadix(radix));
    });
    row.add_controller(gesture);

    (row, val)
}

fn make_digit_btn(ch: char, cb: Rc<dyn Fn(KeyAction)>) -> gtk::Button {
    make_btn(&ch.to_string(), "calc-btn-digit", KeyAction::Digit(ch), cb)
}
