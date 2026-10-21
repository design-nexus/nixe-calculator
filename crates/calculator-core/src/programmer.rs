use crate::error::CalcError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordWidth {
    Byte,  // 8-bit
    Word,  // 16-bit
    Dword, // 32-bit
    #[default]
    Qword, // 64-bit
}

impl WordWidth {
    pub fn bits(&self) -> usize {
        match self {
            Self::Byte => 8,
            Self::Word => 16,
            Self::Dword => 32,
            Self::Qword => 64,
        }
    }

    pub fn mask(&self) -> u64 {
        match self {
            Self::Byte => 0xFF,
            Self::Word => 0xFFFF,
            Self::Dword => 0xFFFF_FFFF,
            Self::Qword => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    pub fn sign_bit(&self) -> u64 {
        1 << (self.bits() - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Signedness {
    #[default]
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Radix {
    Hex,
    #[default]
    Dec,
    Oct,
    Bin,
}

impl Radix {
    pub fn base(&self) -> u32 {
        match self {
            Self::Hex => 16,
            Self::Dec => 10,
            Self::Oct => 8,
            Self::Bin => 2,
        }
    }

    pub fn is_digit_valid(&self, ch: char) -> bool {
        match self {
            Self::Hex => ch.is_ascii_hexdigit(),
            Self::Dec => ch.is_ascii_digit(),
            Self::Oct => matches!(ch, '0'..='7'),
            Self::Bin => ch == '0' || ch == '1',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProgrammerState {
    pub value: u64,
    pub width: WordWidth,
    pub signedness: Signedness,
    pub active_radix: Radix,
}

impl ProgrammerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_value(&mut self, val: u64) {
        self.value = val & self.width.mask();
    }

    pub fn toggle_bit(&mut self, bit_idx: usize) {
        if bit_idx < self.width.bits() {
            self.value ^= 1 << bit_idx;
        }
    }

    pub fn get_bit(&self, bit_idx: usize) -> bool {
        if bit_idx < self.width.bits() { (self.value & (1 << bit_idx)) != 0 } else { false }
    }

    pub fn set_width(&mut self, width: WordWidth) {
        self.width = width;
        self.value &= width.mask();
    }

    pub fn format_hex(&self) -> String {
        let masked = self.value & self.width.mask();
        match self.width {
            WordWidth::Byte => format!("{masked:02X}"),
            WordWidth::Word => format!("{masked:04X}"),
            WordWidth::Dword => format!("{masked:08X}"),
            WordWidth::Qword => format!("{masked:016X}"),
        }
    }

    pub fn format_dec(&self) -> String {
        let masked = self.value & self.width.mask();
        if self.signedness == Signedness::Signed {
            match self.width {
                WordWidth::Byte => (masked as u8 as i8).to_string(),
                WordWidth::Word => (masked as u16 as i16).to_string(),
                WordWidth::Dword => (masked as u32 as i32).to_string(),
                WordWidth::Qword => (masked as i64).to_string(),
            }
        } else {
            masked.to_string()
        }
    }

    pub fn format_oct(&self) -> String {
        let masked = self.value & self.width.mask();
        format!("{masked:o}")
    }

    pub fn format_bin(&self) -> String {
        let total_bits = self.width.bits();
        let mut groups = Vec::new();
        let num_nibbles = total_bits / 4;

        for i in (0..num_nibbles).rev() {
            let nibble = (self.value >> (i * 4)) & 0x0F;
            groups.push(format!("{nibble:04b}"));
        }
        groups.join(" ")
    }

    pub fn bitwise_and(&mut self, rhs: u64) {
        self.value = (self.value & rhs) & self.width.mask();
    }

    pub fn bitwise_or(&mut self, rhs: u64) {
        self.value = (self.value | rhs) & self.width.mask();
    }

    pub fn bitwise_xor(&mut self, rhs: u64) {
        self.value = (self.value ^ rhs) & self.width.mask();
    }

    pub fn bitwise_not(&mut self) {
        self.value = (!self.value) & self.width.mask();
    }

    pub fn shl(&mut self, shift: u32) {
        self.value = (self.value << shift) & self.width.mask();
    }

    pub fn shr_logical(&mut self, shift: u32) {
        self.value = (self.value >> shift) & self.width.mask();
    }

    pub fn shr_arithmetic(&mut self, shift: u32) {
        let bits = self.width.bits() as u32;
        if shift >= bits {
            if (self.value & self.width.sign_bit()) != 0 {
                self.value = self.width.mask();
            } else {
                self.value = 0;
            }
            return;
        }
        let sign = (self.value & self.width.sign_bit()) != 0;
        let mut res = self.value >> shift;
        if sign {
            let fill = self.width.mask() << (bits - shift);
            res |= fill;
        }
        self.value = res & self.width.mask();
    }

    pub fn rol(&mut self, shift: u32) {
        let bits = self.width.bits() as u32;
        let s = shift % bits;
        let left = (self.value << s) & self.width.mask();
        let right = (self.value >> (bits - s)) & self.width.mask();
        self.value = (left | right) & self.width.mask();
    }

    pub fn ror(&mut self, shift: u32) {
        let bits = self.width.bits() as u32;
        let s = shift % bits;
        let right = (self.value >> s) & self.width.mask();
        let left = (self.value << (bits - s)) & self.width.mask();
        self.value = (left | right) & self.width.mask();
    }

    pub fn add(&mut self, rhs: u64) {
        self.value = self.value.wrapping_add(rhs) & self.width.mask();
    }

    pub fn sub(&mut self, rhs: u64) {
        self.value = self.value.wrapping_sub(rhs) & self.width.mask();
    }

    pub fn mul(&mut self, rhs: u64) {
        self.value = self.value.wrapping_mul(rhs) & self.width.mask();
    }

    pub fn div(&mut self, rhs: u64) -> Result<(), CalcError> {
        let rhs_masked = rhs & self.width.mask();
        let res = self.value.checked_div(rhs_masked).ok_or(CalcError::DivisionByZero)?;
        self.value = res & self.width.mask();
        Ok(())
    }

    pub fn rem(&mut self, rhs: u64) -> Result<(), CalcError> {
        let rhs_masked = rhs & self.width.mask();
        let res = self.value.checked_rem(rhs_masked).ok_or(CalcError::DivisionByZero)?;
        self.value = res & self.width.mask();
        Ok(())
    }
}

pub fn parse_programmer_radix(input: &str, radix: Radix, width: WordWidth) -> Result<u64, CalcError> {
    let clean = input.replace([' ', '_', ','], "");
    if clean.is_empty() {
        return Ok(0);
    }
    let val = match radix {
        Radix::Hex => u64::from_str_radix(&clean, 16).map_err(|e| CalcError::SyntaxError(format!("Invalid HEX '{clean}': {e}"))),
        Radix::Dec => {
            if let Some(stripped) = clean.strip_prefix('-') {
                let pos = stripped.parse::<i64>().map_err(|e| CalcError::SyntaxError(format!("Invalid DEC '{clean}': {e}")))?;
                Ok((-pos) as u64)
            } else {
                clean.parse::<u64>().map_err(|e| CalcError::SyntaxError(format!("Invalid DEC '{clean}': {e}")))
            }
        }
        Radix::Oct => u64::from_str_radix(&clean, 8).map_err(|e| CalcError::SyntaxError(format!("Invalid OCT '{clean}': {e}"))),
        Radix::Bin => u64::from_str_radix(&clean, 2).map_err(|e| CalcError::SyntaxError(format!("Invalid BIN '{clean}': {e}"))),
    }?;

    Ok(val & width.mask())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radix_formatting_and_synchronization() {
        let mut state = ProgrammerState::new();
        state.set_width(WordWidth::Byte);
        state.set_value(0b1010_1111); // 0xAF, 175 unsigned, -81 signed

        assert_eq!(state.format_hex(), "AF");
        assert_eq!(state.format_oct(), "257");
        assert_eq!(state.format_bin(), "1010 1111");

        // Signed
        state.signedness = Signedness::Signed;
        assert_eq!(state.format_dec(), "-81");

        // Unsigned
        state.signedness = Signedness::Unsigned;
        assert_eq!(state.format_dec(), "175");
    }

    #[test]
    fn test_bit_toggling() {
        let mut state = ProgrammerState::new();
        state.set_width(WordWidth::Byte);
        state.set_value(0);

        state.toggle_bit(0);
        assert_eq!(state.value, 1);
        assert!(state.get_bit(0));

        state.toggle_bit(7);
        assert_eq!(state.value, 0x81);
        assert!(state.get_bit(7));

        state.toggle_bit(0);
        assert_eq!(state.value, 0x80);
    }

    #[test]
    fn test_bitwise_operations() {
        let mut state = ProgrammerState::new();
        state.set_width(WordWidth::Byte);
        state.set_value(0b1100_1010);

        state.bitwise_and(0b1111_0000);
        assert_eq!(state.value, 0b1100_0000);

        state.bitwise_or(0b0000_1111);
        assert_eq!(state.value, 0b1100_1111);

        state.bitwise_xor(0b1111_0000);
        assert_eq!(state.value, 0b0011_1111);

        state.bitwise_not();
        assert_eq!(state.value, 0b1100_0000);
    }

    #[test]
    fn test_shifts_and_rotates() {
        let mut state = ProgrammerState::new();
        state.set_width(WordWidth::Byte);
        state.set_value(0b0000_1001); // 9

        state.shl(2);
        assert_eq!(state.value, 0b0010_0100); // 36

        state.shr_logical(1);
        assert_eq!(state.value, 0b0001_0010); // 18

        state.set_value(0b1000_0001);
        state.rol(1);
        assert_eq!(state.value, 0b0000_0011);

        state.ror(1);
        assert_eq!(state.value, 0b1000_0001);
    }

    #[test]
    fn test_parse_programmer_radix() {
        assert_eq!(parse_programmer_radix("FF", Radix::Hex, WordWidth::Byte).unwrap(), 0xFF);
        assert_eq!(parse_programmer_radix("255", Radix::Dec, WordWidth::Byte).unwrap(), 0xFF);
        assert_eq!(parse_programmer_radix("377", Radix::Oct, WordWidth::Byte).unwrap(), 0xFF);
        assert_eq!(parse_programmer_radix("1111 1111", Radix::Bin, WordWidth::Byte).unwrap(), 0xFF);
    }
}
