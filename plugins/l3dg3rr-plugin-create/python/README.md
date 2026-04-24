# l3dg3rr-mcp-launcher

Small launcher package for running `ledgerr-mcp-server` via:

- `cargo`
- `binary`
- `docker`

Examples:

```bash
python -m l3dg3rr_mcp_launcher --mode cargo
python -m l3dg3rr_mcp_launcher --mode binary --binary ./target/release/ledgerr-mcp-server
python -m l3dg3rr_mcp_launcher --mode docker --image tax-ledger:dev
```
