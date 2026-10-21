# Nixe Calculator

A modern, fast, lightweight desktop calculator for Omarchy and the Nixe application suite.

Combines the clean elegance of macOS Calculator, the mathematical power of Windows Calculator, the natural-language workflow of Numi, and the interactive graphing of Desmos.

---

## Features (Phase 1: Standard Mode)

- **Pure Decimal Precision:** 128-bit exact decimal arithmetic (`rust_decimal`) with zero floating-point rounding quirks (`0.1 + 0.2 = 0.3`).
- **Pratt Expression Parser:** Predictable operator precedence (`+`, `−`, `×`, `÷`, `^`, `%`, `(`, `)`), implicit multiplication `2(3+4)`, unary operators, and functions (`sqrt`, `sqr`, `recip`).
- **Percentage Engine:** Additive and subtractive percentages (`100 + 10% = 110`, `100 − 10% = 90`, `200 × 15% = 30`).
- **Live Expression Preview:** Real-time result preview updates dynamically as you type.
- **Memory Operations:** Full memory register with `MC`, `MR`, `M+`, `M−`, `MS` and persistent visual indicator.
- **Searchable Calculation History:** SQLite-backed persistent history with instant search, pinned items, and single-click formula restoration.
- **Native GTK4 & Nixe Style:** Built strictly against Nixe `STYLE.md` with flat single-surface styling, JetBrains Mono typography, hairline 1px borders, and zero visible scrollbars.
- **Theme Synchronization:** Seamless live restyling following Omarchy (`colors.toml`), with 14 bundled themes and user theme support.
- **Keyboard-First:** Full numpad and keyboard shortcuts for all operations, clipboard copy/paste, and history toggle (`Ctrl+H`).

---

## Keyboard Shortcuts

| Shortcut | Action |
| --- | --- |
| `0–9`, `.` | Enter digits and decimal point |
| `+`, `−`, `×`, `÷`, `^`, `%` | Arithmetic operations |
| `Enter`, `=` | Evaluate calculation |
| `Backspace` | Delete last character |
| `Esc`, `C` | Clear expression |
| `Ctrl+H` | Toggle history sidebar |
| `Ctrl+C` | Copy displayed result |
| `Ctrl+V` | Paste expression |
| `Ctrl+M` | Memory store (MS) |
| `Ctrl+R` | Memory recall (MR) |
| `Ctrl+P` | Memory add (M+) |
| `Ctrl+Q` / `Ctrl+W` | Close window |
| `Ctrl+L` | Memory clear (MC) |
| `Ctrl+,` | Open settings |
| `F1` | Show shortcuts dialog |

---

## Building and Installing

### Prerequisites

- Rust 1.85+ (Edition 2024)
- GTK4 (`gtk4` package)
- Cairo and SQLite3 (`cairo`, `sqlite`)

### Run Tests

```bash
cargo test --workspace
```

### Build and Install

```bash
./install.sh
```

This compiles the release binary with thin LTO and installs:
- Executable to `~/.local/bin/nixe-calculator`
- Desktop entry to `~/.local/share/applications/io.github.design_nexus.nixe.Calculator.desktop`
- SVG icon to `~/.local/share/icons/hicolor/scalable/apps/io.github.design_nexus.nixe.Calculator.svg`

### Uninstall

```bash
./uninstall.sh
```
