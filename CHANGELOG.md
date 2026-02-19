# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- **Install script** -- remote `curl-install.sh` for pre-built binaries

[Unreleased]: https://github.com/FloodCreux/squrl/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/FloodCreux/squrl/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/FloodCreux/squrl/releases/tag/v0.1.0
