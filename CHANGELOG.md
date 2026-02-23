# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Collection-scoped environments** -- each collection can now define its own named environments (e.g. `dev`, `staging`, `prod`) with per-environment key-value variables. Collection environment variables take highest priority during `{{VAR}}` substitution, followed by global `.env.*` environments, OS env vars, and built-in dynamic variables
- **Embedded environments in JSON/YAML collections** -- environments are stored directly in collection files under `environments` and `selected_environment` fields, with full backward compatibility for existing collections
- **Companion env file for `.http` collections** -- `.http` file collections store environments in a `squrl-env.json` companion file alongside the `.http` files
- **Collection environment CLI commands** -- `squrl collection env <name> list|create|delete|select|info` and `squrl collection env <name> key <env> get|set|add|delete|rename` for full CRUD management of collection-scoped environments
- **`--collection-env` flag** -- specify a collection-scoped environment when sending requests via `squrl request send` or `squrl collection send`
- **Environment editor popup in TUI** -- `Ctrl-e` now opens a centered popup overlay for viewing and editing environment variables, with full keyboard navigation and inline editing
- **Auto-create default environment** -- pressing `Ctrl-e` on a collection with no environments automatically creates a "default" environment, providing an immediate editing surface
- **Environment widget always visible** -- the sidebar environment widget now always renders, showing `(no environment)` when no environment is configured
- **Template URL support in `.http` files** -- URLs containing `{{VAR}}` placeholders are now accepted by the `.http` file parser instead of failing with "relative URL without a base"
- **Windows support** -- squrl now builds, tests, and releases on Windows (x86_64 and aarch64 MSVC targets)
- **Windows CI** -- `windows-latest` added to the CI test matrix
- **Windows release artifacts** -- `.zip` archives for Windows targets alongside `.tar.gz` for Linux and macOS
- **PowerShell export format** -- export requests as PowerShell `Invoke-RestMethod` scripts (`.ps1`) with splatting, `-InFile` for file bodies, `-Form` for multipart (PS 6+), and `-Proxy` support
- **PowerShell install scripts** -- `scripts/install.ps1` (build from source) and `scripts/curl-install.ps1` (download pre-built binary) for Windows
- **Scoop package manager** -- `scoop bucket add squrl` for Windows installation, with automated manifest updates on release

### Changed

- `prepare_request()` now accepts an optional `collection_index` parameter for collection-aware environment variable resolution
- TUI environment cycling (`e` key) is now context-dependent: cycles collection environments when the selected collection has its own, otherwise cycles global environments
- TUI environment editor (`Ctrl-e`) is context-dependent: edits collection environments when available, otherwise edits global environments
- Environment keybindings (`e` and `Ctrl-e`) are now always registered when collections are loaded, rather than only when global `.env.*` files exist
- Moved install scripts into `scripts/` directory (`install.sh`, `curl-install.sh`, `install.ps1`, `curl-install.ps1`)
- `get_user_config_dir()` now uses `ProjectDirs` on Windows instead of `~/.config/squrl`
- `sanitize_name` now strips backslashes (`\`) in addition to forward slashes and quotes
- Test suite is fully cross-platform: replaced hardcoded `/tmp` paths with `std::env::temp_dir()`, gated Unix-specific tilde expansion tests with `#[cfg(not(windows))]`, and made path construction platform-aware
- Added `BSL-1.0` (Boost Software License) to `deny.toml` allowed licenses
- Added Windows target triples to `deny.toml`

### Fixed

- `.http` files with `{{VAR}}` template URLs no longer fail to parse during collection loading
- Environment `path` field no longer serialized into collection JSON files (was leaking filesystem paths into portable collection data)
- Integer overflow in import handlers when `collections` list is empty (`len() - 1` replaced with `saturating_sub(1)`)
- Curl export `--data` flag used `'@/<path>'` instead of `'@<path>'`, producing double-slashes for absolute paths

## [0.1.1] - 2026-02-19

### Added

- **Homebrew tap** -- automated formula updates on release via GitHub Actions
- **Docker support** -- multi-platform builds with cargo-chef and cargo-zigbuild
- **GitHub CI/CD** -- format, clippy, and test workflows; release workflow with cross-compiled binaries for Linux and macOS (x86_64 and aarch64)
- **GitHub issue and PR templates** -- bug report, feature request, and pull request templates
- **Collection write-back for `.http` files** -- modifications to `.http`-based collections are now saved back to the original `.http` files, preserving the HTTP file format with full round-trip support for methods, URLs, query parameters, headers, authentication, and bodies
- **Request source path tracking** -- each request parsed from an `.http` file now records its originating file path, enabling accurate write-back to the correct file

### Changed

- Refactored `App` into smaller, focused components
- Refactored available events dispatching for maintainability
- Refactored CLI command handlers into dedicated modules
- Replaced unwraps with proper error handling throughout the codebase
- Addressed await-holding-lock issues for safer async code
- Avoided cloning OS environment variables on every interpolation (performance)
- Fixed untagged serde deserialization
- Cleaned up module visibility and access modifiers
- Cleaned up logging configuration
- Homepage description text is now properly centered in the TUI using layout constraints
- WebSocket badge in the request list now uses foreground-only coloring instead of foreground + background

### Fixed

- Panic errors replaced with graceful error handling
- Settings options cleanup
- Trailing slash on WebSocket echo URL in demo collection

## [0.1.0] - 2026-02-17

Initial release of squrl.

### Added

#### Core

- **HTTP client** -- support for all 9 standard methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, TRACE, CONNECT) with configurable timeouts, redirects, and proxy support
- **WebSocket support** -- connect, send/receive messages, track connection state, and WebSocket-specific TUI layout
- **Request bodies** -- raw text, JSON, XML, HTML, JavaScript, file upload, URL-encoded form, and multipart
- **Response handling** -- pretty-printed JSON, syntax highlighting, image preview, cookies, and headers
- **Pre/post request scripts** -- JavaScript execution via embedded Boa runtime

#### Authentication

- **Basic auth** -- username/password
- **Bearer token** -- token-based authentication
- **JWT** -- HS256/384/512, RS256/384/512, ES256/384, PS256/384/512, and EdDSA algorithms
- **Digest auth** -- MD5, SHA-256, and SHA-512

#### Collections and Environments

- **Collections** -- organize requests in JSON or YAML files with tree-based navigation
- **Collection folders** -- group requests into folders with automatic folder creation on startup
- **Environments** -- key-value variables with `{{variable}}` substitution across URLs, headers, bodies, auth, and scripts
- **Ephemeral collections** -- guard against accidental modification of auto-loaded collections
- **Rename and delete** -- collections and requests can be renamed or deleted from both TUI and CLI

#### Import and Export

- **Import Postman** -- collections and environments with folder depth control
- **Import cURL** -- single files or recursive directory scanning
- **Import OpenAPI** -- spec files with folder depth control
- **Import .http files** -- standard HTTP methods plus `WEBSOCKET` keyword, with recursive directory scanning
- **Export** -- HTTP, cURL, PHP Guzzle, Node.js Axios, and Rust reqwest formats

#### Terminal UI (TUI)

- **Interactive TUI** -- collection tree sidebar, request editor panels, response viewer, environment editor, cookie viewer, log panel, and theme picker
- **Keyboard navigation** -- navigate response sections and cycle through response tabs
- **Themes** -- 9 built-in themes: Default (Gruber Darker), Dracula, Catppuccin (Mocha, Macchiato, Frappe, Latte), Gruvbox, OpenCode, VS Code Dark
- **Custom themes** -- TOML-based theme files
- **Key bindings** -- Vim, Emacs, and Default modes with full customization via TOML
- **Cookie support** -- view and edit cookies in the TUI
- **Image preview** -- render response images directly in the terminal
- **Status line** -- contextual information display
- **Syntax highlighting** -- for request bodies and responses

#### CLI

- **One-off requests** -- `squrl try` for quick HTTP requests without saving
- **Collection management** -- list, info, new, delete, rename, and send
- **Request management** -- full CRUD for requests, URLs, methods, params, headers, auth, body, scripts, settings, and export
- **Environment management** -- info, key get/set/add/delete/rename
- **Shell completions** -- Bash, Zsh, and Fish via clap
- **Man page generation** -- via clap_mangen
- **Global options** -- working directory, dry-run mode, regex filtering, verbosity levels, and TUI launch flag

#### Configuration

- **Config file** -- `squrl.toml` with theme, syntax highlighting, image preview, response wrapping, proxy, and collection format settings
- **Global config** -- fallback config at `~/.config/squrl/global.toml`
- **Environment variables** -- `SQURL_MAIN_DIR`, `SQURL_THEME`, `SQURL_KEY_BINDINGS`
- **Auto-load .http files** -- from `requests/` subdirectory in git repositories
- **Clipboard support** -- copy response bodies and exports (optional feature)

#### Development and Tooling

- **Justfile** -- task runner with build, test, lint, format, security, coverage, install, and uninstall recipes
- **Nix flake** -- reproducible development environment
- **Pre-commit hooks** -- formatting, linting, and security checks
- **Integration tests** -- CLI, collection, completions, environment, import, request, HTTP send, theme, and try command tests
- **Code coverage** -- via cargo-llvm-cov
- **Security auditing** -- cargo-audit and cargo-deny
- **Install scripts** -- remote `scripts/curl-install.sh` for pre-built binaries and `scripts/curl-install.ps1` for Windows

[Unreleased]: https://github.com/FloodCreux/squrl/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/FloodCreux/squrl/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/FloodCreux/squrl/releases/tag/v0.1.0
