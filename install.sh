#!/usr/bin/env bash
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if [[ ${NIXE_SKIP_BUILD:-0} != 1 ]]; then
  cargo build --release --manifest-path "$root/Cargo.toml" -p nixe-calculator
fi
share="${XDG_DATA_HOME:-$HOME/.local/share}"
binary="${CARGO_TARGET_DIR:-$root/target}/release/nixe-calculator"
[[ -f $binary ]] || { echo "Release binary missing: $binary" >&2; exit 1; }
install -Dm755 "$binary" "$HOME/.local/bin/nixe-calculator"
install -Dm644 "$root/data/io.github.design_nexus.nixe.Calculator.desktop" "$share/applications/io.github.design_nexus.nixe.Calculator.desktop"
install -Dm644 "$root/data/icons/io.github.design_nexus.nixe.Calculator.svg" "$share/icons/hicolor/scalable/apps/io.github.design_nexus.nixe.Calculator.svg"
if command -v update-desktop-database >/dev/null; then update-desktop-database "$share/applications"; fi
printf 'Installed Calculator without opening it.\n'
