use crate::ast::{BinaryOp, Expr, Func, UnaryOp};
use crate::error::CalcError;
use crate::parser::Parser;
use crate::token::tokenize;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

pub fn evaluate_str(input: &str) -> Result<Decimal, CalcError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    evaluate(&ast)
}

/// Evaluates without throwing: returns Some(result) on success, None on error or empty.
pub fn preview_str(input: &str) -> Option<Decimal> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Attempt standard evaluation
    if let Ok(res) = evaluate_str(trimmed) {
        return Some(res);
    }
    // If user is still typing and ended with an operator or opening paren, try evaluating trimmed prefix
    let candidate = trimmed.trim_end_matches(['+', '-', '*', '/', '^', '(', ' ']);
    if !candidate.is_empty()
        && candidate != trimmed
        && let Ok(res) = evaluate_str(candidate)
    {
        return Some(res);
    }
    None
}

pub fn evaluate(expr: &Expr) -> Result<Decimal, CalcError> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Unary { op, expr } => {
            let val = evaluate(expr)?;
            match op {
                UnaryOp::Neg => Ok(-val),
                UnaryOp::Pos => Ok(val),
            }
        }
        Expr::Percent(inner) => {
            let val = evaluate(inner)?;
            val.checked_div(Decimal::from(100)).ok_or(CalcError::DivisionByZero)
        }
        Expr::Function { func, arg } => {
            let val = evaluate(arg)?;
            match func {
                Func::Sqrt => {
                    if val < Decimal::ZERO {
                        return Err(CalcError::InvalidDomain("Cannot take square root of negative number".into()));
                    }
                    val.sqrt().ok_or(CalcError::Overflow)
                }
                Func::Sqr => val.checked_mul(val).ok_or(CalcError::Overflow),
                Func::Recip => {
                    if val.is_zero() {
                        return Err(CalcError::DivisionByZero);
                    }
                    Decimal::ONE.checked_div(val).ok_or(CalcError::Overflow)
                }
                Func::Abs => Ok(val.abs()),
            }
        }
        Expr::Binary { op, left, right } => {
            // Check for special percentage rules with left operand
            if let Expr::Percent(pct_inner) = right.as_ref() {
                let left_val = evaluate(left)?;
                let pct_num = evaluate(pct_inner)?;
                match op {
                    BinaryOp::Add => {
                        // left + pct% -> left + (left * pct / 100)
                        let delta = left_val
                            .checked_mul(pct_num)
                            .and_then(|v| v.checked_div(Decimal::from(100)))
                            .ok_or(CalcError::Overflow)?;
                        left_val.checked_add(delta).ok_or(CalcError::Overflow)
                    }
                    BinaryOp::Sub => {
                        // left - pct% -> left - (left * pct / 100)
                        let delta = left_val
                            .checked_mul(pct_num)
                            .and_then(|v| v.checked_div(Decimal::from(100)))
                            .ok_or(CalcError::Overflow)?;
                        left_val.checked_sub(delta).ok_or(CalcError::Overflow)
                    }
                    BinaryOp::Mul => {
                        // left * pct% -> left * (pct / 100)
                        let frac = pct_num.checked_div(Decimal::from(100)).ok_or(CalcError::DivisionByZero)?;
                        left_val.checked_mul(frac).ok_or(CalcError::Overflow)
                    }
                    BinaryOp::Div => {
                        // left / pct% -> left / (pct / 100)
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
                let left_val = evaluate(left)?;
                let right_val = evaluate(right)?;
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
        BinaryOp::Pow => {
            if right.is_zero() {
                return Ok(Decimal::ONE);
            }
            // Check if right is an integer
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
                // Non-integer power via f64
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
