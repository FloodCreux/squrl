#!/bin/sh
set -eu

# Remote installer for squrl pre-built binaries.
# Usage: curl -fsSL https://raw.githubusercontent.com/FloodCreux/squrl/main/curl-install.sh | sh
#
# Environment variables:
#   VERSION      - version to install (default: latest)
#   INSTALL_DIR  - installation directory (default: $HOME/.local/bin)

REPO_API="https://api.github.com/repos/FloodCreux/squrl"
REPO_RELEASES="https://github.com/FloodCreux/squrl/releases/download"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

err() {
    echo "Error: $*" >&2
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "unknown-linux-gnu" ;;
        Darwin*) echo "apple-darwin" ;;
        *)       err "Unsupported OS: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             err "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Detect download tool
detect_downloader() {
    if command -v curl >/dev/null 2>&1; then
        echo "curl"
    elif command -v wget >/dev/null 2>&1; then
        echo "wget"
    else
        err "Neither curl nor wget found. Please install one of them."
    fi
}

download() {
    url="$1"
    dest="$2"
    case "$DOWNLOADER" in
        curl) curl -fsSL -o "$dest" "$url" ;;
        wget) wget -qO "$dest" "$url" ;;
    esac
}

fetch_url() {
    url="$1"
    case "$DOWNLOADER" in
        curl) curl -fsSL "$url" ;;
        wget) wget -qO- "$url" ;;
    esac
}

# Resolve version
resolve_version() {
    if [ -n "${VERSION:-}" ]; then
        echo "$VERSION"
        return
    fi

    latest="$(fetch_url "$REPO_API/releases/latest" | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4)"

    if [ -z "$latest" ]; then
        err "Could not determine latest version. Set VERSION env var manually."
    fi

    # Strip leading 'v' if present
    echo "$latest" | sed 's/^v//'
}

DOWNLOADER="$(detect_downloader)"
OS="$(detect_os)"
ARCH="$(detect_arch)"
VERSION="$(resolve_version)"
TARGET="${ARCH}-${OS}"
TARBALL="squrl-v${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="${REPO_RELEASES}/v${VERSION}/${TARBALL}"

echo "Installing squrl v${VERSION} for ${TARGET}..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

# Download and extract
echo "Downloading ${DOWNLOAD_URL}..."
download "$DOWNLOAD_URL" "$TMPDIR/$TARBALL"

tar -xzf "$TMPDIR/$TARBALL" -C "$TMPDIR"

if [ ! -f "$TMPDIR/squrl" ]; then
    err "Archive did not contain squrl binary"
fi

# Install binary
mkdir -p "$INSTALL_DIR"
cp "$TMPDIR/squrl" "$INSTALL_DIR/squrl"
chmod +x "$INSTALL_DIR/squrl"
echo "Installed squrl to $INSTALL_DIR/squrl"

# Install completions and man page using the binary itself
PREFIX="${INSTALL_DIR%/bin}"
BASH_COMP_DIR="$PREFIX/share/bash-completion/completions"
ZSH_COMP_DIR="$PREFIX/share/zsh/site-functions"
FISH_COMP_DIR="$PREFIX/share/fish/vendor_completions.d"
MAN_DIR="$PREFIX/share/man/man1"

install_completions() {
    shell="$1"
    src_name="$2"
    dest_dir="$3"
    dest_name="$4"

    if "$INSTALL_DIR/squrl" completions "$shell" "$TMPDIR" 2>/dev/null; then
        if [ -f "$TMPDIR/$src_name" ]; then
            mkdir -p "$dest_dir"
            cp "$TMPDIR/$src_name" "$dest_dir/$dest_name"
            echo "  Installed $shell completions to $dest_dir/$dest_name"
        fi
    fi
}

echo "Installing shell completions..."
if command -v bash >/dev/null 2>&1; then
    install_completions bash squrl.bash "$BASH_COMP_DIR" squrl
fi
if command -v zsh >/dev/null 2>&1; then
    install_completions zsh _squrl "$ZSH_COMP_DIR" _squrl
fi
if command -v fish >/dev/null 2>&1; then
    install_completions fish squrl.fish "$FISH_COMP_DIR" squrl.fish
fi

echo "Installing man page..."
if "$INSTALL_DIR/squrl" man "$TMPDIR" 2>/dev/null; then
    if [ -f "$TMPDIR/squrl.1" ]; then
        mkdir -p "$MAN_DIR"
        cp "$TMPDIR/squrl.1" "$MAN_DIR/squrl.1"
        echo "  Installed man page to $MAN_DIR/squrl.1"
    fi
fi

# PATH check
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        ;;
    *)
        echo ""
        echo "Warning: $INSTALL_DIR is not in your PATH."
        echo "Add it with:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        ;;
esac

echo ""
echo "squrl v${VERSION} installed successfully!"
echo "Run 'squrl --version' to verify."
