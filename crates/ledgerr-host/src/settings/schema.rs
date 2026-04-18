use serde::{Deserialize, Serialize};

use crate::notify::{NotificationBackend, NotificationTestResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsSchemaVersion {
    #[default]
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowNotificationsFor {
    pub approval_required: bool,
    pub transaction_submitted: bool,
    pub run_failed: bool,
    pub run_completed: bool,
}

impl Default for ShowNotificationsFor {
    fn default() -> Self {
        Self {
            approval_required: true,
            transaction_submitted: true,
            run_failed: true,
            run_completed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub schema_version: SettingsSchemaVersion,
    pub toast_enabled: bool,
    pub toast_backend_preference: NotificationBackend,
    pub start_minimized_to_tray: bool,
    pub window_visible_on_start: bool,
    #[serde(default)]
    pub show_notifications_for: ShowNotificationsFor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_test_result: Option<NotificationTestResult>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SettingsSchemaVersion::V1,
            toast_enabled: true,
            toast_backend_preference: NotificationBackend::PowerShell,
            start_minimized_to_tray: false,
            window_visible_on_start: true,
            show_notifications_for: ShowNotificationsFor::default(),
            last_test_result: None,
        }
    }
}
