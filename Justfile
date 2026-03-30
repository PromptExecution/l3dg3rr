set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

mcp-build:
    cargo build -p turbo-mcp --bin turbo-mcp-server

mcp-start:
    cargo run -p turbo-mcp --bin turbo-mcp-server

mcp-start-release:
    ./target/release/turbo-mcp-server

mcp-stop:
    pkill -f turbo-mcp-server || true

mcp-e2e:
    ./scripts/mcp_e2e.sh
