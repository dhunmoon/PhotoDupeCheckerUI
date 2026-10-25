#!/usr/bin/env bash
# Builds the Linux release artifacts into dist/:
#   dupe-finder-<version>-linux-x86_64.tar.gz   portable binary (any distro, glibc >= 2.28)
#   dupe-finder-<version>-linux-x86_64.AppImage single-file app (any distro)
#   dupe-finder_<version>_amd64.deb             Debian / Ubuntu / Mint / Pop!_OS
#   dupe-finder-<version>-1.x86_64.rpm          Fedora / RHEL / openSUSE
#
# Needs: cargo-zigbuild (+ zig), cargo-deb, cargo-generate-rpm, curl.
# The binary is linked against glibc 2.28 via zig so it runs on older distros too.
set -euo pipefail

cd "$(dirname "$0")/.."
VERSION=$(cargo metadata --no-deps --format-version 1 |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
TARGET=x86_64-unknown-linux-gnu
BIN=target/$TARGET/release/dupe_finder
DIST=dist
NAME=dupe-finder-$VERSION-linux-x86_64

mkdir -p "$DIST"

if [[ "${SKIP_BUILD:-}" != 1 ]]; then
    cargo zigbuild --release --locked --target "$TARGET.2.28"
fi

# --- tarball ---
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/$NAME"
cp "$BIN" packaging/linux/dupe-finder.desktop packaging/linux/dupe-finder.png \
    README.md LICENSE "$STAGE/$NAME/"
tar -C "$STAGE" -czf "$DIST/$NAME.tar.gz" "$NAME"

# --- .deb / .rpm (configured in Cargo.toml [package.metadata.*]) ---
cargo deb --no-build --no-strip --target "$TARGET" --output "$DIST/"
cargo generate-rpm --target "$TARGET" --output "$DIST/"

# --- AppImage ---
APPDIR=$STAGE/AppDir
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps"
cp "$BIN" "$APPDIR/usr/bin/"
cp packaging/linux/dupe-finder.desktop "$APPDIR/usr/share/applications/"
cp packaging/linux/dupe-finder.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/"
cp packaging/linux/dupe-finder.desktop packaging/linux/dupe-finder.png "$APPDIR/"
ln -s usr/bin/dupe_finder "$APPDIR/AppRun"

TOOL=${APPIMAGETOOL:-$STAGE/appimagetool}
if [[ ! -x "$TOOL" ]]; then
    curl -sSfL -o "$TOOL" \
        https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$TOOL"
fi
# Extract-and-run avoids needing FUSE on the build machine (e.g. CI containers).
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$TOOL" --no-appstream "$APPDIR" "$DIST/$NAME.AppImage"

ls -l "$DIST"
