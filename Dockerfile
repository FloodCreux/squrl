# syntax=docker/dockerfile:1

# ------------------------------------------------------------------------------
# Stage 1: Base image with Rust nightly + cargo-chef
# Runs on the BUILD platform (host arch) for cross-compilation support
# ------------------------------------------------------------------------------
FROM --platform=$BUILDPLATFORM rust:nightly-alpine3.21 AS chef
RUN apk add --no-cache musl-dev && \
    cargo install --locked cargo-chef
WORKDIR /app

# ------------------------------------------------------------------------------
# Stage 2: Planner - extract dependency recipe from Cargo.toml/Cargo.lock
# The recipe.json only changes when dependencies change, enabling layer caching
# ------------------------------------------------------------------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Stage 3: Builder - compile dependencies (cached) then build the binary
# Uses Zig as a cross-compilation linker via cargo-zigbuild
# ------------------------------------------------------------------------------
FROM chef AS builder

ARG TARGETPLATFORM

# Map Docker platform to Rust target triple, install cross-compilation tooling
RUN case "$TARGETPLATFORM" in \
        "linux/amd64") echo "x86_64-unknown-linux-musl" > /rust_target.txt ;; \
        "linux/arm64") echo "aarch64-unknown-linux-musl" > /rust_target.txt ;; \
        *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac && \
    rustup target add $(cat /rust_target.txt) && \
    apk add --no-cache zig && \
    cargo install --locked cargo-zigbuild

# Build dependencies only (cached until Cargo.toml/Cargo.lock change)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook \
    --release \
    --no-default-features \
    --target $(cat /rust_target.txt) \
    --recipe-path recipe.json \
    --zigbuild

# Build the application
COPY . .
RUN cargo zigbuild \
    --release \
    --no-default-features \
    --target $(cat /rust_target.txt) \
    --bin squrl && \
    cp /app/target/$(cat /rust_target.txt)/release/squrl /squrl

# ------------------------------------------------------------------------------
# Stage 4: Runtime - minimal image with just the binary
# ------------------------------------------------------------------------------
FROM alpine:3.21 AS runtime
COPY --from=builder /squrl /usr/local/bin/squrl
WORKDIR /app
ENTRYPOINT ["squrl"]
