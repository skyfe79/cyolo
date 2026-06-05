#!/usr/bin/env bash
# install.sh — thin wrapper around `cargo install --path .`
#
# Builds cyolo from the current clone and drops the binary into
# ~/.cargo/bin/cyolo. Requires Rust 1.85+ (edition 2024).
#
#   ./install.sh                  # release build
#   ./install.sh --debug          # dev build (faster, larger binary)
#   ./install.sh --locked         # forward --locked to cargo install (CI-friendly)
#   ./install.sh --no-modify-path # don't touch shell rc files (CI-friendly)
#
# When ~/.cargo/bin is not already on PATH, the installer appends an
# `export PATH=...` line to your shell's rc file (.zshrc / .bashrc /
# config.fish / .profile). The edit is idempotent and can be skipped
# with --no-modify-path.

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

MSRV_MAJOR=1
MSRV_MINOR=85

CARGO_FLAGS="--force"
PROFILE_LABEL="release"
MODIFY_PATH=1
for arg in "$@"; do
    case "$arg" in
        --debug)
            CARGO_FLAGS="$CARGO_FLAGS --debug"
            PROFILE_LABEL="debug"
            ;;
        --locked)
            CARGO_FLAGS="$CARGO_FLAGS --locked"
            ;;
        --no-modify-path)
            MODIFY_PATH=0
            ;;
        -h|--help)
            sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'
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

# Append a PATH export to the user's shell rc when needed. Idempotent:
# re-running won't add duplicate lines. fish uses its own syntax.
configure_shell_path() {
    bin_dir="$1"
    shell_name="$(basename "${SHELL:-/bin/sh}")"

    case "$shell_name" in
        zsh)
            rc_file="${ZDOTDIR:-$HOME}/.zshrc"
            path_line="export PATH=\"$bin_dir:\$PATH\""
            ;;
        bash)
            # macOS login shells read .bash_profile; Linux reads .bashrc.
            if [ "$(uname -s)" = "Darwin" ] && [ -f "$HOME/.bash_profile" ]; then
                rc_file="$HOME/.bash_profile"
            else
                rc_file="$HOME/.bashrc"
            fi
            path_line="export PATH=\"$bin_dir:\$PATH\""
            ;;
        fish)
            rc_file="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
            path_line="fish_add_path \"$bin_dir\""
            ;;
        *)
            rc_file="$HOME/.profile"
            path_line="export PATH=\"$bin_dir:\$PATH\""
            ;;
    esac

    mkdir -p "$(dirname "$rc_file")"
    [ -f "$rc_file" ] || : > "$rc_file"

    # Already present (line or the bin dir referenced) -> nothing to do.
    if grep -qF "$bin_dir" "$rc_file" 2>/dev/null; then
        echo ""
        echo "==> $bin_dir already referenced in $rc_file"
        return 0
    fi

    {
        echo ""
        echo "# Added by cyolo install.sh"
        echo "$path_line"
    } >> "$rc_file"

    echo ""
    echo "==> Added $bin_dir to PATH in $rc_file"
    echo "    Restart your shell or run: source $rc_file"
}

case ":$PATH:" in
    *":$CARGO_BIN:"*)
        ;;
    *)
        if [ "$MODIFY_PATH" -eq 1 ]; then
            configure_shell_path "$CARGO_BIN"
        else
            echo ""
            echo "Note: $CARGO_BIN is not on your PATH. Add this line to your shell rc:"
            echo "    export PATH=\"$CARGO_BIN:\$PATH\""
        fi
        ;;
esac

echo ""
echo "Done. Try: cyolo --help"
