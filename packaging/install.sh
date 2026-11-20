#!/bin/sh
# Builds Lingua Onde and adds it to the app launcher (Linux, current user).
# Run it again after pulling changes. `install.sh --uninstall` removes it.
set -eu

bin="$HOME/.local/bin/lingua-onde"
desktop="$HOME/.local/share/applications/lingua-onde.desktop"
icon="$HOME/.local/share/icons/hicolor/scalable/apps/lingua-onde.svg"

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin" "$desktop" "$icon"
    echo "Removed the app. Your progress stays in ~/.local/share/lingua-onde/."
    exit 0
fi

repo="$(cd "$(dirname "$0")/.." && pwd)"
cargo build --release --manifest-path "$repo/Cargo.toml"
mkdir -p "$(dirname "$bin")" "$(dirname "$desktop")" "$(dirname "$icon")"
# Copy, so `cargo clean` never breaks the launcher.
install -m 755 "$repo/target/release/lingua_onde" "$bin"
install -m 644 "$repo/packaging/lingua-onde.svg" "$icon"
# Launchers do not search PATH reliably: point Exec at the installed file.
sed "s|^Exec=.*|Exec=$bin|" "$repo/packaging/lingua-onde.desktop" > "$desktop"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$(dirname "$desktop")" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" || true
echo "Installed: search for \"Lingua Onde\" in your app launcher."
