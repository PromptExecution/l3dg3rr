/// Graph-based AST for the rhai pseudo-DSL.
///
/// Two statement forms are supported:
///   - Pipeline step:    `fn name() -> target`
///   - Conditional:      `if expr -> target`   where expr is e.g. `confidence > 0.8`
///   - Match arm:        `match expr => Arm -> target`
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// A named function/step node.
    Step,
    /// A decision diamond (condition expression).
    Decision,
    /// A match/switch diamond with labeled outgoing arms.
    Match,
}

#[derive(Debug, Clone)]
pub struct Node {
    /// Sanitized Mermaid-safe identifier.
    pub id: String,
    /// Human-readable display label.
    pub label: String,
    pub kind: NodeKind,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    /// Optional edge label, e.g. "true" / "false".
    pub label: Option<String>,
}

#[derive(Debug, Default)]
pub struct Graph {
    /// Insertion-ordered list of node IDs (for deterministic output).
    pub order: Vec<String>,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn add_node(&mut self, id: String, label: String, kind: NodeKind) {
        if !self.nodes.contains_key(&id) {
            self.order.push(id.clone());
            self.nodes.insert(id, Node { id: self.order.last().unwrap().clone(), label, kind });
        }
    }

    pub fn add_edge(&mut self, from: String, to: String, label: Option<String>) {
        self.edges.push(Edge { from, to, label });
    }
}

// ---------------------------------------------------------------------------
// Sanitization
// ---------------------------------------------------------------------------

/// Replace any character that is not alphanumeric or `_` with `_`.
pub fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Conditional {
    lhs: String,
    op: String,
    rhs: String,
    target: String,
}

#[derive(Debug, Clone)]
struct MatchArm {
    expr: String,
    arm: String,
    target: String,
}

/// Parse the rhai pseudo-DSL source into a `Graph`.
///
/// Returns an empty `Graph` when the source contains no parseable statements
/// (empty or comment-only input). Malformed lines are silently skipped so a
/// partial parse always succeeds.
pub fn parse(source: &str) -> Graph {
    let mut pipeline_edges: Vec<(String, String)> = Vec::new();
    let mut conditionals: Vec<Conditional> = Vec::new();
    let mut match_arms: Vec<MatchArm> = Vec::new();

    for raw_line in source.lines() {
        // Strip inline comment.
        let line = match raw_line.find("//") {
            Some(pos) => &raw_line[..pos],
            None => raw_line,
        }
        .trim();

        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("fn ") {
            // Form 1: `fn name() -> target`
            if let Some((name_part, target_part)) = rest.split_once("->") {
                let name = name_part.trim().trim_end_matches("()").trim().to_string();
                let target = target_part.trim().to_string();
                if !name.is_empty() && !target.is_empty() {
                    pipeline_edges.push((name, target));
                }
            }
        } else if let Some(rest) = line.strip_prefix("if ") {
            // Form 2: `if expr -> target`
            if let Some((expr_part, target_part)) = rest.split_once("->") {
                let expr = expr_part.trim().to_string();
                let target = target_part.trim().to_string();
                if !expr.is_empty() && !target.is_empty() {
                    if let Some(cond) = parse_condition(&expr, &target) {
                        conditionals.push(cond);
                    }
                    // If condition doesn't parse as structured, still record it
                    // as a raw decision node with a plain edge.
                    else {
                        let cond_id = sanitize_id(&expr);
                        let target_id = sanitize_id(&target);
                        conditionals.push(Conditional {
                            lhs: cond_id,
                            op: String::new(),
                            rhs: String::new(),
                            target: target_id,
                        });
                    }
                }
            }
        } else if let Some(rest) = line.strip_prefix("match ") {
            // Form 3: `match expr => Arm -> target`
            if let Some((expr_part, arm_target_part)) = rest.split_once("=>") {
                if let Some((arm_part, target_part)) = arm_target_part.split_once("->") {
                    let expr = expr_part.trim().to_string();
                    let arm = arm_part.trim().to_string();
                    let target = target_part.trim().to_string();
                    if !expr.is_empty() && !arm.is_empty() && !target.is_empty() {
                        match_arms.push(MatchArm { expr, arm, target });
                    }
                }
            }
        }
        // Unknown forms are silently skipped.
    }

    build_graph(pipeline_edges, conditionals, match_arms)
}

/// Parse a condition string like `confidence > 0.8` into its parts.
fn parse_condition(expr: &str, target: &str) -> Option<Conditional> {
    // Supported operators (longest first to avoid prefix mis-match).
    let operators = [">=", "<=", "!=", ">", "<", "=="];
    for op in &operators {
        if let Some(pos) = expr.find(op) {
            let lhs = expr[..pos].trim().to_string();
            let rhs = expr[pos + op.len()..].trim().to_string();
            if !lhs.is_empty() && !rhs.is_empty() {
                return Some(Conditional {
                    lhs,
                    op: op.to_string(),
                    rhs,
                    target: target.to_string(),
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Graph builder
// ---------------------------------------------------------------------------

fn build_graph(
    pipeline_edges: Vec<(String, String)>,
    conditionals: Vec<Conditional>,
    match_arms: Vec<MatchArm>,
) -> Graph {
    let mut graph = Graph::default();

    // --- Pipeline steps ---
    for (name, target) in &pipeline_edges {
        let name_id = sanitize_id(name);
        let target_id = sanitize_id(target);
        graph.add_node(name_id.clone(), name.clone(), NodeKind::Step);
        graph.add_node(target_id.clone(), target.clone(), NodeKind::Step);
        graph.add_edge(name_id, target_id, None);
    }

    // --- Conditionals ---
    // Group by LHS variable for threshold-chain building.
    // We only chain when the operator is `>` (descending chain) or `<` (ascending).
    // Other operators (==, !=, >=, <=) are emitted as plain decision nodes.

    // Collect `>` threshold groups per LHS.
    let mut gt_groups: HashMap<String, Vec<(f64, String)>> = HashMap::new();
    let mut lt_groups: HashMap<String, Vec<(f64, String)>> = HashMap::new();
    let mut plain_conds: Vec<Conditional> = Vec::new();

    for cond in &conditionals {
        if cond.op == ">" {
            if let Ok(val) = cond.rhs.parse::<f64>() {
                gt_groups
                    .entry(cond.lhs.clone())
                    .or_default()
                    .push((val, cond.target.clone()));
                continue;
            }
        }
        if cond.op == "<" {
            if let Ok(val) = cond.rhs.parse::<f64>() {
                lt_groups
                    .entry(cond.lhs.clone())
                    .or_default()
                    .push((val, cond.target.clone()));
                continue;
            }
        }
        plain_conds.push(cond.clone());
    }

    // Build `>` threshold chains (sorted descending).
    for (lhs, mut thresholds) in gt_groups {
        thresholds.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        emit_threshold_chain(&mut graph, &lhs, ">", &thresholds);
    }

    // Build `<` threshold chains (sorted ascending).
    for (lhs, mut thresholds) in lt_groups {
        thresholds.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        emit_threshold_chain(&mut graph, &lhs, "<", &thresholds);
    }

    // Plain conditionals.
    for cond in &plain_conds {
        if cond.op.is_empty() {
            // Raw unparsed condition used as node ID directly.
            let cond_id = cond.lhs.clone();
            let target_id = sanitize_id(&cond.target);
            graph.add_node(
                cond_id.clone(),
                cond.lhs.clone(),
                NodeKind::Decision,
            );
            graph.add_node(target_id.clone(), cond.target.clone(), NodeKind::Step);
            graph.add_edge(cond_id, target_id, None);
        } else {
            let expr_label = format!("{} {} {}", cond.lhs, cond.op, cond.rhs);
            let cond_id = sanitize_id(&expr_label);
            let target_id = sanitize_id(&cond.target);
            graph.add_node(cond_id.clone(), expr_label, NodeKind::Decision);
            graph.add_node(target_id.clone(), cond.target.clone(), NodeKind::Step);
            graph.add_edge(cond_id, target_id, Some("true".to_string()));
        }
    }

    emit_match_groups(&mut graph, &match_arms);

    graph
}

fn emit_match_groups(graph: &mut Graph, match_arms: &[MatchArm]) {
    let mut groups: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for arm in match_arms {
        if !groups.contains_key(&arm.expr) {
            order.push(arm.expr.clone());
        }
        groups
            .entry(arm.expr.clone())
            .or_default()
            .push((arm.arm.clone(), arm.target.clone()));
    }

    for expr in order {
        let node_id = format!("match_{}", sanitize_id(&expr));
        let label = format!("match {}", expr);
        graph.add_node(node_id.clone(), label, NodeKind::Match);

        if let Some(arms) = groups.get(&expr) {
            for (arm_label, target) in arms {
                let target_id = sanitize_id(target);
                graph.add_node(target_id.clone(), target.clone(), NodeKind::Step);
                graph.add_edge(node_id.clone(), target_id, Some(arm_label.clone()));
            }
        }
    }
}

/// Emit a chain of threshold decisions into the graph.
///
/// For `>` thresholds sorted descending [0.8 → commit, 0.5 → reconcile]:
///   node_gt_0_8 --true--> commit
///   node_gt_0_8 --false--> node_gt_0_5
///   node_gt_0_5 --true--> reconcile
///   (no false edge on the last node)
fn emit_threshold_chain(graph: &mut Graph, lhs: &str, op: &str, thresholds: &[(f64, String)]) {
    let node_ids: Vec<String> = thresholds
        .iter()
        .map(|(val, _)| sanitize_id(&format!("{}_{}_{}", lhs, op_to_word(op), val_to_safe(val))))
        .collect();

    for (i, (val, target)) in thresholds.iter().enumerate() {
        let node_id = &node_ids[i];
        let label = format!("{} {} {}", lhs, op, val);
        graph.add_node(node_id.clone(), label, NodeKind::Decision);

        let target_id = sanitize_id(target);
        graph.add_node(target_id.clone(), target.clone(), NodeKind::Step);
        graph.add_edge(node_id.clone(), target_id, Some("true".to_string()));

        if i + 1 < thresholds.len() {
            let next_id = node_ids[i + 1].clone();
            graph.add_edge(node_id.clone(), next_id, Some("false".to_string()));
        }
    }
}

fn op_to_word(op: &str) -> &str {
    match op {
        ">" => "gt",
        "<" => "lt",
        ">=" => "gte",
        "<=" => "lte",
        "==" => "eq",
        "!=" => "ne",
        _ => "op",
    }
}

fn val_to_safe(val: &f64) -> String {
    // Render the float; replace `.` with `_` to stay alphanumeric.
    format!("{}", val).replace('.', "_")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn find_node_by_label<'a>(graph: &'a Graph, label: &str) -> Option<&'a Node> {
        graph.nodes.values().find(|n| n.label == label)
    }

    #[test]
    fn test_pipeline_chain() {
        let src = r#"
            fn ingest() -> classify
            fn classify() -> reconcile
            fn reconcile() -> export
        "#;
        let g = parse(src);
        assert!(g.nodes.contains_key("ingest"));
        assert!(g.nodes.contains_key("classify"));
        assert!(g.nodes.contains_key("reconcile"));
        assert!(g.nodes.contains_key("export"));
        assert_eq!(g.edges.len(), 3);
        assert_eq!(g.edges[0].from, "ingest");
        assert_eq!(g.edges[0].to, "classify");
        assert!(g.edges[0].label.is_none());
    }

    #[test]
    fn test_conditional_branch() {
        let src = r#"
            if confidence > 0.8 -> commit
        "#;
        let g = parse(src);
        // Should have a decision node and a step node.
        let decision = g.nodes.values().find(|n| n.kind == NodeKind::Decision);
        assert!(decision.is_some());
        let commit = g.nodes.values().find(|n| n.label == "commit");
        assert!(commit.is_some());
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].label.as_deref(), Some("true"));
    }

    #[test]
    fn test_threshold_chain_ordering() {
        let src = r#"
            if confidence > 0.5 -> reconcile
            if confidence > 0.8 -> commit
        "#;
        let g = parse(src);
        // Decision nodes should be chained: 0.8 first, false edge to 0.5 node.
        let decision_nodes: Vec<&Node> =
            g.nodes.values().filter(|n| n.kind == NodeKind::Decision).collect();
        assert_eq!(decision_nodes.len(), 2);

        // There should be a false edge from the 0.8 node to the 0.5 node.
        let false_edge = g.edges.iter().find(|e| e.label.as_deref() == Some("false"));
        assert!(false_edge.is_some(), "expected a false-chain edge");

        // The false edge should point from the 0.8 node to the 0.5 node.
        let fe = false_edge.unwrap();
        assert!(fe.from.contains("0_8"), "false edge from should reference 0.8 threshold");
        assert!(fe.to.contains("0_5"), "false edge to should reference 0.5 threshold");
    }

    #[test]
    fn test_deduplication() {
        let src = r#"
            fn ingest() -> classify
            fn ingest() -> classify
        "#;
        let g = parse(src);
        // Node IDs must be deduplicated.
        assert_eq!(g.nodes.len(), 2);
        // But edges are not deduplicated (caller's responsibility; emitter handles it).
        assert_eq!(g.edges.len(), 2);
    }

    #[test]
    fn test_sanitized_node_ids() {
        let src = "fn parse docs() -> build index\n";
        let g = parse(src);
        assert!(g.nodes.contains_key("parse_docs"));
        assert!(g.nodes.contains_key("build_index"));
    }

    #[test]
    fn test_operator_coverage() {
        let ops = [
            ("if score >= 0.9 -> high", ">="),
            ("if score <= 0.1 -> low", "<="),
            ("if status == approved -> commit", "=="),
            ("if status != rejected -> continue", "!="),
        ];
        for (src, op) in &ops {
            let g = parse(src);
            let decision = g.nodes.values().find(|n| n.kind == NodeKind::Decision);
            assert!(decision.is_some(), "no decision node for op {}", op);
            let node = decision.unwrap();
            assert!(
                node.label.contains(op),
                "label '{}' does not contain op '{}'",
                node.label,
                op
            );
        }
    }

    #[test]
    fn test_comment_stripping() {
        let src = r#"
            fn ingest() -> classify // this is a comment
            // full-line comment
            fn classify() -> done
        "#;
        let g = parse(src);
        assert_eq!(g.nodes.len(), 3); // ingest, classify, done
    }

    #[test]
    fn test_match_group_builds_single_match_node_with_labeled_arms() {
        let src = r#"
            match result.disposition => Disposition::Unrecoverable -> halt_pipeline
            match result.disposition => Disposition::Recoverable -> repair_and_retry
            match result.disposition => Disposition::Advisory -> record_note
        "#;
        let g = parse(src);

        let match_nodes: Vec<&Node> = g.nodes.values().filter(|n| n.kind == NodeKind::Match).collect();
        assert_eq!(match_nodes.len(), 1);
        assert_eq!(match_nodes[0].label, "match result.disposition");
        assert_eq!(g.edges.len(), 3);
        assert_eq!(g.edges[0].label.as_deref(), Some("Disposition::Unrecoverable"));
        assert_eq!(g.edges[1].label.as_deref(), Some("Disposition::Recoverable"));
        assert_eq!(g.edges[2].label.as_deref(), Some("Disposition::Advisory"));
    }

    #[test]
    fn test_empty_input() {
        let g = parse("// only comments\n\n");
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
    }

    #[test]
    fn test_find_node_by_label_helper() {
        let src = "fn ingest() -> classify\n";
        let g = parse(src);
        assert!(find_node_by_label(&g, "ingest").is_some());
        assert!(find_node_by_label(&g, "nonexistent").is_none());
    }
}
