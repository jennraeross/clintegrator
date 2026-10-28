use crate::terminal::{detect_available_terminals, resolve_terminal};
use crate::ui::AppState;
use eframe::egui::{self, Color32, RichText, Ui};
use std::path::PathBuf;

pub struct TerminalViewState {
    pub test_status: Option<Result<String, String>>,
}

impl Default for TerminalViewState {
    fn default() -> Self {
        Self { test_status: None }
    }
}

pub fn render_terminal_view(app: &mut AppState, ui: &mut Ui) {
    ui.heading("Terminal Emulator Settings");
    ui.label(RichText::new("Configure the terminal emulator Clintegrator launches to run commands.").weak());
    ui.separator();

    let terminals = detect_available_terminals();
    let (resolved_id, resolved_exe) = resolve_terminal(&app.config.preferred_terminal);

    // Current Active Terminal Banner
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Active Terminal:").strong());
            ui.label(
                RichText::new(&resolved_exe)
                    .strong()
                    .color(Color32::from_rgb(100, 220, 120)),
            );
            ui.label(format!("(ID: {})", resolved_id));
        });
    });

    ui.add_space(8.0);

    // Preferred Terminal Choice
    ui.group(|ui| {
        ui.label(RichText::new("Preferred Terminal Selection:").strong());

        let mut current_pref = app.config.preferred_terminal.clone();

        ui.radio_value(&mut current_pref, "auto".to_string(), "Auto-detect (Recommended)");

        for t in &terminals {
            let label = if t.is_available {
                RichText::new(format!("{} [Available]", t.name)).color(Color32::from_rgb(150, 220, 150))
            } else {
                RichText::new(format!("{} [Not Found]", t.name)).weak()
            };

            ui.radio_value(&mut current_pref, t.id.clone(), label);
        }

        ui.radio_value(&mut current_pref, "custom".to_string(), "Custom Terminal Template");

        if current_pref != app.config.preferred_terminal {
            app.config.preferred_terminal = current_pref;
            let _ = app.config.save();
            app.set_status("Terminal preference saved.");
        }

        if app.config.preferred_terminal == "custom" {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Custom Command Template:");
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut app.config.custom_terminal_template)
                            .hint_text("{terminal} -e {command}"),
                    )
                    .changed()
                {
                    let _ = app.config.save();
                }
            });
            ui.label(
                RichText::new("Variables: {terminal} = terminal binary, {command} = user command")
                    .small()
                    .weak(),
            );
        }
    });

    ui.add_space(8.0);

    // Test Terminal Button
    ui.horizontal(|ui| {
        if ui.button(RichText::new("🚀 Test Active Terminal").strong()).clicked() {
            let dummy_file = PathBuf::from("test_file.txt");
            let test_cmd = "echo '=== Clintegrator Terminal Test ==='; echo 'Your terminal is working properly!'; sleep 2";

            let res = crate::terminal::launch_in_terminal(
                &app.config.preferred_terminal,
                &app.config.custom_terminal_template,
                test_cmd,
                &dummy_file,
                true,
                "current",
            );

            match res {
                Ok(_) => {
                    app.terminal_state.test_status = Some(Ok("Test terminal launched successfully!".to_string()));
                }
                Err(e) => {
                    app.terminal_state.test_status = Some(Err(e));
                }
            }
        }

        if let Some(ref status) = app.terminal_state.test_status {
            match status {
                Ok(msg) => {
                    ui.label(RichText::new(format!("✓ {}", msg)).color(Color32::from_rgb(100, 220, 100)));
                }
                Err(err) => {
                    ui.label(RichText::new(format!("✗ {}", err)).color(Color32::from_rgb(250, 100, 100)));
                }
            }
        }
    });
}

