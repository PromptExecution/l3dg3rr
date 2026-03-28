#!/usr/bin/env bash
set -euo pipefail

git config core.hooksPath .githooks
chmod +x .githooks/commit-msg
echo "Git hooks installed: core.hooksPath=.githooks"
echo "Conventional commit enforcement is active."

