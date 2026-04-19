use std::process::Command;

use chrono::Utc;

use super::types::{
    NotificationEvent, NotificationSettings, NotificationStatus, NotificationTestResult, Notifier,
    NotifyError,
};

#[derive(Debug, Clone)]
pub struct PowerShellBurntToastNotifier {
    settings: NotificationSettings,
}

impl PowerShellBurntToastNotifier {
    pub fn new(settings: NotificationSettings) -> Self {
        Self { settings }
    }

    pub fn build_script(title: &str, body: &str) -> String {
        let title = escape_powershell_single_quoted(title);
        let body = escape_powershell_single_quoted(body);
        format!("Import-Module BurntToast; New-BurntToastNotification -Text '{title}', '{body}'")
    }

    fn run_script(&self, script: &str) -> Result<(), NotifyError> {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    NotifyError::PowerShellUnavailable
                } else {
                    NotifyError::Io(err)
                }
            })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{stderr}\n{stdout}");
        if combined.contains("Import-Module") && combined.contains("BurntToast") {
            return Err(NotifyError::BurntToastUnavailable);
        }

        Err(NotifyError::CommandFailed(combined.trim().to_string()))
    }

    fn disabled_result() -> NotificationTestResult {
        NotificationTestResult {
            status: NotificationStatus::Disabled,
            timestamp: Some(Utc::now()),
            message: Some("notifications disabled".to_string()),
        }
    }
}

impl Notifier for PowerShellBurntToastNotifier {
    fn is_enabled(&self) -> bool {
        self.settings.enabled
    }

    fn status(&self) -> NotificationStatus {
        if self.settings.enabled {
            NotificationStatus::Unknown
        } else {
            NotificationStatus::Disabled
        }
    }

    fn test(&self, title: &str, body: &str) -> Result<NotificationTestResult, NotifyError> {
        if !self.settings.enabled {
            return Ok(Self::disabled_result());
        }

        let script = Self::build_script(title, body);
        self.run_script(&script)?;

        Ok(NotificationTestResult {
            status: NotificationStatus::Ready,
            timestamp: Some(Utc::now()),
            message: Some("toast sent".to_string()),
        })
    }

    fn notify(&self, event: &NotificationEvent) -> Result<(), NotifyError> {
        if !self.settings.enabled {
            return Err(NotifyError::Disabled);
        }

        let (title, body) = match event {
            NotificationEvent::RunStarted => ("l3dg3rr", "Run started".to_string()),
            NotificationEvent::ApprovalRequired => ("l3dg3rr", "Approval required".to_string()),
            NotificationEvent::ToolFailed { tool_name, message } => {
                ("l3dg3rr", format!("Tool failed: {tool_name}: {message}"))
            }
            NotificationEvent::TransactionSubmitted { reference } => {
                ("l3dg3rr", format!("Transaction submitted: {reference}"))
            }
            NotificationEvent::RunCompleted => ("l3dg3rr", "Run completed".to_string()),
            NotificationEvent::Test { title, body } => (title.as_str(), body.clone()),
        };

        self.run_script(&Self::build_script(title, &body))
    }
}

fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notify::{NotificationBackend, NotificationEvent};

    #[test]
    fn build_script_escapes_single_quotes() {
        let script = PowerShellBurntToastNotifier::build_script("l3d'g3rr", "it'works");
        assert!(script.contains("'l3d''g3rr'"));
        assert!(script.contains("'it''works'"));
    }

    #[test]
    fn disabled_path_returns_disabled_status() {
        let notifier = PowerShellBurntToastNotifier::new(NotificationSettings {
            enabled: false,
            backend: NotificationBackend::PowerShell,
            last_test_result: None,
        });
        let result = notifier.test("x", "y").unwrap();
        assert_eq!(result.status, NotificationStatus::Disabled);
    }

    #[test]
    fn test_event_uses_explicit_payload() {
        let event = NotificationEvent::Test {
            title: "a".into(),
            body: "b".into(),
        };
        match event {
            NotificationEvent::Test { title, body } => {
                assert_eq!(title, "a");
                assert_eq!(body, "b");
            }
            _ => panic!("unexpected event variant"),
        }
    }
}
