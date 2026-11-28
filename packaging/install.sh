#!/bin/sh
# Installs Marea for the current user (Linux) and adds it to the app launcher.
#
#   From the internet (no Rust needed):
#     curl -fsSL https://raw.githubusercontent.com/Matteo404404/marea/main/packaging/install.sh | sh
#   From a clone (builds from source with cargo):
#     ./packaging/install.sh
#   Remove it (your progress stays):
#     ./packaging/install.sh --uninstall    or    curl ... | sh -s -- --uninstall
#
# Run it again to update.
set -eu

repo="Matteo404404/marea"
bin="$HOME/.local/bin/marea"
desktop="$HOME/.local/share/applications/marea.desktop"
icon="$HOME/.local/share/icons/hicolor/scalable/apps/marea.svg"
# the same files from before the rename to Marea, removed on install
old="$HOME/.local/bin/lingua-onde $HOME/.local/share/applications/lingua-onde.desktop $HOME/.local/share/icons/hicolor/scalable/apps/lingua-onde.svg"

if [ "${1:-}" = "--uninstall" ]; then
    # shellcheck disable=SC2086
    rm -f "$bin" "$desktop" "$icon" $old
    echo "Removed Marea. Your progress stays in ~/.local/share/marea/."
    exit 0
fi

mkdir -p "$(dirname "$bin")" "$(dirname "$desktop")" "$(dirname "$icon")"
here="$(cd "$(dirname "$0")" 2>/dev/null && pwd)" || here=""
if [ -n "$here" ] && [ -f "$here/../Cargo.toml" ] && command -v cargo >/dev/null; then
    cargo build --release --manifest-path "$here/../Cargo.toml"
    # Copy, so `cargo clean` never breaks the launcher.
    install -m 755 "$here/../target/release/marea" "$bin"
    install -m 644 "$here/marea.svg" "$icon"
    entry="$here/marea.desktop"
else
    if [ "$(uname -m)" != "x86_64" ]; then
        echo "Ready-made builds are for x86_64 only. Build from source instead: see the README." >&2
        exit 1
    fi
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    echo "Downloading Marea…"
    url="https://github.com/$repo/releases/latest/download/marea-linux-x86_64"
    if ! curl -fL --progress-bar -o "$tmp/marea" "$url"; then
        echo "Could not download the app. That file exists only after a release is published." >&2
        echo "Until then, clone the repo and run ./packaging/install.sh (that build needs Rust)." >&2
        exit 1
    fi
    # a GitHub error page is small; the app is tens of megabytes
    size=$(wc -c < "$tmp/marea")
    if [ "$size" -lt 1000000 ]; then
        echo "The download is too small to be the app. Check the latest release on GitHub." >&2
        exit 1
    fi
    curl -fsSL -o "$tmp/marea.svg" "https://raw.githubusercontent.com/$repo/main/packaging/marea.svg"
    curl -fsSL -o "$tmp/marea.desktop" "https://raw.githubusercontent.com/$repo/main/packaging/marea.desktop"
    install -m 755 "$tmp/marea" "$bin"
    install -m 644 "$tmp/marea.svg" "$icon"
    entry="$tmp/marea.desktop"
fi

# shellcheck disable=SC2086
rm -f $old
# Launchers do not search PATH reliably: point Exec at the installed file.
sed "s|^Exec=.*|Exec=$bin|" "$entry" > "$desktop"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$(dirname "$desktop")" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$HOME/.local/share/icons/hicolor" || true

echo "Installed. Open \"Marea\" from your app menu, or run $bin"
# the voices play through PipeWire, PulseAudio or ALSA; mpv only adds pause and rewind
if ! command -v mpv >/dev/null && ! command -v pw-play >/dev/null && ! command -v paplay >/dev/null && ! command -v aplay >/dev/null; then
    echo "No sound player found. Install mpv with your package manager, for example:"
    echo "  sudo apt install mpv    (Ubuntu, Debian, Mint)"
    echo "  sudo dnf install mpv    (Fedora)"
    echo "  sudo pacman -S mpv      (Arch)"
fi
# the window loads these by name and the app needs ALSA (the microphone); a desktop usually has them, a minimal install may not
if command -v ldconfig >/dev/null 2>&1; then
    libs=$(ldconfig -p 2>/dev/null || true)
    missing=""
    for lib in libvulkan.so.1 libxkbcommon.so.0 libwayland-client.so.0 libX11.so.6 libasound.so.2; do
        printf '%s\n' "$libs" | grep -q "$lib" || missing="$missing $lib"
    done
    if [ -n "$missing" ]; then
        echo "The window may not open. Missing:$missing"
        echo "On Ubuntu: sudo apt install mesa-vulkan-drivers libxkbcommon0 libwayland-client0 libx11-6 libasound2"
    fi
fi
