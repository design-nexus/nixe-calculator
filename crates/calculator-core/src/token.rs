use crate::error::CalcError;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Number(Decimal),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    LParen,
    RParen,
    Sqrt,
    Sqr,
    Recip,
    Ident(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, CalcError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let len = chars.len();

    while i < len {
        let ch = chars[i];

        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // Numbers: digits or '.' (e.g. .5 or 123.456)
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            let mut has_dot = ch == '.';
            i += 1;
            while i < len {
                let c = chars[i];
                if c.is_ascii_digit() {
                    i += 1;
                } else if c == '.' && !has_dot {
                    has_dot = true;
                    i += 1;
                } else if c == '_' {
                    // Ignore underscores in numbers
                    i += 1;
                } else {
                    break;
                }
            }
            let s: String = chars[start..i].iter().filter(|&&c| c != '_').collect();
            let num = Decimal::from_str(&s).map_err(|e| CalcError::SyntaxError(format!("Invalid number '{s}': {e}")))?;
            tokens.push(Token::Number(num));
            continue;
        }

        match ch {
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' | '−' => {
                // Unicode minus or ASCII hyphen
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' | '×' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '/' | '÷' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            '^' => {
                tokens.push(Token::Caret);
                i += 1;
            }
            '%' => {
                tokens.push(Token::Percent);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '√' => {
                tokens.push(Token::Sqrt);
                i += 1;
            }
            '²' => {
                tokens.push(Token::Sqr);
                i += 1;
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                match name.to_lowercase().as_str() {
                    "sqrt" => tokens.push(Token::Sqrt),
                    "sqr" | "square" => tokens.push(Token::Sqr),
                    "recip" | "inv" => tokens.push(Token::Recip),
                    _ => tokens.push(Token::Ident(name)),
                }
            }
            other => {
                return Err(CalcError::SyntaxError(format!("Unexpected character: '{other}'")));
            }
        }
    }

    Ok(tokens)
}
