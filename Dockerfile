# syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY xtask ./xtask

RUN cargo test --workspace

FROM debian:bookworm-slim AS runtime
WORKDIR /app

COPY --from=builder /app /app

CMD ["/bin/bash", "-lc", "cargo test --workspace && echo 'tax-ledger workspace ready' "]
