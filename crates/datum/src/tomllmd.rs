//! .tomllmd — compound structured document format with summary levels and entanglement.
//!
//! `.tomllmd` extends `.tomllm` with:
//! - **Summary levels** per section: `verbatim` (full), `executive` (compressed), `epigram` (one-liner)
//! - **Entanglement typing**: cross-datum references as `name.type` validated against known types
//! - **Command interpolation**: `{{ cmd: ... }}` rendered at read time
//! - **Compounding**: merge two+ `.tomllmd` at different summary levels via LLM
//!
//! # Format
//!
//! ```toml
//! [meta]
//! name = "topic-name"
//! type = "mcp"          # datum type for entanglement validation
//! hint = "One-liner"
//! tier = "ch0nky"       # sm0l | ch0nky | frontier
//!
//! [compounding]
//! sources = ["a.tomllmd", "b.tomllmd"]
//! merge_strategy = "union_by_action_desc"
//! produces = "compound.tomllmd"
//!
//! [sections.section_name]
//! verbatim = "full markdown\nwith detail"
//! executive = "compressed summary"
//! epigram = "one-liner"
//!
//! [entanglement]
//! mcp = ["b00t-mcp.mcp", "just-mcp.mcp"]
//! cli = ["b00t.cli", "just.cli"]
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Known datum types for entanglement validation.
pub const KNOWN_DATUM_TYPES: &[&str] = &[
    "mcp", "cli", "install", "config", "skill", "workflow",
    "ontology", "agent", "datum", "bridge", "provider", "surface",
];

/// The top-level .tomllmd structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tomllmd {
    #[serde(default)]
    pub meta: Option<MetaSection>,
    #[serde(default)]
    pub compounding: Option<CompoundingSection>,
    #[serde(default)]
    pub sections: HashMap<String, SectionLevels>,
    #[serde(default)]
    pub entanglement: Option<HashMap<String, Vec<String>>>,
}

/// Metadata header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSection {
    pub name: String,
    #[serde(rename = "type")]
    pub datum_type: String,
    #[serde(default)]
    pub hint: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_tier() -> String {
    "sm0l".into()
}

/// Compounding metadata for merging two+ .tomllmd files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundingSection {
    pub sources: Vec<String>,
    #[serde(default = "default_merge_strategy")]
    pub merge_strategy: String,
    #[serde(default)]
    pub produces: String,
    #[serde(default = "default_llm_tier")]
    pub llm_tier: String,
    #[serde(default)]
    pub context_window_tokens: u64,
}

fn default_merge_strategy() -> String {
    "union_by_action_desc".into()
}

fn default_llm_tier() -> String {
    "sm0l".into()
}

/// Three summary levels for a single section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionLevels {
    #[serde(default)]
    pub verbatim: String,
    #[serde(default)]
    pub executive: String,
    #[serde(default)]
    pub epigram: String,
}

impl SectionLevels {
    pub fn select(&self, tier: &str) -> &str {
        match tier {
            "epigram" | "sm0l" if !self.epigram.is_empty() => &self.epigram,
            "executive" | "ch0nky" if !self.executive.is_empty() => &self.executive,
            _ => &self.verbatim,
        }
    }
}

/// An entanglement reference: `name.type`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntanglementRef {
    pub name: String,
    pub datum_type: String,
}

/// Parse an entanglement reference string `name.type` into its components.
pub fn parse_entanglement_ref(s: &str) -> Option<EntanglementRef> {
    let s = s.trim();
    let dot = s.rfind('.')?;
    let name = s[..dot].to_owned();
    let datum_type = s[dot + 1..].to_owned();
    if name.is_empty() || datum_type.is_empty() {
        return None;
    }
    Some(EntanglementRef { name, datum_type })
}

/// Validate that all entanglement references use known types.
pub fn validate_entanglement_refs(
    refs: &[String],
    known_types: &[&str],
) -> Result<(), Vec<String>> {
    let known: HashSet<&str> = known_types.iter().copied().collect();
    let mut errors = Vec::new();

    for r in refs {
        match parse_entanglement_ref(r) {
            Some(parsed) => {
                if !known.contains(parsed.datum_type.as_str()) {
                    errors.push(format!(
                        "unknown entanglement type '{}' in ref '{}' — expected one of: {:?}",
                        parsed.datum_type, r, known_types
                    ));
                }
            }
            None => {
                errors.push(format!(
                    "invalid entanglement ref '{}' — must be `name.type`",
                    r
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Parse a .tomllmd string into its structured form.
pub fn parse_tomllmd(content: &str) -> Result<Tomllmd, String> {
    toml::from_str(content).map_err(|e| format!("tomllmd parse error: {e}"))
}

/// Render a .tomllmd at a given summary tier.
/// Returns the rendered markdown with command interpolation markers preserved.
pub fn render_tomllmd(tomllmd: &Tomllmd, tier: &str) -> String {
    let mut out = String::new();

    if let Some(meta) = &tomllmd.meta {
        out.push_str(&format!("# {}\n\n", meta.name));
        if !meta.hint.is_empty() {
            out.push_str(&format!("> {}\n\n", meta.hint));
        }
    }

    for (name, levels) in &tomllmd.sections {
        let heading = name.replace('_', " ").replace('-', " ");
        let heading = heading
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().next().unwrap()
                } else {
                    c
                }
            })
            .collect::<String>();
        out.push_str(&format!("## {heading}\n\n"));
        let selected = levels.select(tier);
        out.push_str(selected);
        out.push('\n');
        out.push('\n');
    }

    if let Some(ent) = &tomllmd.entanglement {
        out.push_str("## Entanglement\n\n");
        for (kind, refs) in ent {
            out.push_str(&format!("- **{kind}**: {}\n", refs.join(", ")));
        }
        out.push('\n');
    }

    out.trim().to_owned()
}

/// Compile a .tomllmd string: parse, validate entanglement, render at tier.
pub fn compile_tomllmd(content: &str, tier: &str) -> Result<String, Vec<String>> {
    let tomllmd =
        parse_tomllmd(content).map_err(|e| vec![format!("parse: {e}")])?;

    // Validate entanglement refs if present
    if let Some(ent) = &tomllmd.entanglement {
        let all_refs: Vec<String> =
            ent.values().flat_map(|v| v.iter().cloned()).collect();
        if !all_refs.is_empty() {
            validate_entanglement_refs(&all_refs, KNOWN_DATUM_TYPES)?;
        }
    }

    Ok(render_tomllmd(&tomllmd, tier))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_tomllmd() {
        let content = r#"
[meta]
name = "test-datum"
type = "mcp"
hint = "A test datum"
"#;
        let parsed = parse_tomllmd(content).unwrap();
        assert_eq!(parsed.meta.as_ref().unwrap().name, "test-datum");
        assert_eq!(parsed.meta.as_ref().unwrap().datum_type, "mcp");
    }

    #[test]
    fn parse_with_sections_and_levels() {
        let content = r#"
[meta]
name = "full-datum"
type = "skill"
hint = "A full example"

[sections.overview]
verbatim = "Full technical detail with code examples and edge cases."
executive = "Compressed summary of the feature."
epigram = "One-liner."
"#;
        let parsed = parse_tomllmd(content).unwrap();
        let sec = parsed.sections.get("overview").unwrap();
        assert_eq!(sec.verbatim, "Full technical detail with code examples and edge cases.");
        assert_eq!(sec.executive, "Compressed summary of the feature.");
        assert_eq!(sec.epigram, "One-liner.");
    }

    #[test]
    fn select_level_by_tier() {
        let levels = SectionLevels {
            verbatim: "full".into(),
            executive: "summary".into(),
            epigram: "one".into(),
        };
        assert_eq!(levels.select("sm0l"), "one");
        assert_eq!(levels.select("epigram"), "one");
        assert_eq!(levels.select("ch0nky"), "summary");
        assert_eq!(levels.select("executive"), "summary");
        assert_eq!(levels.select("frontier"), "full");
        assert_eq!(levels.select("unknown"), "full");
    }

    #[test]
    fn parse_entanglement_ref_valid() {
        let r = parse_entanglement_ref("b00t-mcp.mcp").unwrap();
        assert_eq!(r.name, "b00t-mcp");
        assert_eq!(r.datum_type, "mcp");
    }

    #[test]
    fn parse_entanglement_ref_invalid() {
        assert!(parse_entanglement_ref("no-dot").is_none());
        assert!(parse_entanglement_ref(".onlytype").is_none());
        assert!(parse_entanglement_ref("onlyname.").is_none());
    }

    #[test]
    fn validate_entanglement_known_types() {
        let refs = vec!["b00t-mcp.mcp".into(), "just-mcp.mcp".into()];
        assert!(validate_entanglement_refs(&refs, KNOWN_DATUM_TYPES).is_ok());
    }

    #[test]
    fn validate_entanglement_unknown_type_fails() {
        let refs = vec!["something.unknown_type".into()];
        let err = validate_entanglement_refs(&refs, KNOWN_DATUM_TYPES).unwrap_err();
        assert!(err[0].contains("unknown entanglement type"));
    }

    #[test]
    fn validate_entanglement_invalid_syntax_fails() {
        let refs = vec!["no-dot-here".into()];
        let err = validate_entanglement_refs(&refs, KNOWN_DATUM_TYPES).unwrap_err();
        assert!(err[0].contains("invalid entanglement ref"));
    }

    #[test]
    fn render_at_sm0l_tier() {
        let content = r#"
[meta]
name = "test"
type = "mcp"
hint = "hint text"

[sections.overview]
verbatim = "Full verbatim text."
executive = "Executive summary."
epigram = "Epigram."
"#;
        let rendered = compile_tomllmd(content, "sm0l").unwrap();
        assert!(rendered.contains("Epigram"));
        assert!(!rendered.contains("Full verbatim text"));
    }

    #[test]
    fn render_at_ch0nky_tier() {
        let content = r#"
[meta]
name = "test"
type = "mcp"

[sections.overview]
verbatim = "Full verbatim text."
executive = "Executive summary."
epigram = "Epigram."
"#;
        let rendered = compile_tomllmd(content, "ch0nky").unwrap();
        assert!(rendered.contains("Executive summary"));
        assert!(!rendered.contains("Full verbatim text"));
        assert!(!rendered.contains("Epigram"));
    }

    #[test]
    fn render_with_entanglement() {
        let content = r#"
[meta]
name = "test"
type = "mcp"

[sections.details]
verbatim = "Detail text."

[entanglement]
mcp = ["b00t-mcp.mcp", "just-mcp.mcp"]
cli = ["b00t.cli"]
"#;
        let rendered = compile_tomllmd(content, "frontier").unwrap();
        assert!(rendered.contains("Detail text"));
        assert!(rendered.contains("b00t-mcp"));
        assert!(rendered.contains("b00t.cli"));
    }

    #[test]
    fn compile_validates_entanglement() {
        let content = r#"
[meta]
name = "bad"
type = "mcp"

[sections.x]
verbatim = "text"

[entanglement]
mcp = ["b00t-mcp.unknown_type"]
"#;
        let err = compile_tomllmd(content, "frontier").unwrap_err();
        assert!(err[0].contains("unknown entanglement type"));
    }

    #[test]
    fn parse_missing_meta_fails() {
        let content = r#"
[sections.x]
verbatim = "text"
"#;
        let parsed = parse_tomllmd(content).unwrap();
        assert!(parsed.meta.is_none());
        assert_eq!(parsed.sections.len(), 1);
    }

    #[test]
    fn render_empty_sections() {
        let content = r#"
[meta]
name = "empty"
type = "config"
"#;
        let rendered = compile_tomllmd(content, "sm0l").unwrap();
        assert!(rendered.contains("empty"));
    }

    #[test]
    fn render_tomllmd_with_compounding() {
        let content = r#"
[meta]
name = "compound-test"
type = "mcp"

[compounding]
sources = ["a.tomllmd", "b.tomllmd"]
merge_strategy = "union_by_action_desc"
produces = "c.tomllmd"
llm_tier = "ch0nky"
context_window_tokens = 128000

[sections.api]
verbatim = "API details"
"#;
        let parsed = parse_tomllmd(content).unwrap();
        let comp = parsed.compounding.unwrap();
        assert_eq!(comp.sources.len(), 2);
        assert_eq!(comp.llm_tier, "ch0nky");
        assert_eq!(comp.context_window_tokens, 128000);
    }
}
