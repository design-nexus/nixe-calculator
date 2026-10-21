use crate::error::CalcError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitCategory {
    Length,
    Mass,
    Temperature,
    Area,
    Volume,
    Speed,
    Time,
    Data,
    Energy,
    Power,
    Pressure,
    Angle,
}

impl UnitCategory {
    pub const ALL: [UnitCategory; 12] = [
        UnitCategory::Length,
        UnitCategory::Mass,
        UnitCategory::Temperature,
        UnitCategory::Area,
        UnitCategory::Volume,
        UnitCategory::Speed,
        UnitCategory::Time,
        UnitCategory::Data,
        UnitCategory::Energy,
        UnitCategory::Power,
        UnitCategory::Pressure,
        UnitCategory::Angle,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            UnitCategory::Length => "Length",
            UnitCategory::Mass => "Mass",
            UnitCategory::Temperature => "Temperature",
            UnitCategory::Area => "Area",
            UnitCategory::Volume => "Volume",
            UnitCategory::Speed => "Speed",
            UnitCategory::Time => "Time",
            UnitCategory::Data => "Data Storage",
            UnitCategory::Energy => "Energy",
            UnitCategory::Power => "Power",
            UnitCategory::Pressure => "Pressure",
            UnitCategory::Angle => "Angles",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
    pub category: UnitCategory,
    pub aliases: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq)]
enum ConversionRatio {
    Linear(Decimal),
    TemperatureAffine { scale: Decimal, offset: Decimal },
    AngleFactor(f64),
}

impl UnitDefinition {
    fn ratio(&self) -> ConversionRatio {
        match self.id {
            // Length (base: meter)
            "mm" => ConversionRatio::Linear(dec!(0.001)),
            "cm" => ConversionRatio::Linear(dec!(0.01)),
            "m" => ConversionRatio::Linear(dec!(1)),
            "km" => ConversionRatio::Linear(dec!(1000)),
            "in" => ConversionRatio::Linear(dec!(0.0254)),
            "ft" => ConversionRatio::Linear(dec!(0.3048)),
            "yd" => ConversionRatio::Linear(dec!(0.9144)),
            "mi" => ConversionRatio::Linear(dec!(1609.344)),
            "nmi" => ConversionRatio::Linear(dec!(1852)),

            // Mass (base: gram)
            "mg" => ConversionRatio::Linear(dec!(0.001)),
            "g" => ConversionRatio::Linear(dec!(1)),
            "kg" => ConversionRatio::Linear(dec!(1000)),
            "t" => ConversionRatio::Linear(dec!(1000000)),
            "oz" => ConversionRatio::Linear(dec!(28.349523125)),
            "lb" => ConversionRatio::Linear(dec!(453.59237)),
            "st" => ConversionRatio::Linear(dec!(6350.29318)),

            // Temperature (base: Kelvin)
            // K = C + 273.15
            "degC" => ConversionRatio::TemperatureAffine { scale: dec!(1), offset: dec!(273.15) },
            // K = (F - 32) * 5 / 9 + 273.15 = (F + 459.67) * 5 / 9
            "degF" => ConversionRatio::TemperatureAffine { scale: dec!(0.5555555555555555555555555556), offset: dec!(459.67) },
            "K" => ConversionRatio::TemperatureAffine { scale: dec!(1), offset: dec!(0) },

            // Area (base: m²)
            "sqmm" => ConversionRatio::Linear(dec!(0.000001)),
            "sqcm" => ConversionRatio::Linear(dec!(0.0001)),
            "sqm" => ConversionRatio::Linear(dec!(1)),
            "sqkm" => ConversionRatio::Linear(dec!(1000000)),
            "sqin" => ConversionRatio::Linear(dec!(0.00064516)),
            "sqft" => ConversionRatio::Linear(dec!(0.09290304)),
            "sqyd" => ConversionRatio::Linear(dec!(0.83612736)),
            "ac" => ConversionRatio::Linear(dec!(4046.8564224)),
            "ha" => ConversionRatio::Linear(dec!(10000)),

            // Volume (base: liter)
            "ml" => ConversionRatio::Linear(dec!(0.001)),
            "l" => ConversionRatio::Linear(dec!(1)),
            "cum" => ConversionRatio::Linear(dec!(1000)),
            "tsp" => ConversionRatio::Linear(dec!(0.00492892159375)),
            "tbsp" => ConversionRatio::Linear(dec!(0.01478676478125)),
            "floz" => ConversionRatio::Linear(dec!(0.0295735295625)),
            "cup" => ConversionRatio::Linear(dec!(0.2365882365)),
            "pt" => ConversionRatio::Linear(dec!(0.473176473)),
            "qt" => ConversionRatio::Linear(dec!(0.946352946)),
            "gal" => ConversionRatio::Linear(dec!(3.785411784)),
            "cuft" => ConversionRatio::Linear(dec!(28.316846592)),
            "cuin" => ConversionRatio::Linear(dec!(0.016387064)),

            // Speed (base: m/s)
            "mps" => ConversionRatio::Linear(dec!(1)),
            "kmh" => ConversionRatio::Linear(dec!(0.2777777777777777777777777778)),
            "mph" => ConversionRatio::Linear(dec!(0.44704)),
            "knot" => ConversionRatio::Linear(dec!(0.5144444444444444444444444444)),
            "fps" => ConversionRatio::Linear(dec!(0.3048)),

            // Time (base: second)
            "ns" => ConversionRatio::Linear(dec!(0.000000001)),
            "us" => ConversionRatio::Linear(dec!(0.000001)),
            "ms" => ConversionRatio::Linear(dec!(0.001)),
            "s" => ConversionRatio::Linear(dec!(1)),
            "min" => ConversionRatio::Linear(dec!(60)),
            "h" => ConversionRatio::Linear(dec!(3600)),
            "d" => ConversionRatio::Linear(dec!(86400)),
            "wk" => ConversionRatio::Linear(dec!(604800)),
            "mo" => ConversionRatio::Linear(dec!(2629800)),
            "yr" => ConversionRatio::Linear(dec!(31557600)),

            // Data (base: Byte)
            "bit" => ConversionRatio::Linear(dec!(0.125)),
            "byte" => ConversionRatio::Linear(dec!(1)),
            "kb" => ConversionRatio::Linear(dec!(1000)),
            "mb" => ConversionRatio::Linear(dec!(1000000)),
            "gb" => ConversionRatio::Linear(dec!(1000000000)),
            "tb" => ConversionRatio::Linear(dec!(1000000000000)),
            "pb" => ConversionRatio::Linear(dec!(1000000000000000)),
            "kib" => ConversionRatio::Linear(dec!(1024)),
            "mib" => ConversionRatio::Linear(dec!(1048576)),
            "gib" => ConversionRatio::Linear(dec!(1073741824)),
            "tib" => ConversionRatio::Linear(dec!(1099511627776)),

            // Energy (base: Joule)
            "j" => ConversionRatio::Linear(dec!(1)),
            "kj" => ConversionRatio::Linear(dec!(1000)),
            "cal" => ConversionRatio::Linear(dec!(4.184)),
            "kcal" => ConversionRatio::Linear(dec!(4184)),
            "wh" => ConversionRatio::Linear(dec!(3600)),
            "kwh" => ConversionRatio::Linear(dec!(3600000)),
            "btu" => ConversionRatio::Linear(dec!(1055.05585262)),

            // Power (base: Watt)
            "w" => ConversionRatio::Linear(dec!(1)),
            "kw" => ConversionRatio::Linear(dec!(1000)),
            "hpm" => ConversionRatio::Linear(dec!(735.49875)),
            "hpi" => ConversionRatio::Linear(dec!(745.69987158)),

            // Pressure (base: Pascal)
            "pa" => ConversionRatio::Linear(dec!(1)),
            "hpa" => ConversionRatio::Linear(dec!(100)),
            "kpa" => ConversionRatio::Linear(dec!(1000)),
            "bar" => ConversionRatio::Linear(dec!(100000)),
            "mbar" => ConversionRatio::Linear(dec!(100)),
            "psi" => ConversionRatio::Linear(dec!(6894.75729317)),
            "atm" => ConversionRatio::Linear(dec!(101325)),
            "torr" => ConversionRatio::Linear(dec!(133.32236842)),

            // Angles (base: Radian)
            "rad" => ConversionRatio::AngleFactor(1.0),
            "deg" => ConversionRatio::AngleFactor(std::f64::consts::PI / 180.0),
            "grad" => ConversionRatio::AngleFactor(std::f64::consts::PI / 200.0),
            "arcmin" => ConversionRatio::AngleFactor(std::f64::consts::PI / (180.0 * 60.0)),
            "arcsec" => ConversionRatio::AngleFactor(std::f64::consts::PI / (180.0 * 3600.0)),

            _ => ConversionRatio::Linear(dec!(1)),
        }
    }
}

pub static UNITS: &[UnitDefinition] = &[
    // Length
    UnitDefinition {
        id: "mm",
        name: "Millimeters",
        symbol: "mm",
        category: UnitCategory::Length,
        aliases: &["mm", "millimeter", "millimeters", "millimetre", "millimetres"],
    },
    UnitDefinition {
        id: "cm",
        name: "Centimeters",
        symbol: "cm",
        category: UnitCategory::Length,
        aliases: &["cm", "centimeter", "centimeters", "centimetre", "centimetres"],
    },
    UnitDefinition {
        id: "m",
        name: "Meters",
        symbol: "m",
        category: UnitCategory::Length,
        aliases: &["m", "meter", "meters", "metre", "metres"],
    },
    UnitDefinition {
        id: "km",
        name: "Kilometers",
        symbol: "km",
        category: UnitCategory::Length,
        aliases: &["km", "kilometer", "kilometers", "kilometre", "kilometres"],
    },
    UnitDefinition {
        id: "in",
        name: "Inches",
        symbol: "in",
        category: UnitCategory::Length,
        aliases: &["in", "inch", "inches", "\""],
    },
    UnitDefinition {
        id: "ft",
        name: "Feet",
        symbol: "ft",
        category: UnitCategory::Length,
        aliases: &["ft", "foot", "feet", "'"],
    },
    UnitDefinition { id: "yd", name: "Yards", symbol: "yd", category: UnitCategory::Length, aliases: &["yd", "yard", "yards"] },
    UnitDefinition { id: "mi", name: "Miles", symbol: "mi", category: UnitCategory::Length, aliases: &["mi", "mile", "miles"] },
    UnitDefinition {
        id: "nmi",
        name: "Nautical Miles",
        symbol: "NM",
        category: UnitCategory::Length,
        aliases: &["nm", "nmi", "nauticalmile", "nauticalmiles", "nautical miles"],
    },
    // Mass
    UnitDefinition {
        id: "mg",
        name: "Milligrams",
        symbol: "mg",
        category: UnitCategory::Mass,
        aliases: &["mg", "milligram", "milligrams"],
    },
    UnitDefinition { id: "g", name: "Grams", symbol: "g", category: UnitCategory::Mass, aliases: &["g", "gram", "grams"] },
    UnitDefinition {
        id: "kg",
        name: "Kilograms",
        symbol: "kg",
        category: UnitCategory::Mass,
        aliases: &["kg", "kilogram", "kilograms", "kilo", "kilos"],
    },
    UnitDefinition {
        id: "t",
        name: "Metric Tons",
        symbol: "t",
        category: UnitCategory::Mass,
        aliases: &["t", "ton", "tons", "tonne", "tonnes"],
    },
    UnitDefinition { id: "oz", name: "Ounces", symbol: "oz", category: UnitCategory::Mass, aliases: &["oz", "ounce", "ounces"] },
    UnitDefinition {
        id: "lb",
        name: "Pounds",
        symbol: "lb",
        category: UnitCategory::Mass,
        aliases: &["lb", "lbs", "pound", "pounds"],
    },
    UnitDefinition { id: "st", name: "Stone", symbol: "st", category: UnitCategory::Mass, aliases: &["st", "stone", "stones"] },
    // Temperature
    UnitDefinition {
        id: "degC",
        name: "Celsius",
        symbol: "°C",
        category: UnitCategory::Temperature,
        aliases: &["c", "°c", "celsius", "centigrade"],
    },
    UnitDefinition {
        id: "degF",
        name: "Fahrenheit",
        symbol: "°F",
        category: UnitCategory::Temperature,
        aliases: &["f", "°f", "fahrenheit"],
    },
    UnitDefinition { id: "K", name: "Kelvin", symbol: "K", category: UnitCategory::Temperature, aliases: &["k", "kelvin"] },
    // Area
    UnitDefinition {
        id: "sqmm",
        name: "Square Millimeters",
        symbol: "mm²",
        category: UnitCategory::Area,
        aliases: &["mm2", "mm²", "sqmm", "square millimeters"],
    },
    UnitDefinition {
        id: "sqcm",
        name: "Square Centimeters",
        symbol: "cm²",
        category: UnitCategory::Area,
        aliases: &["cm2", "cm²", "sqcm", "square centimeters"],
    },
    UnitDefinition {
        id: "sqm",
        name: "Square Meters",
        symbol: "m²",
        category: UnitCategory::Area,
        aliases: &["m2", "m²", "sqm", "square meters"],
    },
    UnitDefinition {
        id: "sqkm",
        name: "Square Kilometers",
        symbol: "km²",
        category: UnitCategory::Area,
        aliases: &["km2", "km²", "sqkm", "square kilometers"],
    },
    UnitDefinition {
        id: "sqin",
        name: "Square Inches",
        symbol: "sq in",
        category: UnitCategory::Area,
        aliases: &["sqin", "sq in", "in2", "in²", "square inches"],
    },
    UnitDefinition {
        id: "sqft",
        name: "Square Feet",
        symbol: "sq ft",
        category: UnitCategory::Area,
        aliases: &["sqft", "sq ft", "ft2", "ft²", "square feet"],
    },
    UnitDefinition {
        id: "sqyd",
        name: "Square Yards",
        symbol: "sq yd",
        category: UnitCategory::Area,
        aliases: &["sqyd", "sq yd", "yd2", "yd²", "square yards"],
    },
    UnitDefinition { id: "ac", name: "Acres", symbol: "ac", category: UnitCategory::Area, aliases: &["ac", "acre", "acres"] },
    UnitDefinition {
        id: "ha",
        name: "Hectares",
        symbol: "ha",
        category: UnitCategory::Area,
        aliases: &["ha", "hectare", "hectares"],
    },
    // Volume
    UnitDefinition {
        id: "ml",
        name: "Milliliters",
        symbol: "mL",
        category: UnitCategory::Volume,
        aliases: &["ml", "milliliter", "milliliters", "millilitre", "millilitres"],
    },
    UnitDefinition {
        id: "l",
        name: "Liters",
        symbol: "L",
        category: UnitCategory::Volume,
        aliases: &["l", "liter", "liters", "litre", "litres"],
    },
    UnitDefinition {
        id: "cum",
        name: "Cubic Meters",
        symbol: "m³",
        category: UnitCategory::Volume,
        aliases: &["m3", "m³", "cum", "cubic meters"],
    },
    UnitDefinition {
        id: "tsp",
        name: "Teaspoons",
        symbol: "tsp",
        category: UnitCategory::Volume,
        aliases: &["tsp", "teaspoon", "teaspoons"],
    },
    UnitDefinition {
        id: "tbsp",
        name: "Tablespoons",
        symbol: "tbsp",
        category: UnitCategory::Volume,
        aliases: &["tbsp", "tablespoon", "tablespoons"],
    },
    UnitDefinition {
        id: "floz",
        name: "Fluid Ounces",
        symbol: "fl oz",
        category: UnitCategory::Volume,
        aliases: &["floz", "fl oz", "fluid ounce", "fluid ounces"],
    },
    UnitDefinition { id: "cup", name: "Cups", symbol: "cup", category: UnitCategory::Volume, aliases: &["cup", "cups"] },
    UnitDefinition { id: "pt", name: "Pints", symbol: "pt", category: UnitCategory::Volume, aliases: &["pt", "pint", "pints"] },
    UnitDefinition {
        id: "qt",
        name: "Quarts",
        symbol: "qt",
        category: UnitCategory::Volume,
        aliases: &["qt", "quart", "quarts"],
    },
    UnitDefinition {
        id: "gal",
        name: "Gallons",
        symbol: "gal",
        category: UnitCategory::Volume,
        aliases: &["gal", "gallon", "gallons"],
    },
    UnitDefinition {
        id: "cuft",
        name: "Cubic Feet",
        symbol: "cu ft",
        category: UnitCategory::Volume,
        aliases: &["cuft", "cu ft", "ft3", "ft³", "cubic feet"],
    },
    UnitDefinition {
        id: "cuin",
        name: "Cubic Inches",
        symbol: "cu in",
        category: UnitCategory::Volume,
        aliases: &["cuin", "cu in", "in3", "in²", "cubic inches"],
    },
    // Speed
    UnitDefinition {
        id: "mps",
        name: "Meters per second",
        symbol: "m/s",
        category: UnitCategory::Speed,
        aliases: &["m/s", "mps", "meter per second", "meters per second"],
    },
    UnitDefinition {
        id: "kmh",
        name: "Kilometers per hour",
        symbol: "km/h",
        category: UnitCategory::Speed,
        aliases: &["km/h", "kmh", "kph", "kilometer per hour", "kilometers per hour"],
    },
    UnitDefinition {
        id: "mph",
        name: "Miles per hour",
        symbol: "mph",
        category: UnitCategory::Speed,
        aliases: &["mph", "mile per hour", "miles per hour"],
    },
    UnitDefinition {
        id: "knot",
        name: "Knots",
        symbol: "kn",
        category: UnitCategory::Speed,
        aliases: &["kn", "knot", "knots", "kt", "kts"],
    },
    UnitDefinition {
        id: "fps",
        name: "Feet per second",
        symbol: "ft/s",
        category: UnitCategory::Speed,
        aliases: &["ft/s", "fps", "foot per second", "feet per second"],
    },
    // Time
    UnitDefinition {
        id: "ns",
        name: "Nanoseconds",
        symbol: "ns",
        category: UnitCategory::Time,
        aliases: &["ns", "nanosecond", "nanoseconds"],
    },
    UnitDefinition {
        id: "us",
        name: "Microseconds",
        symbol: "μs",
        category: UnitCategory::Time,
        aliases: &["us", "μs", "microsecond", "microseconds"],
    },
    UnitDefinition {
        id: "ms",
        name: "Milliseconds",
        symbol: "ms",
        category: UnitCategory::Time,
        aliases: &["ms", "millisecond", "milliseconds"],
    },
    UnitDefinition {
        id: "s",
        name: "Seconds",
        symbol: "s",
        category: UnitCategory::Time,
        aliases: &["s", "sec", "secs", "second", "seconds"],
    },
    UnitDefinition {
        id: "min",
        name: "Minutes",
        symbol: "min",
        category: UnitCategory::Time,
        aliases: &["min", "mins", "minute", "minutes"],
    },
    UnitDefinition {
        id: "h",
        name: "Hours",
        symbol: "h",
        category: UnitCategory::Time,
        aliases: &["h", "hr", "hrs", "hour", "hours"],
    },
    UnitDefinition { id: "d", name: "Days", symbol: "d", category: UnitCategory::Time, aliases: &["d", "day", "days"] },
    UnitDefinition {
        id: "wk",
        name: "Weeks",
        symbol: "wk",
        category: UnitCategory::Time,
        aliases: &["wk", "wks", "week", "weeks"],
    },
    UnitDefinition {
        id: "mo",
        name: "Months",
        symbol: "mo",
        category: UnitCategory::Time,
        aliases: &["mo", "mos", "month", "months"],
    },
    UnitDefinition {
        id: "yr",
        name: "Years",
        symbol: "yr",
        category: UnitCategory::Time,
        aliases: &["yr", "yrs", "year", "years"],
    },
    // Data
    UnitDefinition { id: "bit", name: "Bits", symbol: "b", category: UnitCategory::Data, aliases: &["b", "bit", "bits"] },
    UnitDefinition { id: "byte", name: "Bytes", symbol: "B", category: UnitCategory::Data, aliases: &["byte", "bytes"] },
    UnitDefinition {
        id: "kb",
        name: "Kilobytes (decimal)",
        symbol: "KB",
        category: UnitCategory::Data,
        aliases: &["kb", "kilobyte", "kilobytes"],
    },
    UnitDefinition {
        id: "mb",
        name: "Megabytes (decimal)",
        symbol: "MB",
        category: UnitCategory::Data,
        aliases: &["mb", "megabyte", "megabytes"],
    },
    UnitDefinition {
        id: "gb",
        name: "Gigabytes (decimal)",
        symbol: "GB",
        category: UnitCategory::Data,
        aliases: &["gb", "gigabyte", "gigabytes"],
    },
    UnitDefinition {
        id: "tb",
        name: "Terabytes (decimal)",
        symbol: "TB",
        category: UnitCategory::Data,
        aliases: &["tb", "terabyte", "terabytes"],
    },
    UnitDefinition {
        id: "pb",
        name: "Petabytes (decimal)",
        symbol: "PB",
        category: UnitCategory::Data,
        aliases: &["pb", "petabyte", "petabytes"],
    },
    UnitDefinition {
        id: "kib",
        name: "Kibibytes (binary)",
        symbol: "KiB",
        category: UnitCategory::Data,
        aliases: &["kib", "kibibyte", "kibibytes"],
    },
    UnitDefinition {
        id: "mib",
        name: "Mebibytes (binary)",
        symbol: "MiB",
        category: UnitCategory::Data,
        aliases: &["mib", "mebibyte", "mebibytes"],
    },
    UnitDefinition {
        id: "gib",
        name: "Gibibytes (binary)",
        symbol: "GiB",
        category: UnitCategory::Data,
        aliases: &["gib", "gibibyte", "gibibytes"],
    },
    UnitDefinition {
        id: "tib",
        name: "Tebibytes (binary)",
        symbol: "TiB",
        category: UnitCategory::Data,
        aliases: &["tib", "tebibyte", "tebibytes"],
    },
    // Energy
    UnitDefinition { id: "j", name: "Joules", symbol: "J", category: UnitCategory::Energy, aliases: &["j", "joule", "joules"] },
    UnitDefinition {
        id: "kj",
        name: "Kilojoules",
        symbol: "kJ",
        category: UnitCategory::Energy,
        aliases: &["kj", "kilojoule", "kilojoules"],
    },
    UnitDefinition {
        id: "cal",
        name: "Calories",
        symbol: "cal",
        category: UnitCategory::Energy,
        aliases: &["cal", "calorie", "calories"],
    },
    UnitDefinition {
        id: "kcal",
        name: "Kilocalories",
        symbol: "kcal",
        category: UnitCategory::Energy,
        aliases: &["kcal", "kilocalorie", "kilocalories"],
    },
    UnitDefinition {
        id: "wh",
        name: "Watt-hours",
        symbol: "Wh",
        category: UnitCategory::Energy,
        aliases: &["wh", "watt-hour", "watt-hours"],
    },
    UnitDefinition {
        id: "kwh",
        name: "Kilowatt-hours",
        symbol: "kWh",
        category: UnitCategory::Energy,
        aliases: &["kwh", "kilowatt-hour", "kilowatt-hours"],
    },
    UnitDefinition {
        id: "btu",
        name: "British Thermal Units",
        symbol: "BTU",
        category: UnitCategory::Energy,
        aliases: &["btu", "btus"],
    },
    // Power
    UnitDefinition { id: "w", name: "Watts", symbol: "W", category: UnitCategory::Power, aliases: &["w", "watt", "watts"] },
    UnitDefinition {
        id: "kw",
        name: "Kilowatts",
        symbol: "kW",
        category: UnitCategory::Power,
        aliases: &["kw", "kilowatt", "kilowatts"],
    },
    UnitDefinition {
        id: "hpm",
        name: "Horsepower (metric)",
        symbol: "hp (M)",
        category: UnitCategory::Power,
        aliases: &["ps", "metric hp", "metric horsepower", "hp(m)"],
    },
    UnitDefinition {
        id: "hpi",
        name: "Horsepower (mechanical)",
        symbol: "hp",
        category: UnitCategory::Power,
        aliases: &["hp", "horsepower", "mechanical hp", "hp(i)"],
    },
    // Pressure
    UnitDefinition {
        id: "pa",
        name: "Pascals",
        symbol: "Pa",
        category: UnitCategory::Pressure,
        aliases: &["pa", "pascal", "pascals"],
    },
    UnitDefinition {
        id: "hpa",
        name: "Hectopascals",
        symbol: "hPa",
        category: UnitCategory::Pressure,
        aliases: &["hpa", "hectopascal", "hectopascals"],
    },
    UnitDefinition {
        id: "kpa",
        name: "Kilopascals",
        symbol: "kPa",
        category: UnitCategory::Pressure,
        aliases: &["kpa", "kilopascal", "kilopascals"],
    },
    UnitDefinition { id: "bar", name: "Bar", symbol: "bar", category: UnitCategory::Pressure, aliases: &["bar", "bars"] },
    UnitDefinition {
        id: "mbar",
        name: "Millibar",
        symbol: "mbar",
        category: UnitCategory::Pressure,
        aliases: &["mbar", "mbars", "millibar", "millibars"],
    },
    UnitDefinition {
        id: "psi",
        name: "Pounds per square inch",
        symbol: "PSI",
        category: UnitCategory::Pressure,
        aliases: &["psi", "pounds per square inch"],
    },
    UnitDefinition {
        id: "atm",
        name: "Standard Atmospheres",
        symbol: "atm",
        category: UnitCategory::Pressure,
        aliases: &["atm", "atmosphere", "atmospheres"],
    },
    UnitDefinition {
        id: "torr",
        name: "Torr (mmHg)",
        symbol: "Torr",
        category: UnitCategory::Pressure,
        aliases: &["torr", "mmhg"],
    },
    // Angle
    UnitDefinition {
        id: "deg",
        name: "Degrees",
        symbol: "°",
        category: UnitCategory::Angle,
        aliases: &["deg", "degree", "degrees", "°"],
    },
    UnitDefinition {
        id: "rad",
        name: "Radians",
        symbol: "rad",
        category: UnitCategory::Angle,
        aliases: &["rad", "radian", "radians"],
    },
    UnitDefinition {
        id: "grad",
        name: "Gradians",
        symbol: "grad",
        category: UnitCategory::Angle,
        aliases: &["grad", "grads", "gradian", "gradians", "gon"],
    },
    UnitDefinition {
        id: "arcmin",
        name: "Arcminutes",
        symbol: "arcmin",
        category: UnitCategory::Angle,
        aliases: &["arcmin", "arcminute", "arcminutes", "'"],
    },
    UnitDefinition {
        id: "arcsec",
        name: "Arcseconds",
        symbol: "arcsec",
        category: UnitCategory::Angle,
        aliases: &["arcsec", "arcsecond", "arcseconds", "\""],
    },
];

pub fn find_unit(alias: &str) -> Option<&'static UnitDefinition> {
    let lower = alias.trim().to_lowercase();
    for unit in UNITS {
        if unit.id.eq_ignore_ascii_case(&lower)
            || unit.symbol.eq_ignore_ascii_case(&lower)
            || unit.name.eq_ignore_ascii_case(&lower)
        {
            return Some(unit);
        }
        for &a in unit.aliases {
            if a.eq_ignore_ascii_case(&lower) {
                return Some(unit);
            }
        }
    }
    None
}

pub fn units_for_category(category: UnitCategory) -> Vec<&'static UnitDefinition> {
    UNITS.iter().filter(|u| u.category == category).collect()
}

pub fn convert(val: Decimal, from_str: &str, to_str: &str) -> Result<Decimal, CalcError> {
    let from_unit = find_unit(from_str).ok_or_else(|| CalcError::SyntaxError(format!("Unknown unit '{from_str}'")))?;
    let to_unit = find_unit(to_str).ok_or_else(|| CalcError::SyntaxError(format!("Unknown unit '{to_str}'")))?;

    if from_unit.category != to_unit.category {
        return Err(CalcError::SyntaxError(format!(
            "Cannot convert between {} ({}) and {} ({})",
            from_unit.name,
            from_unit.category.display_name(),
            to_unit.name,
            to_unit.category.display_name()
        )));
    }

    if from_unit.id == to_unit.id {
        return Ok(val);
    }

    match (from_unit.ratio(), to_unit.ratio()) {
        (ConversionRatio::Linear(from_scale), ConversionRatio::Linear(to_scale)) => {
            let base_val = val.checked_mul(from_scale).ok_or(CalcError::Overflow)?;
            base_val.checked_div(to_scale).ok_or(CalcError::Overflow)
        }
        (
            ConversionRatio::TemperatureAffine { scale: f_scale, offset: f_offset },
            ConversionRatio::TemperatureAffine { scale: t_scale, offset: t_offset },
        ) => {
            // Base is Kelvin:
            // For C: scale=1, offset=273.15 -> K = (val + 273.15) * 1
            // For F: scale=5/9, offset=459.67 -> K = (val + 459.67) * 5/9
            // For K: scale=1, offset=0 -> K = (val + 0) * 1
            let kelvin = val.checked_add(f_offset).and_then(|v| v.checked_mul(f_scale)).ok_or(CalcError::Overflow)?;

            // Convert from Kelvin to target:
            // kelvin = (target + t_offset) * t_scale
            // target = (kelvin / t_scale) - t_offset
            let target = kelvin.checked_div(t_scale).and_then(|v| v.checked_sub(t_offset)).ok_or(CalcError::Overflow)?;

            // Clean float artifacts (e.g. 32 F to C is exactly 0 C, 212 F to C is 100 C)
            if (target - target.round()).abs() < dec!(0.000001) { Ok(target.round()) } else { Ok(target.round_dp(6)) }
        }
        (ConversionRatio::AngleFactor(from_factor), ConversionRatio::AngleFactor(to_factor)) => {
            let f = val.to_string().parse::<f64>().map_err(|_| CalcError::Overflow)?;
            let rad = f * from_factor;
            let target_f = rad / to_factor;
            let target_dec = Decimal::from_f64_retain(target_f).ok_or(CalcError::Overflow)?;
            if (target_dec - target_dec.round()).abs() < dec!(0.000001) {
                Ok(target_dec.round())
            } else {
                Ok(target_dec.round_dp(6))
            }
        }
        _ => Err(CalcError::SyntaxError("Incompatible conversion types".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_conversions() {
        assert_eq!(convert(dec!(1), "km", "m").unwrap(), dec!(1000));
        assert_eq!(convert(dec!(1), "mi", "km").unwrap(), dec!(1.609344));
        assert_eq!(convert(dec!(12), "in", "ft").unwrap(), dec!(1));
        assert_eq!(convert(dec!(1), "m", "cm").unwrap(), dec!(100));
    }

    #[test]
    fn test_temperature_affine() {
        assert_eq!(convert(dec!(32), "degF", "degC").unwrap(), dec!(0));
        assert_eq!(convert(dec!(212), "degF", "degC").unwrap(), dec!(100));
        assert_eq!(convert(dec!(0), "degC", "degF").unwrap(), dec!(32));
        assert_eq!(convert(dec!(100), "degC", "degF").unwrap(), dec!(212));
        assert_eq!(convert(dec!(0), "degC", "K").unwrap(), dec!(273.15));
    }

    #[test]
    fn test_data_conversions() {
        assert_eq!(convert(dec!(1), "MB", "KB").unwrap(), dec!(1000));
        assert_eq!(convert(dec!(1), "MiB", "KiB").unwrap(), dec!(1024));
        assert_eq!(convert(dec!(8), "bit", "byte").unwrap(), dec!(1));
        assert_eq!(convert(dec!(1), "GB", "MB").unwrap(), dec!(1000));
    }

    #[test]
    fn test_time_conversions() {
        assert_eq!(convert(dec!(1), "h", "min").unwrap(), dec!(60));
        assert_eq!(convert(dec!(2.5), "h", "s").unwrap(), dec!(9000));
        assert_eq!(convert(dec!(1), "d", "h").unwrap(), dec!(24));
    }

    #[test]
    fn test_category_mismatch_error() {
        assert!(convert(dec!(10), "m", "kg").is_err());
        assert!(convert(dec!(5), "hours", "meters").is_err());
    }
}
