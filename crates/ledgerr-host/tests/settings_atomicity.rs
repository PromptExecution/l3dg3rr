use ledgerr_host::settings::{AppSettings, SettingsStore};

#[test]
fn creates_parent_directory_on_first_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("settings.json");
    let store = SettingsStore::new(path.clone());
    store.save(&AppSettings::default()).unwrap();
    assert!(path.exists());
}

#[test]
fn atomic_save_replaces_old_file_without_partial_contents() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let store = SettingsStore::new(path.clone());
    store.save(&AppSettings::default()).unwrap();

    let mut updated = AppSettings::default();
    updated.toast_enabled = false;
    updated.start_minimized_to_tray = true;
    store.save(&updated).unwrap();

    let raw = std::fs::read_to_string(path).unwrap();
    assert!(raw.contains("\"toast_enabled\": false"));
    assert!(raw.contains("\"start_minimized_to_tray\": true"));
}
