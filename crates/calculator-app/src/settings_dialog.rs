use crate::widgets::{self, Page};
use crate::{prefs, theme, window};
use gtk::prelude::*;
use nixe_ui::settings_card::{self, Spec};

const GROUPS: &[(&str, &str, &str)] = &[
    ("This window", "applications-graphics-symbolic", "Theme, glow, motion"),
    ("Calculations", "accessories-calculator-symbolic", "Precision, separators"),
    ("History", "document-open-recent-symbolic", "Retention, clear history"),
    ("Keyboard", "input-keyboard-symbolic", "Shortcuts"),
];

const SPEC: Spec = Spec { groups: GROUPS, search_text: None };

pub fn open() {
    let (Some(overlay), Some(win)) = (window::overlay(), window::window()) else {
        return;
    };
    settings_card::open(&overlay, &win, SPEC, || {
        let p = widgets::page("settings");
        build_content(&p);
        (p.root.upcast(), p.body)
    });
}

fn build_content(p: &Page) {
    window_group(p);
    calc_group(p);
    history_group(p);
    keyboard_group(p);
}

fn window_group(p: &Page) {
    let group = p.group("This window");
    group.add(&nixe_ui::widgets::start_row());
    let current_prefs = prefs::get();

    let options: Vec<(String, String)> = theme::all().iter().map(|t| (t.id.clone(), t.name.clone())).collect();

    let (theme_row, theme_dd) =
        widgets::choice_row("Theme", "A bundled theme, or one of your own.", options, &current_prefs.theme, |id| {
            prefs::update(|p| {
                p.theme = id;
                p.mode = prefs::ThemeMode::Theme;
            });
            theme::apply();
        });

    theme_dd.set_sensitive(current_prefs.mode == prefs::ThemeMode::Theme || !theme::omarchy_available());

    if theme::omarchy_available() {
        let dd = theme_dd.clone();
        let (row, _) = widgets::switch_row(
            "Follow Omarchy theme",
            "Match the desktop's colors and update when the Omarchy theme changes.",
            current_prefs.mode == prefs::ThemeMode::Omarchy,
            move |on| {
                prefs::update(|p| {
                    p.mode = if on { prefs::ThemeMode::Omarchy } else { prefs::ThemeMode::Theme };
                });
                dd.set_sensitive(!on);
                theme::apply();
            },
        );
        group.add(&row);
    }
    group.add(&theme_row);

    let (row, _) =
        widgets::switch_row("Glow", "Soft accent glow around focused and selected elements.", current_prefs.glow, |on| {
            prefs::update(|p| p.glow = on);
            theme::apply();
        });
    group.add(&row);

    let (row, _) =
        widgets::switch_row("Reduce motion", "Turn off transitions in this window.", current_prefs.reduce_motion, |on| {
            prefs::update(|p| p.reduce_motion = on);
            theme::apply();
        });
    group.add(&row);
}

fn calc_group(p: &Page) {
    let group = p.group("Calculations");
    let current = prefs::get();

    let prec_options = vec![
        ("0".into(), "Automatic".into()),
        ("2".into(), "2 decimal places".into()),
        ("4".into(), "4 decimal places".into()),
        ("6".into(), "6 decimal places".into()),
        ("8".into(), "8 decimal places".into()),
        ("12".into(), "12 decimal places".into()),
    ];

    let (row, _) = widgets::choice_row(
        "Decimal precision",
        "Number of decimal places shown in results.",
        prec_options,
        &current.precision.to_string(),
        |val| {
            let p: usize = val.parse().unwrap_or(0);
            prefs::update(|pr| pr.precision = p);
            window::refresh_display();
        },
    );
    group.add(&row);

    let (row, _) = widgets::switch_row(
        "Thousands separators",
        "Group digits with commas (e.g. 1,234,567.89).",
        current.thousands_separator,
        |on| {
            prefs::update(|pr| pr.thousands_separator = on);
            window::refresh_display();
        },
    );
    group.add(&row);
}

fn history_group(p: &Page) {
    let group = p.group("History");
    let current = prefs::get();

    let (row, _) = widgets::switch_row("Record history", "Save completed calculations locally.", current.history_enabled, |on| {
        prefs::update(|pr| pr.history_enabled = on);
    });
    group.add(&row);

    let ret_options = vec![
        ("50".into(), "50 calculations".into()),
        ("100".into(), "100 calculations".into()),
        ("250".into(), "250 calculations".into()),
        ("1000".into(), "1,000 calculations".into()),
    ];

    let (row, _) = widgets::choice_row(
        "History retention",
        "Maximum number of unpinned calculations to keep.",
        ret_options,
        &current.history_retention.to_string(),
        |val| {
            let n: usize = val.parse().unwrap_or(100);
            prefs::update(|pr| pr.history_retention = n);
        },
    );
    group.add(&row);
}

fn keyboard_group(p: &Page) {
    let group = p.group("Keyboard");
    let shortcuts: &[(&[&str], &str)] = &[
        (&["Alt", "1"], "Standard mode"),
        (&["Alt", "2"], "Scientific mode"),
        (&["Alt", "3"], "Programmer mode"),
        (&["Alt", "4"], "Unit Converter"),
        (&["Alt", "5"], "Calculation Sheets"),
        (&["0–9", "."], "Enter digits and decimal point"),
        (&["+", "−", "×", "÷"], "Basic arithmetic operations"),
        (&["Enter", "="], "Calculate result"),
        (&["Backspace"], "Delete last character"),
        (&["Esc", "C"], "Clear current expression"),
        (&["Ctrl", "H"], "Toggle history sidebar"),
        (&["Ctrl", "C"], "Copy result to clipboard"),
        (&["Ctrl", "V"], "Paste expression"),
        (&["Ctrl", "M"], "Store current value in memory (MS)"),
        (&["Ctrl", "R"], "Recall memory value (MR)"),
        (&["Ctrl", "P"], "Add to memory (M+)"),
        (&["Ctrl", "Q"], "Subtract from memory (M−)"),
        (&["Ctrl", "L"], "Clear memory register (MC)"),
        (&["Ctrl", ","], "Open settings"),
        (&["F1"], "Show shortcuts dialog"),
    ];

    for (keys, what) in shortcuts {
        group.add(&widgets::row(what, "", Some(widgets::key_caps(keys).upcast_ref())));
    }
}
