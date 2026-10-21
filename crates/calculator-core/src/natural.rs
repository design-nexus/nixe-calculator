use crate::error::CalcError;
use crate::evaluator::evaluate_str_with_mode;
use crate::scientific::AngleMode;
use crate::units::{UnitDefinition, convert, find_unit};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LineValue {
    pub value: Decimal,
    pub formatted: String,
    pub currency: Option<char>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SheetLineKind {
    Empty,
    Heading(String),
    Comment(String),
    Result(LineValue),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct SheetLineOutput {
    pub line_index: usize,
    pub original_text: String,
    pub kind: SheetLineKind,
}

#[derive(Debug, Clone)]
pub struct VarEntry {
    pub value: Decimal,
    pub currency: Option<char>,
    pub unit: Option<String>,
    pub is_percent: bool,
}

#[derive(Default)]
pub struct NaturalSession {
    pub variables: HashMap<String, VarEntry>,
    pub line_results: Vec<Option<Decimal>>,
}

impl NaturalSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate_sheet(&mut self, text: &str) -> Vec<SheetLineOutput> {
        self.variables.clear();
        self.line_results.clear();

        let mut outputs = Vec::new();

        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                self.line_results.push(None);
                outputs.push(SheetLineOutput { line_index: idx, original_text: line.to_string(), kind: SheetLineKind::Empty });
                continue;
            }

            // Headings
            if let Some(heading) = trimmed.strip_prefix('#') {
                self.line_results.push(None);
                outputs.push(SheetLineOutput {
                    line_index: idx,
                    original_text: line.to_string(),
                    kind: SheetLineKind::Heading(heading.trim().to_string()),
                });
                continue;
            }

            // Comments
            if trimmed.starts_with("//") || trimmed.starts_with(';') || trimmed.starts_with("--") {
                self.line_results.push(None);
                outputs.push(SheetLineOutput {
                    line_index: idx,
                    original_text: line.to_string(),
                    kind: SheetLineKind::Comment(trimmed.to_string()),
                });
                continue;
            }

            // Check for variable assignment: `identifier = ...`
            if let Some((ident, expr)) = parse_var_assignment(trimmed) {
                let is_percent = expr.trim().ends_with('%');
                match self.evaluate_line(expr) {
                    Ok(res) => {
                        self.variables.insert(
                            ident.to_lowercase(),
                            VarEntry { value: res.value, currency: res.currency, unit: res.unit.clone(), is_percent },
                        );
                        self.line_results.push(Some(res.value));
                        outputs.push(SheetLineOutput {
                            line_index: idx,
                            original_text: line.to_string(),
                            kind: SheetLineKind::Result(res),
                        });
                    }
                    Err(e) => {
                        self.line_results.push(None);
                        outputs.push(SheetLineOutput {
                            line_index: idx,
                            original_text: line.to_string(),
                            kind: SheetLineKind::Error(e),
                        });
                    }
                }
                continue;
            }

            // Ordinary expression
            match self.evaluate_line(trimmed) {
                Ok(res) => {
                    self.line_results.push(Some(res.value));
                    outputs.push(SheetLineOutput {
                        line_index: idx,
                        original_text: line.to_string(),
                        kind: SheetLineKind::Result(res),
                    });
                }
                Err(e) => {
                    self.line_results.push(None);
                    outputs.push(SheetLineOutput {
                        line_index: idx,
                        original_text: line.to_string(),
                        kind: SheetLineKind::Error(e),
                    });
                }
            }
        }

        outputs
    }

    fn evaluate_line(&self, input: &str) -> Result<LineValue, String> {
        let mut s = input.trim().to_string();

        // Substitute previous line references: $N, lineN, prev
        s = self.substitute_references(&s)?;

        // Substitute variable references
        s = self.substitute_variables(&s);

        // Detect currency symbol
        let mut currency = None;
        for c in ['$', '€', '£', '¥'] {
            if s.contains(c) {
                currency = Some(c);
                s = s.replace(c, "");
                break;
            }
        }

        // Check for conversion pattern: `<source> in <target>` or `<source> to <target>`
        if let Some((source_expr, target_str)) = split_conversion(&s) {
            return self.evaluate_conversion(&source_expr, &target_str);
        }

        // Check for natural language patterns
        s = normalize_natural_idioms(&s);

        // Evaluate using math evaluator
        let val = evaluate_str_with_mode(&s, AngleMode::Degrees).map_err(|e| format_calc_error(&e))?;

        let formatted = format_numi_result(val, currency, None);
        Ok(LineValue { value: val, formatted, currency, unit: None })
    }

    fn substitute_references(&self, input: &str) -> Result<String, String> {
        let mut out = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                let mut num_str = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !num_str.is_empty() {
                    if let Ok(line_num) = num_str.parse::<usize>()
                        && line_num >= 1
                        && line_num <= self.line_results.len()
                    {
                        if let Some(val) = self.line_results[line_num - 1] {
                            out.push_str(&val.to_string());
                            continue;
                        } else {
                            return Err(format!("Line {line_num} has no numerical result"));
                        }
                    }
                    // Not a valid line reference, keep original e.g. $250
                    out.push('$');
                    out.push_str(&num_str);
                } else {
                    out.push('$');
                }
            } else if c.is_alphabetic() {
                let mut word = String::new();
                word.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        word.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if word.eq_ignore_ascii_case("prev") {
                    let prev_val = self.line_results.iter().rev().find_map(|r| *r);
                    if let Some(v) = prev_val {
                        out.push_str(&v.to_string());
                    } else {
                        return Err("No previous result available for 'prev'".into());
                    }
                } else if word.to_lowercase().starts_with("line") {
                    let num_str = &word[4..];
                    if let Ok(line_num) = num_str.parse::<usize>() {
                        if line_num >= 1 && line_num <= self.line_results.len() {
                            if let Some(val) = self.line_results[line_num - 1] {
                                out.push_str(&val.to_string());
                            } else {
                                return Err(format!("Line {line_num} has no numerical result"));
                            }
                        } else {
                            return Err(format!("Reference to invalid line number {line_num}"));
                        }
                    } else {
                        out.push_str(&word);
                    }
                } else {
                    out.push_str(&word);
                }
            } else {
                out.push(c);
            }
        }

        Ok(out)
    }

    fn substitute_variables(&self, input: &str) -> String {
        let mut s = input.to_string();
        for (name, var) in &self.variables {
            let replacement = if var.is_percent {
                let pct = (var.value * rust_decimal_macros::dec!(100)).normalize();
                format!("{pct}%")
            } else {
                var.value.to_string()
            };
            s = replace_word(&s, name, &replacement);
        }
        s
    }

    fn evaluate_conversion(&self, source_expr: &str, target_str: &str) -> Result<LineValue, String> {
        let target_unit = find_unit(target_str.trim()).ok_or_else(|| format!("Unknown target unit '{target_str}'"))?;

        // Try parsing multi-unit source: e.g. "2 hours 30 minutes"
        let parts = parse_compound_units(source_expr);
        if !parts.is_empty() {
            let mut total_converted = Decimal::ZERO;
            for (amt, u) in parts {
                let res = convert(amt, u.id, target_unit.id).map_err(|e| format_calc_error(&e))?;
                total_converted = total_converted.checked_add(res).ok_or("Overflow in conversion")?;
            }
            let formatted = format!("{} {}", total_converted, target_unit.symbol);
            return Ok(LineValue {
                value: total_converted,
                formatted,
                currency: None,
                unit: Some(target_unit.symbol.to_string()),
            });
        }

        // Single expression before unit: e.g. `5 miles` or `(10 + 20) km`
        let (val, from_unit) = extract_value_and_unit(source_expr)?;
        let res = convert(val, from_unit.id, target_unit.id).map_err(|e| format_calc_error(&e))?;
        let formatted = format!("{} {}", res, target_unit.symbol);
        Ok(LineValue { value: res, formatted, currency: None, unit: Some(target_unit.symbol.to_string()) })
    }
}

fn parse_var_assignment(s: &str) -> Option<(&str, &str)> {
    if let Some(idx) = s.find('=') {
        let ident = s[..idx].trim();
        let expr = s[idx + 1..].trim();
        if !ident.is_empty()
            && ident.chars().all(|c| c.is_alphanumeric() || c == '_')
            && ident.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Some((ident, expr));
        }
    }
    None
}

fn split_conversion(s: &str) -> Option<(String, String)> {
    // Check " in "
    if let Some(idx) = s.to_lowercase().find(" in ") {
        let left = s[..idx].trim().to_string();
        let right = s[idx + 4..].trim().to_string();
        if find_unit(&right).is_some() {
            return Some((left, right));
        }
    }
    // Check " to "
    if let Some(idx) = s.to_lowercase().find(" to ") {
        let left = s[..idx].trim().to_string();
        let right = s[idx + 4..].trim().to_string();
        if find_unit(&right).is_some() {
            return Some((left, right));
        }
    }
    None
}

fn parse_compound_units(s: &str) -> Vec<(Decimal, &'static UnitDefinition)> {
    let mut results = Vec::new();
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let mut i = 0;
    while i + 1 < tokens.len() {
        if let Ok(amt) = tokens[i].parse::<Decimal>()
            && let Some(u) = find_unit(tokens[i + 1])
        {
            results.push((amt, u));
            i += 2;
            continue;
        }
        i += 1;
    }
    results
}

fn extract_value_and_unit(s: &str) -> Result<(Decimal, &'static UnitDefinition), String> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() >= 2 {
        let unit_part = tokens.last().unwrap();
        if let Some(u) = find_unit(unit_part) {
            let val_part = tokens[..tokens.len() - 1].join(" ");
            let val = evaluate_str_with_mode(&val_part, AngleMode::Degrees).map_err(|e| format_calc_error(&e))?;
            return Ok((val, u));
        }
    }
    Err(format!("Could not extract unit from '{s}'"))
}

fn normalize_natural_idioms(input: &str) -> String {
    let mut s = input.trim().to_string();

    // 15% of 249 -> 249 * 15%
    if let Some(idx) = s.to_lowercase().find(" of ") {
        let left = s[..idx].trim();
        let right = s[idx + 4..].trim();
        if left.ends_with('%') {
            return format!("({right}) * ({left})");
        }
    }

    // 20% tip on 85 -> 85 * 20%
    if let Some(idx) = s.to_lowercase().find(" tip on ") {
        let left = s[..idx].trim();
        let right = s[idx + 8..].trim();
        if left.ends_with('%') {
            return format!("({right}) * ({left})");
        }
    }

    // 150 increased by 12% -> 150 + 12%
    if let Some(idx) = s.to_lowercase().find(" increased by ") {
        let left = s[..idx].trim();
        let right = s[idx + 14..].trim();
        return format!("({left}) + ({right})");
    }

    // 200 decreased by 15% -> 200 - 15%
    if let Some(idx) = s.to_lowercase().find(" decreased by ") {
        let left = s[..idx].trim();
        let right = s[idx + 14..].trim();
        return format!("({left}) - ({right})");
    }

    // 1200 + 8% tax -> 1200 + 8%
    if let Some(idx) = s.to_lowercase().find(" tax") {
        let before = s[..idx].trim();
        s = before.to_string();
    }

    // $250 split between 4 people -> 250 / 4
    if let Some(idx) = s.to_lowercase().find(" split between ") {
        let left = s[..idx].trim();
        let rest = s[idx + 15..].trim();
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if !num_str.is_empty() {
            return format!("({left}) / {num_str}");
        }
    }

    // split by N
    if let Some(idx) = s.to_lowercase().find(" split by ") {
        let left = s[..idx].trim();
        let rest = s[idx + 10..].trim();
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if !num_str.is_empty() {
            return format!("({left}) / {num_str}");
        }
    }

    // Words to operators
    s = s.replace(" plus ", " + ");
    s = s.replace(" minus ", " - ");
    s = s.replace(" times ", " * ");
    s = s.replace(" divided by ", " / ");

    s
}

fn replace_word(text: &str, word: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut remainder = text;

    while let Some(idx) = remainder.to_lowercase().find(&word.to_lowercase()) {
        let before = &remainder[..idx];
        let after = &remainder[idx + word.len()..];

        let before_boundary = before.is_empty() || !before.chars().last().unwrap().is_alphanumeric();
        let after_boundary = after.is_empty() || !after.chars().next().unwrap().is_alphanumeric();

        if before_boundary && after_boundary {
            result.push_str(before);
            result.push_str(replacement);
            remainder = after;
        } else {
            result.push_str(&remainder[..idx + word.len()]);
            remainder = after;
        }
    }
    result.push_str(remainder);
    result
}

fn format_numi_result(val: Decimal, currency: Option<char>, unit: Option<&str>) -> String {
    let mut s = if let Some(c) = currency { format!("{c}{val}") } else { val.to_string() };
    if let Some(u) = unit {
        s.push(' ');
        s.push_str(u);
    }
    s
}

fn format_calc_error(err: &CalcError) -> String {
    match err {
        CalcError::DivisionByZero => "Division by zero".into(),
        CalcError::Overflow => "Numerical overflow".into(),
        CalcError::Underflow => "Numerical underflow".into(),
        CalcError::InvalidDomain(msg) => format!("Domain error: {msg}"),
        CalcError::SyntaxError(msg) => format!("Syntax error: {msg}"),
        CalcError::EmptyExpression => "Empty expression".into(),
        CalcError::UnbalancedParentheses => "Unbalanced parentheses".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_numi_expressions() {
        let mut session = NaturalSession::new();
        let sheet = r#"
# Sample Budget
15% of 249
1200 + 8% tax
20% tip on 85
150 increased by 12%
$250 split between 4 people
"#;
        let outputs = session.evaluate_sheet(sheet);
        assert_eq!(outputs.len(), 7); // Empty, Heading, 5 exprs

        // 15% of 249 = 37.35
        if let SheetLineKind::Result(res) = &outputs[2].kind {
            assert_eq!(res.value, dec!(37.35));
        } else {
            panic!("Expected result for 15% of 249");
        }

        // 1200 + 8% = 1296
        if let SheetLineKind::Result(res) = &outputs[3].kind {
            assert_eq!(res.value, dec!(1296));
        } else {
            panic!("Expected result for 1200 + 8% tax");
        }

        // 20% tip on 85 = 17
        if let SheetLineKind::Result(res) = &outputs[4].kind {
            assert_eq!(res.value, dec!(17));
        } else {
            panic!("Expected result for 20% tip on 85");
        }

        // 150 increased by 12% = 168
        if let SheetLineKind::Result(res) = &outputs[5].kind {
            assert_eq!(res.value, dec!(168));
        } else {
            panic!("Expected result for 150 increased by 12%");
        }

        // $250 split between 4 people = $62.5
        if let SheetLineKind::Result(res) = &outputs[6].kind {
            assert_eq!(res.value, dec!(62.5));
            assert_eq!(res.currency, Some('$'));
        } else {
            panic!("Expected result for split between people");
        }
    }

    #[test]
    fn test_conversions_in_sheet() {
        let mut session = NaturalSession::new();
        let sheet = r#"
5 miles in kilometers
32 fahrenheit to celsius
2 hours 30 minutes in seconds
"#;
        let outputs = session.evaluate_sheet(sheet);

        if let SheetLineKind::Result(res) = &outputs[1].kind {
            assert_eq!(res.value, dec!(8.04672));
        } else {
            panic!("Expected conversion result for 5 miles in km");
        }

        if let SheetLineKind::Result(res) = &outputs[2].kind {
            assert_eq!(res.value, dec!(0));
        } else {
            panic!("Expected conversion result for 32 fahrenheit to celsius");
        }

        if let SheetLineKind::Result(res) = &outputs[3].kind {
            assert_eq!(res.value, dec!(9000));
        } else {
            panic!("Expected compound conversion result for 2h 30m in s");
        }
    }

    #[test]
    fn test_variables_and_line_references() {
        let mut session = NaturalSession::new();
        let sheet = "price = 100\ntax = 10%\ntotal = price + tax\nprev * 2\n$1 + 50";
        let outputs = session.evaluate_sheet(sheet);

        // total = 100 + 10% = 110
        if let SheetLineKind::Result(res) = &outputs[2].kind {
            assert_eq!(res.value, dec!(110));
        } else {
            panic!("Expected result for total");
        }

        // prev * 2 = 220
        if let SheetLineKind::Result(res) = &outputs[3].kind {
            assert_eq!(res.value, dec!(220));
        } else {
            panic!("Expected result for prev * 2");
        }

        // $1 + 50 = 100 + 50 = 150
        match &outputs[4].kind {
            SheetLineKind::Result(res) => assert_eq!(res.value, dec!(150)),
            other => panic!("Expected result for $1 + 50, got {:?}", other),
        }
    }
}
