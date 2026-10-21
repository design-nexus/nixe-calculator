use calculator_core::natural::{NaturalSession, SheetLineKind};
use calculator_core::units::{UnitCategory, convert};
use rust_decimal_macros::dec;

#[test]
fn test_all_twelve_unit_categories() {
    // 1. Length
    assert_eq!(convert(dec!(5), "mi", "km").unwrap(), dec!(8.04672));
    assert_eq!(convert(dec!(100), "cm", "m").unwrap(), dec!(1));

    // 2. Mass
    assert_eq!(convert(dec!(1), "kg", "g").unwrap(), dec!(1000));
    assert_eq!(convert(dec!(1), "lb", "oz").unwrap(), dec!(16));

    // 3. Temperature
    assert_eq!(convert(dec!(32), "degF", "degC").unwrap(), dec!(0));
    assert_eq!(convert(dec!(100), "degC", "degF").unwrap(), dec!(212));
    assert_eq!(convert(dec!(0), "degC", "K").unwrap(), dec!(273.15));

    // 4. Area
    assert_eq!(convert(dec!(1), "sqkm", "ha").unwrap(), dec!(100));
    assert_eq!(convert(dec!(1), "ha", "sqm").unwrap(), dec!(10000));

    // 5. Volume
    assert_eq!(convert(dec!(1), "l", "ml").unwrap(), dec!(1000));
    assert_eq!(convert(dec!(1), "gal", "l").unwrap(), dec!(3.785411784));

    // 6. Speed
    assert_eq!(convert(dec!(100), "km/h", "m/s").unwrap().round_dp(2), dec!(27.78));
    assert_eq!(convert(dec!(60), "mph", "km/h").unwrap().round_dp(2), dec!(96.56));

    // 7. Time
    assert_eq!(convert(dec!(1), "h", "min").unwrap(), dec!(60));
    assert_eq!(convert(dec!(1), "d", "h").unwrap(), dec!(24));

    // 8. Data
    assert_eq!(convert(dec!(1), "GB", "MB").unwrap(), dec!(1000));
    assert_eq!(convert(dec!(1), "GiB", "MiB").unwrap(), dec!(1024));
    assert_eq!(convert(dec!(8), "bit", "byte").unwrap(), dec!(1));

    // 9. Energy
    assert_eq!(convert(dec!(1), "kWh", "Wh").unwrap(), dec!(1000));
    assert_eq!(convert(dec!(1), "cal", "J").unwrap(), dec!(4.184));

    // 10. Power
    assert_eq!(convert(dec!(1), "kW", "W").unwrap(), dec!(1000));

    // 11. Pressure
    assert_eq!(convert(dec!(1), "bar", "kPa").unwrap(), dec!(100));
    assert_eq!(convert(dec!(1), "atm", "Pa").unwrap(), dec!(101325));

    // 12. Angles
    assert_eq!(convert(dec!(180), "deg", "rad").unwrap().round_dp(4), dec!(3.1416));
    assert_eq!(convert(dec!(360), "deg", "grad").unwrap(), dec!(400));
}

#[test]
fn test_category_listing() {
    assert_eq!(UnitCategory::ALL.len(), 12);
    for cat in UnitCategory::ALL {
        let units = calculator_core::units_for_category(cat);
        assert!(!units.is_empty(), "Category {:?} has no units", cat);
    }
}

#[test]
fn test_natural_numi_calculations() {
    let mut session = NaturalSession::new();
    let text = r#"
# Vacation Planning
hotel = $1200
flight = 450
subtotal = hotel + flight
tax = 8%
total = subtotal + tax
$250 split between 4 people
5 miles in kilometers
32 fahrenheit to celsius
2 hours 30 minutes in seconds
"#;

    let outputs = session.evaluate_sheet(text);

    // subtotal = 1200 + 450 = 1650
    if let SheetLineKind::Result(res) = &outputs[4].kind {
        assert_eq!(res.value, dec!(1650));
    } else {
        panic!("Failed to evaluate subtotal");
    }

    // total = 1650 + 8% = 1782
    if let SheetLineKind::Result(res) = &outputs[6].kind {
        assert_eq!(res.value, dec!(1782));
    } else {
        panic!("Failed to evaluate total with tax");
    }

    // split between 4 people = 62.5
    if let SheetLineKind::Result(res) = &outputs[7].kind {
        assert_eq!(res.value, dec!(62.5));
        assert_eq!(res.currency, Some('$'));
    } else {
        panic!("Failed to split");
    }

    // 5 miles in kilometers = 8.04672 km
    if let SheetLineKind::Result(res) = &outputs[8].kind {
        assert_eq!(res.value, dec!(8.04672));
    } else {
        panic!("Failed 5 miles in kilometers");
    }

    // 32 fahrenheit to celsius = 0 °C
    if let SheetLineKind::Result(res) = &outputs[9].kind {
        assert_eq!(res.value, dec!(0));
    } else {
        panic!("Failed 32 fahrenheit to celsius");
    }

    // 2 hours 30 minutes in seconds = 9000 s
    if let SheetLineKind::Result(res) = &outputs[10].kind {
        assert_eq!(res.value, dec!(9000));
    } else {
        panic!("Failed 2 hours 30 minutes in seconds");
    }
}
