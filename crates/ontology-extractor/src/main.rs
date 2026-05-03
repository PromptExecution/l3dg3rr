//! Ontology Extractor CLI
//!
//! Command-line interface for extracting Rust AST and building ontological graphs.

use anyhow::Result;
use clap::Parser;
use ontology_extractor::{RustAstExtractor, TermClassification};

#[derive(Parser, Debug)]
#[command(author = "PromptExecution", version, about = "Extract Rust AST for ontological knowledge graph", long_about = None)]
struct Cli {
    /// Path to Rust crate root to analyze
    #[arg(short, long, default_value = ".")]
    crate_path: String,

    /// Output format: json or summary
    #[arg(short, long, default_value = "summary")]
    format: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🔍 Extracting Rust idioms from: {}", cli.crate_path);
    }

    let extractor = RustAstExtractor::new();
    let terms = extractor
        .extract_rust_idioms(std::path::Path::new(&cli.crate_path))
        .await?;

    if cli.format == "json" {
        let graph = extractor.build_ontology_graph(&terms);
        let json = graph.to_json()?;
        println!("{}", json);
    } else {
        // Summary format
        println!("📊 Ontology Extraction Summary");
        println!("═════════════════════════════");
        println!("Total terms extracted: {}", terms.len());

        let entity_count = terms
            .iter()
            .filter(|t| matches!(&t.classification, TermClassification::Entity { .. }))
            .count();
        let action_count = terms
            .iter()
            .filter(|t| matches!(&t.classification, TermClassification::Action { .. }))
            .count();

        println!("  Entities: {}", entity_count);
        println!("  Actions: {}", action_count);

        if cli.verbose {
            println!("\n📋 Top 5 Terms:");
            for (i, term) in terms.iter().take(5).enumerate() {
                println!("  {}. {} ({:?})", i + 1, term.name, term.classification);
            }
        }
    }

    Ok(())
}
