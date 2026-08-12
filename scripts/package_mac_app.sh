#!/bin/sh
set -eu

PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
APP_DIR=${1:-"$PROJECT_ROOT/target/release/bundle/osx/Tweet Scrolls.app"}
MACOS_DIR="$APP_DIR/Contents/MacOS"

cd "$PROJECT_ROOT"
cargo build --release --features gpui-app --bin tweet-scrolls-mac

mkdir -p "$MACOS_DIR"
install -m 755 \
    "$PROJECT_ROOT/target/release/tweet-scrolls-mac" \
    "$MACOS_DIR/tweet-scrolls-mac"
install -m 644 \
    "$PROJECT_ROOT/packaging/macos/Info.plist" \
    "$APP_DIR/Contents/Info.plist"

printf '%s\n' "$APP_DIR"
