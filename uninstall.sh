#!/usr/bin/env bash
set -euo pipefail
share="${XDG_DATA_HOME:-$HOME/.local/share}"
rm -f "$HOME/.local/bin/nixe-calculator"
rm -f "$share/applications/io.github.design_nexus.nixe.Calculator.desktop"
rm -f "$share/icons/hicolor/scalable/apps/io.github.design_nexus.nixe.Calculator.svg"
if command -v update-desktop-database >/dev/null; then update-desktop-database "$share/applications"; fi
printf 'Uninstalled Calculator.\n'
