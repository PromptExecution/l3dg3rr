use std::path::PathBuf;

use clap::{Parser, Subcommand};

use xtask_mcpb::{
    bundler::McpbBundler,
    manifest::{
        ManifestAuthor, ManifestServer, McpConfig, McpbManifest,
        ServerType,
    },
    publisher::{GitHubPublisher, McpRegistryPublisher},
    server_json::ServerJson,
    verify::verify_bundle,
};

#[derive(Parser)]
#[command(name = "xtask", about = "l3dg3rr build and publish automation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile binary + assemble a deterministic .mcpb bundle
    Bundle {
        /// Path to the compiled ledgerr-mcp-server binary
        #[arg(long, default_value = "target/release/ledgerr-mcp-server")]
        binary: PathBuf,
        /// Output .mcpb path
        #[arg(long, default_value = "dist/ledgerr-mcp.mcpb")]
        output: PathBuf,
        /// Version string to embed in manifest (e.g. v0.1.0)
        #[arg(long)]
        version: String,
    },
    /// Print the manifest.json for a given version (no bundle created)
    Manifest {
        #[arg(long)]
        version: String,
    },
    /// Upload a .mcpb artifact to a GitHub release (requires gh CLI + GITHUB_TOKEN)
    PublishGithub {
        #[arg(long)]
        release_tag: String,
        #[arg(long)]
        artifact: PathBuf,
        /// Override repository (e.g. PromptExecution/l3dg3rr)
        #[arg(long)]
        repo: Option<String>,
    },
    /// Submit bundle to MCP Registry (requires mcp-publisher on PATH + auth)
    PublishRegistry {
        #[arg(long)]
        release_tag: String,
        /// Public download URL of the .mcpb artifact
        #[arg(long)]
        artifact_url: String,
        /// Hex SHA-256 of the .mcpb file (from `xtask bundle` output)
        #[arg(long)]
        sha256: String,
        #[arg(long, default_value = "io.github.prompt-execution/ledgerr-mcp")]
        server_name: String,
    },
    /// Validate a .mcpb bundle: ZIP structure, manifest, and entry_point presence
    Verify {
        path: PathBuf,
    },
    /// Update server.json version, mcpb identifier URL, and fileSha256 for a release.
    /// Run this before `mcp-publisher publish`.
    UpdateServerJson {
        /// Release version tag (e.g. v0.1.0)
        #[arg(long)]
        version: String,
        /// Public download URL of the canonical .mcpb artifact
        #[arg(long)]
        artifact_url: String,
        /// Hex SHA-256 of the .mcpb file (printed by `xtask-mcpb bundle`)
        #[arg(long)]
        sha256: String,
        /// Path to server.json (default: ./server.json)
        #[arg(long, default_value = "server.json")]
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Bundle { binary, output, version } => {
            let manifest = ledgerr_manifest(&version);
            let bundler = McpbBundler::new(manifest, binary, output);
            let artifact = bundler.bundle()?;
            println!("bundled: {}", artifact.path.display());
            println!("sha256:  {}", artifact.sha256);
            println!("size:    {} bytes", artifact.size_bytes);
        }

        Commands::Manifest { version } => {
            let manifest = ledgerr_manifest(&version);
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }

        Commands::PublishGithub { release_tag, artifact, repo } => {
            let mut publisher = GitHubPublisher::new(&release_tag);
            if let Some(r) = repo {
                publisher = publisher.with_repo(r);
            }
            publisher.upload(&artifact)?;
            println!("uploaded {} → release {}", artifact.display(), release_tag);
        }

        Commands::PublishRegistry { release_tag, artifact_url, sha256, server_name } => {
            let manifest = ledgerr_manifest(&release_tag);
            let publisher =
                McpRegistryPublisher::new(&release_tag, &server_name, &manifest.description);
            publisher.publish(&artifact_url, &sha256)?;
            println!("published {server_name} to MCP Registry @ {release_tag}");
        }

        Commands::Verify { path } => {
            let manifest = verify_bundle(&path)?;
            println!("ok: {} ({} {})", path.display(), manifest.name, manifest.version);
        }

        Commands::UpdateServerJson { version, artifact_url, sha256, path } => {
            let mut server_json = ServerJson::load(&path)?;
            server_json.update_mcpb(&version, &artifact_url, &sha256)?;
            server_json.save(&path)?;
            println!(
                "updated {}: version={version} sha256={sha256}",
                path.display()
            );
        }
    }
    Ok(())
}

/// Canonical manifest definition for ledgerr-mcp.
/// Update this when the server's configuration surface changes.
fn ledgerr_manifest(version: &str) -> McpbManifest {
    McpbManifest {
        manifest_version: "0.3".into(),
        name: "ledgerr-mcp".into(),
        version: version.into(),
        description: "Local-first U.S. expat tax document intelligence MCP server. \
            Ingests PDF statements, classifies transactions, and produces \
            CPA-auditable Excel workbooks — no data leaves your machine."
            .into(),
        author: ManifestAuthor {
            name: "l3dg3rr".into(),
            email: None,
            url: Some("https://github.com/PromptExecution/l3dg3rr".into()),
        },
        server: ManifestServer {
            server_type: ServerType::Binary,
            entry_point: "ledgerr-mcp-server".into(),
            mcp_config: McpConfig {
                command: "./ledgerr-mcp-server".into(),
                args: vec![],
                env: None,
            },
        },
        configuration: None,
    }
}
