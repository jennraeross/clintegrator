use crate::os_integration::{OsIntegration, OsIntegrationStatus};
use crate::ui::AppState;
use eframe::egui::{self, Color32, RichText, Ui};
use std::path::PathBuf;

pub struct OsViewState {
    pub message: Option<Result<String, String>>,
}

impl Default for OsViewState {
    fn default() -> Self {
        Self { message: None }
    }
}

pub fn render_os_view(app: &mut AppState, ui: &mut Ui) {
    ui.heading("Operating System Integration");
    ui.label(
        RichText::new("Register Clintegrator with the OS so file managers and 'Open With' menus can use it for all filetypes.")
            .weak(),
    );
    ui.separator();

    #[cfg(target_os = "linux")]
    {
        render_linux_section(app, ui);
    }

    #[cfg(target_os = "windows")]
    {
        render_windows_section(app, ui);
    }

    #[cfg(target_os = "macos")]
    {
        render_macos_section(app, ui);
    }

    // Always offer cross-platform export options for portability
    #[cfg(not(target_os = "windows"))]
    {
        ui.add_space(8.0);
        render_windows_export_card(app, ui);
    }
}

#[cfg(target_os = "linux")]
fn render_linux_section(app: &mut AppState, ui: &mut Ui) {
    let status = OsIntegration::status();
    let desktop_path = OsIntegration::linux_desktop_file_path();

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Linux Desktop Integration Status:").strong());
            match &status {
                OsIntegrationStatus::Installed(path) => {
                    ui.label(
                        RichText::new("✓ Active & Installed")
                            .color(Color32::from_rgb(100, 220, 100))
                            .strong(),
                    );
                    ui.label(format!("({})", path.display()));
                }
                OsIntegrationStatus::NotInstalled => {
                    ui.label(
                        RichText::new("⚠ Not Installed")
                            .color(Color32::from_rgb(240, 180, 50))
                            .strong(),
                    );
                }
                OsIntegrationStatus::Outdated(path) => {
                    ui.label(
                        RichText::new("🔄 Outdated Exe Path")
                            .color(Color32::from_rgb(240, 180, 50))
                            .strong(),
                    );
                    ui.label(format!("({})", path.display()));
                }
            }
        });

        ui.add_space(4.0);
        ui.label("Current Binary Path:");
        ui.label(RichText::new(OsIntegration::current_executable().to_string_lossy()).monospace());

        ui.label("Desktop File Target:");
        ui.label(RichText::new(desktop_path.to_string_lossy()).monospace());

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(RichText::new("📥 Register Clintegrator with System").strong()).clicked() {
                match OsIntegration::install_linux_desktop() {
                    Ok(path) => {
                        app.os_state.message = Some(Ok(format!("Installed desktop integration to {}", path.display())));
                        app.set_status("OS Integration registered.");
                    }
                    Err(e) => {
                        app.os_state.message = Some(Err(e));
                    }
                }
            }

            if ui.button("🗑 Remove Integration").clicked() {
                match OsIntegration::uninstall_linux_desktop() {
                    Ok(_) => {
                        app.os_state.message = Some(Ok("Desktop entry removed successfully.".to_string()));
                        app.set_status("OS Integration removed.");
                    }
                    Err(e) => {
                        app.os_state.message = Some(Err(e));
                    }
                }
            }
        });

        if let Some(ref msg) = app.os_state.message {
            ui.add_space(4.0);
            match msg {
                Ok(text) => {
                    ui.label(RichText::new(format!("✓ {}", text)).color(Color32::from_rgb(100, 220, 100)));
                }
                Err(err) => {
                    ui.label(RichText::new(format!("✗ {}", err)).color(Color32::from_rgb(250, 100, 100)));
                }
            }
        }
    });

    ui.add_space(8.0);

    egui::CollapsingHeader::new("ℹ How Linux 'Open All Filetypes' Works")
        .default_open(false)
        .show(ui, |ui| {
            ui.label("Clintegrator registers broad MIME types in its .desktop file:");
            ui.label(RichText::new("MimeType=application/octet-stream;text/plain;*/*;all/all;all/allfiles;").monospace());
            ui.label("This informs GUI file managers (such as Nautilus, Dolphin, Thunar, PCManFM) that Clintegrator can handle any file.");
            ui.label("To make it default, you can right-click any file in your file manager -> 'Open With Other Application' -> select Clintegrator -> 'Always use for this file type'.");
        });
}

fn render_windows_export_card(app: &mut AppState, ui: &mut Ui) {
    egui::CollapsingHeader::new("🪟 Windows Integration Helper (.reg)")
        .default_open(false)
        .show(ui, |ui| {
            ui.label("On Windows, Clintegrator can register into Windows Explorer for all files via Registry:");
            ui.label(RichText::new("HKEY_CURRENT_USER\\Software\\Classes\\*\\shell\\Open with Clintegrator").monospace());

            if ui.button("Export 'clintegrator-register.reg'").clicked() {
                let dest = PathBuf::from("clintegrator-register.reg");
                match OsIntegration::export_windows_reg_file(&dest) {
                    Ok(_) => {
                        app.set_status("Exported clintegrator-register.reg");
                    }
                    Err(e) => {
                        app.os_state.message = Some(Err(e));
                    }
                }
            }
        });
}

#[cfg(target_os = "windows")]
fn render_windows_section(app: &mut AppState, ui: &mut Ui) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Windows Registry Association").strong());
        ui.label("Associate Clintegrator with all files (*) in the Windows Explorer context menu.");

        if ui.button("Export and Register .reg file").clicked() {
            let dest = PathBuf::from("clintegrator-register.reg");
            let _ = OsIntegration::export_windows_reg_file(&dest);
            app.set_status("Created clintegrator-register.reg");
        }
    });
}

#[cfg(target_os = "macos")]
fn render_macos_section(_app: &mut AppState, ui: &mut Ui) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("macOS LaunchServices Integration").strong());
        ui.label("On macOS, Clintegrator can be wrapped into Clintegrator.app bundle declaring 'public.item' CFBundleDocumentTypes.");
        egui::CollapsingHeader::new("View Info.plist template").show(ui, |ui| {
            ui.label(RichText::new(OsIntegration::generate_macos_info_plist()).monospace().small());
        });
    });
}
