# squrl

A terminal-based HTTP and WebSocket client built with Rust. Think Postman or Insomnia, but in your terminal -- no GUI required.

![squrl homepage](assets/home.png)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [TUI (interactive)](#tui)
  - [CLI](#cli)
- [Configuration](#configuration)
- [Themes](#themes)
- [Key Bindings](#key-bindings)
- [Development](#development)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Features

- **Dual interface** -- interactive TUI and full-featured CLI
- **HTTP client** -- all 9 standard methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, TRACE, CONNECT) with configurable timeouts, redirects, and proxy support
- **WebSocket support** -- connect, send/receive messages, and track connection state
- **Collections** -- organize requests in JSON or YAML files with tree-based navigation
- **Environments** -- key-value variables with `{{variable}}` substitution across URLs, headers, bodies, auth, and scripts
- **Authentication** -- Basic, Bearer Token, JWT (HS/RS/ES/PS/EdDSA), and Digest (MD5, SHA-256, SHA-512)
- **Request bodies** -- raw text, JSON, XML, HTML, JavaScript, file upload, URL-encoded form, and multipart
- **Pre/post request scripts** -- JavaScript execution via embedded Boa runtime
- **Response handling** -- pretty-printed JSON, syntax highlighting, image preview, cookies, and headers
- **Import** -- Postman collections & environments, cURL commands, OpenAPI specs, and `.http` files
- **Export** -- HTTP, cURL, PHP Guzzle, Node.js Axios, and Rust reqwest
- **Themes** -- 9 built-in themes (Gruber Darker, Dracula, Catppuccin variants, Gruvbox, and more) plus custom TOML themes
- **Key bindings** -- fully customizable with Vim, Emacs, and default modes
- **Clipboard** -- copy response bodies and exports (optional feature)
- **Shell completions & man pages** -- Bash, Zsh, and Fish via clap

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

This builds a release binary and installs the binary, shell completions, and man page to `~/.local`.

## Usage

### TUI

Launch the interactive terminal UI:

```sh
squrl
```

The TUI provides a collection tree sidebar, request editor panels, response viewer, environment editor, cookie viewer, log panel, and theme picker -- all navigable via keyboard.

### CLI

#### One-off requests

```sh
squrl try <url> [options]
```

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

#### Themes (CLI)

```sh
squrl theme list             # List available themes
squrl theme preview <name>   # Preview a theme
squrl theme export <name>    # Export a theme as TOML
```

#### Utilities

```sh
squrl completions <shell> [<dir>]   # Generate shell completions (bash, zsh, fish)
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

squrl reads its config from `squrl.toml` in the working directory. A global fallback config can be placed at `~/.config/squrl/global.toml`.

```toml
theme = "dracula"
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

### Environment variables

| Variable | Description |
|---|---|
| `SQURL_MAIN_DIR` | Working directory |
| `SQURL_THEME` | Path to a custom theme TOML file |
| `SQURL_KEY_BINDINGS` | Path to a custom keybindings TOML file |

### Working directory layout

```
squrl_main_dir/
  collection1.json      # or .yaml -- request collections
  collection2.yaml
  .env.production       # KEY=VALUE environment files
  .env.staging
  squrl.toml            # Local configuration
  squrl.log             # Auto-generated log file (TUI mode)
```

squrl also auto-loads `.http` files from a `requests/` subdirectory when inside a git repository.

## Themes

squrl ships with 9 built-in themes:

- **Default** (Gruber Darker)
- **Dracula**
- **Catppuccin** (Mocha, Macchiato, Frappe, Latte)
- **Gruvbox**
- **OpenCode**
- **VS Code Dark**

Custom themes are TOML files placed in `~/.config/squrl/themes/`. Theme priority: CLI flag > `SQURL_THEME` env var > `~/.config/squrl/theme.toml` > config file setting > default.

## Key Bindings

squrl supports three text editor modes out of the box: **Vim**, **Emacs**, and **Default**. You can also define fully custom key bindings via a TOML file at `~/.config/squrl/keybindings.toml` or via the `SQURL_KEY_BINDINGS` environment variable.

## Development

Requires [Rust](https://www.rust-lang.org/tools/install) (nightly) and [just](https://github.com/casey/just). A [Nix flake](flake.nix) is also provided for reproducible development environments.

```sh
just build          # Debug build
just build-release  # Release build
just test           # Run all tests
just test-verbose   # Run tests with output
just lint           # Run clippy lints
just fmt            # Format code
just fmt-check      # Check formatting
just security       # Run cargo-deny and cargo-audit
just coverage       # Generate code coverage report
just clean          # Clean build artifacts
just install        # Install binary, completions, and man page
just uninstall      # Remove all installed files
```

Run `just` with no arguments to see all available recipes.

### Pre-commit hooks

This project uses [pre-commit](https://pre-commit.com/) for automated checks before each commit.

```sh
# Install pre-commit (choose one)
pip install pre-commit
brew install pre-commit
# or via nix develop (included in devShell)

# Install the git hooks
pre-commit install

# Run hooks manually on all files
pre-commit run --all-files
```

## Acknowledgements

squrl draws heavy inspiration from [ATAC](https://github.com/Julien-cpsn/ATAC) by [Julien-cpsn](https://github.com/Julien-cpsn) -- a fantastic terminal API client that pioneered many of the ideas and patterns found in this project. Much of squrl's architecture, feature set, and TUI design was informed by ATAC's excellent work. If you like squrl, you should check out ATAC as well.

## License

[MIT](LICENSE)
