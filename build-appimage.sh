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

# 7. Package using appimagetool
APPIMAGETOOL="/tmp/appimagetool-extracted/AppRun"
if [ ! -f "$APPIMAGETOOL" ]; then
    if command -v appimagetool >/dev/null 2>&1; then
        APPIMAGETOOL="appimagetool"
    else
        echo "❌ appimagetool not found. Please install appimagetool or download it."
        exit 1
    fi
fi

EXTRA_ARGS=""
if [ -f "/tmp/runtime-x86_64" ]; then
    EXTRA_ARGS="--runtime-file /tmp/runtime-x86_64"
fi

ARCH=x86_64 "$APPIMAGETOOL" $EXTRA_ARGS "$APPDIR" "$DIR/neko-log-v1.AppImage"

echo "✨ Successfully generated: $DIR/neko-log-v1.AppImage"
chmod +x "$DIR/neko-log-v1.AppImage"
