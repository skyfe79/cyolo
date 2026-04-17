#!/usr/bin/env bash
# install.sh — thin wrapper around `cargo install --path .`
#
# Builds cyolo from the current clone and drops the binary into
# ~/.cargo/bin/cyolo. Requires Rust 1.85+ (edition 2024).
#
#   ./install.sh            # release build
#   ./install.sh --debug    # dev build (faster, larger binary)
#   ./install.sh --locked   # forward --locked to cargo install (CI-friendly)

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

MSRV_MAJOR=1
MSRV_MINOR=85

CARGO_FLAGS="--force"
PROFILE_LABEL="release"
for arg in "$@"; do
    case "$arg" in
        --debug)
            CARGO_FLAGS="$CARGO_FLAGS --debug"
            PROFILE_LABEL="debug"
            ;;
        --locked)
            CARGO_FLAGS="$CARGO_FLAGS --locked"
            ;;
        -h|--help)
            sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "install.sh: unknown argument '$arg'" >&2
            echo "Try: $0 --help" >&2
            exit 2
            ;;
    esac
done

if ! command -v cargo >/dev/null 2>&1; then
    echo "install.sh: 'cargo' not found on PATH." >&2
    echo "Install the Rust toolchain first: https://rustup.rs" >&2
    exit 1
fi

RUSTC_VERSION="$(rustc --version 2>/dev/null | awk '{print $2}')"
RUSTC_MAJOR="$(echo "$RUSTC_VERSION" | cut -d. -f1)"
RUSTC_MINOR="$(echo "$RUSTC_VERSION" | cut -d. -f2)"
if [ -z "$RUSTC_MAJOR" ] || [ -z "$RUSTC_MINOR" ]; then
    echo "install.sh: could not parse 'rustc --version' output: '$RUSTC_VERSION'" >&2
    exit 1
fi
if [ "$RUSTC_MAJOR" -lt "$MSRV_MAJOR" ] || { [ "$RUSTC_MAJOR" -eq "$MSRV_MAJOR" ] && [ "$RUSTC_MINOR" -lt "$MSRV_MINOR" ]; }; then
    echo "install.sh: Rust $MSRV_MAJOR.$MSRV_MINOR+ required, found $RUSTC_VERSION." >&2
    echo "Upgrade with: rustup update stable" >&2
    exit 1
fi

echo "==> Building cyolo ($PROFILE_LABEL) with rustc $RUSTC_VERSION"
# shellcheck disable=SC2086
cargo install --path . $CARGO_FLAGS

CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
INSTALLED="$CARGO_BIN/cyolo"
if [ ! -x "$INSTALLED" ]; then
    echo "install.sh: expected binary at $INSTALLED but none found." >&2
    echo "Check 'cargo install' output above." >&2
    exit 1
fi

echo ""
echo "==> Installed: $INSTALLED"
"$INSTALLED" --version 2>/dev/null || true

case ":$PATH:" in
    *":$CARGO_BIN:"*)
        ;;
    *)
        echo ""
        echo "Note: $CARGO_BIN is not on your PATH. Add this line to your shell rc:"
        echo "    export PATH=\"$CARGO_BIN:\$PATH\""
        ;;
esac

echo ""
echo "Done. Try: cyolo --help"
