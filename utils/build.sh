#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

BIN_NAME="pinel"
OUT_DIR="$ROOT_DIR/dist"
mkdir -p "$OUT_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="linux-x86_64" ;;
      aarch64|arm64) TARGET="linux-arm64" ;;
      *) echo "Unsupported Linux architecture: $ARCH" >&2; exit 1 ;;
    esac
    SRC="target/release/$BIN_NAME"
    ;;
  Darwin)
    case "$ARCH" in
      arm64) TARGET="macos-arm64" ;;
      x86_64) TARGET="macos-x86_64" ;;
      *) echo "Unsupported macOS architecture: $ARCH" >&2; exit 1 ;;
    esac
    SRC="target/release/$BIN_NAME"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    case "$ARCH" in
      x86_64|amd64) TARGET="windows-x86_64" ;;
      aarch64|arm64) TARGET="windows-arm64" ;;
      *) echo "Unsupported Windows architecture: $ARCH" >&2; exit 1 ;;
    esac
    SRC="target/release/${BIN_NAME}.exe"
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

cargo build --release --bin "$BIN_NAME"

if [[ "$OS" == "MINGW"* || "$OS" == "MSYS"* || "$OS" == "CYGWIN"* ]]; then
  DEST_DIR="$OUT_DIR/${BIN_NAME}-${TARGET}"
  rm -rf "$DEST_DIR"
  mkdir -p "$DEST_DIR"
  cp "$SRC" "$DEST_DIR/${BIN_NAME}.exe"
  (cd "$OUT_DIR" && zip -qr "${BIN_NAME}-${TARGET}.zip" "${BIN_NAME}-${TARGET}")
  echo "Created $OUT_DIR/${BIN_NAME}-${TARGET}.zip"
else
  DEST_DIR="$OUT_DIR/${BIN_NAME}-${TARGET}"
  rm -rf "$DEST_DIR"
  mkdir -p "$DEST_DIR"
  cp "$SRC" "$DEST_DIR/${BIN_NAME}"
  (cd "$OUT_DIR" && tar -czf "${BIN_NAME}-${TARGET}.tar.gz" "${BIN_NAME}-${TARGET}")
  echo "Created $OUT_DIR/${BIN_NAME}-${TARGET}.tar.gz"
fi
