#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "🌸 Packaging neko-log-v1.AppImage..."

# 1. Ensure release binary is compiled
if [ ! -f "target/release/nekolog" ]; then
    echo "🔨 Building release binary..."
    cargo build --release
fi

# 2. Setup AppDir structure
APPDIR="$DIR/AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"

# 3. Copy binary
cp "target/release/nekolog" "$APPDIR/usr/bin/nekolog"
chmod +x "$APPDIR/usr/bin/nekolog"

# 4. Copy Desktop Entry
cp "nekolog.desktop" "$APPDIR/nekolog.desktop"

# 5. Copy Icon
if [ -f "nekolog.png" ]; then
    cp "nekolog.png" "$APPDIR/nekolog.png"
    cp "nekolog.png" "$APPDIR/.DirIcon"
fi

# 6. Create AppRun script
cat << 'EOF' > "$APPDIR/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/nekolog" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# 7. Ensure appimagetool and runtime exist in .tools
TOOLS_DIR="$DIR/.tools"
mkdir -p "$TOOLS_DIR"

if [ ! -d "$TOOLS_DIR/appimagetool-extracted" ]; then
    echo "⬇️ Downloading appimagetool..."
    curl -L -o "$TOOLS_DIR/appimagetool.AppImage" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$TOOLS_DIR/appimagetool.AppImage"
    (cd "$TOOLS_DIR" && ./appimagetool.AppImage --appimage-extract && mv squashfs-root appimagetool-extracted && rm -f appimagetool.AppImage)
fi

if [ ! -f "$TOOLS_DIR/runtime-x86_64" ]; then
    echo "⬇️ Downloading runtime-x86_64..."
    curl -L -o "$TOOLS_DIR/runtime-x86_64" https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64
fi

APPIMAGETOOL="$TOOLS_DIR/appimagetool-extracted/AppRun"
RUNTIME_FILE="$TOOLS_DIR/runtime-x86_64"

ARCH=x86_64 "$APPIMAGETOOL" --runtime-file "$RUNTIME_FILE" "$APPDIR" "$DIR/neko-log-v1.AppImage"

echo "✨ Successfully generated: $DIR/neko-log-v1.AppImage"
chmod +x "$DIR/neko-log-v1.AppImage"
