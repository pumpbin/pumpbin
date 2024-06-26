#!/bin/bash

export MACOSX_DEPLOYMENT_TARGET="11.0"
cargo build --release --target=x86_64-apple-darwin
cargo build --release --target=aarch64-apple-darwin

TARGET="pumpbin"
ASSETS_DIR="assets"
RELEASE_DIR="target/release"
APP_NAME="PumpBin.app"
APP_TEMPLATE="$ASSETS_DIR/macos/$APP_NAME"
APP_TEMPLATE_PLIST="$APP_TEMPLATE/Contents/Info.plist"
APP_DIR="$RELEASE_DIR/macos-pumpbin"
APP_BINARY="$RELEASE_DIR/$TARGET"
APP_BINARY_DIR="$APP_DIR/$APP_NAME/Contents/MacOS"
APP_EXTRAS_DIR="$APP_DIR/$APP_NAME/Contents/Resources"

TARGET_MAKER="maker"
APP_NAME_MAKER="PumpBin-Maker.app"
APP_TEMPLATE_MAKER="$ASSETS_DIR/macos/$APP_NAME_MAKER"
APP_TEMPLATE_PLIST_MAKER="$APP_TEMPLATE_MAKER/Contents/Info.plist"
APP_DIR_MAKER="$RELEASE_DIR/macos-maker"
APP_BINARY_MAKER="$RELEASE_DIR/$TARGET_MAKER"
APP_BINARY_DIR_MAKER="$APP_DIR_MAKER/$APP_NAME_MAKER/Contents/MacOS"
APP_EXTRAS_DIR_MAKER="$APP_DIR_MAKER/$APP_NAME_MAKER/Contents/Resources"

DMG_NAME="PumpBin.dmg"
DMG_DIR="$RELEASE_DIR/macos-pumpbin"

DMG_NAME_MAKER="PumpBin-Maker.dmg"
DMG_DIR_MAKER="$RELEASE_DIR/macos-maker"

VERSION=$(cat VERSION)
BUILD=$(git describe --always --dirty --exclude='*')

# update version and build
sed -i '' -e "s/{{ VERSION }}/$VERSION/g" "$APP_TEMPLATE_PLIST"
sed -i '' -e "s/{{ BUILD }}/$BUILD/g" "$APP_TEMPLATE_PLIST"

sed -i '' -e "s/{{ VERSION }}/$VERSION/g" "$APP_TEMPLATE_PLIST_MAKER"
sed -i '' -e "s/{{ BUILD }}/$BUILD/g" "$APP_TEMPLATE_PLIST_MAKER"

# build binary
lipo "target/x86_64-apple-darwin/release/$TARGET" "target/aarch64-apple-darwin/release/$TARGET" -create -output "$APP_BINARY"

# build app
mkdir -p "$APP_BINARY_DIR"
mkdir -p "$APP_EXTRAS_DIR"
cp -fRp "$APP_TEMPLATE" "$APP_DIR"
cp -fp "$APP_BINARY" "$APP_BINARY_DIR"
touch -r "$APP_BINARY" "$APP_DIR/$APP_NAME"
echo "Created '$APP_NAME' in '$APP_DIR'"

# package dmg
echo "Packing disk image..."
ln -sf /Applications "$DMG_DIR/Applications"
hdiutil create "$DMG_DIR/$DMG_NAME" -volname "PumpBin" -fs HFS+ -srcfolder "$APP_DIR" -ov -format UDZO
echo "Packed '$APP_NAME' in '$APP_DIR'"

lipo "target/x86_64-apple-darwin/release/$TARGET_MAKER" "target/aarch64-apple-darwin/release/$TARGET_MAKER" -create -output "$APP_BINARY_MAKER"

mkdir -p "$APP_BINARY_DIR_MAKER"
mkdir -p "$APP_EXTRAS_DIR_MAKER"
cp -fRp "$APP_TEMPLATE_MAKER" "$APP_DIR_MAKER"
cp -fp "$APP_BINARY_MAKER" "$APP_BINARY_DIR_MAKER"
touch -r "$APP_BINARY_MAKER" "$APP_DIR_MAKER/$APP_NAME_MAKER"
echo "Created '$APP_NAME_MAKER' in '$APP_DIR_MAKER'"

ln -sf /Applications "$DMG_DIR_MAKER/Applications"
hdiutil create "$DMG_DIR_MAKER/$DMG_NAME_MAKER" -volname "PumpBin-Maker" -fs HFS+ -srcfolder "$APP_DIR_MAKER" -ov -format UDZO
echo "Packed '$APP_NAME_MAKER' in '$APP_DIR_MAKER'"
