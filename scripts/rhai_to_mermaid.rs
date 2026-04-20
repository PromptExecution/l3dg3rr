# Rhai Pattern Visualization
# Auto-generates Mermaid diagrams from executable Rhai code samples

use rhai::{Engine, Dynamic};

fn visualize_rhai_pattern(script: &str) -> String {
    let engine = Engine::new_raw();
    
    let mut mermaid = String::from("flowchart TD\n");
    
    // Parse function definitions to extract nodes
    let lines: Vec<&str> = script.lines().collect();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.starts_with("let ") {
            // Extract function/call patterns
            if let Some(name) = extract_identifier(trimmed) {
                mermaid.push_str(&format!("    {}[\"{}\"]\n", name, name));
            }
        }
        if trimmed.contains("->") {
            // Extract transitions
            let parts: Vec<&str> = trimmed.split("->").collect();
            if parts.len() == 2 {
                let from = extract_identifier(parts[0]).unwrap_or("?");
                let to = extract_identifier(parts[1]).unwrap_or("?");
                mermaid.push_str(&format!("    {} --> {}\n", from, to));
            }
        }
    }
    
    mermaid
}

fn extract_identifier(line: &str) -> Option<String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    // Find first identifier-like word
    for word in &words {
        if word.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) 
            && !word.starts_with("//")
            && !word.starts_with("fn")
            && !word.starts_with("let")
        {
            return Some(word.trim_matches(|c| c == '(' || c == ')' || c == ';').to_string());
        }
    }
    None
}

fn main() {
    let examples = vec![
        r#"
fn ingest() -> validate
fn validate() -> classify
fn classify() -> reconcile
fn reconcile() -> commit
        "#,
        r#"
if confidence > 0.8 -> commit
if confidence < 0.5 -> review
        "#,
    ];
    
    for (i, script) in examples.iter().enumerate() {
        let diagram = visualize_rhai_pattern(script);
        println!("\n=== Pattern {} ===\n{}", i + 1, diagram);
    }
}