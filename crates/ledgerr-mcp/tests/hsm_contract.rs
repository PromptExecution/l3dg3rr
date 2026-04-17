mod common;

use ledgerr_mcp::{HsmStatusRequest, HsmTransitionRequest, TurboLedgerService};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("hsm-contract");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest")
}

#[test]
fn hsm_01_advances_in_deterministic_lifecycle_order() {
    let svc = service();

    let initial = svc
        .hsm_status_tool(HsmStatusRequest)
        .expect("initial status");
    assert_eq!(initial.state, "ingest");
    assert_eq!(initial.substate, "pending");
    assert_eq!(initial.display_state, "ingest.pending");
    assert_eq!(initial.next_hint, "advance_to_normalize");
    assert_eq!(initial.blockers, Vec::<String>::new());

    let steps = [
        ("normalize", "ready"),
        ("validate", "ready"),
        ("reconcile", "ready"),
        ("commit", "ready"),
        ("summarize", "ready"),
    ];

    for (state, substate) in steps {
        let transitioned = svc
            .hsm_transition_tool(HsmTransitionRequest {
                target_state: state.to_string(),
                target_substate: substate.to_string(),
            })
            .expect("transition");
        assert_eq!(transitioned.status, "advanced");
        assert_eq!(transitioned.state, state);
        assert_eq!(transitioned.substate, substate);
        assert_eq!(
            transitioned.state_marker,
            format!("{state}:{substate}:advanced")
        );
    }
}

#[test]
fn hsm_02_invalid_transition_returns_deterministic_guard_reason_and_evidence() {
    let svc = service();
    let blocked = svc
        .hsm_transition_tool(HsmTransitionRequest {
            target_state: "reconcile".to_string(),
            target_substate: "ready".to_string(),
        })
        .expect("blocked transition");

    assert_eq!(blocked.status, "blocked");
    assert_eq!(blocked.guard_reason, Some("invalid_transition".to_string()));
    assert_eq!(
        blocked.transition_evidence,
        vec![
            "from=ingest.pending".to_string(),
            "to=reconcile.ready".to_string(),
            "allowed=normalize.ready".to_string(),
        ]
    );
    assert_eq!(blocked.state, "ingest");
    assert_eq!(blocked.substate, "pending");
}

#[test]
fn hsm_02_status_always_includes_deterministic_small_model_hints() {
    let svc = service();
    let status = svc.hsm_status_tool(HsmStatusRequest).expect("status");

    assert_eq!(status.display_state, "ingest.pending");
    assert_eq!(status.next_hint, "advance_to_normalize");
    assert_eq!(status.resume_hint, "resume_from_ingest.pending");
    assert_eq!(status.blockers, Vec::<String>::new());
}
