//! LFMF Counter CLI
//!
//! Command-line interface for viewing LFMF lesson counters and statistics.

use anyhow::Result;
use clap::{Parser, Subcommand};
use lfmf_counter::{CounterDisplay, LfmfCounter};

#[derive(Parser, Debug)]
#[command(author = "PromptExecution", version, about = "LFMF lesson counter statistics", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show summary statistics
    Summary(SummaryArgs),

    /// Show per-tool counters
    List(ListArgs),

    /// Show problematic tools (high error rate)
    Problematic(ProblematicArgs),

    /// Show top active tools
    Top(TopArgs),
}

#[derive(Parser, Debug)]
struct SummaryArgs {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Parser, Debug)]
struct ListArgs {
    //  Sort by activity (default) or tool name
    #[arg(short, long)]
    sort_by_activity: bool,

    // Show suggestions for tools
    #[arg(short, long)]
    show_suggestions: bool,

    // Limit number of tools to show
    #[arg(short, long, default_value = "10")]
    limit: usize,
}

#[derive(Parser, Debug)]
struct ProblematicArgs {
    /// Number of problematic tools to show
    #[arg(short, long, default_value = "5")]
    limit: usize,
}

#[derive(Parser, Debug)]
struct TopArgs {
    /// Number of top tools to show
    #[arg(short, long, default_value = "10")]
    limit: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let counter = LfmfCounter::load().await?;

    match cli.command {
        Commands::Summary(args) => {
            let summary = counter.get_summary();
            print_summary(&summary, args.verbose);
        }
        Commands::List(args) => {
            let tools = counter.get_stats().await?;

            if args.sort_by_activity {
                let active_tools = counter
                    .get_tools_by_activity()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>();
                print_tools_list(&active_tools, args.show_suggestions, args.limit);
            } else {
                print_tools_list(&tools, args.show_suggestions, args.limit);
            }
        }
        Commands::Problematic(args) => {
            let problematic = counter.get_problematic_tools();
            print_problematic_tools(&problematic, args.limit);
        }
        Commands::Top(args) => {
            let active_tools = counter.get_tools_by_activity();
            print_top_tools(&active_tools, args.limit);
        }
    }

    Ok(())
}

fn print_summary(summary: &lfmf_counter::CounterSummary, verbose: bool) {
    println!("📊 LFMF Counter Summary");
    println!("═════════════════════════════");
    println!("Total Tools: {}", summary.total_tools);
    println!("🦨 Total Skunks: {}", summary.total_skunk);
    println!("🐛 Total Bugs: {}", summary.total_bugs);
    println!("📚 Total Lessons: {}", summary.total_lessons);
    println!("⚠️  Problematic Tools: {}", summary.problematic_tools);

    if let Some(most_active) = &summary.most_active {
        println!("🏆 Most Active: {}", most_active);
    }

    if verbose {
        println!("\n💡 Tip: Use `b00t lfmf <tool> <topic>: <body>` to record lessons");
        println!("💡 Tip: Include 🦨 for code changes, 🐛 for lessons learned");
    }
}

fn print_tools_list(tools: &[lfmf_counter::ToolCounter], show_suggestions: bool, limit: usize) {
    println!("📋 Tools Counter");
    println!("═══════════════");

    for counter in tools.iter().take(limit) {
        let display = CounterDisplay::from_counter(counter);
        println!("{}", display.format());

        if show_suggestions {
            if let Some(suggestion) = display.get_suggestion() {
                println!("  💡 {}", suggestion);
            }
        }
    }
}

fn print_problematic_tools(tools: &[&lfmf_counter::ToolCounter], limit: usize) {
    if tools.is_empty() {
        println!("✅ No problematic tools found!");
        return;
    }

    println!("⚠️  Problematic Tools (High Error Rate)");
    println!("═════════════════════════════════════");

    for (i, counter) in tools.iter().take(limit).enumerate() {
        println!("  {}. {}", i + 1, counter.tool);
        println!(
            "     🐛: {} 🦨: {} ({}% error rate)",
            counter.bug_count,
            counter.skunk_count,
            (counter.bug_count as f64 / counter.total_lessons as f64 * 100.0) as usize
        );
    }
}

fn print_top_tools(tools: &[&lfmf_counter::ToolCounter], limit: usize) {
    println!("🏆 Top Active Tools");
    println!("═════════════════════════════");

    for (i, counter) in tools.iter().take(limit).enumerate() {
        let display = CounterDisplay::from_counter(counter);
        println!("  {}. {}", i + 1, display.format());
        println!("     📚 {} total lessons", counter.total_lessons);
    }
}
