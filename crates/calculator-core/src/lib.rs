pub mod ast;
pub mod error;
pub mod evaluator;
pub mod formatting;
pub mod memory;
pub mod natural;
pub mod parser;
pub mod programmer;
pub mod scientific;
pub mod token;
pub mod units;

pub use error::CalcError;
pub use evaluator::{eval_f64, evaluate_str, evaluate_str_with_mode, preview_str, preview_str_with_mode};
pub use formatting::{FormatOptions, format_decimal};
pub use memory::MemoryRegister;
pub use natural::{LineValue, NaturalSession, SheetLineKind, SheetLineOutput};
pub use parser::parse_str;
pub use programmer::{ProgrammerState, Radix, Signedness, WordWidth, parse_programmer_radix};
pub use scientific::{AngleMode, CONST_E, CONST_PI, CONST_TAU, SciFunc, eval_sci_func};
pub use units::{UNITS, UnitCategory, UnitDefinition, convert, find_unit, units_for_category};

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
    fn test_scientific_trig_and_powers() {
        // Trig in Degrees
        let sin90 = evaluate_str("sin(90)").unwrap();
        assert_eq!(sin90, dec!(1));

        let cos0 = evaluate_str("cos(0)").unwrap();
        assert_eq!(cos0, dec!(1));

        // Trig in Radians
        let sin_rad = evaluate_str_with_mode("sin(pi / 2)", AngleMode::Radians).unwrap();
        assert_eq!(sin_rad, dec!(1));

        // Factorials
        assert_eq!(evaluate_str("5!").unwrap(), dec!(120));
        assert_eq!(evaluate_str("fact(5)").unwrap(), dec!(120));

        // Logarithms
        assert_eq!(evaluate_str("log(100)").unwrap(), dec!(2));
        assert_eq!(evaluate_str_with_mode("ln(e)", AngleMode::Radians).unwrap(), dec!(1));

        // Powers and roots
        assert_eq!(evaluate_str("2^10").unwrap(), dec!(1024));
        assert_eq!(evaluate_str("cbrt(27)").unwrap(), dec!(3));
    }

    #[test]
    fn test_scientific_notation() {
        assert_eq!(evaluate_str("1e3 + 500").unwrap(), dec!(1500));
        assert_eq!(evaluate_str("2.5e-2 * 100").unwrap(), dec!(2.5));
    }

    #[test]
    fn test_programmer_operations() {
        assert_eq!(evaluate_str("0xFF & 0x0F").unwrap(), dec!(15));
        assert_eq!(evaluate_str("12 | 3").unwrap(), dec!(15));
        assert_eq!(evaluate_str("8 ^ 3").unwrap(), dec!(512)); // ^ is power in math, or xor in programmer mode
        assert_eq!(evaluate_str("1 << 4").unwrap(), dec!(16));
        assert_eq!(evaluate_str("32 >> 2").unwrap(), dec!(8));
    }

    #[test]
    fn test_percentages() {
        assert_eq!(evaluate_str("100 + 10%").unwrap(), dec!(110));
        assert_eq!(evaluate_str("200 + 15%").unwrap(), dec!(230));
        assert_eq!(evaluate_str("100 - 10%").unwrap(), dec!(90));
        assert_eq!(evaluate_str("200 * 15%").unwrap(), dec!(30));
        assert_eq!(evaluate_str("50%").unwrap(), dec!(0.5));
    }
}
