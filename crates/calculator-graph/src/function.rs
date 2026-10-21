use calculator_core::ast::Expr;
use calculator_core::error::CalcError;
use calculator_core::evaluator::eval_f64;
use calculator_core::parser::parse_str;
use calculator_core::scientific::AngleMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlottedFunction {
    pub id: usize,
    pub formula: String,
    #[serde(skip)]
    pub parsed: Option<Expr>,
    pub color: (f64, f64, f64),
    pub visible: bool,
    pub show_derivative: bool,
    pub error: Option<String>,
}

impl PlottedFunction {
    pub fn new(id: usize, formula: &str, color: (f64, f64, f64)) -> Self {
        let mut func =
            Self { id, formula: formula.to_string(), parsed: None, color, visible: true, show_derivative: false, error: None };
        func.compile();
        func
    }

    pub fn set_formula(&mut self, new_formula: &str) {
        self.formula = new_formula.to_string();
        self.compile();
    }

    pub fn compile(&mut self) {
        let trimmed = self.formula.trim();
        if trimmed.is_empty() {
            self.parsed = None;
            self.error = None;
            return;
        }

        match parse_str(trimmed) {
            Ok(expr) => {
                self.parsed = Some(expr);
                self.error = None;
            }
            Err(e) => {
                self.parsed = None;
                self.error = Some(e.to_string());
            }
        }
    }

    pub fn eval(&self, x: f64) -> Result<f64, CalcError> {
        if let Some(expr) = &self.parsed { eval_f64(expr, x, AngleMode::Radians) } else { Err(CalcError::EmptyExpression) }
    }

    pub fn eval_derivative(&self, x: f64) -> Result<f64, CalcError> {
        let h = 1e-6;
        let y1 = self.eval(x + h)?;
        let y0 = self.eval(x - h)?;
        Ok((y1 - y0) / (2.0 * h))
    }
}

pub const DEFAULT_COLORS: &[(f64, f64, f64)] = &[
    (0.35, 0.55, 0.95), // Accent Blue
    (0.92, 0.35, 0.35), // Red / Coral
    (0.30, 0.80, 0.55), // Emerald Green
    (0.95, 0.70, 0.20), // Amber / Gold
    (0.70, 0.40, 0.90), // Purple / Violet
    (0.20, 0.85, 0.85), // Cyan / Teal
];
