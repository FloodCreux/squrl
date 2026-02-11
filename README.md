# squrl

A terminal HTTP client built with Rust, powered by [ratatui](https://github.com/ratatui/ratatui), [reqwest](https://github.com/seanmonstar/reqwest), and [Tokio](https://tokio.rs/).

## Features

- **Dual interface** -- interactive TUI and full-featured CLI
- **Async HTTP** -- all 9 standard methods with configurable timeouts and cancellation
- **Collections** -- organize requests in JSON or YAML files with tree-based navigation
- **Environments** -- key-value variables with `{{variable}}` substitution across URLs, headers, bodies, auth, and scripts
- **Authentication** -- Basic, Bearer Token, JWT (HS/RS/ES/PS/EdDSA), and Digest (MD5, SHA-256, SHA-512)
- **Request bodies** -- raw text, JSON, XML, HTML, JavaScript, file upload, form, and multipart
- **Pre/post scripts** -- JavaScript execution via embedded runtime
- **Response handling** -- pretty-printed JSON, syntax highlighting, image preview, cookies, and headers
- **Import** -- Postman collections/environments, cURL commands, OpenAPI specs, and `.http` files
- **Export** -- HTTP, cURL, PHP Guzzle, Node.js Axios, and Rust reqwest
- **Clipboard** -- copy response bodies (optional feature)
- **Shell completions & man pages** -- generated via clap

## Installation

### Pre-built binary (recommended)

```sh
curl -fsSL https://codeberg.org/flood-mike/squrl/raw/branch/main/curl-install.sh | sh
```

### Build from source

```sh
git clone https://codeberg.org/flood-mike/squrl.git
cd squrl
./install.sh
```

### Using just

```sh
just install
```

## Usage

### TUI

Run `squrl` with no subcommand to launch the interactive terminal UI.

### CLI

#### Collections

```sh
squrl collection list [--request-names]
squrl collection info <name> [--without-request-names]
squrl collection new <name>
squrl collection delete <name>
squrl collection rename <name> <new-name>
squrl collection send <name> [--env <env-name>]
```

#### Requests

Requests are referenced as `<collection>/<request>`.

```sh
squrl request info <collection>/<request>
squrl request new <collection>/<request> [--url <url>] [--method <method>]
squrl request delete <collection>/<request>
squrl request rename <collection>/<request> <new-name>
squrl request send <collection>/<request> [--env <env-name>]

# Modify request properties
squrl request url <collection>/<request> set|get|add <url>
squrl request method <collection>/<request> set|get <method>
squrl request params <collection>/<request> get|set|add|delete|rename <key> [<value>]
squrl request header <collection>/<request> get|set|add|delete|rename <key> [<value>]
squrl request auth <collection>/<request> <auth-type> [args]
squrl request body <collection>/<request> set|get|add|delete <type> [content]
squrl request scripts <collection>/<request> set|get <pre|post> [content]
squrl request settings <collection>/<request> get|set <setting> [value]
squrl request export <collection>/<request> <format>
```

#### Environments

```sh
squrl env info <name> [--os-vars]
squrl env key <name> get <key>
squrl env key <name> set <key> <value>
squrl env key <name> add <key> <value>
squrl env key <name> delete <key>
squrl env key <name> rename <key> <new-key>
```

#### Import

```sh
squrl import postman <path> [--max-depth <n>]
squrl import postman-env <path> [--force-uppercase-keys] [--use-disabled]
squrl import curl <path> <collection-name> [<request-name>] [--recursive] [--max-depth <n>]
squrl import openapi <path> [--max-depth <n>]
squrl import http-file <path> [<collection-name>] [--recursive] [--max-depth <n>]
```

#### One-off requests

```sh
squrl try <url> [options]
```

#### Utilities

```sh
squrl completions <shell> [<dir>]   # Generate shell completions
squrl man [<output-dir>]            # Generate man pages
```

#### Global options

```sh
squrl --directory <dir>   # Set working directory (or SQURL_MAIN_DIR env var)
squrl --dry-run           # Test without saving changes
squrl --filter <regex>    # Only load collections matching regex
squrl --tui               # Launch TUI after a CLI command
squrl --verbose/-v        # Increase verbosity level
```

## Configuration

squrl reads its config from `squrl.toml` in the working directory:

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

### Working directory structure

```
squrl_main_dir/
  collection1.json      # or .yaml
  collection2.json
  environment1          # KEY=VALUE format
  environment2
  squrl.toml            # Configuration
```

## Development

Requires [Rust](https://www.rust-lang.org/tools/install) (nightly) and [just](https://github.com/casey/just).

```sh
just build          # Build the project
just build-release  # Build in release mode
just test           # Run all tests
just test-verbose   # Run tests with output
just lint           # Run clippy lints
just fmt            # Format code
just fmt-check      # Check formatting
just clean          # Clean build artifacts
just install        # Install binary, completions, and man page
just uninstall      # Remove all installed files
```

Run `just` to see all available commands.

## License

[MIT](LICENSE)
