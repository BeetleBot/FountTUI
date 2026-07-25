#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ARTIFACTS_DIR="$PROJECT_ROOT/Release/artifacts"

VERSION=$(grep -oP '^version = "\K[^"]+' "$PROJECT_ROOT/Cargo.toml" | head -1)
echo "==> Building Fount v$VERSION (Linux Tarball)"

echo ""
echo -n "Enter version number (current is $VERSION, press Enter to keep): "
read -r NEW_VERSION
if [ -n "$NEW_VERSION" ]; then
    VERSION="$NEW_VERSION"
    echo "==> Updating version to $VERSION"
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$PROJECT_ROOT/Cargo.toml"
    echo "==> Version updated"
fi

echo "==> Building Rust binary..."
cd "$PROJECT_ROOT"
cargo build --release

echo "==> Packaging tarball"
TARBALL_DIR="$PROJECT_ROOT/target/tarball"
mkdir -p "$TARBALL_DIR/usr/share/fount"
mkdir -p "$TARBALL_DIR/usr/share/applications"
mkdir -p "$TARBALL_DIR/usr/share/icons/hicolor/256x256/apps"

cp "$PROJECT_ROOT/target/release/fount" "$TARBALL_DIR/usr/share/fount/"
cp "$PROJECT_ROOT/assets/linux/fount.desktop" "$TARBALL_DIR/usr/share/applications/"
cp "$PROJECT_ROOT/assets/icons/FountTUI_Logo.png" "$TARBALL_DIR/usr/share/icons/hicolor/256x256/apps/fount.png"

cp "$PROJECT_ROOT/assets/linux/install.sh" "$TARBALL_DIR/"
cp "$PROJECT_ROOT/assets/linux/uninstall.sh" "$TARBALL_DIR/"
chmod +x "$TARBALL_DIR/install.sh" "$TARBALL_DIR/uninstall.sh"

TARBALL_FILE="Fount-Linux-x64-$VERSION.tar.gz"
mkdir -p "$ARTIFACTS_DIR"
cd "$TARBALL_DIR"
tar -czf "$ARTIFACTS_DIR/$TARBALL_FILE" .

rm -rf "$TARBALL_DIR"

echo "==> Done! Artifact:"
echo "  $ARTIFACTS_DIR/$TARBALL_FILE"
