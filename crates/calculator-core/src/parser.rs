use crate::ast::{BinaryOp, Expr, Func, UnaryOp};
use crate::error::CalcError;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    pub fn parse(&mut self) -> Result<Expr, CalcError> {
        if self.tokens.is_empty() {
            return Err(CalcError::EmptyExpression);
        }
        let expr = self.parse_expr(0)?;
        if let Some(tok) = self.peek() {
            return Err(CalcError::SyntaxError(format!("Unexpected token after expression: {tok:?}")));
        }
        Ok(expr)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, CalcError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Check for postfix operators (e.g. %, sqr)
            if let Some(tok) = self.peek() {
                if *tok == Token::Percent {
                    self.next();
                    lhs = Expr::Percent(Box::new(lhs));
                    continue;
                } else if *tok == Token::Sqr {
                    self.next();
                    lhs = Expr::Function { func: Func::Sqr, arg: Box::new(lhs) };
                    continue;
                }
            }

            // Check for implicit multiplication: e.g. 2(3) or (2)(3) or 2 sqrt(4)
            if let Some(next_tok) = self.peek()
                && matches!(next_tok, Token::Number(_) | Token::LParen | Token::Sqrt | Token::Ident(_))
            {
                let (l_bp, r_bp) = (20, 21);
                if l_bp < min_bp {
                    break;
                }
                let rhs = self.parse_expr(r_bp)?;
                lhs = Expr::Binary { op: BinaryOp::Mul, left: Box::new(lhs), right: Box::new(rhs) };
                continue;
            }

            let op = match self.peek() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Sub,
                Some(Token::Star) => BinaryOp::Mul,
                Some(Token::Slash) => BinaryOp::Div,
                Some(Token::Caret) => BinaryOp::Pow,
                _ => break,
            };

            let (l_bp, r_bp) = infix_binding_power(op);
            if l_bp < min_bp {
                break;
            }

            self.next(); // consume operator
            let rhs = self.parse_expr(r_bp)?;
            lhs = Expr::Binary { op, left: Box::new(lhs), right: Box::new(rhs) };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, CalcError> {
        let tok = self.next().ok_or(CalcError::SyntaxError("Unexpected end of expression".into()))?;

        match tok {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Plus => {
                let expr = self.parse_expr(25)?;
                Ok(Expr::Unary { op: UnaryOp::Pos, expr: Box::new(expr) })
            }
            Token::Minus => {
                let expr = self.parse_expr(25)?;
                Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            Token::LParen => {
                let expr = self.parse_expr(0)?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err(CalcError::UnbalancedParentheses),
                }
            }
            Token::Sqrt => {
                // If followed by LParen, parse parenthesized; otherwise parse next expression with high precedence
                let arg = if self.peek() == Some(&Token::LParen) {
                    self.next();
                    let inner = self.parse_expr(0)?;
                    if self.next() != Some(Token::RParen) {
                        return Err(CalcError::UnbalancedParentheses);
                    }
                    inner
                } else {
                    self.parse_expr(25)?
                };
                Ok(Expr::Function { func: Func::Sqrt, arg: Box::new(arg) })
            }
            Token::Sqr => {
                let arg = self.parse_prefix()?;
                Ok(Expr::Function { func: Func::Sqr, arg: Box::new(arg) })
            }
            Token::Recip => {
                let arg = if self.peek() == Some(&Token::LParen) {
                    self.next();
                    let inner = self.parse_expr(0)?;
                    if self.next() != Some(Token::RParen) {
                        return Err(CalcError::UnbalancedParentheses);
                    }
                    inner
                } else {
                    self.parse_expr(25)?
                };
                Ok(Expr::Function { func: Func::Recip, arg: Box::new(arg) })
            }
            Token::Ident(name) => match name.to_lowercase().as_str() {
                "abs" => {
                    let arg = if self.peek() == Some(&Token::LParen) {
                        self.next();
                        let inner = self.parse_expr(0)?;
                        if self.next() != Some(Token::RParen) {
                            return Err(CalcError::UnbalancedParentheses);
                        }
                        inner
                    } else {
                        self.parse_expr(25)?
                    };
                    Ok(Expr::Function { func: Func::Abs, arg: Box::new(arg) })
                }
                other => Err(CalcError::SyntaxError(format!("Unknown identifier: '{other}'"))),
            },
            Token::RParen => Err(CalcError::SyntaxError("Unexpected closing parenthesis ')'".into())),
            other => Err(CalcError::SyntaxError(format!("Unexpected token: {other:?}"))),
        }
    }
}

fn infix_binding_power(op: BinaryOp) -> (u8, u8) {
    match op {
        BinaryOp::Add | BinaryOp::Sub => (10, 11),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => (20, 21),
        BinaryOp::Pow => (31, 30), // right-associative
    }
}
