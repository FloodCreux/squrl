#!/bin/sh
set -e

usage() {
    echo "Usage: install.sh [OPTIONS]"
    echo ""
    echo "Build and install squrl from source."
    echo ""
    echo "Options:"
    echo "  --prefix=<path>  Installation prefix (default: \$HOME/.local or \$PREFIX)"
    echo "  --uninstall      Remove installed files"
    echo "  --help           Show this help message"
}

PREFIX="${PREFIX:-$HOME/.local}"
UNINSTALL=0

for arg in "$@"; do
    case "$arg" in
        --prefix=*)
            PREFIX="${arg#--prefix=}"
            ;;
        --uninstall)
            UNINSTALL=1
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            usage
            exit 1
            ;;
    esac
done

BIN_DIR="$PREFIX/bin"
MAN_DIR="$PREFIX/share/man/man1"
BASH_COMP_DIR="$PREFIX/share/bash-completion/completions"
ZSH_COMP_DIR="$PREFIX/share/zsh/site-functions"
FISH_COMP_DIR="$PREFIX/share/fish/vendor_completions.d"

INSTALLED_FILES="
$BIN_DIR/squrl
$BASH_COMP_DIR/squrl
$ZSH_COMP_DIR/_squrl
$FISH_COMP_DIR/squrl.fish
$MAN_DIR/squrl.1
"

if [ "$UNINSTALL" -eq 1 ]; then
    echo "Uninstalling squrl from $PREFIX..."
    for f in $INSTALLED_FILES; do
        if [ -f "$f" ]; then
            rm -f "$f"
            echo "  Removed $f"
        fi
    done
    echo "Done."
    exit 0
fi

# Check prerequisites
if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo is not installed. Install Rust from https://rustup.rs/" >&2
    exit 1
fi

if ! rustup toolchain list 2>/dev/null | grep -q nightly; then
    echo "Error: Rust nightly toolchain is not installed." >&2
    echo "Install it with: rustup toolchain install nightly" >&2
    exit 1
fi

# Build
echo "Building squrl in release mode..."
cargo build --release

SQURL_BIN="target/release/squrl"

if [ ! -f "$SQURL_BIN" ]; then
    echo "Error: build did not produce $SQURL_BIN" >&2
    exit 1
fi

# Install binary
echo "Installing binary to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp "$SQURL_BIN" "$BIN_DIR/squrl"
chmod +x "$BIN_DIR/squrl"

# Install completions
TMPDIR_COMP="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_COMP"' EXIT

install_completions() {
    shell="$1"
    src_name="$2"
    dest_dir="$3"
    dest_name="$4"

    if "$BIN_DIR/squrl" completions "$shell" "$TMPDIR_COMP" 2>/dev/null; then
        if [ -f "$TMPDIR_COMP/$src_name" ]; then
            mkdir -p "$dest_dir"
            cp "$TMPDIR_COMP/$src_name" "$dest_dir/$dest_name"
            echo "  Installed $shell completions to $dest_dir/$dest_name"
        fi
    fi
}

echo "Installing shell completions..."

# Bash
if command -v bash >/dev/null 2>&1; then
    install_completions bash squrl.bash "$BASH_COMP_DIR" squrl
fi

# Zsh
if command -v zsh >/dev/null 2>&1; then
    install_completions zsh _squrl "$ZSH_COMP_DIR" _squrl
fi

# Fish
if command -v fish >/dev/null 2>&1; then
    install_completions fish squrl.fish "$FISH_COMP_DIR" squrl.fish
fi

# Install man page
echo "Installing man page..."
if "$BIN_DIR/squrl" man "$TMPDIR_COMP" 2>/dev/null; then
    if [ -f "$TMPDIR_COMP/squrl.1" ]; then
        mkdir -p "$MAN_DIR"
        cp "$TMPDIR_COMP/squrl.1" "$MAN_DIR/squrl.1"
        echo "  Installed man page to $MAN_DIR/squrl.1"
    fi
fi

# PATH check
case ":$PATH:" in
    *":$BIN_DIR:"*)
        ;;
    *)
        echo ""
        echo "Warning: $BIN_DIR is not in your PATH."
        echo "Add it with:"
        echo "  export PATH=\"$BIN_DIR:\$PATH\""
        ;;
esac

echo ""
echo "squrl installed successfully!"
echo "Run 'squrl --version' to verify."
