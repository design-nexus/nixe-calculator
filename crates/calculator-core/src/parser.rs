use crate::ast::{BinaryOp, Constant, Expr, UnaryOp};
use crate::error::CalcError;
use crate::scientific::SciFunc;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

pub fn parse_str(input: &str) -> Result<Expr, CalcError> {
    let tokens = crate::token::tokenize(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
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
            // Check for postfix operators: %, !, ²
            if let Some(tok) = self.peek() {
                if *tok == Token::Percent {
                    self.next();
                    lhs = Expr::Percent(Box::new(lhs));
                    continue;
                } else if *tok == Token::Exclamation {
                    self.next();
                    lhs = Expr::Factorial(Box::new(lhs));
                    continue;
                } else if *tok == Token::Sqr {
                    self.next();
                    lhs = Expr::Function { func: SciFunc::Sqr, arg: Box::new(lhs) };
                    continue;
                }
            }

            // Check for implicit multiplication: e.g. 2(3), 2pi, (2)(3), 2 sqrt(4)
            if let Some(next_tok) = self.peek()
                && matches!(
                    next_tok,
                    Token::Number(_)
                        | Token::ConstPi
                        | Token::ConstE
                        | Token::ConstTau
                        | Token::LParen
                        | Token::Sqrt
                        | Token::Sci(_)
                        | Token::Ident(_)
                )
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
                Some(Token::BitAnd) => BinaryOp::BitAnd,
                Some(Token::BitOr) => BinaryOp::BitOr,
                Some(Token::BitXor) => BinaryOp::BitXor,
                Some(Token::Shl) => BinaryOp::Shl,
                Some(Token::Shr) => BinaryOp::Shr,
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
            Token::ConstPi => Ok(Expr::Const(Constant::Pi)),
            Token::ConstE => Ok(Expr::Const(Constant::E)),
            Token::ConstTau => Ok(Expr::Const(Constant::Tau)),
            Token::Plus => {
                let expr = self.parse_expr(25)?;
                Ok(Expr::Unary { op: UnaryOp::Pos, expr: Box::new(expr) })
            }
            Token::Minus => {
                let expr = self.parse_expr(25)?;
                Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            Token::BitNot => {
                let expr = self.parse_expr(25)?;
                Ok(Expr::Unary { op: UnaryOp::BitNot, expr: Box::new(expr) })
            }
            Token::LParen => {
                let expr = self.parse_expr(0)?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err(CalcError::UnbalancedParentheses),
                }
            }
            Token::Sqrt => self.parse_func_arg(SciFunc::Sqrt),
            Token::Sqr => self.parse_func_arg(SciFunc::Sqr),
            Token::Recip => {
                let arg = self.parse_prefix()?;
                Ok(Expr::Binary {
                    op: BinaryOp::Div,
                    left: Box::new(Expr::Number(rust_decimal::Decimal::ONE)),
                    right: Box::new(arg),
                })
            }
            Token::Sci(func) => self.parse_func_arg(func),
            Token::Ident(name) => {
                if name.eq_ignore_ascii_case("mod") {
                    Err(CalcError::SyntaxError("Expected left operand before 'mod'".into()))
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Token::RParen => Err(CalcError::SyntaxError("Unexpected closing parenthesis ')'".into())),
            other => Err(CalcError::SyntaxError(format!("Unexpected token: {other:?}"))),
        }
    }

    fn parse_func_arg(&mut self, func: SciFunc) -> Result<Expr, CalcError> {
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
        Ok(Expr::Function { func, arg: Box::new(arg) })
    }
}

fn infix_binding_power(op: BinaryOp) -> (u8, u8) {
    match op {
        BinaryOp::BitOr => (4, 5),
        BinaryOp::BitXor => (6, 7),
        BinaryOp::BitAnd => (8, 9),
        BinaryOp::Shl | BinaryOp::Shr => (10, 11),
        BinaryOp::Add | BinaryOp::Sub => (12, 13),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => (20, 21),
        BinaryOp::Pow => (31, 30), // right-associative
    }
}
