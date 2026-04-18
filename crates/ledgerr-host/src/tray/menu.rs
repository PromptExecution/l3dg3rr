use crate::notify::NotificationStatus;

use super::state::TrayState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenuLabels {
    pub version: String,
    pub toast_enabled: &'static str,
    pub test_toast: &'static str,
    pub status: String,
    pub show_window: &'static str,
    pub exit: &'static str,
}

pub fn tray_menu_labels(state: &TrayState) -> TrayMenuLabels {
    TrayMenuLabels {
        version: format!("Version: {}", env!("CARGO_PKG_VERSION")),
        toast_enabled: "Toast Enabled",
        test_toast: "Test Toast",
        status: format!("Status: {}", status_label(state.notification_status)),
        show_window: "Show Window",
        exit: "Exit",
    }
}

fn status_label(status: NotificationStatus) -> &'static str {
    match status {
        NotificationStatus::Disabled => "Disabled",
        NotificationStatus::Unknown => "Unknown",
        NotificationStatus::Ready => "Ready",
        NotificationStatus::Degraded => "Degraded",
        NotificationStatus::Failed => "Failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_label_is_deterministic() {
        let state = TrayState {
            toast_enabled: true,
            notification_status: NotificationStatus::Ready,
            window_visible: false,
        };
        let labels = tray_menu_labels(&state);
        assert_eq!(labels.version, format!("Version: {}", env!("CARGO_PKG_VERSION")));
        assert_eq!(labels.status, "Status: Ready");
        assert_eq!(labels.toast_enabled, "Toast Enabled");
        assert_eq!(labels.exit, "Exit");
    }
}
