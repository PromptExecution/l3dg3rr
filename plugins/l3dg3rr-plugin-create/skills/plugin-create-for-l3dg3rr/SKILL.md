---
description: Bootstrap and validate l3dg3rr MCP usage in Claude Cowork Plugin Create workflows
---

Use this skill to set up and verify `l3dg3rr` MCP access in Cowork:

1. Confirm the plugin is installed from `promptexecution-tools`.
2. Run `tools/list` and verify `l3dg3rr_*` entries exist.
3. Run one safe capability call first:
   - `l3dg3rr_context_summary`
4. If MCP is unavailable, check:
   - `cargo` is installed
   - repository path is correct
   - `turbo-mcp-server` starts cleanly

When extending the plugin:

- Keep responses deterministic and concise for small models.
- Prefer l3dg3rr service tools over ad-hoc shell parsing of financial data.
- Preserve permission boundaries and surface explicit blocked diagnostics.
