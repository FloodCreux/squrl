# squrl

[![CI](https://github.com/FloodCreux/squrl/actions/workflows/ci.yml/badge.svg)](https://github.com/FloodCreux/squrl/actions/workflows/ci.yml)
[![Release](https://github.com/FloodCreux/squrl/actions/workflows/release.yml/badge.svg)](https://github.com/FloodCreux/squrl/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-nightly-orange.svg)](https://www.rust-lang.org/)

A terminal-based HTTP and WebSocket client built with Rust. Think Postman or Insomnia, but in your terminal -- no GUI required.

![squrl homepage](assets/home.png)
![squrl request](assets/request.png)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Docker](#docker)
- [Usage](#usage)
  - [TUI (interactive)](#tui)
  - [CLI](#cli)
- [Configuration](#configuration)
- [Parameterizing Requests](#parameterizing-requests)
  - [Environment Variables](#environment-variables-1)
  - [Built-in Dynamic Variables](#built-in-dynamic-variables)
  - [Path Parameters](#path-parameters)
  - [Pre/Post Request Scripts](#prepost-request-scripts-1)
- [Themes](#themes)
- [Key Bindings](#key-bindings)
- [Development](#development)
- [Acknowledgements](#acknowledgements)
- [License](#license)

## Features

- **Dual interface** -- interactive TUI and full-featured CLI
- **HTTP client** -- all 9 standard methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, TRACE, CONNECT) with configurable timeouts, redirects, and proxy support
- **WebSocket support** -- connect, send/receive messages, and track connection state
- **Collections** -- organize requests in JSON, YAML, or `.http` files with tree-based navigation, optional folder grouping, and round-trip write-back for `.http` collections
- **Environments** -- key-value variables with `{{variable}}` substitution across URLs, headers, bodies, auth, and scripts
- **Collection-scoped environments** -- define per-collection environments (e.g. `dev`, `staging`, `prod`) with variables embedded directly in collection files, overriding global environments
- **Authentication** -- Basic, Bearer Token, JWT (HS/RS/ES/PS/EdDSA), and Digest (MD5, SHA-256, SHA-512)
- **Request bodies** -- raw text, JSON, XML, HTML, JavaScript, file upload, URL-encoded form, and multipart
- **Pre/post request scripts** -- JavaScript execution via embedded Boa runtime
- **Response handling** -- pretty-printed JSON, syntax highlighting, image preview, cookies, and headers
- **Import** -- Postman collections & environments, cURL commands, OpenAPI specs, and `.http` files (including `WEBSOCKET` requests)
- **Export** -- HTTP, cURL, PHP Guzzle, Node.js Axios, Rust reqwest, and PowerShell
- **Themes** -- 9 built-in themes (Gruber Darker, Dracula, Catppuccin variants, Gruvbox, and more) plus custom TOML themes
- **Key bindings** -- fully customizable with Vim, Emacs, and default modes
- **Clipboard** -- copy response bodies and exports (optional feature)
- **Shell completions & man pages** -- Bash, Zsh, Fish, and PowerShell via clap

## Installation

### Homebrew

```sh
brew tap FloodCreux/squrl
brew install squrl
```

### Scoop (Windows)

```powershell
scoop bucket add squrl https://github.com/FloodCreux/scoop-squrl
scoop install squrl
```

### Pre-built binary

**macOS / Linux:**

```sh
curl -fsSL https://raw.githubusercontent.com/FloodCreux/squrl/main/scripts/curl-install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/FloodCreux/squrl/main/scripts/curl-install.ps1 | iex
```

### Build from source

**macOS / Linux:**

```sh
git clone https://github.com/FloodCreux/squrl.git
cd squrl
./scripts/install.sh
```

**Windows (PowerShell):**

```powershell
git clone https://github.com/FloodCreux/squrl.git
cd squrl
.\scripts\install.ps1
```

### Using just

```sh
just install
```

This builds a release binary and installs the binary, shell completions, and man page to `~/.local`.

## Docker

A multi-stage Dockerfile is included for building a minimal Docker image containing the statically linked `squrl` binary. It uses [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) for dependency caching and [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) for cross-compilation.

### Build

```sh
# Build for your current platform
docker build -t squrl .

# Build for both amd64 and arm64
docker buildx build --platform linux/amd64,linux/arm64 -t squrl .
```

### Run

```sh
# Launch the TUI (mount your working directory so squrl can access collections and environments)
docker run -it --rm -v "$(pwd):/app" squrl

# Run a CLI command
docker run -it --rm -v "$(pwd):/app" squrl try GET https://httpbin.org/get
```

> **Note:** The Docker image is built without clipboard support since there is no display server available inside the container.

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
squrl collection send <name> [--env <env-name>] [--collection-env <env-name>]
```

#### Collection Environments

```sh
squrl collection env <collection-name> list
squrl collection env <collection-name> create <env-name>
squrl collection env <collection-name> delete <env-name>
squrl collection env <collection-name> select [<env-name>]
squrl collection env <collection-name> info <env-name>
squrl collection env <collection-name> key <env-name> get <key>
squrl collection env <collection-name> key <env-name> set <key> <value>
squrl collection env <collection-name> key <env-name> add <key> <value>
squrl collection env <collection-name> key <env-name> delete <key>
squrl collection env <collection-name> key <env-name> rename <key> <new-key>
```

#### Requests

Requests are referenced as `<collection>/<request>`.

```sh
squrl request info <collection>/<request>
squrl request new <collection>/<request> [--url <url>] [--method <method>]
squrl request delete <collection>/<request>
squrl request rename <collection>/<request> <new-name>
squrl request send <collection>/<request> [--env <env-name>] [--collection-env <env-name>]

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
squrl completions <shell> [<dir>]   # Generate shell completions (bash, zsh, fish, powershell)
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

| Variable             | Description                            |
| -------------------- | -------------------------------------- |
| `SQURL_MAIN_DIR`     | Working directory                      |
| `SQURL_THEME`        | Path to a custom theme TOML file       |
| `SQURL_KEY_BINDINGS` | Path to a custom keybindings TOML file |

### Working directory layout

```
squrl_main_dir/
  collection1.json      # or .yaml -- request collections (may contain embedded environments)
  collection2.yaml
  requests/             # .http file collections (auto-loaded in git repos)
    example.http
    squrl-env.json      # companion file for .http collection environments (optional)
  .env.production       # KEY=VALUE global environment files
  .env.staging
  squrl.toml            # Local configuration
  squrl.log             # Auto-generated log file (TUI mode)
```

Collection files support an optional `folders` field for grouping requests:

```json
{
  "name": "My API",
  "folders": [
    {
      "name": "Users",
      "requests": [...]
    }
  ],
  "requests": [...]
}
```

Folders appear in the collection tree between the collection and its root-level requests. When creating a new request in the TUI, the popup includes a folder selector -- choose "None (root)" to add to the collection root, or pick a folder to add the request directly into it. If your cursor is already inside a folder, it is pre-selected. Deleting a folder moves its requests to the collection root. Existing collection files without folders continue to work unchanged.

squrl also auto-loads `.http` files from a `requests/` subdirectory when inside a git repository. Subdirectories are searched recursively, and files found inside a child directory of `requests/` are grouped into a folder named after that top-level child directory. Modifications to these collections are saved back to the original `.http` files, preserving the HTTP file format.

`.http` files support standard HTTP methods as well as `WEBSOCKET` for WebSocket connections:

```http
### List Users
GET https://api.example.com/users

### Create User
POST https://api.example.com/users
Content-Type: application/json

{"name": "Jane Doe"}

### Echo WebSocket
WEBSOCKET wss://echo.websocket.org
```

WebSocket entries support headers and authentication just like HTTP requests.

```
requests/
  example.http              # root-level request (no folder)
  auth/
    login.http              # grouped into "auth" folder
    tokens/
      refresh.http          # also grouped into "auth" folder
  users/
    crud.http               # grouped into "users" folder
```

Folders are ordered alphabetically, and requests within each folder are ordered alphabetically by file path.

## Parameterizing Requests

squrl supports several ways to parameterize requests so you can reuse the same request definitions across different environments and contexts.

### Environment Variables

Define variables in `.env.*` files in your working directory and reference them with `{{VARIABLE_NAME}}` syntax. The environment name is derived from the filename (e.g. `.env.development` becomes `development`).

**Example `.env.development`:**

```
# API configuration
BASE_URL=https://jsonplaceholder.typicode.com
API_KEY=dev-secret-key

# Default resource IDs
USER_ID=1

# User data
USERNAME=janedoe
EMAIL=jane@example.com
```

**Example `.env.production`:**

```
BASE_URL=https://api.myapp.com
API_KEY=prod-secret-key
USER_ID=42
USERNAME=admin
EMAIL=admin@myapp.com
```

Variables can then be used anywhere in a request -- URLs, headers, and bodies:

```http
### List Users
GET {{BASE_URL}}/users
Authorization: Bearer {{API_KEY}}

### Create User
POST {{BASE_URL}}/users
Content-Type: application/json

{
  "username": "{{USERNAME}}",
  "email": "{{EMAIL}}"
}
```

Select the active environment in the TUI or pass `--env <name>` on the CLI:

```sh
squrl request send my-collection/create-user --env development
squrl request send my-collection/create-user --env production
```

**Variable file format notes:**

- Lines starting with `#` are comments.
- Values can be unquoted (`KEY=value`), double-quoted (`KEY="value with spaces"` -- supports `\\`, `\"`, `\n`, `\r`, `\t` escapes), or single-quoted (`KEY='literal value'` -- no escape processing).
- Values containing `=` are fine: `CONNECTION_STRING=host=db port=5432` parses correctly.
- Empty values are valid: `OPTIONAL_HEADER=` yields an empty string.
- If no environment is selected, or a `{{VARIABLE}}` has no match, the placeholder is left as-is in the request.

**Resolution order:** Collection-scoped environment variables are checked first (highest priority), then global environment variables, then OS environment variables (all system env vars are available as `{{VAR}}`), then built-in dynamic variables.

### Collection-Scoped Environments

In addition to global `.env.*` files, each collection can define its own named environments with per-environment variables. Collection environment variables take priority over global ones, allowing you to share common variables globally while overriding specific values per-collection.

**Embedded in JSON/YAML collections:**

```json
{
  "name": "My API",
  "selected_environment": "dev",
  "environments": [
    {
      "name": "dev",
      "values": {
        "BASE_URL": "http://localhost:3000",
        "API_KEY": "dev-key-123"
      }
    },
    {
      "name": "prod",
      "values": {
        "BASE_URL": "https://api.example.com",
        "API_KEY": "prod-key-456"
      }
    }
  ],
  "requests": [...]
}
```

**Companion file for `.http` collections:**

`.http` file collections store environments in a `squrl-env.json` file alongside the `.http` files:

```json
{
  "selected_environment": "dev",
  "environments": {
    "dev": {
      "BASE_URL": "http://localhost:3000"
    },
    "prod": {
      "BASE_URL": "https://api.example.com"
    }
  }
}
```

**CLI management:**

```sh
# Create environments
squrl collection env my-api create dev
squrl collection env my-api create prod

# Add variables
squrl collection env my-api key dev add BASE_URL http://localhost:3000
squrl collection env my-api key prod add BASE_URL https://api.example.com

# Select the active environment
squrl collection env my-api select dev

# Send with a specific collection environment
squrl request send my-api/get-users --collection-env prod
```

**TUI:**

- Press `e` to cycle through the selected collection's environments (falls back to global environments if the collection has none)
- Press `Ctrl-e` to open the environment editor popup, where you can view, add, edit, and delete variables
- If a collection has no environments when `Ctrl-e` is pressed, a "default" environment is auto-created

### Built-in Dynamic Variables

Four dynamic variables are always available, regardless of environment selection. Each generates a fresh value on every request:

| Variable | Description | Example value |
| -------------- | ------------------------ | ------------------------------------- |
| `{{NOW}}` | Current UTC date/time | `2025-07-15 14:30:00.123456789 UTC` |
| `{{TIMESTAMP}}` | Current Unix timestamp | `1752588600` |
| `{{UUIDv4}}` | Random UUID v4 | `550e8400-e29b-41d4-a716-446655440000` |
| `{{UUIDv7}}` | Time-sortable UUID v7 | `01908a6e-7a5c-7000-8000-000000000001` |

```http
### Create Resource with Generated ID
POST {{BASE_URL}}/resources
Content-Type: application/json
X-Idempotency-Key: {{UUIDv4}}
X-Request-Timestamp: {{TIMESTAMP}}

{
  "id": "{{UUIDv4}}",
  "createdAt": "{{NOW}}"
}
```

### Path Parameters

Use `{param}` (single braces) in URLs to define path parameters. Unlike environment variables, path parameters are managed per-request and appear as editable key-value pairs in the TUI.

```http
### Get User
GET https://api.example.com/users/{userId}

### Get Post Comments
GET https://api.example.com/posts/{postId}/comments
```

Path parameters are distinct from environment variables: `{userId}` is a path parameter while `{{USER_ID}}` is an environment variable. Both can be used in the same URL:

```http
### Mixed: env var for base URL, path param for resource ID
GET {{BASE_URL}}/users/{userId}
```

When importing OpenAPI specs, path parameters from the spec (e.g. `/users/{id}`) are automatically converted to squrl path parameters.

### Pre/Post Request Scripts

For advanced parameterization, squrl supports JavaScript pre-request and post-request scripts executed via an embedded Boa runtime. Scripts can read and modify environment variables programmatically.

**Pre-request script** -- runs before the request is sent:

```javascript
// Set a dynamic header value
request.headers["X-Request-ID"] = env.UUIDv4;

// Compute an auth signature
const timestamp = Date.now().toString();
request.headers["X-Timestamp"] = timestamp;

// Override the URL based on environment
if (env.USE_STAGING === "true") {
  request.url = "https://staging.api.example.com/users";
}
```

**Post-request script** -- runs after the response is received:

```javascript
// Extract a token from the response and store it for later requests
const body = JSON.parse(response.body);
env.AUTH_TOKEN = body.token;
env.USER_ID = body.user.id.toString();

console.log("Stored auth token for user " + env.USER_ID);
```

Changes made to `env` in scripts are persisted back to the active environment file, making them available to subsequent requests. This is useful for chaining requests -- for example, logging in first and then using the returned token in later requests.

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
