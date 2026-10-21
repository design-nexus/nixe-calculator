use calculator_core::{evaluate_str, format_decimal, preview_str, CalcError, FormatOptions, MemoryRegister};
use calculator_storage::CalculatorDb;
use rust_decimal_macros::dec;

#[test]
fn test_complex_precedence_and_grouping() {
    // Basic operations & precedence
    assert_eq!(evaluate_str("10 + 20 * 3").unwrap(), dec!(70));
    assert_eq!(evaluate_str("(10 + 20) * 3").unwrap(), dec!(90));
    assert_eq!(evaluate_str("100 - 50 / 2").unwrap(), dec!(75));
    assert_eq!(evaluate_str("(100 - 50) / 2").unwrap(), dec!(25));

    // Nested parentheses and implicit multiplication
    assert_eq!(evaluate_str("2(3 + 4(2 + 1))").unwrap(), dec!(30)); // 2 * (3 + 4 * 3) = 2 * 15 = 30
    assert_eq!(evaluate_str("3(2 + 1)(4 - 2)").unwrap(), dec!(18)); // 3 * 3 * 2 = 18

    // Powers and roots
    assert_eq!(evaluate_str("2 ^ 3 ^ 2").unwrap(), dec!(512)); // right-associative: 2^(3^2) = 2^9 = 512
    assert_eq!(evaluate_str("sqrt(64) + 4").unwrap(), dec!(12));
    assert_eq!(evaluate_str("sqr(5) - 10").unwrap(), dec!(15));
    assert_eq!(evaluate_str("recip(0.5) * 10").unwrap(), dec!(20));
}

#[test]
fn test_percentage_rules() {
    // Additive
    assert_eq!(evaluate_str("100 + 10%").unwrap(), dec!(110));
    assert_eq!(evaluate_str("150 + 20%").unwrap(), dec!(180));

    // Subtractive
    assert_eq!(evaluate_str("100 - 10%").unwrap(), dec!(90));
    assert_eq!(evaluate_str("80 - 25%").unwrap(), dec!(60));

    // Multiplicative
    assert_eq!(evaluate_str("200 * 15%").unwrap(), dec!(30));
    assert_eq!(evaluate_str("50 * 50%").unwrap(), dec!(25));

    // Standalone
    assert_eq!(evaluate_str("15%").unwrap(), dec!(0.15));
}

#[test]
fn test_memory_and_formatting() {
    let mut mem = MemoryRegister::new();
    assert!(mem.is_empty());

    let val1 = evaluate_str("125 * 4").unwrap(); // 500
    mem.store(val1);
    assert_eq!(mem.recall(), dec!(500));

    let val2 = evaluate_str("50").unwrap();
    mem.add(val2).unwrap();
    assert_eq!(mem.recall(), dec!(550));

    mem.sub(dec!(100)).unwrap();
    assert_eq!(mem.recall(), dec!(450));

    let opts = FormatOptions {
        thousands_separator: true,
        precision: Some(2),
    };
    assert_eq!(format_decimal(mem.recall(), &opts), "450.00");
}

#[test]
fn test_live_preview_and_error_handling() {
    // Valid preview
    assert_eq!(preview_str("125 + 4"), Some(dec!(129)));
    // Trailing operator preview (gracefully falls back to completed prefix)
    assert_eq!(preview_str("125 + 4 * "), Some(dec!(129)));

    // Division by zero
    assert_eq!(evaluate_str("10 / 0"), Err(CalcError::DivisionByZero));
    assert_eq!(evaluate_str("10 / (5 - 5)"), Err(CalcError::DivisionByZero));

    // Domain error
    assert!(matches!(evaluate_str("sqrt(-16)"), Err(CalcError::InvalidDomain(_))));
}

#[test]
fn test_database_history_lifecycle() {
    let db = CalculatorDb::open_in_memory().unwrap();

    let id1 = db.save_calculation("125 + 4", "129", "129", "standard").unwrap();
    let id2 = db.save_calculation("100 + 10%", "110", "110", "standard").unwrap();

    let list = db.list_history(None, 10, 0).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, id2); // newest first

    // Pin older calculation
    db.toggle_pin(id1).unwrap();
    let repinned = db.list_history(None, 10, 0).unwrap();
    assert_eq!(repinned[0].id, id1); // pinned comes first

    // Search
    let search_res = db.list_history(Some("10%"), 10, 0).unwrap();
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_res[0].id, id2);

    // Delete
    db.delete_history(id1).unwrap();
    assert_eq!(db.list_history(None, 10, 0).unwrap().len(), 1);

    // Clear
    db.clear_history().unwrap();
    assert_eq!(db.list_history(None, 10, 0).unwrap().len(), 0);
}
