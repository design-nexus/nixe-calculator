pub use nyx_theme::ThemeMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub mode: ThemeMode,
    pub theme: String,
    pub reduce_motion: bool,
    pub glow: bool,
    pub history_open: bool,
    pub precision: usize, // 0 = auto, 1..=12 = fixed decimals
    pub thousands_separator: bool,
    pub history_enabled: bool,
    pub history_retention: usize, // max items to retain
    pub calculator_mode: String,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Omarchy,
            theme: "tokyo-night".into(),
            reduce_motion: false,
            glow: true,
            history_open: false,
            precision: 0,
            thousands_separator: true,
            history_enabled: true,
            history_retention: 100,
            calculator_mode: "standard".into(),
        }
    }
}

nyx_core::prefs_store!(Prefs);
