#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use ledgerr_host::chat::{send_chat_message, ChatRole, ChatTurn};
#[cfg(windows)]
use ledgerr_host::settings::{default_settings_path, ChatSettings, SettingsStore};

#[cfg(windows)]
slint::slint! {
    import { Button, LineEdit, ScrollView, TextEdit } from "std-widgets.slint";

    export component HostWindow inherits Window {
        width: 840px;
        height: 720px;
        title: "l3dg3rr";

        in-out property <string> version_text: "Version";
        in-out property <string> status_text: "Ready";
        in-out property <string> endpoint_text;
        in-out property <string> model_text;
        in-out property <string> api_key_text;
        in-out property <string> system_prompt_text;
        in-out property <string> transcript_text;
        in-out property <string> draft_message_text;
        in-out property <bool> busy: false;

        callback save_settings();
        callback send_message();

        Rectangle {
            background: #f3f7fb;

            VerticalLayout {
                padding: 16px;
                spacing: 12px;

                Text {
                    text: "l3dg3rr tool tray";
                    color: #114477;
                    font-size: 26px;
                }

                Text {
                    text: root.version_text;
                    color: #445566;
                }

                Text {
                    text: root.status_text;
                    color: #223344;
                    wrap: word-wrap;
                }

                Rectangle {
                    background: #e4edf6;
                    border-color: #c7d8ea;
                    border-width: 1px;
                    border-radius: 8px;

                    VerticalLayout {
                        padding: 12px;
                        spacing: 8px;

                        Text {
                            text: "Remote chat configuration";
                            color: #223344;
                            font-size: 18px;
                        }

                        Text { text: "Endpoint URL"; color: #445566; }
                        LineEdit { text <=> root.endpoint_text; enabled: !root.busy; }

                        Text { text: "Model"; color: #445566; }
                        LineEdit { text <=> root.model_text; enabled: !root.busy; }

                        Text { text: "API Key (persisted in settings.json)"; color: #445566; }
                        LineEdit { text <=> root.api_key_text; enabled: !root.busy; }

                        Text { text: "System Prompt"; color: #445566; }
                        TextEdit {
                            text <=> root.system_prompt_text;
                            enabled: !root.busy;
                            height: 90px;
                        }

                        HorizontalLayout {
                            spacing: 8px;

                            Button {
                                text: root.busy ? "Working..." : "Save Settings";
                                enabled: !root.busy;
                                clicked => { root.save_settings(); }
                            }

                            Text {
                                text: "Show Window from the tray to reopen this panel.";
                                color: #556677;
                                wrap: word-wrap;
                                vertical-alignment: center;
                            }
                        }
                    }
                }

                Rectangle {
                    background: #ffffff;
                    border-color: #d4dfe9;
                    border-width: 1px;
                    border-radius: 8px;
                    height: 340px;

                    VerticalLayout {
                        padding: 12px;
                        spacing: 8px;

                        Text {
                            text: "Chat";
                            color: #223344;
                            font-size: 18px;
                        }

                        ScrollView {
                            height: 180px;

                            Text {
                                width: parent.width;
                                text: root.transcript_text;
                                color: #223344;
                                wrap: word-wrap;
                            }
                        }

                        Text { text: "Message"; color: #445566; }
                        TextEdit {
                            text <=> root.draft_message_text;
                            enabled: !root.busy;
                            height: 100px;
                        }

                        HorizontalLayout {
                            spacing: 8px;

                            Button {
                                text: root.busy ? "Sending..." : "Send";
                                enabled: !root.busy;
                                clicked => { root.send_message(); }
                            }

                            Text {
                                text: "Uses the configured OpenAI-compatible endpoint.";
                                color: #556677;
                                wrap: word-wrap;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(windows)]
fn main() -> Result<(), slint::PlatformError> {
    let store = SettingsStore::new(default_settings_path());
    let settings = store
        .load()
        .map_err(|error| slint::PlatformError::from(error.to_string()))?;

    let app = HostWindow::new()?;
    app.set_version_text(format!("Version {}", env!("CARGO_PKG_VERSION")).into());
    app.set_status_text(format!("Editing {}", store.path().display()).into());
    app.set_endpoint_text(settings.chat.endpoint_url.clone().into());
    app.set_model_text(settings.chat.model.clone().into());
    app.set_api_key_text(settings.chat.api_key.clone().into());
    app.set_system_prompt_text(settings.chat.system_prompt.clone().into());
    app.set_transcript_text(
        "Tool tray chat is ready.\n\nSave the endpoint, model, and API key, then send a message."
            .into(),
    );

    let store = Arc::new(store);
    let history = Arc::new(Mutex::new(Vec::<ChatTurn>::new()));

    {
        let app_handle = app.as_weak();
        let store = Arc::clone(&store);
        app.on_save_settings(move || {
            let Some(app) = app_handle.upgrade() else {
                return;
            };

            let mut settings = match store.load() {
                Ok(settings) => settings,
                Err(error) => {
                    app.set_status_text(format!("Failed to load settings: {error}").into());
                    return;
                }
            };

            settings.chat = chat_settings_from_window(&app);
            match store.save(&settings) {
                Ok(()) => app.set_status_text(
                    format!("Saved chat settings to {}", store.path().display()).into(),
                ),
                Err(error) => {
                    app.set_status_text(format!("Failed to save settings: {error}").into())
                }
            }
        });
    }

    {
        let app_handle = app.as_weak();
        let store = Arc::clone(&store);
        let history = Arc::clone(&history);
        app.on_send_message(move || {
            let Some(app) = app_handle.upgrade() else {
                return;
            };

            let draft_message = app.get_draft_message_text().to_string();
            if draft_message.trim().is_empty() {
                app.set_status_text("Enter a message before sending.".into());
                return;
            }

            let mut settings = match store.load() {
                Ok(settings) => settings,
                Err(error) => {
                    app.set_status_text(format!("Failed to load settings: {error}").into());
                    return;
                }
            };
            settings.chat = chat_settings_from_window(&app);
            if let Err(error) = store.save(&settings) {
                app.set_status_text(format!("Failed to save settings: {error}").into());
                return;
            }

            let user_turn = ChatTurn {
                role: ChatRole::User,
                content: draft_message.trim().to_string(),
            };
            let history_snapshot = {
                let mut history = history.lock().expect("chat history poisoned");
                history.push(user_turn.clone());
                history.clone()
            };

            app.set_busy(true);
            app.set_status_text(
                format!(
                    "Sending to {} with model {}",
                    settings.chat.endpoint_url, settings.chat.model
                )
                .into(),
            );
            app.set_transcript_text(render_transcript(&history_snapshot).into());

            let app_handle = app.as_weak();
            let history = Arc::clone(&history);
            std::thread::spawn(move || {
                let result = send_chat_message(
                    &settings.chat,
                    &history_snapshot[..history_snapshot.len() - 1],
                    &user_turn.content,
                );

                let _ = slint::invoke_from_event_loop(move || {
                    let Some(app) = app_handle.upgrade() else {
                        return;
                    };

                    let mut history = history.lock().expect("chat history poisoned");
                    match result {
                        Ok(response) => {
                            history.push(ChatTurn {
                                role: ChatRole::Assistant,
                                content: response,
                            });
                            app.set_transcript_text(render_transcript(&history).into());
                            app.set_draft_message_text("".into());
                            app.set_status_text("Remote chat response received.".into());
                        }
                        Err(error) => {
                            app.set_transcript_text(render_transcript(&history).into());
                            app.set_status_text(format!("Chat request failed: {error}").into());
                        }
                    }
                    app.set_busy(false);
                });
            });
        });
    }

    app.run()
}

#[cfg(windows)]
fn chat_settings_from_window(app: &HostWindow) -> ChatSettings {
    ChatSettings {
        endpoint_url: app.get_endpoint_text().trim().to_string(),
        model: app.get_model_text().trim().to_string(),
        api_key: app.get_api_key_text().trim().to_string(),
        system_prompt: app.get_system_prompt_text().trim().to_string(),
    }
}

#[cfg(windows)]
fn render_transcript(history: &[ChatTurn]) -> String {
    if history.is_empty() {
        return "No messages yet.".to_string();
    }

    history
        .iter()
        .map(|turn| {
            let speaker = match turn.role {
                ChatRole::System => "System",
                ChatRole::User => "You",
                ChatRole::Assistant => "Assistant",
            };
            format!("{speaker}\n{}\n", turn.content.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(not(windows))]
fn main() {
    eprintln!("host-window is currently supported on Windows builds only");
    std::process::exit(1);
}
