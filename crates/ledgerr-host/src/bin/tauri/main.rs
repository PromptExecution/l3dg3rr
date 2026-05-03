#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod state;

use std::sync::{Arc, Mutex};

use ledgerr_host::chat::{ChatTurn, ReviewLog};
use ledgerr_host::internal_openai::InternalOpenAiHandle;
use ledgerr_host::settings::{default_settings_path, SettingsStore};

use state::AppState;

fn main() {
    // Telemetry UUID signal path — read from env, echo back so test harness can verify
    if let Ok(uuid) = std::env::var("TAURI_TEST_UUID") {
        eprintln!("[telemetry] TAURI_TEST_UUID={uuid}");
        let _ = std::fs::write(
            std::env::temp_dir().join("host-tauri-telemetry-signal.txt"),
            format!("TAURI_TEST_UUID={uuid}\n"),
        );
    }
    // Kill delay for autorun countdown — passed to JS via HTML meta tag
    if let Ok(delay) = std::env::var("TAURI_TEST_KILL_DELAY") {
        eprintln!("[telemetry] TAURI_TEST_KILL_DELAY={delay}");
        let _ = std::fs::write(
            std::env::temp_dir().join("host-tauri-kill-delay.txt"),
            format!("TAURI_TEST_KILL_DELAY={delay}\n"),
        );
    }
    if let Ok(shots) = std::env::var("TAURI_TEST_SCREENSHOT_PATH") {
        eprintln!("[telemetry] TAURI_TEST_SCREENSHOT_PATH={shots}");
    }

    let store = Arc::new(SettingsStore::new(default_settings_path()));
    let history: Arc<Mutex<Vec<ChatTurn>>> = Arc::new(Mutex::new(Vec::new()));
    let review_log: Arc<Mutex<ReviewLog>> = Arc::new(Mutex::new(ReviewLog::default()));
    let internal_endpoint: Arc<Mutex<Option<InternalOpenAiHandle>>> = Arc::new(Mutex::new(None));

    let app_state = AppState {
        store,
        history,
        review_log,
        internal_endpoint,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            if let Some(w) = app.get_webview_window("main") {
                let build = std::env::var("TAURI_BUILD_NUMBER")
                    .unwrap_or_else(|_| "0".to_string());
                let title = format!("ledgrrr v{}+b{}", env!("CARGO_PKG_VERSION"), build);
                let _ = w.set_title(&title);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_state,
            commands::save_settings,
            commands::send_message,
            commands::load_rhai_rule_prompt,
            commands::use_internal_phi,
            commands::use_foundry_local,
            commands::use_cloud_model,
            commands::open_docs_playbook,
            commands::get_test_harness_config,
            commands::write_dom_dump,
            commands::get_cargo_pkg_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ledgerr-tauri application");
}
