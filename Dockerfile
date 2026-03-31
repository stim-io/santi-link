# syntax=docker/dockerfile:1.7

FROM rust:1.91-alpine AS builder

WORKDIR /app

RUN apk add --no-cache curl musl-dev

ENV CARGO_HOME=/usr/local/cargo \
    CARGO_TARGET_DIR=/tmp/cargo-target \
    CARGO_INCREMENTAL=1

COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/provider-openai-compatible/Cargo.toml ./crates/provider-openai-compatible/Cargo.toml
COPY crates/provider-openai-auth/Cargo.toml ./crates/provider-openai-auth/Cargo.toml
COPY crates/api/src ./crates/api/src
COPY crates/provider-openai-compatible/src ./crates/provider-openai-compatible/src
COPY crates/provider-openai-auth/src ./crates/provider-openai-auth/src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch --manifest-path crates/api/Cargo.toml --locked

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/tmp/cargo-target \
    cargo build --locked --profile container-dev -p provider-api \
    && mkdir -p /opt/artifacts \
    && cp /tmp/cargo-target/container-dev/provider-api /opt/artifacts/provider-api

FROM alpine:3.22

WORKDIR /app

RUN apk add --no-cache ca-certificates

COPY --from=builder /opt/artifacts/provider-api /usr/local/bin/provider-api

ENV PORT=8080

EXPOSE 8080

CMD ["provider-api"]
