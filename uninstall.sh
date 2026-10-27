#!/usr/bin/env bash
set -euo pipefail
share="${XDG_DATA_HOME:-$HOME/.local/share}"
rm -f "$HOME/.local/bin/nyx-calculator"
rm -f "$share/applications/io.github.design_nexus.nyx.Calculator.desktop"
rm -f "$share/icons/hicolor/scalable/apps/io.github.design_nexus.nyx.Calculator.svg"
if command -v update-desktop-database >/dev/null; then update-desktop-database "$share/applications"; fi

hypr="${XDG_CONFIG_HOME:-$HOME/.config}/hypr"
if [[ -f $hypr/nyx.lua ]] && grep -q 'nyx\.Calculator' "$hypr/nyx.lua"; then
  sed -i '/nyx\\\.Calculator/d' "$hypr/nyx.lua"
  if command -v hyprctl >/dev/null 2>&1; then hyprctl reload >/dev/null 2>&1 || true; fi
fi

printf 'Uninstalled Calculator.\n'
