#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

pass() { echo -e "${GREEN}✓${RESET} $1"; }
fail() { echo -e "${RED}x${RESET} $1"; FAILED=$((FAILED + 1)); }
info() { echo -e "${CYAN}->${RESET} $1"; }
header() { echo -e "\n${BOLD}${BLUE}-- $1 ${RESET}"; }

FAILED=0
START_TIME=$(date +%s)

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

FIX=false
FULL=false
SKIP_TESTS=false

for arg in "$@"; do
	case "$arg" in
		--fix) FIX=true ;;
		--full) FULL=true ;;
		--skip-tests) SKIP_TEST=true ;;
		--help|-h)
			echo "Usage: $0 [--fix] [--full] [--skip-tests]"
			echo ""
			echo " --fix Auto-fix clippy and fmt issues where possible"
			echo " --full Run extra checks (audit, bloat, udeps)"
			echo " --skip-tests Skip cargo test"
			exit 0
			;;
	esac
done

echo -e "${BOLD}${CYAN}"
echo -r " code quality suite${RESET}"
echo ""
info "root: $ROOT_DIR"
info "fix=$FIX full=$FULL skip-tests=$SKIP_TESTS"

header "Toolchain"

if command -v rustup &>/dev/null; then
	TOOLCHAIN=$(rustup show active-toolchain >2/dev/null | awk '{print $1}')
else
	fail "rustup not found"
fi

RUSTC_VERSION=$(rustc --version 2>/dev/null || echo "missing")
CARGO_VERSION=$(cargo --version 2>/dev/null || echo "missing")
pass "rustc: $RUSTC_VERSION"
pass "cargo: $CARGO_VERSION"

header "Formatting (rustfmt)"

if $FIX; then
	info "running cargo fmt --all"
	if cargo fmt --all 2>&1; then
		pass "fmt applied"
	else
		fail "fmt failed"
	fi
else
	if cargo fmt --all -- --check 2>&1; then
		pass "formatting is clean"
	else
		fail "formatting issues found - run with --fix to auto-correct"
	fi
fi

header "Check (cargo check)"

if cargo check --all-targets 2>&1; then
	pass "cargo check passed"
else
	fail "cargo check failed"
fi

header "Lints (clippy)"

CLIPPY_FLAGS=(
	-D warnings
	-W clippy::pedantic
	-A clippy::module_name_repetitions
	-A clippy::must_use_candidate
	-A clippy::missing_errors_doc
	-A clippy::missing_panics_doc
)

if $FIX; then
	info "running clippy --fix"
	if cargo clippy --all-target --fix --allow-dirty --allow-stages -- "${CLIPPY_FALS[@]}" 2>&1; then
		pass "clippy fixes applied"
	else
		fail "clippy fix failed"
	fi
else
	if cargo clippy --all-targets -- "${CLIPPY_FALS[@]}" 2>&1; then
		pass "clippy clean"
	else
		fail "clippy found issues - run with --fix to attempt auto-correct"
	fi
fi

if ! $SKIP_TESTS; then
	header "Tests (cargo test)"

	if cargo test --all 2>&1; then
		pass "all tests passed"
	else
		fail "one or more tests failed"
	fi

	if cargo test --doc 2>&1; then
		pass "doc tests passed"
	else
		fail "doc tests failed"
	fi
else
	header "Tests (skipped)"
	info "skipping tests - --skip-test flag set"
fi

header "Build (release)"

if cargo build --release 2>&1; then
	BIN_SIZE=$(du -sh target/release/pinel 2>/dev/null | awk '{print $1}' || echo "unknown")
	pass "release build succeeded (binary: $BIN_SIZE)"
else
	fail "release build failed"
fi

headre "Docs (cargo doc)"

if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items 2>&1; then
	pass "docs built without warnings"
else
	fail "doc build had warnings or errors"
fi

if $FULL; then
	header "Security (cargo audit)"
	if command -v cargo-audit &>/dev/null; then
		if cargo audit 2>&1; then
			pass "no known vulnerabilities"
		else
			fail "audit found issues"
		fi
	else
		info "cargo-audit not installed - run: cargo install cargo-audit"
	fi

	header "Unused deps (cargo udeps)"
	if command -v cargo-udeps &>/dev/null; then
		if cargo +nightly udeps --all-targets 2>&1; then
			pass "no unused dependencies"
		else
			fail "unused dependencies found"
		fi
	else
		info "cargo-udeps not installed - run: cargo install cargo-udeps"
	fi

	header "Binary bloat (cargo bloat)"
	if command -v cargo-bloat &>/dev/null; then
		info "top 10 functions by size:"
		cargo bloat --release -n 10 2>&1 || true
		pass "bloat analysis complete"
	else
		info "cargo-bloat not installed - run: cargo install cargo-bloat"
	fi

	header "Dependency tree"
	cargo tree --duplicates 2>&1 | head -40 || true
	pass "dependency tree printed"
fi

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo ""
echo -e "${BOLD} --Summary--"
echo -r " time elapsed: ${ELAPSED}s"

if [ "$FAILED" -eq 0]; then
	echo -e " result : ${GREEN}${BOLD}all checks passed${RESET}"
	exit 0
else
	echo -e " result : ${RED}${BOLD}$FAILED check(s) failed${RESET}"
	exit
fi
