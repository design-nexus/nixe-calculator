#!/usr/bin/env bash
set -euo pipefail
share="${XDG_DATA_HOME:-$HOME/.local/share}"
rm -f "$HOME/.local/bin/nixe-calculator"
rm -f "$share/applications/io.github.design_nexus.nixe.Calculator.desktop"
rm -f "$share/icons/hicolor/scalable/apps/io.github.design_nexus.nixe.Calculator.svg"
if command -v update-desktop-database >/dev/null; then update-desktop-database "$share/applications"; fi

hypr="${XDG_CONFIG_HOME:-$HOME/.config}/hypr"
if [[ -f $hypr/nixe.lua ]] && grep -q 'nixe\.Calculator' "$hypr/nixe.lua"; then
  sed -i '/nixe\\\.Calculator/d' "$hypr/nixe.lua"
  if command -v hyprctl >/dev/null 2>&1; then hyprctl reload >/dev/null 2>&1 || true; fi
fi

printf 'Uninstalled Calculator.\n'
