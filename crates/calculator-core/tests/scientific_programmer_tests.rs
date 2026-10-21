use calculator_core::programmer::{ProgrammerState, Radix, Signedness, WordWidth, parse_programmer_radix};
use calculator_core::scientific::AngleMode;
use calculator_core::{CalcError, evaluate_str, evaluate_str_with_mode};
use rust_decimal_macros::dec;

#[test]
fn test_scientific_trig_functions() {
    // Degree mode
    assert_eq!(evaluate_str_with_mode("sin(90)", AngleMode::Degrees).unwrap(), dec!(1));
    assert_eq!(evaluate_str_with_mode("cos(0)", AngleMode::Degrees).unwrap(), dec!(1));
    assert_eq!(evaluate_str_with_mode("tan(45)", AngleMode::Degrees).unwrap(), dec!(1));

    // Radian mode
    assert_eq!(evaluate_str_with_mode("sin(pi / 2)", AngleMode::Radians).unwrap(), dec!(1));
    assert_eq!(evaluate_str_with_mode("cos(pi)", AngleMode::Radians).unwrap(), dec!(-1));

    // Domain errors
    assert!(matches!(evaluate_str_with_mode("tan(90)", AngleMode::Degrees), Err(CalcError::InvalidDomain(_))));
    assert!(matches!(evaluate_str_with_mode("asin(2)", AngleMode::Degrees), Err(CalcError::InvalidDomain(_))));
}

#[test]
fn test_scientific_hyperbolics_and_logs() {
    // Hyperbolics
    assert_eq!(evaluate_str("sinh(0)").unwrap(), dec!(0));
    assert_eq!(evaluate_str("cosh(0)").unwrap(), dec!(1));

    // Logarithms
    assert_eq!(evaluate_str("log(1000)").unwrap(), dec!(3));
    assert_eq!(evaluate_str("log2(64)").unwrap(), dec!(6));
    assert_eq!(evaluate_str_with_mode("ln(e)", AngleMode::Radians).unwrap(), dec!(1));

    // Domain error on log of negative or zero
    assert!(matches!(evaluate_str("log(0)"), Err(CalcError::InvalidDomain(_))));
    assert!(matches!(evaluate_str("ln(-5)"), Err(CalcError::InvalidDomain(_))));
}

#[test]
fn test_scientific_powers_factorials_constants() {
    assert_eq!(evaluate_str("2^8").unwrap(), dec!(256));
    assert_eq!(evaluate_str("cbrt(64)").unwrap(), dec!(4));
    assert_eq!(evaluate_str("5!").unwrap(), dec!(120));
    assert_eq!(evaluate_str("0!").unwrap(), dec!(1));

    // Constants
    let pi_val = evaluate_str("pi").unwrap();
    assert!(pi_val > dec!(3.14) && pi_val < dec!(3.15));

    let e_val = evaluate_str("e").unwrap();
    assert!(e_val > dec!(2.71) && e_val < dec!(2.72));

    let tau_val = evaluate_str("tau").unwrap();
    assert!(tau_val > dec!(6.28) && tau_val < dec!(6.29));
}

#[test]
fn test_programmer_state_and_radix_synchronization() {
    let mut prog = ProgrammerState::new();
    prog.set_width(WordWidth::Dword);
    prog.set_value(0x1A2B_3C4D);

    assert_eq!(prog.format_hex(), "1A2B3C4D");
    assert_eq!(prog.format_dec(), "439041101");
    assert_eq!(prog.format_oct(), "3212636115");
    assert_eq!(prog.format_bin(), "0001 1010 0010 1011 0011 1100 0100 1101");

    // Switching width truncates to mask
    prog.set_width(WordWidth::Byte);
    assert_eq!(prog.format_hex(), "4D");
    assert_eq!(prog.format_dec(), "77");
    assert_eq!(prog.format_bin(), "0100 1101");

    // Signedness negative
    prog.set_value(0xFE);
    prog.signedness = Signedness::Signed;
    assert_eq!(prog.format_dec(), "-2");
    prog.signedness = Signedness::Unsigned;
    assert_eq!(prog.format_dec(), "254");
}

#[test]
fn test_programmer_bit_toggling_and_shifts() {
    let mut prog = ProgrammerState::new();
    prog.set_width(WordWidth::Byte);
    prog.set_value(0);

    // Toggle bits 0, 1, 2 -> 7
    prog.toggle_bit(0);
    prog.toggle_bit(1);
    prog.toggle_bit(2);
    assert_eq!(prog.value, 7);

    // Arithmetic shift right preserves sign
    prog.set_value(0x80); // -128 in signed 8-bit
    prog.shr_arithmetic(1);
    assert_eq!(prog.value, 0xC0); // -64
}

#[test]
fn test_programmer_radix_parsing() {
    assert_eq!(parse_programmer_radix("DEADBEEF", Radix::Hex, WordWidth::Dword).unwrap(), 0xDEAD_BEEF);
    assert_eq!(parse_programmer_radix("123456", Radix::Dec, WordWidth::Dword).unwrap(), 123456);
    assert_eq!(parse_programmer_radix("777", Radix::Oct, WordWidth::Word).unwrap(), 0o777);
    assert_eq!(parse_programmer_radix("1010 1010", Radix::Bin, WordWidth::Byte).unwrap(), 0b1010_1010);
}
