# squrl

An async HTTP request client library for Rust, built on top of [reqwest](https://github.com/seanmonstar/reqwest) and [Tokio](https://tokio.rs/).

## Features

- Async HTTP request execution with configurable timeouts
- Request cancellation via `CancellationToken`
- Response parsing for text, JSON, and image content types
- Pretty-printed JSON responses
- Header and cookie capture
- Middleware support via `reqwest-middleware`
- Structured logging with `tracing`

## Usage

```rust
use squrl::app::request::http::send::send_http_request;
use squrl::models::request::Request;
use std::sync::Arc;
use parking_lot::RwLock;

let request = Arc::new(RwLock::new(Request::default()));
let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
let request_builder = client.get("https://httpbin.org/get");

let response = send_http_request(request_builder, request).await?;
```

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
