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

mcp-cli-basic:
    ./scripts/mcp_cli_demo.sh basic

mcp-cli-spinning-wheels:
    ./scripts/mcp_cli_demo.sh spinning-wheels

mcp-doc-validate:
    ./scripts/mcp_cli_demo.sh basic
    ./scripts/mcp_cli_demo.sh spinning-wheels
    ./scripts/mcp_e2e.sh
