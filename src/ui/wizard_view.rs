use crate::config::{parse_filetypes_input, Rule};
use crate::ui::{AppMode, AppState};
use eframe::egui::{self, Color32, RichText, Ui};
use std::path::PathBuf;

pub struct WizardState {
    pub target_file: PathBuf,
    pub rule_name: String,
    pub filetypes_input: String,
    pub command: String,
    pub hold_open: bool,
    pub working_dir: String,
    pub error_message: Option<String>,
}

impl WizardState {
    pub fn new(target_file: PathBuf) -> Self {
        // Extract default extension or filename
        let default_type = if let Some(ext) = target_file.extension().and_then(|e| e.to_str()) {
            ext.to_lowercase()
        } else if let Some(name) = target_file.file_name().and_then(|n| n.to_str()) {
            name.to_lowercase()
        } else {
            "unknown".to_string()
        };

        Self {
            target_file,
            rule_name: format!("Rule for .{}", default_type),
            filetypes_input: default_type,
            command: "nvim \"{file}\"".to_string(),
            hold_open: false,
            working_dir: "file_dir".to_string(),
            error_message: None,
        }
    }
}

pub fn render_wizard_view(app: &mut AppState, ui: &mut Ui) {
    let wizard = match app.wizard_state.as_mut() {
        Some(w) => w,
        None => return,
    };

    let target_path = wizard.target_file.clone();
    let file_display = target_path.to_string_lossy().to_string();
    let file_name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let ext_display = target_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_else(|| "(no extension)".to_string());

    let mut switch_to_manager = false;

    egui::CentralPanel::default().show(ui, |ui| {
        // Hero Header
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.heading(
                RichText::new("⚡ Unrecognized Filetype Detected")
                    .size(22.0)
                    .color(Color32::from_rgb(255, 190, 70))
                    .strong(),
            );
            ui.label(RichText::new("Configure a terminal command to open this filetype:").weak());
            ui.add_space(4.0);
        });

        ui.separator();

        // Target File Card
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📄 Target File:").strong());
                ui.label(RichText::new(&file_name).strong().color(Color32::from_rgb(100, 200, 255)));
                ui.label(RichText::new(format!("({})", ext_display)).weak());
            });
            ui.horizontal(|ui| {
                ui.label("Full Path:");
                ui.label(RichText::new(&file_display).monospace().small());
            });
        });

        ui.add_space(8.0);

        // Wizard Form
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Rule Label (Optional):");
                ui.text_edit_singleline(&mut wizard.rule_name);
            });
            ui.add_space(4.0);

            ui.label(RichText::new("Step 1: Filetype(s) to match").strong());
            ui.horizontal(|ui| {
                ui.label("Extensions/Names:");
                ui.add(
                    egui::TextEdit::singleline(&mut wizard.filetypes_input)
                        .hint_text("e.g. xyz, log, custom (comma-separated)"),
                );
            });

            let parsed = parse_filetypes_input(&wizard.filetypes_input);
            ui.horizontal_wrapped(|ui| {
                ui.label("Recognized types:");
                if parsed.is_empty() {
                    ui.label(RichText::new("None").weak());
                } else {
                    for ft in &parsed {
                        ui.label(
                            RichText::new(format!(".{}", ft))
                                .background_color(Color32::from_rgb(35, 60, 85))
                                .color(Color32::from_rgb(180, 220, 255)),
                        );
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();

            ui.label(RichText::new("Step 2: Terminal Command").strong());
            ui.horizontal(|ui| {
                ui.label("Command:");
                ui.add(
                    egui::TextEdit::singleline(&mut wizard.command)
                        .hint_text("e.g. nvim \"{file}\" or bat \"{file}\""),
                );
            });

            // Quick Presets
            ui.horizontal_wrapped(|ui| {
                ui.label("Presets:");
                if ui.button("nvim \"{file}\"").clicked() {
                    wizard.command = "nvim \"{file}\"".to_string();
                    wizard.hold_open = false;
                }
                if ui.button("bat \"{file}\"").clicked() {
                    wizard.command = "bat --paging=always \"{file}\"".to_string();
                    wizard.hold_open = false;
                }
                if ui.button("hx \"{file}\"").clicked() {
                    wizard.command = "hx \"{file}\"".to_string();
                    wizard.hold_open = false;
                }
                if ui.button("less \"{file}\"").clicked() {
                    wizard.command = "less \"{file}\"".to_string();
                    wizard.hold_open = false;
                }
                if ui.button("cat \"{file}\"").clicked() {
                    wizard.command = "cat \"{file}\"".to_string();
                    wizard.hold_open = true;
                }
                if ui.button("python3 \"{file}\"").clicked() {
                    wizard.command = "python3 \"{file}\"".to_string();
                    wizard.hold_open = true;
                }
                if ui.button("bash \"{file}\"").clicked() {
                    wizard.command = "bash \"{file}\"".to_string();
                    wizard.hold_open = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();

            ui.label(RichText::new("Step 3: Options").strong());
            ui.checkbox(&mut wizard.hold_open, "Keep terminal open after command exits");

            ui.horizontal(|ui| {
                ui.label("Working Directory:");
                egui::ComboBox::from_id_salt("wizard_working_dir")
                    .selected_text(if wizard.working_dir == "file_dir" {
                        "File's Directory"
                    } else {
                        "Current Working Directory"
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut wizard.working_dir, "file_dir".to_string(), "File's Directory");
                        ui.selectable_value(&mut wizard.working_dir, "current".to_string(), "Current Working Directory");
                    });
            });

            ui.add_space(8.0);
            ui.separator();

            // Preview
            let formatted = crate::terminal::format_command(&wizard.command, &target_path);
            let (term_id, _) = crate::terminal::resolve_terminal(&app.config.preferred_terminal);
            ui.horizontal(|ui| {
                ui.label("Terminal:");
                ui.label(RichText::new(term_id).strong());
            });
            ui.horizontal(|ui| {
                ui.label("Will Execute:");
                ui.label(RichText::new(&formatted).monospace().color(Color32::from_rgb(120, 220, 150)));
            });
        });

        if let Some(err) = &wizard.error_message {
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Error: {}", err)).color(Color32::from_rgb(250, 100, 100)));
        }

        ui.add_space(12.0);

        // Action Buttons
        ui.horizontal(|ui| {
            let parsed_types = parse_filetypes_input(&wizard.filetypes_input);
            let can_proceed = !parsed_types.is_empty() && !wizard.command.trim().is_empty();

            if ui
                .add_enabled(
                    can_proceed,
                    egui::Button::new(RichText::new("🚀 Save Rule & Open File").strong().size(15.0)),
                )
                .clicked()
            {
                let name = if wizard.rule_name.trim().is_empty() {
                    format!("{} files", parsed_types.join(", "))
                } else {
                    wizard.rule_name.clone()
                };

                let mut rule = Rule::new(
                    name,
                    parsed_types,
                    wizard.command.clone(),
                );
                rule.hold_open = wizard.hold_open;
                rule.working_dir = wizard.working_dir.clone();

                // Save to config
                app.config.rules.insert(0, rule.clone());
                if let Err(e) = app.config.save() {
                    wizard.error_message = Some(format!("Failed to save config: {}", e));
                    return;
                }

                // Launch terminal
                let launch_res = crate::terminal::launch_in_terminal(
                    &app.config.preferred_terminal,
                    &app.config.custom_terminal_template,
                    &rule.command,
                    &target_path,
                    rule.hold_open,
                    &rule.working_dir,
                );

                match launch_res {
                    Ok(_) => {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Err(e) => {
                        wizard.error_message = Some(e);
                    }
                }
            }

            if ui
                .add_enabled(
                    !wizard.command.trim().is_empty(),
                    egui::Button::new(RichText::new("⚡ Open Once (Without Saving)").size(15.0)),
                )
                .clicked()
            {
                let launch_res = crate::terminal::launch_in_terminal(
                    &app.config.preferred_terminal,
                    &app.config.custom_terminal_template,
                    &wizard.command,
                    &target_path,
                    wizard.hold_open,
                    &wizard.working_dir,
                );

                match launch_res {
                    Ok(_) => {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Err(e) => {
                        wizard.error_message = Some(e);
                    }
                }
            }

            if ui.button("🛠 Open Full Manager").clicked() {
                switch_to_manager = true;
            }

            if ui.button("Cancel").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    });

    if switch_to_manager {
        app.mode = AppMode::Manager;
    }
}
