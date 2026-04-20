mod emitter;
mod parser;

use emitter::emit_mermaid;
use parser::parse;
use serde_json::Value;
use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // mdbook calls `mdbook-rhai-mermaid supports <renderer>` to check support.
    if args.len() >= 2 && args[1] == "supports" {
        // We support all renderers; just exit 0.
        std::process::exit(0);
    }

    // Normal invocation: read [PreprocessorContext, Book] from stdin.
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("mdbook-rhai-mermaid: failed to read stdin: {}", e);
        std::process::exit(1);
    }

    let pair: Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("mdbook-rhai-mermaid: failed to parse JSON input: {}", e);
            std::process::exit(1);
        }
    };

    // pair is [context, book]; we only need to mutate and return the book.
    let book = match pair.get(1) {
        Some(b) => b.clone(),
        None => {
            eprintln!("mdbook-rhai-mermaid: unexpected JSON structure (expected [ctx, book])");
            std::process::exit(1);
        }
    };

    let modified = process_book(book);

    let output = match serde_json::to_string(&modified) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mdbook-rhai-mermaid: failed to serialize output: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = io::stdout().write_all(output.as_bytes()) {
        eprintln!("mdbook-rhai-mermaid: failed to write stdout: {}", e);
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Book processing
// ---------------------------------------------------------------------------

/// Walk the entire book value, mutating chapter content in-place.
fn process_book(mut book: Value) -> Value {
    // mdbook 0.5.x serializes Book with key "items"; older versions used "sections"
    let key = if book.get("items").is_some() { "items" } else { "sections" };
    process_sections(book.get_mut(key));
    book
}

fn process_sections(sections: Option<&mut Value>) {
    if let Some(Value::Array(book_items)) = sections {
        for item in book_items.iter_mut() {
            process_book_item(item);
        }
    }
}

/// A BookItem is one of:
///   `{ "Chapter": { "content": "...", "sub_items": [...] } }`
///   `"Separator"`
///   `{ "PartTitle": "..." }`
fn process_book_item(item: &mut Value) {
    if let Some(chapter) = item.get_mut("Chapter") {
        process_chapter(chapter);
    }
    // Separator and PartTitle are pass-through.
}

fn process_chapter(chapter: &mut Value) {
    // Process this chapter's content.
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        *content = inject_mermaid_blocks(content);
    }

    // Recurse into sub_items.
    if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
        for sub in sub_items.iter_mut() {
            process_book_item(sub);
        }
    }
}

// ---------------------------------------------------------------------------
// Rhai block detection and mermaid injection
// ---------------------------------------------------------------------------

/// Scan `content` for ` ```rhai ` blocks and inject a mermaid block after each.
fn inject_mermaid_blocks(content: &str) -> String {
    let mut result = String::with_capacity(content.len() + 256);
    let mut remaining = content;

    loop {
        // Find the next ```rhai fence.
        let fence_start = match find_rhai_fence_start(remaining) {
            Some(pos) => pos,
            None => {
                // No more rhai blocks; append the rest and finish.
                result.push_str(remaining);
                break;
            }
        };

        // Append everything before the fence.
        result.push_str(&remaining[..fence_start]);

        let after_open = &remaining[fence_start..];

        // Find the end of the opening fence line.
        let body_start = match after_open.find('\n') {
            Some(pos) => pos + 1,
            None => {
                // Malformed (no newline after opening fence); pass through unchanged.
                result.push_str(after_open);
                break;
            }
        };

        let body_and_rest = &after_open[body_start..];

        // Find the closing ``` fence.
        let close_pos = match find_closing_fence(body_and_rest) {
            Some(pos) => pos,
            None => {
                // Unclosed block; pass through unchanged.
                result.push_str(after_open);
                break;
            }
        };

        let rhai_body = &body_and_rest[..close_pos];

        // Length of the closing fence line (``` + optional whitespace + newline).
        let close_line_end = find_close_line_end(&body_and_rest[close_pos..]);

        let rhai_block_end = body_start + close_pos + close_line_end;

        // Append the original rhai block verbatim.
        result.push_str(&after_open[..rhai_block_end]);

        // Parse and emit mermaid.
        let graph = parse(rhai_body);
        if !graph.nodes.is_empty() || !graph.edges.is_empty() {
            let mermaid = emit_mermaid(&graph);
            result.push_str("\n\n```mermaid\n");
            result.push_str(&mermaid);
            result.push_str("```\n");
        }

        // Advance past the processed block.
        remaining = &remaining[fence_start + rhai_block_end..];
    }

    result
}

/// Find the byte offset of the next ` ```rhai ` opening fence in `s`.
/// The line must start with ` ``` ` (no leading spaces in standard markdown fences)
/// optionally followed by `rhai` and optional trailing whitespace.
fn find_rhai_fence_start(s: &str) -> Option<usize> {
    let mut search = s;
    let mut offset = 0;

    loop {
        let pos = search.find("```rhai")?;
        let abs_pos = offset + pos;

        // Verify the fence is at the start of a line (pos == 0 or preceded by '\n').
        let at_line_start = pos == 0 || search.as_bytes().get(pos - 1) == Some(&b'\n');

        // Verify the rest of the opening fence line is only whitespace.
        let after_fence = &search[pos + 7..]; // skip "```rhai"
        let rest_of_line = match after_fence.find('\n') {
            Some(nl) => &after_fence[..nl],
            None => after_fence,
        };
        let only_whitespace = rest_of_line.chars().all(|c| c.is_whitespace());

        if at_line_start && only_whitespace {
            return Some(abs_pos);
        }

        // Skip past this match and keep looking.
        offset += pos + 7;
        search = &search[pos + 7..];
    }
}

/// Find the byte offset (within `s`) of a closing ` ``` ` fence line.
fn find_closing_fence(s: &str) -> Option<usize> {
    let mut search = s;
    let mut offset = 0;

    loop {
        let pos = search.find("```")?;
        let abs_pos = offset + pos;

        let at_line_start = pos == 0 || search.as_bytes().get(pos - 1) == Some(&b'\n');

        let after = &search[pos + 3..];
        let rest_of_line = match after.find('\n') {
            Some(nl) => &after[..nl],
            None => after,
        };
        let only_whitespace = rest_of_line.chars().all(|c| c.is_whitespace());

        if at_line_start && only_whitespace {
            return Some(abs_pos);
        }

        offset += pos + 3;
        search = &search[pos + 3..];
    }
}

/// Return the length of the closing fence line (including the trailing newline if present).
fn find_close_line_end(s: &str) -> usize {
    match s.find('\n') {
        Some(pos) => pos + 1,
        None => s.len(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_simple_pipeline() {
        let content = "# Title\n\n```rhai\nfn ingest() -> classify\n```\n\nSome text.\n";
        let result = inject_mermaid_blocks(content);
        assert!(result.contains("```rhai\n"), "should preserve rhai block");
        assert!(result.contains("```mermaid\n"), "should inject mermaid block");
        assert!(result.contains("flowchart TD\n"), "mermaid should be a flowchart");
        // Mermaid block must come after the rhai block.
        let rhai_pos = result.find("```rhai").unwrap();
        let mermaid_pos = result.find("```mermaid").unwrap();
        assert!(mermaid_pos > rhai_pos, "mermaid block should be after rhai block");
    }

    #[test]
    fn test_inject_multiple_blocks() {
        let content =
            "```rhai\nfn a() -> b\n```\n\nMiddle.\n\n```rhai\nfn c() -> d\n```\n";
        let result = inject_mermaid_blocks(content);
        let count = result.matches("```mermaid").count();
        assert_eq!(count, 2, "should inject one mermaid block per rhai block");
    }

    #[test]
    fn test_no_rhai_blocks_unchanged() {
        let content = "# Plain\n\nNo code blocks here.\n";
        let result = inject_mermaid_blocks(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_empty_rhai_block_no_injection() {
        let content = "```rhai\n// only a comment\n```\n";
        let result = inject_mermaid_blocks(content);
        // Graph is empty — no mermaid block should be injected.
        assert!(!result.contains("```mermaid"), "empty graph should not inject mermaid");
    }

    #[test]
    fn test_other_code_blocks_unaffected() {
        let content = "```rust\nfn main() {}\n```\n";
        let result = inject_mermaid_blocks(content);
        assert_eq!(result, content);
        assert!(!result.contains("```mermaid"));
    }
}
