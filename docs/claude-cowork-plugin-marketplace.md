# Claude Cowork Plugin Marketplace Approach

## Goal

Distribute this repository as an installable Cowork plugin marketplace entry, with a dedicated `l3dg3rr-plugin-create` plugin that exposes `turbo-mcp-server`.

## Artifacts in this repo

- Marketplace catalog: `.claude-plugin/marketplace.json`
- Plugin manifest: `plugins/l3dg3rr-plugin-create/.claude-plugin/plugin.json`
- Plugin skill: `plugins/l3dg3rr-plugin-create/skills/plugin-create-for-l3dg3rr/SKILL.md`

## Cowork install flow

In Claude Code/Cowork:

```text
/plugin marketplace add https://github.com/PromptExecution/l3dg3rr
/plugin install l3dg3rr-plugin-create@promptexecution-tools
```

Then validate:

```text
/plugin list
/plugin show l3dg3rr-plugin-create
```

In a Cowork task, run:

```text
tools/list
tools/call l3dg3rr_context_summary {}
```

## Organization distribution model

1. Host this marketplace on GitHub (recommended distribution path).
2. Team admins add it as an approved marketplace.
3. Users install `l3dg3rr-plugin-create` from the marketplace catalog.
4. For stricter controls, combine with managed marketplace restrictions in team settings.

## CI publication assist

Use the official Claude Code GitHub Action for PR/issue automation while maintaining this marketplace catalog in-repo:

- Marketplace listing: `https://github.com/marketplace/actions/claude-code-action-official`
- Keep plugin artifacts versioned and reviewed via PR before publishing updates.

## Notes

- This approach uses Git-hosted marketplace distribution and relative plugin source paths.
- The MCP server command currently assumes `cargo` availability on the host running Cowork/plugin execution.
- For container-first environments, replace `mcpServers.l3dg3rr.command` with a docker launcher profile.
