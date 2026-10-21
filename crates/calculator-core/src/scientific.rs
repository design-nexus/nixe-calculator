use crate::error::CalcError;
use std::f64::consts::{E, PI, TAU};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AngleMode {
    #[default]
    Degrees,
    Radians,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SciFunc {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Ln,
    Log10,
    Log2,
    Exp,
    Sqrt,
    Cbrt,
    Sqr,
    Fact,
    Abs,
}

pub fn snap_float(val: f64) -> f64 {
    if val.abs() < 1e-12 {
        return 0.0;
    }
    if (val - val.round()).abs() < 1e-12 {
        return val.round();
    }
    val
}

pub fn eval_sci_func(func: SciFunc, x: f64, angle_mode: AngleMode) -> Result<f64, CalcError> {
    let raw = match func {
        SciFunc::Sin => {
            if angle_mode == AngleMode::Degrees {
                let norm = (x % 360.0 + 360.0) % 360.0;
                if norm.abs() < 1e-10 || (norm - 180.0).abs() < 1e-10 || (norm - 360.0).abs() < 1e-10 {
                    return Ok(0.0);
                }
                if (norm - 90.0).abs() < 1e-10 {
                    return Ok(1.0);
                }
                if (norm - 270.0).abs() < 1e-10 {
                    return Ok(-1.0);
                }
                if (norm - 30.0).abs() < 1e-10 || (norm - 150.0).abs() < 1e-10 {
                    return Ok(0.5);
                }
                if (norm - 210.0).abs() < 1e-10 || (norm - 330.0).abs() < 1e-10 {
                    return Ok(-0.5);
                }
            }
            let rad = to_radians(x, angle_mode);
            Ok(rad.sin())
        }
        SciFunc::Cos => {
            if angle_mode == AngleMode::Degrees {
                let norm = (x % 360.0 + 360.0) % 360.0;
                if (norm - 90.0).abs() < 1e-10 || (norm - 270.0).abs() < 1e-10 {
                    return Ok(0.0);
                }
                if norm.abs() < 1e-10 || (norm - 360.0).abs() < 1e-10 {
                    return Ok(1.0);
                }
                if (norm - 180.0).abs() < 1e-10 {
                    return Ok(-1.0);
                }
                if (norm - 60.0).abs() < 1e-10 || (norm - 300.0).abs() < 1e-10 {
                    return Ok(0.5);
                }
                if (norm - 120.0).abs() < 1e-10 || (norm - 240.0).abs() < 1e-10 {
                    return Ok(-0.5);
                }
            }
            let rad = to_radians(x, angle_mode);
            Ok(rad.cos())
        }
        SciFunc::Tan => {
            if angle_mode == AngleMode::Degrees {
                let norm = (x % 180.0 + 180.0) % 180.0;
                if (norm - 90.0).abs() < 1e-10 {
                    return Err(CalcError::InvalidDomain("Tangent undefined at 90° + k·180°".into()));
                }
                if norm.abs() < 1e-10 || (norm - 180.0).abs() < 1e-10 {
                    return Ok(0.0);
                }
                if (norm - 45.0).abs() < 1e-10 {
                    return Ok(1.0);
                }
                if (norm - 135.0).abs() < 1e-10 {
                    return Ok(-1.0);
                }
            }
            let rad = to_radians(x, angle_mode);
            let res = rad.tan();
            if res.is_infinite() { Err(CalcError::InvalidDomain("Tangent undefined".into())) } else { Ok(res) }
        }
        SciFunc::Asin => {
            if !(-1.0..=1.0).contains(&x) {
                Err(CalcError::InvalidDomain("Arcsine domain is [-1, 1]".into()))
            } else {
                from_radians(x.asin(), angle_mode)
            }
        }
        SciFunc::Acos => {
            if !(-1.0..=1.0).contains(&x) {
                Err(CalcError::InvalidDomain("Arccosine domain is [-1, 1]".into()))
            } else {
                from_radians(x.acos(), angle_mode)
            }
        }
        SciFunc::Atan => from_radians(x.atan(), angle_mode),
        SciFunc::Sinh => Ok(x.sinh()),
        SciFunc::Cosh => Ok(x.cosh()),
        SciFunc::Tanh => Ok(x.tanh()),
        SciFunc::Ln => {
            if x <= 0.0 {
                Err(CalcError::InvalidDomain("Logarithm domain is x > 0".into()))
            } else {
                Ok(x.ln())
            }
        }
        SciFunc::Log10 => {
            if x <= 0.0 {
                Err(CalcError::InvalidDomain("Logarithm domain is x > 0".into()))
            } else {
                Ok(x.log10())
            }
        }
        SciFunc::Log2 => {
            if x <= 0.0 {
                Err(CalcError::InvalidDomain("Logarithm domain is x > 0".into()))
            } else {
                Ok(x.log2())
            }
        }
        SciFunc::Exp => {
            let res = x.exp();
            if res.is_infinite() { Err(CalcError::Overflow) } else { Ok(res) }
        }
        SciFunc::Sqrt => {
            if x < 0.0 {
                Err(CalcError::InvalidDomain("Square root domain is x ≥ 0".into()))
            } else {
                Ok(x.sqrt())
            }
        }
        SciFunc::Sqr => Ok(x * x),
        SciFunc::Cbrt => Ok(x.cbrt()),
        SciFunc::Fact => factorial(x),
        SciFunc::Abs => Ok(x.abs()),
    }?;

    Ok(snap_float(raw))
}

pub fn factorial(x: f64) -> Result<f64, CalcError> {
    if x < 0.0 {
        return Err(CalcError::InvalidDomain("Factorial domain is n ≥ 0".into()));
    }
    if x > 170.0 {
        return Err(CalcError::Overflow);
    }
    // If integer, compute exact product
    if (x - x.round()).abs() < 1e-10 {
        let n = x.round() as u64;
        let mut acc = 1.0;
        for i in 2..=n {
            acc *= i as f64;
        }
        Ok(acc)
    } else {
        // Gamma function approximation for non-integers
        Err(CalcError::InvalidDomain("Factorial requires an integer".into()))
    }
}

pub fn to_radians(val: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => val,
        AngleMode::Degrees => val * (PI / 180.0),
    }
}

pub fn from_radians(rad: f64, mode: AngleMode) -> Result<f64, CalcError> {
    let res = match mode {
        AngleMode::Radians => rad,
        AngleMode::Degrees => rad * (180.0 / PI),
    };
    if res.is_nan() || res.is_infinite() { Err(CalcError::Overflow) } else { Ok(res) }
}

pub const CONST_PI: f64 = PI;
pub const CONST_E: f64 = E;
pub const CONST_TAU: f64 = TAU;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trig_degrees_and_radians() {
        // Degrees
        let sin90 = eval_sci_func(SciFunc::Sin, 90.0, AngleMode::Degrees).unwrap();
        assert!((sin90 - 1.0).abs() < 1e-10);

        let cos0 = eval_sci_func(SciFunc::Cos, 0.0, AngleMode::Degrees).unwrap();
        assert!((cos0 - 1.0).abs() < 1e-10);

        let cos90 = eval_sci_func(SciFunc::Cos, 90.0, AngleMode::Degrees).unwrap();
        assert!(cos90.abs() < 1e-10);

        let tan45 = eval_sci_func(SciFunc::Tan, 45.0, AngleMode::Degrees).unwrap();
        assert!((tan45 - 1.0).abs() < 1e-10);

        // Tan 90 undefined
        assert!(eval_sci_func(SciFunc::Tan, 90.0, AngleMode::Degrees).is_err());

        // Radians
        let sin_pi_2 = eval_sci_func(SciFunc::Sin, PI / 2.0, AngleMode::Radians).unwrap();
        assert!((sin_pi_2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_advanced_functions() {
        assert_eq!(eval_sci_func(SciFunc::Fact, 5.0, AngleMode::Degrees).unwrap(), 120.0);
        assert_eq!(eval_sci_func(SciFunc::Fact, 0.0, AngleMode::Degrees).unwrap(), 1.0);
        assert!(eval_sci_func(SciFunc::Fact, -1.0, AngleMode::Degrees).is_err());

        let ln_e = eval_sci_func(SciFunc::Ln, E, AngleMode::Radians).unwrap();
        assert!((ln_e - 1.0).abs() < 1e-10);

        let log10_100 = eval_sci_func(SciFunc::Log10, 100.0, AngleMode::Radians).unwrap();
        assert!((log10_100 - 2.0).abs() < 1e-10);

        let cbrt27 = eval_sci_func(SciFunc::Cbrt, 27.0, AngleMode::Radians).unwrap();
        assert!((cbrt27 - 3.0).abs() < 1e-10);
    }
}
