use crate::notify::NotificationStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayState {
    pub toast_enabled: bool,
    pub notification_status: NotificationStatus,
    pub window_visible: bool,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            toast_enabled: true,
            notification_status: NotificationStatus::Unknown,
            window_visible: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayCommand {
    ToggleToast(bool),
    TestToast,
    ShowWindow,
    Quit,
}
