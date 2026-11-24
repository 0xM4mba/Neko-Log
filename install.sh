#!/usr/bin/env bash
set -e

echo "🌸 Installing NekoLog (ログニャン) - Anime Linux Log Viewer..."

BIN_SRC="$(dirname "$0")/target/release/nekolog"
if [ ! -f "$BIN_SRC" ]; then
    echo "⚠️  Release binary not found. Building with cargo..."
    cargo build --release
fi

TARGET_BIN_DIR="$HOME/.local/bin"
TARGET_APP_DIR="$HOME/.local/share/applications"

mkdir -p "$TARGET_BIN_DIR"
mkdir -p "$TARGET_APP_DIR"

cp "$BIN_SRC" "$TARGET_BIN_DIR/nekolog"
chmod +x "$TARGET_BIN_DIR/nekolog"

sed "s|Exec=nekolog|Exec=$TARGET_BIN_DIR/nekolog|g" "$(dirname "$0")/nekolog.desktop" > "$TARGET_APP_DIR/nekolog.desktop"

echo "✨ NekoLog installed successfully!"
echo "📍 Binary: $TARGET_BIN_DIR/nekolog"
echo "🖥️  Desktop Entry: $TARGET_APP_DIR/nekolog.desktop"
echo "🌸 Run 'nekolog' from terminal or launch it from your desktop application menu!"
