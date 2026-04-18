use ledgerr_host::notify::{NotificationBackend, NotificationStatus, NotificationTestResult};
use ledgerr_host::settings::{AppSettings, SettingsStore};

#[test]
fn load_defaults_when_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().join("settings.json"));
    let settings = store.load().unwrap();
    assert!(settings.toast_enabled);
    assert_eq!(
        settings.toast_backend_preference,
        NotificationBackend::PowerShell
    );
}

#[test]
fn save_then_reload_roundtrips_settings() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::new(dir.path().join("settings.json"));
    let settings = AppSettings {
        toast_enabled: false,
        window_visible_on_start: false,
        last_test_result: Some(NotificationTestResult {
            status: NotificationStatus::Ready,
            timestamp: None,
            message: Some("ok".into()),
        }),
        ..AppSettings::default()
    };

    store.save(&settings).unwrap();
    let reloaded = store.load().unwrap();
    assert_eq!(reloaded, settings);
}

#[test]
fn malformed_json_falls_back_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, "{bad json").unwrap();
    let store = SettingsStore::new(path);
    let settings = store.load().unwrap();
    assert_eq!(settings, AppSettings::default());
}

#[test]
fn toggle_toast_enabled_persists_across_fresh_store_instance() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let store = SettingsStore::new(path.clone());
    let mut settings = store.load().unwrap();
    settings.toast_enabled = false;
    store.save(&settings).unwrap();

    let fresh_store = SettingsStore::new(path);
    let reloaded = fresh_store.load().unwrap();
    assert!(!reloaded.toast_enabled);
}
