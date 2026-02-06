# squrl

A terminal HTTP client built with Rust, powered by [ratatui](https://github.com/ratatui/ratatui), [reqwest](https://github.com/seanmonstar/reqwest), and [Tokio](https://tokio.rs/).

## Features

- TUI and CLI modes
- Async HTTP request execution with configurable timeouts and cancellation
- Environment management with key-value variable support
- Response parsing for text, JSON, and image content types
- Pretty-printed JSON responses
- Header and cookie capture
- Man page generation

## Installation

```sh
cargo install --path .
```

## Usage

### CLI

```sh
# Environment management
squrl env info <name>              # Show environment details
squrl env info <name> --os-vars    # Include OS environment variables
squrl env key <name> get <key>     # Get a key value
squrl env key <name> set <key> <value>   # Update a key
squrl env key <name> add <key> <value>   # Add a new key
squrl env key <name> delete <key>        # Delete a key
squrl env key <name> rename <key> <new>  # Rename a key

# Man pages
squrl man [output_dir]             # Generate man pages

# Options
squrl --directory <dir>            # Set working directory (or SQURL_MAIN_DIR env var)
squrl --dry-run                    # Test without saving changes
```

### TUI

Run `squrl` with no subcommand to launch the interactive terminal UI.

## Development

Requires [Rust](https://www.rust-lang.org/tools/install) (nightly) and [just](https://github.com/casey/just).

```sh
just build    # Build the project
just test     # Run all tests
just lint     # Run clippy lints
just fmt      # Format code
```

Run `just` to see all available commands.

## License

[MIT](LICENSE)
