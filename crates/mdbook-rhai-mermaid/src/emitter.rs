/// Emit a Mermaid `flowchart TD` diagram from a `Graph`.
use crate::parser::{Graph, NodeKind};

/// Render the graph as a Mermaid flowchart string.
pub fn emit_mermaid(graph: &Graph) -> String {
    let mut out = String::from("flowchart TD\n");

    // 1. Node declarations in insertion order.
    for id in &graph.order {
        if let Some(node) = graph.nodes.get(id) {
            let declaration = match node.kind {
                NodeKind::Step => format!("    {}[\"{}\"]\n", node.id, escape_label(&node.label)),
                NodeKind::Decision => {
                    format!("    {}{{\"{}\"}}\n", node.id, escape_label(&node.label))
                }
            };
            out.push_str(&declaration);
        }
    }

    // 2. Edges.
    for edge in &graph.edges {
        let line = match &edge.label {
            Some(lbl) => format!("    {} -->|\"{}\"|{}\n", edge.from, lbl, edge.to),
            None => format!("    {} --> {}\n", edge.from, edge.to),
        };
        out.push_str(&line);
    }

    out
}

/// Escape double-quotes inside a Mermaid label.
fn escape_label(s: &str) -> String {
    s.replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_emit_pipeline() {
        let src = "fn ingest() -> classify\nfn classify() -> done\n";
        let graph = parse(src);
        let out = emit_mermaid(&graph);
        assert!(out.starts_with("flowchart TD\n"));
        assert!(out.contains("ingest[\"ingest\"]"));
        assert!(out.contains("classify[\"classify\"]"));
        assert!(out.contains("ingest --> classify"));
        assert!(out.contains("classify --> done"));
    }

    #[test]
    fn test_emit_decision_diamond() {
        let src = "if confidence > 0.8 -> commit\n";
        let graph = parse(src);
        let out = emit_mermaid(&graph);
        // Decision nodes use curly braces.
        assert!(out.contains('{'), "expected decision diamond syntax");
        assert!(out.contains("|\"true\"|"));
    }

    #[test]
    fn test_emit_threshold_chain_has_false_edge() {
        let src = "if confidence > 0.5 -> reconcile\nif confidence > 0.8 -> commit\n";
        let graph = parse(src);
        let out = emit_mermaid(&graph);
        assert!(out.contains("|\"false\"|"), "expected false chain edge in output");
        assert!(out.contains("|\"true\"|"), "expected true edge in output");
    }
}
