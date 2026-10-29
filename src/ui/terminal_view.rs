use crate::ui::{form_label, AppState};
use eframe::egui::{self, Label, RichText, Ui};
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
    let c = app.colors;
    ui.heading("Terminal Emulator Settings");
    ui.label(RichText::new("Configure the terminal emulator Clintegrator launches to run commands.").weak());
    ui.separator();

    let (resolved_id, resolved_exe) = app.resolved_terminal.clone();

    // Current Active Terminal Banner
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Active Terminal:").strong());
            ui.add(
                Label::new(
                    RichText::new(&resolved_exe)
                        .strong()
                        .color(c.success),
                )
                .truncate(),
            )
            .on_hover_text(&resolved_exe);
            ui.label(RichText::new(format!("(ID: {})", resolved_id)).weak());
        });
    });

    ui.add_space(8.0);

    // Preferred Terminal Choice
    ui.group(|ui| {
        ui.label(RichText::new("Preferred Terminal Selection:").strong());

        let mut current_pref = app.config.preferred_terminal.clone();

        ui.radio_value(&mut current_pref, "auto".to_string(), "Auto-detect (Recommended)");

        for t in &app.terminals {
            let label = if t.is_available {
                RichText::new(format!("{} [Available]", t.name)).color(c.success)
            } else {
                RichText::new(format!("{} [Not Found]", t.name)).weak()
            };

            ui.radio_value(&mut current_pref, t.id.clone(), label);
        }

        ui.radio_value(&mut current_pref, "custom".to_string(), "Custom Terminal Template");

        if current_pref != app.config.preferred_terminal {
            app.config.preferred_terminal = current_pref;
            app.refresh_resolved_terminal();
            let _ = app.config.save();
            app.set_status("Terminal preference saved.");
        }

        if app.config.preferred_terminal == "custom" {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                form_label(ui, "Custom Command Template:");
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

    // Terminal actions
    ui.horizontal_wrapped(|ui| {
        if ui.button(RichText::new("🔄 Rescan Terminals").strong()).clicked() {
            app.refresh_terminal_cache();
            app.set_status("Terminal list refreshed.");
        }

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
                    ui.label(RichText::new(format!("✓ {}", msg)).color(c.success));
                }
                Err(err) => {
                    ui.label(RichText::new(format!("✗ {}", err)).color(c.error));
                }
            }
        }
    });
}
