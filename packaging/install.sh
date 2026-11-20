#!/bin/sh
# Installs Lingua Onde for the current user (Linux) and adds it to the app launcher.
#
#   From the internet (no Rust needed):
#     curl -fsSL https://raw.githubusercontent.com/Matteo404404/lingua-onde/main/packaging/install.sh | sh
#   From a clone (builds from source with cargo):
#     ./packaging/install.sh
#   Remove it (your progress stays):
#     ./packaging/install.sh --uninstall    or    curl ... | sh -s -- --uninstall
#
# Run it again to update.
set -eu

repo="Matteo404404/lingua-onde"
bin="$HOME/.local/bin/lingua-onde"
desktop="$HOME/.local/share/applications/lingua-onde.desktop"
icon="$HOME/.local/share/icons/hicolor/scalable/apps/lingua-onde.svg"

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin" "$desktop" "$icon"
    echo "Removed Lingua Onde. Your progress stays in ~/.local/share/lingua-onde/."
    exit 0
fi

mkdir -p "$(dirname "$bin")" "$(dirname "$desktop")" "$(dirname "$icon")"
here="$(cd "$(dirname "$0")" 2>/dev/null && pwd)" || here=""
if [ -n "$here" ] && [ -f "$here/../Cargo.toml" ] && command -v cargo >/dev/null; then
    cargo build --release --manifest-path "$here/../Cargo.toml"
    # Copy, so `cargo clean` never breaks the launcher.
    install -m 755 "$here/../target/release/lingua_onde" "$bin"
    install -m 644 "$here/lingua-onde.svg" "$icon"
    entry="$here/lingua-onde.desktop"
else
    if [ "$(uname -m)" != "x86_64" ]; then
        echo "Ready-made builds are for x86_64 only. Build from source instead: see the README." >&2
        exit 1
    fi
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    echo "Downloading Lingua Onde…"
    curl -fL --progress-bar -o "$tmp/lingua-onde" "https://github.com/$repo/releases/latest/download/lingua-onde-linux-x86_64"
    curl -fsSL -o "$tmp/lingua-onde.svg" "https://raw.githubusercontent.com/$repo/main/packaging/lingua-onde.svg"
    curl -fsSL -o "$tmp/lingua-onde.desktop" "https://raw.githubusercontent.com/$repo/main/packaging/lingua-onde.desktop"
    install -m 755 "$tmp/lingua-onde" "$bin"
    install -m 644 "$tmp/lingua-onde.svg" "$icon"
    entry="$tmp/lingua-onde.desktop"
fi

# Launchers do not search PATH reliably: point Exec at the installed file.
sed "s|^Exec=.*|Exec=$bin|" "$entry" > "$desktop"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$(dirname "$desktop")" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" || true

echo "Installed. Open \"Lingua Onde\" from your app menu."
if ! command -v mpv >/dev/null; then
    echo "For sound, install mpv with your package manager, for example:"
    echo "  sudo apt install mpv    (Ubuntu, Debian, Mint)"
    echo "  sudo dnf install mpv    (Fedora)"
    echo "  sudo pacman -S mpv      (Arch)"
fi
