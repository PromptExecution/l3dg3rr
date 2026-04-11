set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

mcp-build:
    cargo build -p ledgerr-mcp --bin ledgerr-mcp-server

mcp-start:
    cargo run -p ledgerr-mcp --bin ledgerr-mcp-server

mcp-start-release:
    ./target/release/ledgerr-mcp-server

mcp-stop:
    pkill -f ledgerr-mcp-server || true

mcp-e2e:
    ./scripts/mcp_e2e.sh

mcp-cli-basic:
    ./scripts/mcp_cli_demo.sh basic

mcp-cli-spinning-wheels:
    ./scripts/mcp_cli_demo.sh spinning-wheels

mcp-doc-demo:
    ./scripts/mcp_cli_demo.sh basic
    ./scripts/mcp_cli_demo.sh spinning-wheels
    ./scripts/mcp_e2e.sh

test:
    cargo build -p ledgerr-mcp --bin ledgerr-mcp-server --bin mcp-outcome-test
    ./target/debug/mcp-outcome-test
