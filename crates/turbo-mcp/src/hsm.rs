#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LifecycleState {
    Ingest,
    Normalize,
    Validate,
    Reconcile,
    Commit,
    Summarize,
}

impl LifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ingest => "ingest",
            Self::Normalize => "normalize",
            Self::Validate => "validate",
            Self::Reconcile => "reconcile",
            Self::Commit => "commit",
            Self::Summarize => "summarize",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ingest" => Some(Self::Ingest),
            "normalize" => Some(Self::Normalize),
            "validate" => Some(Self::Validate),
            "reconcile" => Some(Self::Reconcile),
            "commit" => Some(Self::Commit),
            "summarize" => Some(Self::Summarize),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LifecycleSubstate {
    Pending,
    Ready,
}

impl LifecycleSubstate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "ready" => Some(Self::Ready),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifecycleNode {
    pub state: LifecycleState,
    pub substate: LifecycleSubstate,
}

impl LifecycleNode {
    pub fn token(self) -> String {
        format!("{}.{}", self.state.as_str(), self.substate.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HsmTransitionRequest {
    pub target_state: String,
    pub target_substate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HsmTransitionResponse {
    pub state: String,
    pub substate: String,
    pub status: String,
    pub guard_reason: Option<String>,
    pub transition_evidence: Vec<String>,
    pub state_marker: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HsmStatusRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HsmStatusResponse {
    pub state: String,
    pub substate: String,
    pub display_state: String,
    pub next_hint: String,
    pub resume_hint: String,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HsmMachine {
    pub current: LifecycleNode,
}

impl Default for HsmMachine {
    fn default() -> Self {
        Self {
            current: LifecycleNode {
                state: LifecycleState::Ingest,
                substate: LifecycleSubstate::Pending,
            },
        }
    }
}

pub fn parse_node(state: &str, substate: &str) -> Option<LifecycleNode> {
    Some(LifecycleNode {
        state: LifecycleState::parse(state)?,
        substate: LifecycleSubstate::parse(substate)?,
    })
}

pub fn allowed_next_node(current: LifecycleNode) -> Option<LifecycleNode> {
    use LifecycleState as S;
    use LifecycleSubstate as T;
    let next_state = match current.state {
        S::Ingest => S::Normalize,
        S::Normalize => S::Validate,
        S::Validate => S::Reconcile,
        S::Reconcile => S::Commit,
        S::Commit => S::Summarize,
        S::Summarize => return None,
    };
    Some(LifecycleNode {
        state: next_state,
        substate: T::Ready,
    })
}

pub fn next_hint_for(node: LifecycleNode) -> String {
    match node.state {
        LifecycleState::Ingest => "advance_to_normalize",
        LifecycleState::Normalize => "advance_to_validate",
        LifecycleState::Validate => "advance_to_reconcile",
        LifecycleState::Reconcile => "advance_to_commit",
        LifecycleState::Commit => "advance_to_summarize",
        LifecycleState::Summarize => "lifecycle_complete",
    }
    .to_string()
}

pub fn transition_blocked_response(current: LifecycleNode, requested: LifecycleNode) -> HsmTransitionResponse {
    let mut transition_evidence = vec![
        format!("from={}", current.token()),
        format!("to={}", requested.token()),
    ];
    if let Some(next) = allowed_next_node(current) {
        transition_evidence.push(format!("allowed={}", next.token()));
    } else {
        transition_evidence.push("allowed=none".to_string());
    }

    HsmTransitionResponse {
        state: current.state.as_str().to_string(),
        substate: current.substate.as_str().to_string(),
        status: "blocked".to_string(),
        guard_reason: Some("invalid_transition".to_string()),
        transition_evidence,
        state_marker: format!("{}:{}:blocked", current.state.as_str(), current.substate.as_str()),
    }
}

pub fn transition_advanced_response(node: LifecycleNode) -> HsmTransitionResponse {
    HsmTransitionResponse {
        state: node.state.as_str().to_string(),
        substate: node.substate.as_str().to_string(),
        status: "advanced".to_string(),
        guard_reason: None,
        transition_evidence: vec![format!("advanced_to={}", node.token())],
        state_marker: format!("{}:{}:advanced", node.state.as_str(), node.substate.as_str()),
    }
}

pub fn status_response(node: LifecycleNode, mut blockers: Vec<String>) -> HsmStatusResponse {
    blockers.sort();
    blockers.dedup();
    HsmStatusResponse {
        state: node.state.as_str().to_string(),
        substate: node.substate.as_str().to_string(),
        display_state: node.token(),
        next_hint: next_hint_for(node),
        resume_hint: format!("resume_from_{}", node.token()),
        blockers,
    }
}
