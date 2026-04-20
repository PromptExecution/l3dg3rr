set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set dotenv-load := true

# Canonical build/test/run recipes live here. If a workflow needs a command,
# add or update the relevant `just` recipe and reference it from AGENTS.md.

mcp-build:
    cargo build -p ledgerr-mcp --bin ledgerr-mcp-server

mcp-start:
    cargo run -p ledgerr-mcp --bin ledgerr-mcp-server

mcp-start-release:
    ./target/release/ledgerr-mcp-server

mcp-stop:
    pkill -f ledgerr-mcp-server || true

# Build the Windows host binaries from WSL via PowerShell. This is the canonical
# path for `host-tray.exe` and `host-window.exe`.
wsl2-pwsh-build:
    powershell.exe -NoProfile -Command '$env:PATH = "C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;" + $env:PATH; Set-Location "C:\Users\wendy\l3dg3rr"; cargo build -p ledgerr-host --bin host-tray --bin host-window'

# Rebuild and launch the tray host on Windows.
wsl2-pwsh-run-tray:
    powershell.exe -NoProfile -Command '$env:PATH = "C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;" + $env:PATH; Set-Location "C:\Users\wendy\l3dg3rr"; cargo build -p ledgerr-host --bin host-tray | Out-Null; Get-Process host-tray -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 250; Start-Process -FilePath "C:\Users\wendy\l3dg3rr\target\debug\host-tray.exe" -WorkingDirectory "C:\Users\wendy\l3dg3rr"'

# Rebuild and launch the separate Slint host window on Windows.
wsl2-pwsh-run-window:
    powershell.exe -NoProfile -Command '$env:PATH = "C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;" + $env:PATH; Set-Location "C:\Users\wendy\l3dg3rr"; cargo build -p ledgerr-host --bin host-window | Out-Null; Start-Process -FilePath "C:\Users\wendy\l3dg3rr\target\debug\host-window.exe" -WorkingDirectory "C:\Users\wendy\l3dg3rr"'

# Build docs, start Windows-local HTTP server, and open browser for live Rhai diagram editing.
wsl2-pwsh-docserve:
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "C:\Users\wendy\l3dg3rr\scripts\docserve-live.ps1"

# Pull and run the MCP server from GHCR using podman (stdio transport)
# Usage: just mcp-podman-run        — latest image on main
#        just mcp-podman-run v0.2.0 — specific release tag
mcp-podman-run tag="main":
    @command -v podman >/dev/null || { echo "error: podman not found — install podman first"; exit 1; }
    podman pull ghcr.io/promptexecution/l3dg3rr:{{tag}}
    podman run --rm -i \
      -v "${LEDGER_DATA_DIR:-$PWD/data}:/data" \
      ghcr.io/promptexecution/l3dg3rr:{{tag}}

# Verify the GHCR image exists for a given tag without pulling the full image
mcp-podman-verify tag="main":
    @command -v podman >/dev/null || { echo "error: podman not found"; exit 1; }
    podman manifest inspect ghcr.io/promptexecution/l3dg3rr:{{tag}}

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
    cargo test --workspace --all-targets --all-features
    cargo build -p ledgerr-mcp --bin mcp-outcome-test
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

# ─── MCPB bundle + publish ────────────────────────────────────────────────────

# Build release binary and assemble a deterministic .mcpb bundle for one target
bundle target="x86_64-unknown-linux-musl":
    cargo build -p ledgerr-mcp --release --bin ledgerr-mcp-server --target {{target}}
    cargo xtask-mcpb bundle \
        --binary target/{{target}}/release/ledgerr-mcp-server \
        --output dist/ledgerr-mcp-{{target}}.mcpb \
        --version $(just v)

# Bundle for all tier-1 distribution targets (requires cross-compilation toolchains)
bundle-all:
    just bundle x86_64-unknown-linux-musl
    just bundle x86_64-apple-darwin
    just bundle aarch64-apple-darwin

# Print the manifest.json for the current version (no bundle created)
manifest:
    cargo xtask-mcpb manifest --version $(just v)

# Verify a .mcpb bundle's structure and manifest
verify path:
    cargo xtask-mcpb verify {{path}}

# Upload all dist/*.mcpb artifacts to a GitHub release
publish-mcpb tag="":
    #!/bin/bash
    set -euo pipefail
    TAG="{{tag}}"
    if [ -z "$TAG" ]; then TAG=$(gh release list --limit 1 --json tagName --jq '.[0].tagName'); fi
    shopt -s nullglob
    bundles=(dist/*.mcpb)
    if [ ${#bundles[@]} -eq 0 ]; then
      echo "error: no .mcpb files found in dist/ — run 'just bundle' first"
      exit 1
    fi
    for f in "${bundles[@]}"; do
      cargo xtask-mcpb publish-github --release-tag "$TAG" --artifact "$f"
    done

# Update server.json with the current release version + sha256 from a bundle artifact.
# Run before `mcp-publisher publish`.
update-server-json artifact sha256="":
    #!/bin/bash
    set -euo pipefail
    SHA="{{sha256}}"
    if [ -z "$SHA" ]; then
        SHA=$(sha256sum "{{artifact}}" | cut -d' ' -f1)
    fi
    VERSION=$(just v)
    FILENAME=$(basename "{{artifact}}")
    cargo xtask-mcpb update-server-json \
        --version "$VERSION" \
        --sha256 "$SHA" \
        --artifact-url "https://github.com/PromptExecution/l3dg3rr/releases/download/$VERSION/$FILENAME"

# Submit bundle to MCP Registry (requires mcp-publisher on PATH + registry auth)
publish-registry tag artifact-url sha256:
    cargo xtask-mcpb publish-registry \
        --release-tag {{tag}} \
        --artifact-url {{artifact-url}} \
        --sha256 {{sha256}}

# ─── Cocogitto release recipe (major|minor|patch, defaults to patch) ──────────

[private]
ensure-cog:
    @PATH="${HOME}/.cargo/bin:${PATH}" bash -eu -o pipefail -c 'if command -v cog >/dev/null 2>&1; then echo "Using existing cog"; else echo "cog not found; installing cocogitto..."; cargo install cocogitto; fi'

# Cocogitto release recipe (major|minor|patch, defaults to patch)
release version="patch": ensure-cog
    #!/bin/bash
    set -euo pipefail
    export PATH="${HOME}/.cargo/bin:${PATH}"
    case "{{version}}" in
        major|minor|patch) ;;
        *) echo "Invalid version: {{version}} (use major, minor, or patch)" && exit 1 ;;
    esac
    echo "Running pre-release checks..."
    cargo test --workspace --all-targets --all-features
    ./scripts/e2e_mvp.sh
    echo "Bumping {{version}} version with cocogitto..."
    cog bump --{{version}}
    cog changelog
    echo "Pushing tags..."
    git push --follow-tags

# Show current version
v: ensure-cog
    @PATH="${HOME}/.cargo/bin:${PATH}" cog get-version

# Validate commits
validate: ensure-cog
    @PATH="${HOME}/.cargo/bin:${PATH}" cog check

# Show changelog
changelog: ensure-cog
    @PATH="${HOME}/.cargo/bin:${PATH}" cog changelog

# Show release stats
stats:
    @echo "Tags:"
    @git tag -l
    @echo ""
    @echo "Recent commits:"
    @git log --oneline -5

# Build mdbook documentation locally
# Requires: cargo install mdbook mdbook-mermaid && cargo install --path crates/mdbook-rhai-mermaid
docgen:
    @if [ ! -x ~/.cargo/bin/mdbook ]; then echo "error: mdbook not found — run: cargo install mdbook mdbook-mermaid"; exit 1; fi
    @if [ ! -x ~/.cargo/bin/mdbook-mermaid ]; then echo "error: mdbook-mermaid not found — run: cargo install mdbook-mermaid"; exit 1; fi
    @if [ ! -x ~/.cargo/bin/mdbook-rhai-mermaid ]; then cargo install --path crates/mdbook-rhai-mermaid --quiet; fi
    PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/mdbook build book
    @echo "Docs built in book/book/ — serve with: npx serve book/book"

# Build and serve mdbook locally with the live Rhai editor enabled
docserve host="127.0.0.1" port="3000":
    @if [ ! -x ~/.cargo/bin/mdbook ]; then echo "error: mdbook not found — run: cargo install mdbook mdbook-mermaid"; exit 1; fi
    @if [ ! -x ~/.cargo/bin/mdbook-mermaid ]; then echo "error: mdbook-mermaid not found — run: cargo install mdbook-mermaid"; exit 1; fi
    @if [ ! -x ~/.cargo/bin/mdbook-rhai-mermaid ]; then cargo install --path crates/mdbook-rhai-mermaid --quiet; fi
    PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/mdbook build book
    @echo "Serving http://{{host}}:{{port}}"
    cd book/book && python3 -m http.server {{port}} --bind {{host}}

# Verify docs build, rhai→mermaid injection happened, diagrams render, cross-references valid
docgen-check:
    @if [ ! -x ~/.cargo/bin/mdbook ]; then echo "error: mdbook not found — run: cargo install mdbook mdbook-mermaid"; exit 1; fi
    @if [ ! -x ~/.cargo/bin/mdbook-mermaid ]; then echo "error: mdbook-mermaid not found — run: cargo install mdbook-mermaid"; exit 1; fi
    @if [ ! -x ~/.cargo/bin/mdbook-rhai-mermaid ]; then cargo install --path crates/mdbook-rhai-mermaid --quiet; fi
    PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/mdbook build book
    @echo "Checking for rendered SVG diagrams..."
    @grep -q '<svg' book/book/theory.html && echo "✓ theory.html has SVG diagrams" || { echo "error: no SVG in theory.html"; exit 1; }
    @grep -q '<svg' book/book/pipeline.html && echo "✓ pipeline.html has SVG diagrams" || { echo "error: no SVG in pipeline.html"; exit 1; }
    @grep -q '<svg' book/book/visualize.html && echo "✓ visualize.html has SVG diagrams" || { echo "error: no SVG in visualize.html"; exit 1; }
    @echo "Verifying cross-references..."
    @grep -q 'href="./graph.html"' book/book/intro.html && echo "✓ intro.html references graph.html" || exit 1
    @grep -q 'href="./validation.html"' book/book/pipeline.html && echo "✓ pipeline.html references validation.html" || exit 1
    @grep -q 'href="./pipeline.html"' book/book/validation.html && echo "✓ validation.html references pipeline.html" || exit 1
    @echo "Verifying rhai→mermaid injection..."
    @grep -q 'class="language-rhai"' book/book/theory.html && echo "✓ theory.html has rhai source blocks" || exit 1
    @grep -q 'class="mermaid"' book/book/theory.html && echo "✓ theory.html has generated mermaid blocks" || { echo "error: rhai→mermaid injection missing in theory.html"; exit 1; }
    @grep -q 'theme/rhai-live-' book/book/theory.html && echo "✓ theory.html loads live-editor assets" || { echo "error: live-editor JS missing in theory.html"; exit 1; }
    @echo "Running live-editor unit tests..."
    @node --test book/theme/rhai-live-core.test.js
    @echo "All documentation diagrams validated!"
