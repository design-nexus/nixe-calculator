use crate::error::CalcError;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryRegister {
    value: Decimal,
    is_set: bool,
}

impl MemoryRegister {
    pub fn new() -> Self {
        Self { value: Decimal::ZERO, is_set: false }
    }

    pub fn store(&mut self, val: Decimal) {
        self.value = val;
        self.is_set = true;
    }

    pub fn recall(&self) -> Decimal {
        self.value
    }

    pub fn add(&mut self, val: Decimal) -> Result<(), CalcError> {
        let new_val = self.value.checked_add(val).ok_or(CalcError::Overflow)?;
        self.value = new_val;
        self.is_set = true;
        Ok(())
    }

    pub fn sub(&mut self, val: Decimal) -> Result<(), CalcError> {
        let new_val = self.value.checked_sub(val).ok_or(CalcError::Overflow)?;
        self.value = new_val;
        self.is_set = true;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.value = Decimal::ZERO;
        self.is_set = false;
    }

    pub fn is_empty(&self) -> bool {
        !self.is_set || self.value.is_zero()
    }

    pub fn is_active(&self) -> bool {
        self.is_set
    }

    pub fn value(&self) -> Decimal {
        self.value
    }
}
