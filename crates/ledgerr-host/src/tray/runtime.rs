use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::notify::{
    NotificationBackend, NotificationEvent, NotificationSettings, Notifier,
    PowerShellBurntToastNotifier,
};
use crate::settings::{AppSettings, SettingsStore};

use super::{TrayCommand, TrayState, tray_menu_labels};

#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
};

pub fn run(store: SettingsStore) -> Result<(), Box<dyn std::error::Error>> {
    let settings = store.load()?;
    let state = Arc::new(Mutex::new(TrayState {
        toast_enabled: settings.toast_enabled,
        notification_status: if settings.toast_enabled {
            crate::notify::NotificationStatus::Ready
        } else {
            crate::notify::NotificationStatus::Disabled
        },
        window_visible: settings.window_visible_on_start,
    }));

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
            } else if event.id.as_ref() == tray.test_toast_id.as_str() {
                TrayCommand::TestToast
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
    test_toast_id: String,
    show_window_id: String,
    exit_id: String,
}

enum TrayControl {
    SetToastEnabled(bool),
    SetStatus(String),
    Quit,
}

fn tray_thread_main(
    settings: AppSettings,
    ready_tx: mpsc::Sender<Result<TrayThreadHandle, String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let labels = tray_menu_labels(&TrayState {
        toast_enabled: settings.toast_enabled,
        notification_status: if settings.toast_enabled {
            crate::notify::NotificationStatus::Ready
        } else {
            crate::notify::NotificationStatus::Disabled
        },
        window_visible: settings.window_visible_on_start,
    });

    let menu = Menu::new();
    let version = MenuItem::new(labels.version, false, None);
    let toast_enabled =
        CheckMenuItem::new(labels.toast_enabled, true, settings.toast_enabled, None);
    let test_toast = MenuItem::new(labels.test_toast, true, None);
    let status_item = MenuItem::new(labels.status, false, None);
    let show_window = MenuItem::new(labels.show_window, true, None);
    let exit = MenuItem::new(labels.exit, true, None);

    menu.append(&version)?;
    menu.append(&toast_enabled)?;
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
            test_toast_id: test_toast.id().as_ref().to_string(),
            show_window_id: show_window.id().as_ref().to_string(),
            exit_id: exit.id().as_ref().to_string(),
        }))
        .map_err(|error| error.to_string())?;

    run_tray_pump(tray_icon, toast_enabled, status_item, control_rx)?;
    Ok(())
}

fn run_tray_pump(
    _tray_icon: TrayIcon,
    toast_enabled: CheckMenuItem,
    status_item: MenuItem,
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
            Ok(TrayControl::SetToastEnabled(enabled)) => {
                toast_enabled.set_checked(enabled);
            }
            Ok(TrayControl::SetStatus(status)) => {
                status_item.set_text(status);
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

            let mut state = state.lock().expect("tray state poisoned");
            state.toast_enabled = enabled;
            state.notification_status = if enabled {
                crate::notify::NotificationStatus::Unknown
            } else {
                crate::notify::NotificationStatus::Disabled
            };
            let status = tray_menu_labels(&state).status;
            let _ = tray.control_tx.send(TrayControl::SetToastEnabled(enabled));
            let _ = tray.control_tx.send(TrayControl::SetStatus(status));
            Ok(false)
        }
        TrayCommand::TestToast => {
            let settings = store.load()?;
            let notify_settings = NotificationSettings {
                enabled: settings.toast_enabled,
                backend: settings.toast_backend_preference,
                last_test_result: settings.last_test_result.clone(),
            };
            let notifier = PowerShellBurntToastNotifier::new(notify_settings);
            let result = notifier.test("l3dg3rr", "tray test toast");

            let mut settings = settings;
            let mut state = state.lock().expect("tray state poisoned");
            match result {
                Ok(test_result) => {
                    settings.last_test_result = Some(test_result);
                    settings.toast_backend_preference = NotificationBackend::PowerShell;
                    store.save(&settings)?;
                    state.notification_status = if state.toast_enabled {
                        crate::notify::NotificationStatus::Ready
                    } else {
                        crate::notify::NotificationStatus::Disabled
                    };
                }
                Err(_) => {
                    state.notification_status = crate::notify::NotificationStatus::Failed;
                }
            }

            let status = tray_menu_labels(&state).status;
            let _ = tray.control_tx.send(TrayControl::SetStatus(status));
            Ok(false)
        }
        TrayCommand::ShowWindow => {
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

fn send_best_effort_toast(settings: &AppSettings, event: NotificationEvent) {
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
