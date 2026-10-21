use crate::error::CalcError;
use crate::scientific::SciFunc;
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
    Exclamation, // ! for factorial
    LParen,
    RParen,
    Sqrt,
    Sqr,
    Recip,
    Sci(SciFunc),
    ConstPi,
    ConstE,
    ConstTau,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
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

        // Check for 0x (hex), 0b (bin), 0o (oct) prefixes
        if ch == '0' && i + 1 < len {
            let next_ch = chars[i + 1];
            if next_ch == 'x' || next_ch == 'X' {
                i += 2;
                let start = i;
                while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().filter(|&&c| c != '_').collect();
                let val = u64::from_str_radix(&s, 16).map_err(|e| CalcError::SyntaxError(format!("Invalid hex '0x{s}': {e}")))?;
                tokens.push(Token::Number(Decimal::from(val)));
                continue;
            } else if next_ch == 'b' || next_ch == 'B' {
                i += 2;
                let start = i;
                while i < len && (chars[i] == '0' || chars[i] == '1' || chars[i] == '_') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().filter(|&&c| c != '_').collect();
                let val =
                    u64::from_str_radix(&s, 2).map_err(|e| CalcError::SyntaxError(format!("Invalid binary '0b{s}': {e}")))?;
                tokens.push(Token::Number(Decimal::from(val)));
                continue;
            } else if next_ch == 'o' || next_ch == 'O' {
                i += 2;
                let start = i;
                while i < len && (matches!(chars[i], '0'..='7') || chars[i] == '_') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().filter(|&&c| c != '_').collect();
                let val =
                    u64::from_str_radix(&s, 8).map_err(|e| CalcError::SyntaxError(format!("Invalid octal '0o{s}': {e}")))?;
                tokens.push(Token::Number(Decimal::from(val)));
                continue;
            }
        }

        // Numbers: digits or '.' (e.g. .5, 123.456, or 1.25e+3)
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
                } else if (c == 'e' || c == 'E') && i + 1 < len {
                    let next = chars[i + 1];
                    if next.is_ascii_digit() || ((next == '+' || next == '-') && i + 2 < len && chars[i + 2].is_ascii_digit()) {
                        i += 1; // consume 'e'
                        if chars[i] == '+' || chars[i] == '-' {
                            i += 1;
                        }
                        while i < len && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                        break;
                    } else {
                        break;
                    }
                } else if c == '_' {
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
            '!' => {
                tokens.push(Token::Exclamation);
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
            'π' => {
                tokens.push(Token::ConstPi);
                i += 1;
            }
            'τ' => {
                tokens.push(Token::ConstTau);
                i += 1;
            }
            '&' => {
                tokens.push(Token::BitAnd);
                i += 1;
            }
            '|' => {
                tokens.push(Token::BitOr);
                i += 1;
            }
            '~' => {
                tokens.push(Token::BitNot);
                i += 1;
            }
            '<' if i + 1 < len && chars[i + 1] == '<' => {
                tokens.push(Token::Shl);
                i += 2;
            }
            '>' if i + 1 < len && chars[i + 1] == '>' => {
                tokens.push(Token::Shr);
                i += 2;
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
                    "sin" => tokens.push(Token::Sci(SciFunc::Sin)),
                    "cos" => tokens.push(Token::Sci(SciFunc::Cos)),
                    "tan" => tokens.push(Token::Sci(SciFunc::Tan)),
                    "asin" => tokens.push(Token::Sci(SciFunc::Asin)),
                    "acos" => tokens.push(Token::Sci(SciFunc::Acos)),
                    "atan" => tokens.push(Token::Sci(SciFunc::Atan)),
                    "sinh" => tokens.push(Token::Sci(SciFunc::Sinh)),
                    "cosh" => tokens.push(Token::Sci(SciFunc::Cosh)),
                    "tanh" => tokens.push(Token::Sci(SciFunc::Tanh)),
                    "ln" => tokens.push(Token::Sci(SciFunc::Ln)),
                    "log" | "log10" => tokens.push(Token::Sci(SciFunc::Log10)),
                    "log2" => tokens.push(Token::Sci(SciFunc::Log2)),
                    "exp" => tokens.push(Token::Sci(SciFunc::Exp)),
                    "cbrt" => tokens.push(Token::Sci(SciFunc::Cbrt)),
                    "fact" => tokens.push(Token::Sci(SciFunc::Fact)),
                    "abs" => tokens.push(Token::Sci(SciFunc::Abs)),
                    "pi" => tokens.push(Token::ConstPi),
                    "e" => tokens.push(Token::ConstE),
                    "tau" => tokens.push(Token::ConstTau),
                    "and" => tokens.push(Token::BitAnd),
                    "or" => tokens.push(Token::BitOr),
                    "xor" => tokens.push(Token::BitXor),
                    "not" => tokens.push(Token::BitNot),
                    "shl" | "lsh" => tokens.push(Token::Shl),
                    "shr" | "rsh" => tokens.push(Token::Shr),
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
