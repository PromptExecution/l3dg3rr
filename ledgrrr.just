# ── ledgrrr — ledgerr-mcp lifecycle (just module example) ─────────────────
#
# Just module syntax:
#   In parent justfile:  mod ledgrrr 'vendor/ledgrrr/ledgrrr.just'
#   Invocation:          just ledgrrr build
#   List recipes:        just --list ledgrrr
#
# Module scope:
#   - Recipes and variables are scoped to the module
#   - Recipes run in the module's source directory
#   - Environment variables loaded only in root justfile
#
# symlink: vendor/ledgrrr -> vendor/l3dg3rr  (polyseme mapping)
#
# Reference: https://just.systems/man/en/modules.html

# ─── Build ─────────────────────────────────────────────────────────────────────

# Build ledgerr-mcp binary from vendor workspace
build:
    cargo build --release -p ledgerr-mcp

# Build Docker image for ledgerr-mcp
docker-build:
    docker build -f Dockerfile.ledgerr-mcp -t ledgerr-mcp:latest .

# ─── Start/Stop ────────────────────────────────────────────────────────────────

# Run ledgerr-mcp container on port 8001
docker-run:
    docker rm -f ledgerr-mcp 2>/dev/null || true
    docker run -d --name ledgerr-mcp -p 8001:8001 ledgerr-mcp:latest
    echo "started on :8001"

# Stop and remove ledgerr-mcp container
docker-stop:
    docker rm -f ledgerr-mcp 2>/dev/null || echo "no container"

# Rebuild and restart
docker-restart: docker-build docker-stop docker-run

# ─── Status ────────────────────────────────────────────────────────────────────

# Check if ledgerr-mcp container is running
docker-status:
    docker inspect ledgerr-mcp --format '{{ "{{" }}.State.Status{{ "}}" }}' 2>/dev/null || echo "not running"

# Follow container logs
docker-logs:
    docker logs -f ledgerr-mcp

# ─── Install ───────────────────────────────────────────────────────────────────

# Build native binary and copy to ~/.cargo/bin
install: build
    cp target/release/ledgerr-mcp-server {{env_var_or_default("HOME", "~")}}/.cargo/bin/ledgerr-mcp
    echo "installed"
