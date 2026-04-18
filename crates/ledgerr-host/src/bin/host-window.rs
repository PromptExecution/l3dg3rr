#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
slint::slint! {
    export component HostWindow inherits Window {
        width: 360px;
        height: 180px;
        title: "l3dg3rr";
        in property <string> version_text: "Version";

        Rectangle {
            background: #f3f7fb;

            VerticalLayout {
                padding: 20px;
                spacing: 10px;

                Text {
                    text: "l3dg3rr";
                    color: #114477;
                    font-size: 24px;
                }

                Text {
                    text: "Desktop host window";
                    color: #223344;
                    font-size: 16px;
                }

                Text {
                    text: version_text;
                    color: #445566;
                }

                Text {
                    text: "Tray -> Show Window launches this process.";
                    color: #556677;
                    wrap: word-wrap;
                }
            }
        }
    }
}

#[cfg(windows)]
fn main() -> Result<(), slint::PlatformError> {
    let app = HostWindow::new()?;
    app.set_version_text(format!("Version {}", env!("CARGO_PKG_VERSION")).into());
    app.run()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("host-window is currently supported on Windows builds only");
    std::process::exit(1);
}
