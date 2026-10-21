use crate::scientific::SciFunc;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Pos,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constant {
    Pi,
    E,
    Tau,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Number(Decimal),
    Const(Constant),
    Variable(String),
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Percent(Box<Expr>),
    Factorial(Box<Expr>),
    Function { func: SciFunc, arg: Box<Expr> },
}
