# List available recipes
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run the project
run:
    cargo run

# Check the project compiles without building
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run pre-commit checks
pre-commit: fmt-check lint

# Install git pre-commit hook
setup-hooks:
    echo '#!/bin/sh\njust pre-commit' > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    @echo "Pre-commit hook installed"

# Clean build artifacts
clean:
    cargo clean

# ============================================================================
# Security
# ============================================================================

# Run cargo-audit to check for known vulnerabilities
audit:
    cargo audit

# Run cargo-deny to check advisories, licenses, and bans
deny:
    cargo deny check

# Run all security checks
security: audit deny

# ============================================================================
# Code Coverage
# ============================================================================

# Run tests with coverage using cargo-llvm-cov
coverage:
    cargo llvm-cov --all-features

# Run tests with coverage and generate HTML report
coverage-html:
    cargo llvm-cov --all-features --html
    @echo "Coverage report generated at target/llvm-cov/html/index.html"

# Run tests with coverage and generate lcov report (for CI integration)
coverage-lcov:
    cargo llvm-cov --all-features --lcov --output-path target/llvm-cov/lcov.info
    @echo "LCOV report generated at target/llvm-cov/lcov.info"

# Run tests with coverage and open HTML report in browser
coverage-open:
    cargo llvm-cov --all-features --open

# Set install prefix
PREFIX := env("PREFIX", home_directory() / ".local")

# Install squrl, completions, and man page
install: build-release _install-completions _install-man
    cargo install --path . --force
    @echo "squrl installed successfully"

# Uninstall squrl, completions, and man page
uninstall:
    cargo uninstall squrl || true
    rm -f "{{ PREFIX }}/share/bash-completion/completions/squrl"
    rm -f "{{ PREFIX }}/share/zsh/site-functions/_squrl"
    rm -f "{{ PREFIX }}/share/fish/vendor_completions.d/squrl.fish"
    rm -f "{{ PREFIX }}/share/man/man1/squrl.1"
    @echo "squrl uninstalled"

[private]
_install-completions:
    #!/bin/sh
    set -e
    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT
    BIN="target/release/squrl"
    if command -v bash >/dev/null 2>&1; then
        "$BIN" completions bash "$TMPDIR"
        mkdir -p "{{ PREFIX }}/share/bash-completion/completions"
        cp "$TMPDIR/squrl.bash" "{{ PREFIX }}/share/bash-completion/completions/squrl"
        echo "  Installed bash completions"
    fi
    if command -v zsh >/dev/null 2>&1; then
        "$BIN" completions zsh "$TMPDIR"
        mkdir -p "{{ PREFIX }}/share/zsh/site-functions"
        cp "$TMPDIR/_squrl" "{{ PREFIX }}/share/zsh/site-functions/_squrl"
        echo "  Installed zsh completions"
    fi
    if command -v fish >/dev/null 2>&1; then
        "$BIN" completions fish "$TMPDIR"
        mkdir -p "{{ PREFIX }}/share/fish/vendor_completions.d"
        cp "$TMPDIR/squrl.fish" "{{ PREFIX }}/share/fish/vendor_completions.d/squrl.fish"
        echo "  Installed fish completions"
    fi

[private]
_install-man:
    #!/bin/sh
    set -e
    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT
    BIN="target/release/squrl"
    "$BIN" man "$TMPDIR"
    mkdir -p "{{ PREFIX }}/share/man/man1"
    cp "$TMPDIR/squrl.1" "{{ PREFIX }}/share/man/man1/squrl.1"
    echo "  Installed man page"
