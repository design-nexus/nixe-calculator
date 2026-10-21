use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    DivisionByZero,
    Overflow,
    Underflow,
    InvalidDomain(String),
    SyntaxError(String),
    EmptyExpression,
    UnbalancedParentheses,
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivisionByZero => write!(f, "Cannot divide by zero"),
            Self::Overflow => write!(f, "Numerical overflow"),
            Self::Underflow => write!(f, "Numerical underflow"),
            Self::InvalidDomain(msg) => write!(f, "Invalid domain: {msg}"),
            Self::SyntaxError(msg) => write!(f, "Syntax error: {msg}"),
            Self::EmptyExpression => write!(f, "Empty expression"),
            Self::UnbalancedParentheses => write!(f, "Unbalanced parentheses"),
        }
    }
}

impl std::error::Error for CalcError {}
