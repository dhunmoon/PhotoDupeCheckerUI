#!/usr/bin/env bash
# Builds "Dupe Finder.app" and wraps it in a drag-to-Applications .dmg in dist/.
# Must run on macOS (uses codesign / hdiutil).
#
#   packaging/package-macos.sh aarch64-apple-darwin   -> ...-macos-apple-silicon.dmg
#   packaging/package-macos.sh x86_64-apple-darwin    -> ...-macos-intel.dmg
#
# The app is only ad-hoc signed (no Apple Developer ID), so users see a Gatekeeper
# prompt on first launch; see README "Installing" for the workaround.
set -euo pipefail

cd "$(dirname "$0")/.."
TARGET=${1:-aarch64-apple-darwin}
case "$TARGET" in
    aarch64-apple-darwin) LABEL=apple-silicon ;;
    x86_64-apple-darwin) LABEL=intel ;;
    *) echo "unsupported target: $TARGET" >&2; exit 1 ;;
esac
VERSION=$(cargo metadata --no-deps --format-version 1 |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
NAME=dupe-finder-$VERSION-macos-$LABEL
DIST=dist

export MACOSX_DEPLOYMENT_TARGET=11.0
cargo build --release --locked --target "$TARGET"

STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
APP="$STAGE/Dupe Finder.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "target/$TARGET/release/dupe_finder" "$APP/Contents/MacOS/"
cp packaging/macos/AppIcon.icns "$APP/Contents/Resources/"
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist >"$APP/Contents/Info.plist"
# Apple Silicon refuses to run unsigned code; an ad-hoc signature is enough.
codesign --force --deep --sign - "$APP"
ln -s /Applications "$STAGE/Applications"

mkdir -p "$DIST"
hdiutil create -volname "Dupe Finder" -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov \
    "$DIST/$NAME.dmg"
ls -l "$DIST"
