mod common;

use ledgerr_mcp::{HsmResumeRequest, HsmStatusRequest, HsmTransitionRequest, TurboLedgerService};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("hsm-resume");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest")
}

#[test]
fn hsm_03_resume_uses_last_valid_checkpoint_marker() {
    let svc = service();
    let transitioned = svc
        .hsm_transition_tool(HsmTransitionRequest {
            target_state: "normalize".to_string(),
            target_substate: "ready".to_string(),
        })
        .expect("transition to normalize");
    assert_eq!(transitioned.state_marker, "normalize:ready:advanced");

    let resumed = svc
        .hsm_resume_tool(HsmResumeRequest {
            state_marker: transitioned.state_marker.clone(),
        })
        .expect("resume");
    assert!(resumed.resumed);
    assert_eq!(resumed.resume_from, "normalize:ready:advanced");
    assert_eq!(resumed.resume_hint, "advance_to_validate");
    assert_eq!(resumed.blockers, Vec::<String>::new());

    let status = svc.hsm_status_tool(HsmStatusRequest).expect("status");
    assert_eq!(status.display_state, "normalize.ready");
    assert_eq!(status.next_hint, "advance_to_validate");
}

#[test]
fn hsm_03_resume_from_unknown_checkpoint_is_blocked_deterministically() {
    let svc = service();
    let resumed = svc
        .hsm_resume_tool(HsmResumeRequest {
            state_marker: "validate:ready:advanced".to_string(),
        })
        .expect("blocked resume");

    assert!(!resumed.resumed);
    assert_eq!(resumed.resume_from, "ingest:pending:advanced");
    assert_eq!(resumed.resume_hint, "resume_from_ingest.pending");
    assert_eq!(resumed.blockers, vec!["checkpoint_unknown".to_string()]);
}

#[test]
fn hsm_03_resume_response_exposes_deterministic_small_model_fields() {
    let svc = service();
    let resumed = svc
        .hsm_resume_tool(HsmResumeRequest {
            state_marker: "ingest:pending:advanced".to_string(),
        })
        .expect("resume from initial checkpoint");

    assert!(resumed.resumed);
    assert_eq!(resumed.resume_from, "ingest:pending:advanced");
    assert_eq!(resumed.resume_hint, "advance_to_normalize");
    assert_eq!(resumed.blockers, Vec::<String>::new());
}
