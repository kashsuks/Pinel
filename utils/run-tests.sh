#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
RESET='\033[0m'

# stored inside the git dir itself (not the repo root)
MARKER_FILE="$(git rev-parse --git-dir)/PINEL_LAST_TESTED_COMMIT"

echo -e "${CYAN}->${RESET} running cargo test --all"

if cargo test --all; then
    CURRENT_HASH="$(git rev-parse HEAD)"
    echo "$CURRENT_HASH" > "$MARKER_FILE"
    echo -e "${GREEN}✓${RESET} test passed - recorded commit ${CURRENT_HASH:0:8} as tested"
    exit 0
else
    echo -e "${RED}x${RESET} tests failed - not recording this commit as tested"
    exit 1
fi
