# syntax=docker/dockerfile:1.7

# ── dependency cache layer (cargo-chef) ──────────────────────────────────────
FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY xtask ./xtask
RUN cargo chef prepare --recipe-path recipe.json

# ── build ─────────────────────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY xtask ./xtask

RUN cargo test --workspace --all-features
RUN cargo build -p ledgerr-mcp --release --bin ledgerr-mcp-server

# ── runtime ───────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/ledgerr-mcp-server /usr/local/bin/ledgerr-mcp-server

ENV LEDGER_WORKBOOK_PATH=/data/tax-ledger.xlsx
ENV LEDGER_PDF_INBOX=/data/inbox

CMD ["/usr/local/bin/ledgerr-mcp-server"]
