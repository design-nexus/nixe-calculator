use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub thousands_separator: bool,
    pub precision: Option<usize>,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { thousands_separator: true, precision: None }
    }
}

pub fn format_decimal(val: Decimal, opts: &FormatOptions) -> String {
    let normalized = val.normalize();

    let formatted_str =
        if let Some(p) = opts.precision { format!("{normalized:.prec$}", prec = p) } else { normalized.to_string() };

    if !opts.thousands_separator {
        return formatted_str;
    }

    // Split into sign, integer part, and fractional part
    let (is_neg, rest) =
        if let Some(stripped) = formatted_str.strip_prefix('-') { (true, stripped) } else { (false, formatted_str.as_str()) };

    let mut parts = rest.split('.');
    let int_part = parts.next().unwrap_or("0");
    let frac_part = parts.next();

    // Group integer part by 3 digits from the right
    let mut grouped_int = String::with_capacity(int_part.len() + int_part.len() / 3);
    let int_chars: Vec<char> = int_part.chars().collect();
    let num_digits = int_chars.len();

    for (idx, ch) in int_chars.into_iter().enumerate() {
        if idx > 0 && (num_digits - idx).is_multiple_of(3) {
            grouped_int.push(',');
        }
        grouped_int.push(ch);
    }

    let mut result = String::with_capacity(grouped_int.len() + 10);
    if is_neg {
        result.push('−'); // Unicode minus for beautiful presentation
    }
    result.push_str(&grouped_int);

    if let Some(frac) = frac_part {
        result.push('.');
        result.push_str(frac);
    }

    result
}
