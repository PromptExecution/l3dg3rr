set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set dotenv-load := true

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

mcp-doc-demo:
    ./scripts/mcp_cli_demo.sh basic
    ./scripts/mcp_cli_demo.sh spinning-wheels
    ./scripts/mcp_e2e.sh

test:
    cargo build -p turbo-mcp --bin turbo-mcp-server --bin mcp-outcome-test
    ./target/debug/mcp-outcome-test

gh-secrets-help:
    @echo "Expected .env values (optional):"
    @echo "  CRATES_IO_TOKEN=..."
    @echo "  PYPI_API_TOKEN=..."
    @echo ""
    @echo "Recipes:"
    @echo "  just gh-secrets-set-repo"
    @echo "  just gh-secrets-set-repo repo=PromptExecution/l3dg3rr force=true"
    @echo "  just gh-secrets-set-org org=PromptExecution repos=l3dg3rr"
    @echo "  just gh-secrets-set-org org=PromptExecution repos=l3dg3rr force=true"

gh-secrets-set-repo repo="PromptExecution/l3dg3rr" force="false":
    @command -v gh >/dev/null || { echo "gh CLI not found"; exit 1; }
    @gh auth status >/dev/null
    @for name in CRATES_IO_TOKEN PYPI_API_TOKEN; do \
      value="${!name:-}"; \
      if [ -z "$value" ]; then \
        echo "SKIP $name: not set in .env"; \
        continue; \
      fi; \
      if gh secret list -R "{{repo}}" | awk '{print $1}' | grep -qx "$name"; then \
        if [ "{{force}}" = "true" ]; then \
          printf "%s" "$value" | gh secret set "$name" -R "{{repo}}"; \
          echo "UPDATE $name: repo={{repo}}"; \
        else \
          echo "SKIP $name: already exists in repo={{repo}} (force=true to overwrite)"; \
        fi; \
      else \
        printf "%s" "$value" | gh secret set "$name" -R "{{repo}}"; \
        echo "SET $name: repo={{repo}}"; \
      fi; \
    done

gh-secrets-set-org org="PromptExecution" repos="l3dg3rr" force="false":
    @command -v gh >/dev/null || { echo "gh CLI not found"; exit 1; }
    @gh auth status >/dev/null
    @for name in CRATES_IO_TOKEN PYPI_API_TOKEN; do \
      value="${!name:-}"; \
      if [ -z "$value" ]; then \
        echo "SKIP $name: not set in .env"; \
        continue; \
      fi; \
      if gh secret list --org "{{org}}" | awk '{print $1}' | grep -qx "$name"; then \
        if [ "{{force}}" = "true" ]; then \
          printf "%s" "$value" | gh secret set "$name" --org "{{org}}" --visibility selected --repos "{{repos}}"; \
          echo "UPDATE $name: org={{org}} repos={{repos}}"; \
        else \
          echo "SKIP $name: already exists in org={{org}} (force=true to overwrite)"; \
        fi; \
      else \
        printf "%s" "$value" | gh secret set "$name" --org "{{org}}" --visibility selected --repos "{{repos}}"; \
        echo "SET $name: org={{org}} repos={{repos}}"; \
      fi; \
    done
