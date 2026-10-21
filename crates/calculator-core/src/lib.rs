pub mod ast;
pub mod error;
pub mod evaluator;
pub mod formatting;
pub mod memory;
pub mod parser;
pub mod token;

pub use error::CalcError;
pub use evaluator::{evaluate_str, preview_str};
pub use formatting::{FormatOptions, format_decimal};
pub use memory::MemoryRegister;

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate_str("2 + 3").unwrap(), dec!(5));
        assert_eq!(evaluate_str("10 - 4").unwrap(), dec!(6));
        assert_eq!(evaluate_str("6 * 7").unwrap(), dec!(42));
        assert_eq!(evaluate_str("6 × 7").unwrap(), dec!(42));
        assert_eq!(evaluate_str("15 / 3").unwrap(), dec!(5));
        assert_eq!(evaluate_str("15 ÷ 3").unwrap(), dec!(5));
    }

    #[test]
    fn test_precedence_and_parentheses() {
        assert_eq!(evaluate_str("2 + 3 * 4").unwrap(), dec!(14));
        assert_eq!(evaluate_str("(2 + 3) * 4").unwrap(), dec!(20));
        assert_eq!(evaluate_str("2 * (3 + 4)").unwrap(), dec!(14));
        assert_eq!(evaluate_str("2(3 + 4)").unwrap(), dec!(14)); // implicit multiplication
        assert_eq!(evaluate_str("(2 + 3)(4 - 2)").unwrap(), dec!(10));
    }

    #[test]
    fn test_exact_decimal_arithmetic() {
        // In IEEE-754 floats, 0.1 + 0.2 != 0.3. In our decimal engine, it is exact!
        assert_eq!(evaluate_str("0.1 + 0.2").unwrap(), dec!(0.3));
        assert_eq!(evaluate_str("1.000000000000001 - 1").unwrap(), dec!(0.000000000000001));
    }

    #[test]
    fn test_unary_operators() {
        assert_eq!(evaluate_str("-5 + 10").unwrap(), dec!(5));
        assert_eq!(evaluate_str("-(-5)").unwrap(), dec!(5));
        assert_eq!(evaluate_str("+5 * -3").unwrap(), dec!(-15));
        assert_eq!(evaluate_str("-(2 + 3)").unwrap(), dec!(-5));
    }

    #[test]
    fn test_percentages() {
        // Additive percentage
        assert_eq!(evaluate_str("100 + 10%").unwrap(), dec!(110));
        assert_eq!(evaluate_str("200 + 15%").unwrap(), dec!(230));

        // Subtractive percentage
        assert_eq!(evaluate_str("100 - 10%").unwrap(), dec!(90));
        assert_eq!(evaluate_str("50 - 20%").unwrap(), dec!(40));

        // Multiplicative percentage
        assert_eq!(evaluate_str("200 * 15%").unwrap(), dec!(30));
        assert_eq!(evaluate_str("50%").unwrap(), dec!(0.5));
    }

    #[test]
    fn test_powers_and_functions() {
        assert_eq!(evaluate_str("2 ^ 3").unwrap(), dec!(8));
        assert_eq!(evaluate_str("sqrt(144)").unwrap(), dec!(12));
        assert_eq!(evaluate_str("√144").unwrap(), dec!(12));
        assert_eq!(evaluate_str("sqr(5)").unwrap(), dec!(25));
        assert_eq!(evaluate_str("5²").unwrap(), dec!(25));
        assert_eq!(evaluate_str("recip(4)").unwrap(), dec!(0.25));
        assert_eq!(evaluate_str("abs(-42)").unwrap(), dec!(42));
    }

    #[test]
    fn test_error_handling() {
        assert_eq!(evaluate_str("5 / 0"), Err(CalcError::DivisionByZero));
        assert!(matches!(evaluate_str("sqrt(-4)"), Err(CalcError::InvalidDomain(_))));
        assert_eq!(evaluate_str("(2 + 3"), Err(CalcError::UnbalancedParentheses));
        assert_eq!(evaluate_str(""), Err(CalcError::EmptyExpression));
    }

    #[test]
    fn test_memory_register() {
        let mut mem = MemoryRegister::new();
        assert!(mem.is_empty());

        mem.store(dec!(100));
        assert_eq!(mem.recall(), dec!(100));
        assert!(!mem.is_empty());

        mem.add(dec!(50)).unwrap();
        assert_eq!(mem.recall(), dec!(150));

        mem.sub(dec!(30)).unwrap();
        assert_eq!(mem.recall(), dec!(120));

        mem.clear();
        assert!(mem.is_empty());
        assert_eq!(mem.recall(), dec!(0));
    }

    #[test]
    fn test_number_formatting() {
        let opts = FormatOptions { thousands_separator: true, precision: None };
        assert_eq!(format_decimal(dec!(1234567.89), &opts), "1,234,567.89");
        assert_eq!(format_decimal(dec!(-9876543.21), &opts), "−9,876,543.21");
        assert_eq!(format_decimal(dec!(0), &opts), "0");

        let opts_fixed = FormatOptions { thousands_separator: true, precision: Some(2) };
        assert_eq!(format_decimal(dec!(42), &opts_fixed), "42.00");
    }

    #[test]
    fn test_preview_str() {
        assert_eq!(preview_str("2 + 3 * 4"), Some(dec!(14)));
        // Incomplete expression while user is typing
        assert_eq!(preview_str("2 + 3 * "), Some(dec!(5))); // prefix "2 + 3"
        assert_eq!(preview_str(""), None);
    }
}
