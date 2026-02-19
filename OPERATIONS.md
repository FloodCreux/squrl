# Operations

## Building

Requires Rust nightly and [just](https://github.com/casey/just).

```sh
just build          # Debug build
just build-release  # Release build
```

## Installation

### Pre-built binary

**macOS / Linux:**

```sh
curl -fsSL https://raw.githubusercontent.com/FloodCreux/squrl/main/scripts/curl-install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/FloodCreux/squrl/main/scripts/curl-install.ps1 | iex
```

### From source

```sh
just install        # Builds release binary, installs completions and man page
just uninstall      # Removes all installed files
```

## Configuration

squrl reads its config from `squrl.toml` in the working directory (or the directory set via `--directory` / `SQURL_MAIN_DIR`):

```toml
disable_syntax_highlighting = false
save_requests_response = false
disable_images_preview = false
disable_graphical_protocol = false
wrap_responses = false
preferred_collection_file_format = "json"

[proxy]
http_proxy = "http://..."
https_proxy = "https://..."
```

## Working Directory Structure

```
squrl_main_dir/
  collection1.json      # or .yaml
  collection2.json
  environment1          # KEY=VALUE format
  environment2
  squrl.toml            # Configuration
```

## Testing

```sh
just test           # Run all tests
just test-verbose   # Run tests with output
```

## Linting and Formatting

```sh
just lint           # Run clippy
just fmt            # Format code
just fmt-check      # Check formatting without modifying files
```

## CI/CD

GitHub Actions workflows live in `.github/workflows/`:

### CI (`ci.yml`)

Runs on every push to `main` and on pull requests:

- **Format** — `cargo fmt -- --check`
- **Clippy** — `cargo clippy -- -D warnings`
- **Test** — `cargo test` on Ubuntu, macOS, and Windows
- **Cargo Deny** — license, advisory, and source checks

Ensure `just lint`, `just fmt-check`, and `just test` pass before pushing.

### Release (`release.yml`)

Runs when a version tag is pushed (e.g. `git tag v0.1.0 && git push --tags`):

1. Runs the full CI pipeline first
2. Cross-compiles release binaries for all six targets:
   - `x86_64-unknown-linux-gnu`
   - `aarch64-unknown-linux-gnu`
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
   - `x86_64-pc-windows-msvc`
   - `aarch64-pc-windows-msvc`
3. Packages each binary into a tarball (`.tar.gz` for Linux/macOS, `.zip` for Windows)
4. Creates a GitHub Release with auto-generated release notes and all tarballs attached

These tarballs are consumed by the `scripts/curl-install.sh` remote installer (and `.zip` archives by `scripts/curl-install.ps1` on Windows).
