use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::notify::{
    NotificationBackend, NotificationEvent, NotificationSettings, NotificationStatus,
    NotificationTestResult, Notifier, NotifyError, PowerShellBurntToastNotifier,
};
use crate::settings::{AppSettings, SettingsStore};

use super::{tray_menu_labels, TrayCommand, TrayState};

#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

pub fn run(store: SettingsStore) -> Result<(), Box<dyn std::error::Error>> {
    let settings = store.load()?;
    let state = Arc::new(Mutex::new(TrayState::from_settings(&settings)));

    let (ready_tx, ready_rx) = mpsc::channel::<Result<TrayThreadHandle, String>>();
    let tray_thread = std::thread::spawn({
        let settings = settings.clone();
        let ready_tx_for_error = ready_tx.clone();
        move || {
            if let Err(error) = tray_thread_main(settings, ready_tx) {
                let _ = ready_tx_for_error.send(Err(error.to_string()));
            }
        }
    });

    let tray = match ready_rx.recv()? {
        Ok(handle) => handle,
        Err(message) => return Err(message.into()),
    };

    send_best_effort_toast(
        &settings,
        NotificationEvent::Test {
            title: "l3dg3rr".to_string(),
            body: format!("Hello from l3dg3rr {}", env!("CARGO_PKG_VERSION")),
        },
    );

    loop {
        if let Ok(event) = MenuEvent::receiver().recv_timeout(Duration::from_millis(250)) {
            let command = if event.id.as_ref() == tray.toast_enabled_id.as_str() {
                let enabled = !state.lock().expect("tray state poisoned").toast_enabled;
                TrayCommand::ToggleToast(enabled)
            } else if event.id.as_ref() == tray.cycle_backend_id.as_str() {
                TrayCommand::CycleBackend
            } else if event.id.as_ref() == tray.test_toast_id.as_str() {
                TrayCommand::TestToast
            } else if event.id.as_ref() == tray.start_minimized_to_tray_id.as_str() {
                let enabled = !state
                    .lock()
                    .expect("tray state poisoned")
                    .start_minimized_to_tray;
                TrayCommand::ToggleStartMinimizedToTray(enabled)
            } else if event.id.as_ref() == tray.window_visible_on_start_id.as_str() {
                let enabled = !state
                    .lock()
                    .expect("tray state poisoned")
                    .window_visible_on_start;
                TrayCommand::ToggleWindowVisibleOnStart(enabled)
            } else if event.id.as_ref() == tray.notify_approval_required_id.as_str() {
                let enabled = !state
                    .lock()
                    .expect("tray state poisoned")
                    .show_notifications_for
                    .approval_required;
                TrayCommand::ToggleApprovalRequired(enabled)
            } else if event.id.as_ref() == tray.notify_transaction_submitted_id.as_str() {
                let enabled = !state
                    .lock()
                    .expect("tray state poisoned")
                    .show_notifications_for
                    .transaction_submitted;
                TrayCommand::ToggleTransactionSubmitted(enabled)
            } else if event.id.as_ref() == tray.notify_run_failed_id.as_str() {
                let enabled = !state
                    .lock()
                    .expect("tray state poisoned")
                    .show_notifications_for
                    .run_failed;
                TrayCommand::ToggleRunFailed(enabled)
            } else if event.id.as_ref() == tray.notify_run_completed_id.as_str() {
                let enabled = !state
                    .lock()
                    .expect("tray state poisoned")
                    .show_notifications_for
                    .run_completed;
                TrayCommand::ToggleRunCompleted(enabled)
            } else if event.id.as_ref() == tray.show_window_id.as_str() {
                TrayCommand::ShowWindow
            } else if event.id.as_ref() == tray.exit_id.as_str() {
                TrayCommand::Quit
            } else {
                continue;
            };

            let should_quit = handle_command(command, &store, &state, &tray)?;
            if should_quit {
                break;
            }
        }
    }

    let _ = tray.control_tx.send(TrayControl::Quit);
    let _ = tray_thread.join();
    Ok(())
}

struct TrayThreadHandle {
    control_tx: mpsc::Sender<TrayControl>,
    toast_enabled_id: String,
    cycle_backend_id: String,
    test_toast_id: String,
    start_minimized_to_tray_id: String,
    window_visible_on_start_id: String,
    notify_approval_required_id: String,
    notify_transaction_submitted_id: String,
    notify_run_failed_id: String,
    notify_run_completed_id: String,
    show_window_id: String,
    exit_id: String,
}

enum TrayControl {
    SetState(TrayState),
    Quit,
}

fn tray_thread_main(
    settings: AppSettings,
    ready_tx: mpsc::Sender<Result<TrayThreadHandle, String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = TrayState::from_settings(&settings);
    let labels = tray_menu_labels(&state);

    let menu = Menu::new();
    let version = MenuItem::new(labels.version, false, None);
    let backend = MenuItem::new(labels.backend, false, None);
    let toast_enabled =
        CheckMenuItem::new(labels.toast_enabled, true, settings.toast_enabled, None);
    let cycle_backend = MenuItem::new(labels.cycle_backend, true, None);
    let last_test = MenuItem::new(labels.last_test, false, None);
    let start_minimized_to_tray = CheckMenuItem::new(
        labels.start_minimized_to_tray,
        true,
        settings.start_minimized_to_tray,
        None,
    );
    let window_visible_on_start = CheckMenuItem::new(
        labels.window_visible_on_start,
        true,
        settings.window_visible_on_start,
        None,
    );
    let notify_approval_required = CheckMenuItem::new(
        labels.notify_approval_required,
        true,
        settings.show_notifications_for.approval_required,
        None,
    );
    let notify_transaction_submitted = CheckMenuItem::new(
        labels.notify_transaction_submitted,
        true,
        settings.show_notifications_for.transaction_submitted,
        None,
    );
    let notify_run_failed = CheckMenuItem::new(
        labels.notify_run_failed,
        true,
        settings.show_notifications_for.run_failed,
        None,
    );
    let notify_run_completed = CheckMenuItem::new(
        labels.notify_run_completed,
        true,
        settings.show_notifications_for.run_completed,
        None,
    );
    let test_toast = MenuItem::new(labels.test_toast, true, None);
    let status_item = MenuItem::new(labels.status, false, None);
    let show_window = MenuItem::new(labels.show_window, true, None);
    let exit = MenuItem::new(labels.exit, true, None);

    menu.append(&version)?;
    menu.append(&backend)?;
    menu.append(&toast_enabled)?;
    menu.append(&cycle_backend)?;
    menu.append(&last_test)?;
    menu.append(&start_minimized_to_tray)?;
    menu.append(&window_visible_on_start)?;
    menu.append(&notify_approval_required)?;
    menu.append(&notify_transaction_submitted)?;
    menu.append(&notify_run_failed)?;
    menu.append(&notify_run_completed)?;
    menu.append(&test_toast)?;
    menu.append(&status_item)?;
    menu.append(&show_window)?;
    menu.append(&exit)?;

    let icon = make_icon()?;
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip(&format!("l3dg3rr {}", env!("CARGO_PKG_VERSION")))
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(true)
        .with_menu_on_right_click(true)
        .with_icon(icon)
        .build()?;

    let (control_tx, control_rx) = mpsc::channel::<TrayControl>();
    ready_tx
        .send(Ok(TrayThreadHandle {
            control_tx,
            toast_enabled_id: toast_enabled.id().as_ref().to_string(),
            cycle_backend_id: cycle_backend.id().as_ref().to_string(),
            test_toast_id: test_toast.id().as_ref().to_string(),
            start_minimized_to_tray_id: start_minimized_to_tray.id().as_ref().to_string(),
            window_visible_on_start_id: window_visible_on_start.id().as_ref().to_string(),
            notify_approval_required_id: notify_approval_required.id().as_ref().to_string(),
            notify_transaction_submitted_id: notify_transaction_submitted.id().as_ref().to_string(),
            notify_run_failed_id: notify_run_failed.id().as_ref().to_string(),
            notify_run_completed_id: notify_run_completed.id().as_ref().to_string(),
            show_window_id: show_window.id().as_ref().to_string(),
            exit_id: exit.id().as_ref().to_string(),
        }))
        .map_err(|error| error.to_string())?;

    run_tray_pump(
        tray_icon,
        TrayMenuItems {
            backend,
            toast_enabled,
            last_test,
            start_minimized_to_tray,
            window_visible_on_start,
            notify_approval_required,
            notify_transaction_submitted,
            notify_run_failed,
            notify_run_completed,
            status_item,
        },
        control_rx,
    )?;
    Ok(())
}

struct TrayMenuItems {
    backend: MenuItem,
    toast_enabled: CheckMenuItem,
    last_test: MenuItem,
    start_minimized_to_tray: CheckMenuItem,
    window_visible_on_start: CheckMenuItem,
    notify_approval_required: CheckMenuItem,
    notify_transaction_submitted: CheckMenuItem,
    notify_run_failed: CheckMenuItem,
    notify_run_completed: CheckMenuItem,
    status_item: MenuItem,
}

fn run_tray_pump(
    _tray_icon: TrayIcon,
    menu_items: TrayMenuItems,
    control_rx: mpsc::Receiver<TrayControl>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        #[cfg(windows)]
        {
            let mut msg = MSG::default();
            while unsafe { PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE) } != 0 {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }

        match control_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(TrayControl::SetState(state)) => {
                let labels = tray_menu_labels(&state);
                menu_items.backend.set_text(&labels.backend);
                menu_items.toast_enabled.set_checked(state.toast_enabled);
                menu_items.last_test.set_text(&labels.last_test);
                menu_items
                    .start_minimized_to_tray
                    .set_checked(state.start_minimized_to_tray);
                menu_items
                    .window_visible_on_start
                    .set_checked(state.window_visible_on_start);
                menu_items
                    .notify_approval_required
                    .set_checked(state.show_notifications_for.approval_required);
                menu_items
                    .notify_transaction_submitted
                    .set_checked(state.show_notifications_for.transaction_submitted);
                menu_items
                    .notify_run_failed
                    .set_checked(state.show_notifications_for.run_failed);
                menu_items
                    .notify_run_completed
                    .set_checked(state.show_notifications_for.run_completed);
                menu_items.status_item.set_text(&labels.status);
            }
            Ok(TrayControl::Quit) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}

fn handle_command(
    command: TrayCommand,
    store: &SettingsStore,
    state: &Arc<Mutex<TrayState>>,
    tray: &TrayThreadHandle,
) -> Result<bool, Box<dyn std::error::Error>> {
    match command {
        TrayCommand::ToggleToast(enabled) => {
            let mut settings = store.load()?;
            settings.toast_enabled = enabled;
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::CycleBackend => {
            let mut settings = store.load()?;
            settings.toast_backend_preference = next_backend(settings.toast_backend_preference);
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::TestToast => {
            let mut settings = store.load()?;
            let test_result = match run_notification_test(&settings) {
                Ok(result) => result,
                Err(error) => NotificationTestResult {
                    status: NotificationStatus::Failed,
                    timestamp: Some(Utc::now()),
                    message: Some(error.to_string()),
                },
            };
            settings.last_test_result = Some(test_result);
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::ToggleWindowVisibleOnStart(enabled) => {
            let mut settings = store.load()?;
            settings.window_visible_on_start = enabled;
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::ToggleApprovalRequired(enabled) => {
            let mut settings = store.load()?;
            settings.show_notifications_for.approval_required = enabled;
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::ToggleTransactionSubmitted(enabled) => {
            let mut settings = store.load()?;
            settings.show_notifications_for.transaction_submitted = enabled;
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::ToggleRunFailed(enabled) => {
            let mut settings = store.load()?;
            settings.show_notifications_for.run_failed = enabled;
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::ToggleRunCompleted(enabled) => {
            let mut settings = store.load()?;
            settings.show_notifications_for.run_completed = enabled;
            store.save(&settings)?;

            sync_state(state, &settings, tray);
            Ok(false)
        }
        TrayCommand::ShowWindow => {
            if let Ok(mut state) = state.lock() {
                state.window_visible = true;
            }
            show_window_process()?;
            Ok(false)
        }
        TrayCommand::Quit => {
            let settings = store.load()?;
            send_best_effort_toast(
                &settings,
                NotificationEvent::Test {
                    title: "l3dg3rr".to_string(),
                    body: "Goodbye from l3dg3rr".to_string(),
                },
            );
            Ok(true)
        }
    }
}

fn sync_state(state: &Arc<Mutex<TrayState>>, settings: &AppSettings, tray: &TrayThreadHandle) {
    let mut state = state.lock().expect("tray state poisoned");
    state.apply_settings(settings);
    let _ = tray.control_tx.send(TrayControl::SetState(state.clone()));
}

fn next_backend(current: NotificationBackend) -> NotificationBackend {
    match current {
        NotificationBackend::Auto => NotificationBackend::PowerShell,
        NotificationBackend::PowerShell => NotificationBackend::Noop,
        NotificationBackend::Noop => NotificationBackend::Auto,
    }
}

fn run_notification_test(settings: &AppSettings) -> Result<NotificationTestResult, NotifyError> {
    match settings.toast_backend_preference {
        NotificationBackend::Noop => Ok(NotificationTestResult {
            status: NotificationStatus::Disabled,
            timestamp: Some(Utc::now()),
            message: Some("noop backend selected".to_string()),
        }),
        NotificationBackend::Auto | NotificationBackend::PowerShell => {
            let notify_settings = NotificationSettings {
                enabled: settings.toast_enabled,
                backend: settings.toast_backend_preference,
                last_test_result: settings.last_test_result.clone(),
            };
            let notifier = PowerShellBurntToastNotifier::new(notify_settings);
            notifier.test("l3dg3rr", "tray test toast")
        }
    }
}

fn send_best_effort_toast(settings: &AppSettings, event: NotificationEvent) {
    if matches!(settings.toast_backend_preference, NotificationBackend::Noop) {
        return;
    }

    let notify_settings = NotificationSettings {
        enabled: settings.toast_enabled,
        backend: settings.toast_backend_preference,
        last_test_result: settings.last_test_result.clone(),
    };
    let notifier = PowerShellBurntToastNotifier::new(notify_settings);
    let _ = notifier.notify(&event);
}

fn show_window_process() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let host_window = current_exe.with_file_name("host-window.exe");
    std::process::Command::new(host_window).spawn()?;
    Ok(())
}

fn make_icon() -> Result<Icon, tray_icon::BadIcon> {
    let width = 16;
    let height = 16;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let border = x == 0 || y == 0 || x == width - 1 || y == height - 1;
            let fill = (2..=13).contains(&x) && (2..=13).contains(&y);
            let stem = (4..=6).contains(&x) && (4..=11).contains(&y);
            let foot = (4..=11).contains(&x) && (10..=12).contains(&y);

            let pixel = if border {
                [0x0D, 0x47, 0xA1, 0xFF]
            } else if stem || foot {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else if fill {
                [0x19, 0x7A, 0xD9, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    Icon::from_rgba(rgba, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_cycle_covers_all_known_variants() {
        assert_eq!(
            next_backend(NotificationBackend::Auto),
            NotificationBackend::PowerShell
        );
        assert_eq!(
            next_backend(NotificationBackend::PowerShell),
            NotificationBackend::Noop
        );
        assert_eq!(
            next_backend(NotificationBackend::Noop),
            NotificationBackend::Auto
        );
    }

    #[test]
    fn noop_backend_test_returns_disabled_result() {
        let settings = AppSettings {
            toast_backend_preference: NotificationBackend::Noop,
            ..AppSettings::default()
        };

        let result = run_notification_test(&settings).expect("noop backend should not fail");
        assert_eq!(result.status, NotificationStatus::Disabled);
        assert_eq!(result.message.as_deref(), Some("noop backend selected"));
    }

    #[test]
    fn powershell_backend_test_respects_disabled_setting() {
        let settings = AppSettings {
            toast_enabled: false,
            toast_backend_preference: NotificationBackend::PowerShell,
            ..AppSettings::default()
        };

        let result = run_notification_test(&settings).expect("disabled path should be ok");
        assert_eq!(result.status, NotificationStatus::Disabled);
    }
}
