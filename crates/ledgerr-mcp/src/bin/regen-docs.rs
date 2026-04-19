/// Regenerates docs/ and scripts/ files from the Rust contract source.
/// Run: cargo run -p ledgerr-mcp --bin regen-docs
use std::path::PathBuf;

use ledgerr_mcp::contract;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn write(root: &PathBuf, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::write(&path, content).unwrap_or_else(|e| panic!("write {rel}: {e}"));
    println!("wrote {rel}");
}

fn main() {
    let root = repo_root();
    write(
        &root,
        "docs/mcp-capability-contract.md",
        &contract::generated_capability_contract_markdown(),
    );
    write(
        &root,
        "docs/agent-mcp-runbook.md",
        &contract::generated_agent_runbook_markdown(),
    );
    write(
        &root,
        "scripts/mcp_cli_demo.sh",
        &contract::generated_mcp_cli_demo_script(),
    );
    println!("done");
}
