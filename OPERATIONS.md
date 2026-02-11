# Operations

## Building

Requires Rust nightly and [just](https://github.com/casey/just).

```sh
just build          # Debug build
just build-release  # Release build
```

## Installation

### Pre-built binary

```sh
curl -fsSL https://codeberg.org/flood-mike/squrl/raw/branch/main/curl-install.sh | sh
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

## CI

The `.github` directory contains CI workflows. Ensure `just lint`, `just fmt-check`, and `just test` pass before pushing.
