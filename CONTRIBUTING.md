# Contributing to squrl

Thank you for your interest in contributing to squrl!

## Getting Started

1. Fork and clone the repository:
   ```sh
   git clone https://codeberg.org/flood-mike/squrl.git
   cd squrl
   ```

2. Install prerequisites:
   - [Rust](https://www.rust-lang.org/tools/install) (nightly toolchain)
   - [just](https://github.com/casey/just)

3. Build and run tests:
   ```sh
   just build
   just test
   ```

## Development Workflow

```sh
just build          # Build the project
just test           # Run all tests
just lint           # Run clippy lints
just fmt            # Format code
just fmt-check      # Check formatting
```

Run `just` to see all available commands.

## Submitting Changes

1. Create a feature branch from `main`.
2. Make your changes and ensure `just lint` and `just test` pass.
3. Format your code with `just fmt`.
4. Submit a pull request with a clear description of what you changed and why.

## Reporting Issues

Open an issue on the [issue tracker](https://codeberg.org/flood-mike/squrl/issues) with:
- A clear description of the problem or suggestion
- Steps to reproduce (for bugs)
- Expected vs actual behavior

## Code Style

- Follow existing patterns in the codebase.
- Run `just fmt` before committing.
- Ensure `just lint` produces no warnings.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
