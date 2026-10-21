use crate::ast::{BinaryOp, Constant, Expr, UnaryOp};
use crate::error::CalcError;
use crate::parser::Parser;
use crate::scientific::{AngleMode, CONST_E, CONST_PI, CONST_TAU, eval_sci_func};
use crate::token::tokenize;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

pub fn evaluate_str(input: &str) -> Result<Decimal, CalcError> {
    evaluate_str_with_mode(input, AngleMode::Degrees)
}

pub fn evaluate_str_with_mode(input: &str, angle_mode: AngleMode) -> Result<Decimal, CalcError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    evaluate(&ast, angle_mode)
}

pub fn preview_str(input: &str) -> Option<Decimal> {
    preview_str_with_mode(input, AngleMode::Degrees)
}

pub fn preview_str_with_mode(input: &str, angle_mode: AngleMode) -> Option<Decimal> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(res) = evaluate_str_with_mode(trimmed, angle_mode) {
        return Some(res);
    }
    let candidate = trimmed.trim_end_matches(['+', '-', '*', '/', '^', '(', ' ', '&', '|', '~']);
    if !candidate.is_empty()
        && candidate != trimmed
        && let Ok(res) = evaluate_str_with_mode(candidate, angle_mode)
    {
        return Some(res);
    }
    None
}

pub fn evaluate(expr: &Expr, angle_mode: AngleMode) -> Result<Decimal, CalcError> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Variable(name) => Err(CalcError::SyntaxError(format!("Undefined variable '{name}'"))),
        Expr::Const(c) => match c {
            Constant::Pi => Decimal::from_f64_retain(CONST_PI).ok_or(CalcError::Overflow),
            Constant::E => Decimal::from_f64_retain(CONST_E).ok_or(CalcError::Overflow),
            Constant::Tau => Decimal::from_f64_retain(CONST_TAU).ok_or(CalcError::Overflow),
        },
        Expr::Unary { op, expr } => {
            let val = evaluate(expr, angle_mode)?;
            match op {
                UnaryOp::Neg => Ok(-val),
                UnaryOp::Pos => Ok(val),
                UnaryOp::BitNot => {
                    let int_val = val.to_i64().ok_or(CalcError::Overflow)?;
                    Ok(Decimal::from(!int_val))
                }
            }
        }
        Expr::Percent(inner) => {
            let val = evaluate(inner, angle_mode)?;
            val.checked_div(Decimal::from(100)).ok_or(CalcError::DivisionByZero)
        }
        Expr::Factorial(inner) => {
            let val = evaluate(inner, angle_mode)?;
            let f = val.to_f64().ok_or(CalcError::Overflow)?;
            let res = eval_sci_func(crate::scientific::SciFunc::Fact, f, angle_mode)?;
            Decimal::from_f64_retain(res).ok_or(CalcError::Overflow)
        }
        Expr::Function { func, arg } => {
            let val = evaluate(arg, angle_mode)?;
            // Check for exact decimal sqrt if func is Sqrt
            if *func == crate::scientific::SciFunc::Sqrt {
                if val < Decimal::ZERO {
                    return Err(CalcError::InvalidDomain("Cannot take square root of negative number".into()));
                }
                return val.sqrt().ok_or(CalcError::Overflow);
            }
            if *func == crate::scientific::SciFunc::Sqr {
                return val.checked_mul(val).ok_or(CalcError::Overflow);
            }
            if *func == crate::scientific::SciFunc::Abs {
                return Ok(val.abs());
            }

            let f = val.to_f64().ok_or(CalcError::Overflow)?;
            let res = eval_sci_func(*func, f, angle_mode)?;
            Decimal::from_f64_retain(res).ok_or(CalcError::Overflow)
        }
        Expr::Binary { op, left, right } => {
            // Check for special percentage rules with left operand
            if let Expr::Percent(pct_inner) = right.as_ref() {
                let left_val = evaluate(left, angle_mode)?;
                let pct_num = evaluate(pct_inner, angle_mode)?;
                match op {
                    BinaryOp::Add => {
                        let delta = left_val
                            .checked_mul(pct_num)
                            .and_then(|v| v.checked_div(Decimal::from(100)))
                            .ok_or(CalcError::Overflow)?;
                        left_val.checked_add(delta).ok_or(CalcError::Overflow)
                    }
                    BinaryOp::Sub => {
                        let delta = left_val
                            .checked_mul(pct_num)
                            .and_then(|v| v.checked_div(Decimal::from(100)))
                            .ok_or(CalcError::Overflow)?;
                        left_val.checked_sub(delta).ok_or(CalcError::Overflow)
                    }
                    BinaryOp::Mul => {
                        let frac = pct_num.checked_div(Decimal::from(100)).ok_or(CalcError::DivisionByZero)?;
                        left_val.checked_mul(frac).ok_or(CalcError::Overflow)
                    }
                    BinaryOp::Div => {
                        let frac = pct_num.checked_div(Decimal::from(100)).ok_or(CalcError::DivisionByZero)?;
                        if frac.is_zero() {
                            return Err(CalcError::DivisionByZero);
                        }
                        left_val.checked_div(frac).ok_or(CalcError::Overflow)
                    }
                    _ => {
                        let right_val = pct_num.checked_div(Decimal::from(100)).ok_or(CalcError::DivisionByZero)?;
                        apply_binary_op(*op, left_val, right_val)
                    }
                }
            } else {
                let left_val = evaluate(left, angle_mode)?;
                let right_val = evaluate(right, angle_mode)?;
                apply_binary_op(*op, left_val, right_val)
            }
        }
    }
}

fn apply_binary_op(op: BinaryOp, left: Decimal, right: Decimal) -> Result<Decimal, CalcError> {
    match op {
        BinaryOp::Add => left.checked_add(right).ok_or(CalcError::Overflow),
        BinaryOp::Sub => left.checked_sub(right).ok_or(CalcError::Overflow),
        BinaryOp::Mul => left.checked_mul(right).ok_or(CalcError::Overflow),
        BinaryOp::Div => {
            if right.is_zero() {
                Err(CalcError::DivisionByZero)
            } else {
                left.checked_div(right).ok_or(CalcError::Overflow)
            }
        }
        BinaryOp::Mod => {
            if right.is_zero() {
                Err(CalcError::DivisionByZero)
            } else {
                left.checked_rem(right).ok_or(CalcError::Overflow)
            }
        }
        BinaryOp::BitAnd => {
            let l = left.to_i64().ok_or(CalcError::Overflow)?;
            let r = right.to_i64().ok_or(CalcError::Overflow)?;
            Ok(Decimal::from(l & r))
        }
        BinaryOp::BitOr => {
            let l = left.to_i64().ok_or(CalcError::Overflow)?;
            let r = right.to_i64().ok_or(CalcError::Overflow)?;
            Ok(Decimal::from(l | r))
        }
        BinaryOp::BitXor => {
            let l = left.to_i64().ok_or(CalcError::Overflow)?;
            let r = right.to_i64().ok_or(CalcError::Overflow)?;
            Ok(Decimal::from(l ^ r))
        }
        BinaryOp::Shl => {
            let l = left.to_i64().ok_or(CalcError::Overflow)?;
            let r = right.to_u32().ok_or(CalcError::Overflow)?;
            Ok(Decimal::from(l.wrapping_shl(r)))
        }
        BinaryOp::Shr => {
            let l = left.to_i64().ok_or(CalcError::Overflow)?;
            let r = right.to_u32().ok_or(CalcError::Overflow)?;
            Ok(Decimal::from(l.wrapping_shr(r)))
        }
        BinaryOp::Pow => {
            if right.is_zero() {
                return Ok(Decimal::ONE);
            }
            if right.fract().is_zero() {
                let exp = right.to_i64().ok_or(CalcError::Overflow)?;
                if exp >= 0 {
                    left.checked_powi(exp).ok_or(CalcError::Overflow)
                } else {
                    let positive = left.checked_powi(-exp).ok_or(CalcError::Overflow)?;
                    if positive.is_zero() {
                        return Err(CalcError::DivisionByZero);
                    }
                    Decimal::ONE.checked_div(positive).ok_or(CalcError::Overflow)
                }
            } else {
                let l_f64 = left.to_f64().ok_or(CalcError::Overflow)?;
                let r_f64 = right.to_f64().ok_or(CalcError::Overflow)?;
                if l_f64 < 0.0 {
                    return Err(CalcError::InvalidDomain("Negative base with fractional exponent".into()));
                }
                let res_f64 = l_f64.powf(r_f64);
                if res_f64.is_infinite() || res_f64.is_nan() {
                    return Err(CalcError::Overflow);
                }
                Decimal::from_f64_retain(res_f64).ok_or(CalcError::Overflow)
            }
        }
    }
}

pub fn eval_f64(expr: &Expr, x: f64, angle_mode: AngleMode) -> Result<f64, CalcError> {
    match expr {
        Expr::Number(n) => n.to_f64().ok_or(CalcError::Overflow),
        Expr::Const(c) => match c {
            Constant::Pi => Ok(CONST_PI),
            Constant::E => Ok(CONST_E),
            Constant::Tau => Ok(CONST_TAU),
        },
        Expr::Variable(name) => {
            if name.eq_ignore_ascii_case("x")
                || name.eq_ignore_ascii_case("t")
                || name.eq_ignore_ascii_case("theta")
                || name.eq_ignore_ascii_case("θ")
            {
                Ok(x)
            } else {
                Err(CalcError::SyntaxError(format!("Undefined variable '{name}'")))
            }
        }
        Expr::Unary { op, expr } => {
            let val = eval_f64(expr, x, angle_mode)?;
            match op {
                UnaryOp::Neg => Ok(-val),
                UnaryOp::Pos => Ok(val),
                UnaryOp::BitNot => Ok(!(val as i64) as f64),
            }
        }
        Expr::Percent(inner) => {
            let val = eval_f64(inner, x, angle_mode)?;
            Ok(val / 100.0)
        }
        Expr::Factorial(inner) => {
            let val = eval_f64(inner, x, angle_mode)?;
            eval_sci_func(crate::scientific::SciFunc::Fact, val, angle_mode)
        }
        Expr::Function { func, arg } => {
            let val = eval_f64(arg, x, angle_mode)?;
            eval_sci_func(*func, val, angle_mode)
        }
        Expr::Binary { op, left, right } => {
            let l = eval_f64(left, x, angle_mode)?;
            let r = eval_f64(right, x, angle_mode)?;
            match op {
                BinaryOp::Add => Ok(l + r),
                BinaryOp::Sub => Ok(l - r),
                BinaryOp::Mul => Ok(l * r),
                BinaryOp::Div => {
                    if r.abs() < 1e-15 {
                        Err(CalcError::DivisionByZero)
                    } else {
                        Ok(l / r)
                    }
                }
                BinaryOp::Mod => {
                    if r.abs() < 1e-15 {
                        Err(CalcError::DivisionByZero)
                    } else {
                        Ok(l % r)
                    }
                }
                BinaryOp::Pow => {
                    let res = l.powf(r);
                    if res.is_nan() || res.is_infinite() {
                        Err(CalcError::InvalidDomain("Undefined power".into()))
                    } else {
                        Ok(res)
                    }
                }
                BinaryOp::BitAnd => Ok(((l as i64) & (r as i64)) as f64),
                BinaryOp::BitOr => Ok(((l as i64) | (r as i64)) as f64),
                BinaryOp::BitXor => Ok(((l as i64) ^ (r as i64)) as f64),
                BinaryOp::Shl => Ok(((l as i64).wrapping_shl(r as u32)) as f64),
                BinaryOp::Shr => Ok(((l as i64).wrapping_shr(r as u32)) as f64),
            }
        }
    }
}
