use ledgerr_host::notify::NotificationStatus;
use ledgerr_host::tray::{TrayCommand, TrayState, tray_menu_labels};

#[test]
fn tray_default_state_is_sane() {
    let state = TrayState::default();
    assert!(state.toast_enabled);
    assert_eq!(state.notification_status, NotificationStatus::Unknown);
}

#[test]
fn tray_menu_renders_disabled_status() {
    let state = TrayState {
        toast_enabled: false,
        notification_status: NotificationStatus::Disabled,
        window_visible: false,
    };
    let labels = tray_menu_labels(&state);
    assert_eq!(labels.status, "Status: Disabled");
}

#[test]
fn tray_command_toggle_preserves_explicit_value() {
    let command = TrayCommand::ToggleToast(false);
    match command {
        TrayCommand::ToggleToast(value) => assert!(!value),
        _ => panic!("unexpected tray command"),
    }
}
