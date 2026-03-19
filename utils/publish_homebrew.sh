#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

REPO="${PINEL_REPO:-kashsuks/Pinel}"
FORMULA_NAME="${PINEL_HOMEBREW_FORMULA_NAME:-pinel}"
CLASS_NAME="${PINEL_HOMEBREW_CLASS_NAME:-Pinel}"
DESCRIPTION="${PINEL_HOMEBREW_DESC:-A lightweight editor built with rust}"
HOMEPAGE="${PINEL_HOMEBREW_HOMEPAGE:-https://github.com/${REPO}}"
LICENSE_ID="${PINEL_HOMEBREW_LICENSE:-GPL-3.0-only}"
MACOS_X86_ASSET="${PINEL_HOMEBREW_X86_ASSET:-pinel-macos-x86_64}"
MACOS_ARM_ASSET="${PINEL_HOMEBREW_ARM_ASSET:-pinel-macos-arm64}"
TAP_DIR="${PINEL_HOMEBREW_TAP_DIR:-}"
COMMIT_CHANGES="false"
PUSH_CHANGES="false"
VERSION=""
TAG=""

log() {
  printf '%s\n' "$*" >&2
}

fail() {
  log "error: $*"
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

usage() {
  cat <<'EOF'
Usage:
  utils/publish_homebrew.sh --version <version> --tap-dir <path> [options]

Options:
  --version <version>     Release version without the leading v (required unless --tag is used)
  --tag <tag>             Release tag, for example v0.4.3
  --tap-dir <path>        Local checkout of the Homebrew tap repo
  --formula-name <name>   Formula file name, defaults to pinel
  --class-name <name>     Ruby class name, defaults to Pinel
  --x86-asset <name>      macOS Intel GitHub release asset, defaults to pinel-macos-x86_64
  --arm-asset <name>      macOS Apple Silicon release asset, defaults to pinel-macos-arm64
  --repo <owner/repo>     GitHub repository, defaults to kashsuks/Pinel
  --desc <text>           Formula description
  --homepage <url>        Formula homepage
  --license <id>          SPDX license identifier
  --commit                Create a git commit in the tap repo
  --push                  Push the tap repo after committing
  --help                  Show this help text

Environment overrides:
  PINEL_REPO
  PINEL_HOMEBREW_FORMULA_NAME
  PINEL_HOMEBREW_CLASS_NAME
  PINEL_HOMEBREW_DESC
  PINEL_HOMEBREW_HOMEPAGE
  PINEL_HOMEBREW_LICENSE
  PINEL_HOMEBREW_X86_ASSET
  PINEL_HOMEBREW_ARM_ASSET
  PINEL_HOMEBREW_TAP_DIR

Notes:
  - This script generates a formula with separate Intel and Apple Silicon URLs and sha256 values.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --tag)
      TAG="${2:-}"
      shift 2
      ;;
    --tap-dir)
      TAP_DIR="${2:-}"
      shift 2
      ;;
    --formula-name)
      FORMULA_NAME="${2:-}"
      shift 2
      ;;
    --class-name)
      CLASS_NAME="${2:-}"
      shift 2
      ;;
    --x86-asset)
      MACOS_X86_ASSET="${2:-}"
      shift 2
      ;;
    --arm-asset)
      MACOS_ARM_ASSET="${2:-}"
      shift 2
      ;;
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --desc)
      DESCRIPTION="${2:-}"
      shift 2
      ;;
    --homepage)
      HOMEPAGE="${2:-}"
      shift 2
      ;;
    --license)
      LICENSE_ID="${2:-}"
      shift 2
      ;;
    --commit)
      COMMIT_CHANGES="true"
      shift
      ;;
    --push)
      COMMIT_CHANGES="true"
      PUSH_CHANGES="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$VERSION" && -n "$TAG" ]]; then
  VERSION="${TAG#v}"
fi

if [[ -z "$TAG" && -n "$VERSION" ]]; then
  TAG="v${VERSION}"
fi

[[ -n "$VERSION" ]] || fail "--version or --tag is required"
[[ -n "$TAP_DIR" ]] || fail "--tap-dir is required"

need_cmd curl
need_cmd shasum
need_cmd git

[[ -d "$TAP_DIR/.git" ]] || fail "tap dir must be a git checkout: $TAP_DIR"

FORMULA_DIR="$TAP_DIR/Formula"
FORMULA_PATH="$FORMULA_DIR/${FORMULA_NAME}.rb"
X86_ASSET_URL="https://github.com/${REPO}/releases/download/${TAG}/${MACOS_X86_ASSET}"
ARM_ASSET_URL="https://github.com/${REPO}/releases/download/${TAG}/${MACOS_ARM_ASSET}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
X86_ASSET_PATH="$TMP_DIR/${MACOS_X86_ASSET}"
ARM_ASSET_PATH="$TMP_DIR/${MACOS_ARM_ASSET}"

log "fetching ${X86_ASSET_URL}"
curl -fsSL "$X86_ASSET_URL" -o "$X86_ASSET_PATH"
X86_SHA256="$(shasum -a 256 "$X86_ASSET_PATH" | awk '{print $1}')"

log "fetching ${ARM_ASSET_URL}"
curl -fsSL "$ARM_ASSET_URL" -o "$ARM_ASSET_PATH"
ARM_SHA256="$(shasum -a 256 "$ARM_ASSET_PATH" | awk '{print $1}')"

mkdir -p "$FORMULA_DIR"

cat > "$FORMULA_PATH" <<EOF
class ${CLASS_NAME} < Formula
  desc "${DESCRIPTION}"
  homepage "${HOMEPAGE}"
  version "${VERSION}"
  license "${LICENSE_ID}"

  on_intel do
    url "${X86_ASSET_URL}"
    sha256 "${X86_SHA256}"
  end

  on_arm do
    url "${ARM_ASSET_URL}"
    sha256 "${ARM_SHA256}"
  end

  def install
    if Hardware::CPU.intel?
      bin.install "${MACOS_X86_ASSET}" => "${FORMULA_NAME}"
    else
      bin.install "${MACOS_ARM_ASSET}" => "${FORMULA_NAME}"
    end
  end

  test do
    system "#{bin}/${FORMULA_NAME}", "--help"
  end
end
EOF

log "wrote ${FORMULA_PATH}"

if [[ "$COMMIT_CHANGES" == "true" ]]; then
  git -C "$TAP_DIR" add "Formula/${FORMULA_NAME}.rb"

  if git -C "$TAP_DIR" diff --cached --quiet; then
    log "no tap changes to commit"
  else
    git -C "$TAP_DIR" commit -m "pinel ${VERSION}"
    log "committed formula update in ${TAP_DIR}"
  fi
fi

if [[ "$PUSH_CHANGES" == "true" ]]; then
  git -C "$TAP_DIR" push
  log "pushed tap changes"
fi

log "homebrew formula is ready at ${FORMULA_PATH}"
