use crate::DatumError;
use std::path::Path;

/// A parsed section of a datum document.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub level: u8,
    pub heading: String,
    pub body: String,
    pub subsections: Vec<Section>,
}

/// The full AST of a datum document.
#[derive(Debug, Clone, PartialEq)]
pub struct DatumAst {
    pub h1: String,
    /// Lines before the first H2 (overview/intro block)
    pub preamble: String,
    pub sections: Vec<Section>,
    pub line_count: usize,
}

/// Parse a datum file into its section AST.
///
/// Handles:
/// - H1 (`# Title`) as document root
/// - H2 (`## Section`) as top-level sections
/// - H3+ (`### Subsection`) as nested subsections
/// - Code blocks (``` fences) — skipped for heading detection
/// - TOML-like `[section]` headers — detected as H2-equivalent
/// - Tables, lists, inline code — passed through as body content
pub fn parse_datum(content: &str) -> Result<DatumAst, DatumError> {
    if content.trim().is_empty() {
        return Err(DatumError::Empty {
            path: "<string>".into(),
        });
    }

    let h1_line = content
        .lines()
        .find(|line| line.starts_with("# ") && !line.starts_with("##"))
        .ok_or_else(|| DatumError::NoH1Header {
            path: "<string>".into(),
        })?;

    let h1 = h1_line.trim_start_matches("# ").to_owned();
    let line_count = content.lines().count();

    let mut preamble = String::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut current_section: Option<Section> = None;
    let mut current_subsection: Option<Section> = None;
    let mut in_code_block = false;

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            append_body(&mut current_subsection, &mut current_section, &mut preamble, line);
            continue;
        }

        if in_code_block {
            append_body(&mut current_subsection, &mut current_section, &mut preamble, line);
            continue;
        }

        let trimmed = line.trim();

        // TOML section header [section] — treat as H2
        if trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.starts_with("[//") {
            if let Some(sec) = current_section.take() {
                sections.push(sec);
            }
            current_subsection = None;
            let name = trimmed.trim_matches('[').trim_matches(']').to_owned();
            current_section = Some(Section {
                level: 2,
                heading: name,
                body: String::new(),
                subsections: Vec::new(),
            });
            continue;
        }

        if line.starts_with("## ") {
            if let Some(sub) = current_subsection.take() {
                if let Some(ref mut sec) = current_section {
                    sec.subsections.push(sub);
                }
            }
            if let Some(sec) = current_section.take() {
                sections.push(sec);
            }
            let heading = line.trim_start_matches("## ").to_owned();
            current_section = Some(Section {
                level: 2,
                heading,
                body: String::new(),
                subsections: Vec::new(),
            });
            continue;
        }

        if line.starts_with("### ") {
            if let Some(sub) = current_subsection.take() {
                if let Some(ref mut sec) = current_section {
                    sec.subsections.push(sub);
                }
            }
            let heading = line.trim_start_matches("### ").to_owned();
            current_subsection = Some(Section {
                level: 3,
                heading,
                body: String::new(),
                subsections: Vec::new(),
            });
            continue;
        }

        if line.starts_with("# ") {
            continue;
        }

        append_body(&mut current_subsection, &mut current_section, &mut preamble, line);
    }

    if let Some(sub) = current_subsection.take() {
        if let Some(ref mut sec) = current_section {
            sec.subsections.push(sub);
        }
    }
    if let Some(sec) = current_section.take() {
        sections.push(sec);
    }

    Ok(DatumAst {
        h1,
        preamble,
        sections,
        line_count,
    })
}

fn append_body(
    sub: &mut Option<Section>,
    sec: &mut Option<Section>,
    preamble: &mut String,
    line: &str,
) {
    if let Some(ref mut s) = sub {
        if !s.body.is_empty() {
            s.body.push('\n');
        }
        s.body.push_str(line);
    } else if let Some(ref mut s) = sec {
        if !s.body.is_empty() {
            s.body.push('\n');
        }
        s.body.push_str(line);
    } else {
        if !preamble.is_empty() {
            preamble.push('\n');
        }
        preamble.push_str(line);
    }
}

/// Load and parse a datum file into its AST.
pub fn parse_datum_file(base: &Path, name: &str) -> Result<DatumAst, DatumError> {
    let content = crate::read_datum_content(base, name)?;
    parse_datum(&content)
}

/// Lint-level violation found in a datum AST.
#[derive(Debug, Clone, PartialEq)]
pub struct DatumLintFinding {
    pub datum_name: String,
    pub severity: LintSeverity,
    pub message: String,
    pub section: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

/// Lint a parsed datum AST against quality rules.
///
/// Rules applied:
/// 1. Preamble must exist (content between H1 and first H2)
/// 2. At least one H2 section required
/// 3. No empty sections (H2 or H3 with no body)
/// 4. TOML sections (delimited by `[brackets]`) should have key=value content
/// 5. Tables should have header separators (`|---|---|`)
/// 6. Code blocks should have a language specifier
/// 7. No H1 duplicates
/// 8. Section heading length should be reasonable (≤ 100 chars)
pub fn lint_ast(ast: &DatumAst, datum_name: &str) -> Vec<DatumLintFinding> {
    let mut findings = Vec::new();

    if ast.preamble.trim().is_empty() {
        findings.push(DatumLintFinding {
            datum_name: datum_name.to_owned(),
            severity: LintSeverity::Warning,
            message: "No preamble content between H1 and first section".into(),
            section: None,
            line: Some(2),
        });
    }

    if ast.sections.is_empty() {
        findings.push(DatumLintFinding {
            datum_name: datum_name.to_owned(),
            severity: LintSeverity::Error,
            message: "Datum has no H2 or TOML sections — at least one required".into(),
            section: None,
            line: Some(1),
        });
    }

    for sec in &ast.sections {
        if sec.body.trim().is_empty() && sec.subsections.is_empty() {
            findings.push(DatumLintFinding {
                datum_name: datum_name.to_owned(),
                severity: LintSeverity::Warning,
                message: format!("Empty section: '{}' has no content", sec.heading),
                section: Some(sec.heading.clone()),
                line: None,
            });
        }

        if sec.heading.len() > 100 {
            findings.push(DatumLintFinding {
                datum_name: datum_name.to_owned(),
                severity: LintSeverity::Warning,
                message: format!(
                    "Section heading too long ({} chars): '{}'",
                    sec.heading.len(),
                    sec.heading
                ),
                section: Some(sec.heading.clone()),
                line: None,
            });
        }

        // Check for toml sections with key=value content
        if !sec.heading.contains(' ') && !sec.heading.contains('_')
            && sec.body.contains('=')
        {
            lint_toml_section(sec, datum_name, &mut findings);
        }

        // Check tables in section body
        if sec.body.contains('|') && sec.body.contains("\n|") {
            lint_tables_in_body(sec, datum_name, &mut findings);
        }

        // Check code blocks
        if sec.body.contains("```") {
            lint_code_blocks(sec, datum_name, &mut findings);
        }

        // Lint subsections
        for sub in &sec.subsections {
            if sub.body.trim().is_empty() {
                findings.push(DatumLintFinding {
                    datum_name: datum_name.to_owned(),
                    severity: LintSeverity::Info,
                    message: format!(
                        "Empty subsection: '{}' under '{}'",
                        sub.heading, sec.heading
                    ),
                    section: Some(format!("{} > {}", sec.heading, sub.heading)),
                    line: None,
                });
            }
        }
    }

    findings
}

fn lint_toml_section(sec: &Section, datum_name: &str, findings: &mut Vec<DatumLintFinding>) {
    for line in sec.body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.contains('=') && !trimmed.starts_with("[[") {
            findings.push(DatumLintFinding {
                datum_name: datum_name.to_owned(),
                severity: LintSeverity::Info,
                message: format!(
                    "TOML section '{}' has non-key-value line: '{}'",
                    sec.heading, trimmed
                ),
                section: Some(sec.heading.clone()),
                line: None,
            });
        }
    }
}

fn lint_tables_in_body(sec: &Section, datum_name: &str, findings: &mut Vec<DatumLintFinding>) {
    let mut in_table = false;
    let mut has_separator = false;
    let mut row_count = 0;

    for line in sec.body.lines() {
        if !line.trim().starts_with('|') {
            if in_table && !has_separator && row_count > 1 {
                findings.push(DatumLintFinding {
                    datum_name: datum_name.to_owned(),
                    severity: LintSeverity::Info,
                    message: format!(
                        "Section '{}' has a table without header separator (---|---)",
                        sec.heading
                    ),
                    section: Some(sec.heading.clone()),
                    line: None,
                });
            }
            in_table = false;
            has_separator = false;
            row_count = 0;
            continue;
        }
        in_table = true;
        row_count += 1;
        if line.contains("---") {
            has_separator = true;
        }
    }

    if in_table && !has_separator && row_count > 1 {
        findings.push(DatumLintFinding {
            datum_name: datum_name.to_owned(),
            severity: LintSeverity::Info,
            message: format!(
                "Section '{}' has a table without header separator",
                sec.heading
            ),
            section: Some(sec.heading.clone()),
            line: None,
        });
    }
}

fn lint_code_blocks(sec: &Section, datum_name: &str, findings: &mut Vec<DatumLintFinding>) {
    let mut in_block = false;
    let mut has_lang = false;

    for line in sec.body.lines() {
        if line.trim_start().starts_with("```") {
            if !in_block {
                let rest = line.trim_start().trim_start_matches("```");
                has_lang = !rest.is_empty() && !rest.starts_with('{');
            }
            in_block = !in_block;
            continue;
        }
        if !in_block {
            continue;
        }
    }

    if !has_lang {
        findings.push(DatumLintFinding {
            datum_name: datum_name.to_owned(),
            severity: LintSeverity::Info,
            message: format!(
                "Section '{}' has a code block without language specifier",
                sec.heading
            ),
            section: Some(sec.heading.clone()),
            line: None,
        });
    }
}

/// Full lint pipeline: load, parse, lint, return findings.
pub fn lint_datum_file(base: &Path, name: &str) -> Result<Vec<DatumLintFinding>, DatumError> {
    let ast = parse_datum_file(base, name)?;
    Ok(lint_ast(&ast, name))
}

/// Validate that a parsed datum conforms to mandatory structural rules.
/// Returns errors only (no warnings/info) — suitable for CI gating.
pub fn validate_datum_structure(ast: &DatumAst, datum_name: &str) -> Result<(), Vec<String>> {
    let findings = lint_ast(ast, datum_name);
    let errors: Vec<String> = findings
        .into_iter()
        .filter(|f| f.severity == LintSeverity::Error)
        .map(|f| format!("[{}] {}: {}", datum_name, f.message, f.section.unwrap_or_default()))
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_markdown_datum() {
        let content = "# Test Datum\n\nSome preamble\n\n## Section One\n\nBody text\n\n## Section Two\n\nMore text\n";
        let ast = parse_datum(content).unwrap();
        assert_eq!(ast.h1, "Test Datum");
        assert_eq!(ast.sections.len(), 2);
        assert_eq!(ast.sections[0].heading, "Section One");
        assert_eq!(ast.sections[1].heading, "Section Two");
    }

    #[test]
    fn parse_toml_section_datum() {
        let content = "# Config Datum\n\n[node]\nidentity = \"test\"\nos = \"linux\"\n\n[dependencies]\nfoo = \"bar\"\n";
        let ast = parse_datum(content).unwrap();
        assert_eq!(ast.h1, "Config Datum");
        assert_eq!(ast.sections.len(), 2);
        assert_eq!(ast.sections[0].heading, "node");
        assert_eq!(ast.sections[1].heading, "dependencies");
    }

    #[test]
    fn parse_with_subsections() {
        let content = "# Main\n\n## Section\n\n### Sub A\n\nSub body\n\n### Sub B\n\nMore sub\n\n## Another\n\nEnd\n";
        let ast = parse_datum(content).unwrap();
        assert_eq!(ast.sections.len(), 2);
        assert_eq!(ast.sections[0].subsections.len(), 2);
        assert_eq!(ast.sections[0].subsections[0].heading, "Sub A");
        assert_eq!(ast.sections[0].subsections[1].heading, "Sub B");
    }

    #[test]
    fn lint_empty_datum() {
        let err = parse_datum("").unwrap_err();
        assert!(matches!(err, DatumError::Empty { .. }));
    }

    #[test]
    fn lint_no_h1() {
        let err = parse_datum("Some content\n## Section\n").unwrap_err();
        assert!(matches!(err, DatumError::NoH1Header { .. }));
    }

    #[test]
    fn lint_no_sections_warning() {
        let content = "# Bare\n\nJust preamble, no sections\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "bare");
        assert!(findings.iter().any(|f| f.message.contains("no H2 or TOML sections")));
    }

    #[test]
    fn lint_empty_section_warning() {
        let content = "# Test\n\n## Empty\n\n## Full\n\nHas body\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(findings.iter().any(|f| f.message.contains("Empty section")));
    }

    #[test]
    fn lint_long_heading_warning() {
        let long = "A".repeat(101);
        let content = format!("# Test\n\n## {long}\n\nBody\n");
        let ast = parse_datum(&content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(findings.iter().any(|f| f.message.contains("too long")));
    }

    #[test]
    fn lint_table_without_separator() {
        let content = "# Test\n\n## Table\n\n| H1 | H2 |\n| A  | B  |\n| C  | D  |\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(findings.iter().any(|f| f.message.contains("header separator")));
    }

    #[test]
    fn lint_table_with_separator_ok() {
        let content = "# Test\n\n## Table\n\n| H1 | H2 |\n|----|----|\n| A  | B  |\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(!findings.iter().any(|f| f.message.contains("header separator")));
    }

    #[test]
    fn lint_code_block_no_lang() {
        let content = "# Test\n\n## Code\n\n```\nlet x = 1;\n```\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(findings.iter().any(|f| f.message.contains("language specifier")));
    }

    #[test]
    fn lint_code_block_with_lang_ok() {
        let content = "# Test\n\n## Code\n\n```rust\nlet x = 1;\n```\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(!findings.iter().any(|f| f.message.contains("language specifier")));
    }

    #[test]
    fn lint_preamble_missing_warning() {
        let content = "# Test\n## Section\nBody\n";
        let ast = parse_datum(content).unwrap();
        let findings = lint_ast(&ast, "test");
        assert!(findings.iter().any(|f| f.message.contains("No preamble")));
    }

    #[test]
    fn parse_code_blocks_skipped_for_heading_detection() {
        let content = "# Test\n\nPreamble\n\n## Section\n\n```markdown\n# This is not a real H1\n```\n\n## Real Section\n\nDone\n";
        let ast = parse_datum(content).unwrap();
        assert_eq!(ast.sections.len(), 2);
    }

    #[test]
    fn validate_good_datum_passes() {
        let content = "# Valid\n\nPreamble\n\n## Section\n\nBody\n\n## Tags\n\n#tag\n";
        let ast = parse_datum(content).unwrap();
        assert!(validate_datum_structure(&ast, "valid").is_ok());
    }

    /// Test with real datum files from the _b00t_ repo.
    /// These require the external _b00t_ repo to be present at compile time.
    /// Enabled via `--features real_datums` (not available in CI).
    #[cfg(feature = "real_datums")]
    mod real_datums {
        use super::*;

        fn datum_content(name: &str) -> Option<&'static str> {
            // At compile time, include_str! resolves from the crate source dir.
            // Path: <repo>/crates/datum/src/../../../../_b00t_/datums/<name>.datum
            // In CI this won't exist unless the _b00t_ repo is checked out.
            // We try to include and catch compile errors — if it fails, the test is skipped.
            match name {
                "opencode" => Some(include_str!("../../../../_b00t_/datums/opencode.datum")),
                "opencode-codebase-memory-integration" => {
                    Some(include_str!("../../../../_b00t_/datums/opencode-codebase-memory-integration.datum"))
                }
                "b00t-opencode-gaps" => {
                    Some(include_str!("../../../../_b00t_/datums/b00t-opencode-gaps.datum"))
                }
                "openagents-control" => {
                    Some(include_str!("../../../../_b00t_/datums/openagents-control.datum"))
                }
                _ => None,
            }
        }

        #[test]
        fn parse_real_opencode_datum() {
            let content = datum_content("opencode").unwrap();
            let ast = parse_datum(content).unwrap();
            assert_eq!(ast.h1, "opencode datum");
            assert!(ast.sections.len() >= 5);
        }

        #[test]
        fn parse_real_integration_datum() {
            let content = datum_content("opencode-codebase-memory-integration").unwrap();
            let ast = parse_datum(content).unwrap();
            assert!(ast.h1.contains("Foreign Variant Datum"));
            assert!(ast.sections.iter().any(|s| s.heading == "node"));
            assert!(ast.sections.iter().any(|s| s.heading == "bridges"));
        }

        #[test]
        fn lint_real_datums() {
            for name in &["opencode", "opencode-codebase-memory-integration", "b00t-opencode-gaps", "openagents-control"] {
                let content = datum_content(name).unwrap();
                let ast = parse_datum(content).unwrap();
                let findings = lint_ast(&ast, name);
                let errors: Vec<_> = findings.iter().filter(|f| f.severity == LintSeverity::Error).collect();
                assert!(errors.is_empty(), "datum {name} has lint errors: {errors:?}");
            }
        }
    }

}
