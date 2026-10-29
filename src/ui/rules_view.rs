use crate::config::{parse_filetypes_input, Rule};
use crate::ui::{form_label, AppState};
use eframe::egui::{self, Label, RichText, Ui};
use std::path::PathBuf;

pub struct RulesViewState {
    pub search_query: String,
    pub editing_rule: Option<RuleEditor>,
    pub testing_rule: Option<RuleTester>,
}

pub struct RuleEditor {
    pub original_id: Option<String>, // None if adding new rule
    pub name: String,
    pub filetypes_input: String,
    pub command: String,
    pub run_in_terminal: bool,
    pub hold_open: bool,
    pub working_dir: String,
}

pub struct RuleTester {
    pub rule: Rule,
    pub sample_path: String,
    pub test_result: Option<Result<String, String>>,
}

impl Default for RulesViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            editing_rule: None,
            testing_rule: None,
        }
    }
}

pub fn render_rules_view(app: &mut AppState, ui: &mut Ui) {
    let c = app.colors;

    ui.horizontal_wrapped(|ui| {
        ui.heading("Filetype Rules");

        if ui.button(RichText::new("+ Add New Pair").strong().color(c.success)).clicked() {
            app.rules_state.editing_rule = Some(RuleEditor {
                original_id: None,
                name: String::new(),
                filetypes_input: String::new(),
                command: "nvim \"{file}\"".to_string(),
                run_in_terminal: true,
                hold_open: app.config.default_hold_open,
                working_dir: "file_dir".to_string(),
            });
        }

        let search_width = ui.available_width().clamp(140.0, 260.0);
        ui.add_sized(
            [search_width, ui.spacing().interact_size.y],
            egui::TextEdit::singleline(&mut app.rules_state.search_query)
                .hint_text("🔍 Search filetypes or commands..."),
        );
    });

    ui.separator();

    // If currently editing a rule, show the editor card
    if let Some(mut editor) = app.rules_state.editing_rule.take() {
        let is_new = editor.original_id.is_none();
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading(if is_new { "Add Value Pair" } else { "Edit Value Pair" });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                form_label(ui, "Rule Label (Optional):");
                ui.add(egui::TextEdit::singleline(&mut editor.name));
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                form_label(ui, "Filetype(s):");
                ui.add(
                    egui::TextEdit::singleline(&mut editor.filetypes_input)
                        .hint_text("e.g. md, txt, rs (comma or space separated)"),
                );
            });

            let parsed = parse_filetypes_input(&editor.filetypes_input);
            ui.horizontal_wrapped(|ui| {
                ui.label("Parsed types:");
                if parsed.is_empty() {
                    ui.label(RichText::new("No filetypes specified yet").italics().weak());
                } else {
                    for ft in &parsed {
                        ui.label(
                            RichText::new(format!(".{}", ft))
                                .background_color(c.chip_bg)
                                .color(c.chip_fg),
                        );
                    }
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                form_label(ui, "Terminal Command:");
                ui.add(
                    egui::TextEdit::singleline(&mut editor.command)
                        .hint_text("e.g. nvim \"{file}\" or bat \"{file}\""),
                );
            });

            // Quick suggestion buttons
            ui.horizontal_wrapped(|ui| {
                ui.label("Quick templates:");
                if ui.button("nvim \"{file}\"").clicked() {
                    editor.command = "nvim \"{file}\"".to_string();
                }
                if ui.button("bat \"{file}\"").clicked() {
                    editor.command = "bat --paging=always \"{file}\"".to_string();
                }
                if ui.button("hx \"{file}\"").clicked() {
                    editor.command = "hx \"{file}\"".to_string();
                }
                if ui.button("vd \"{file}\"").clicked() {
                    editor.command = "vd \"{file}\"".to_string();
                }
                if ui.button("python3 \"{file}\"").clicked() {
                    editor.command = "python3 \"{file}\"".to_string();
                    editor.hold_open = true;
                }
                if ui.button("bash \"{file}\"").clicked() {
                    editor.command = "bash \"{file}\"".to_string();
                    editor.hold_open = true;
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.checkbox(&mut editor.run_in_terminal, "Run in terminal emulator");
            });

            if !editor.run_in_terminal {
                ui.label(
                    RichText::new("Direct Mode: runs command directly in background without opening a terminal window.")
                        .small()
                        .weak(),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut editor.hold_open, "Keep terminal open after command exits");
                });
            }

            ui.horizontal(|ui| {
                form_label(ui, "Working Directory:");
                egui::ComboBox::from_id_salt("editor_working_dir")
                    .selected_text(if editor.working_dir == "file_dir" {
                        "File's Directory"
                    } else {
                        "Current Working Directory"
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editor.working_dir, "file_dir".to_string(), "File's Directory");
                        ui.selectable_value(&mut editor.working_dir, "current".to_string(), "Current Working Directory");
                    });
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let can_save = !parsed.is_empty() && !editor.command.trim().is_empty();
                if ui.add_enabled(can_save, egui::Button::new(RichText::new("Save Pair").strong())).clicked() {
                    let mut rule = if let Some(ref orig_id) = editor.original_id {
                        app.config.rules.iter().find(|r| &r.id == orig_id).cloned().unwrap_or_else(|| {
                            Rule::new(editor.name.clone(), parsed.clone(), editor.command.clone())
                        })
                    } else {
                        Rule::new(editor.name.clone(), parsed.clone(), editor.command.clone())
                    };

                    rule.name = if editor.name.trim().is_empty() {
                        format!("{} -> {}", parsed.join(", "), editor.command.split_whitespace().next().unwrap_or(""))
                    } else {
                        editor.name.clone()
                    };
                    rule.filetypes = parsed;
                    rule.command = editor.command.clone();
                    rule.run_in_terminal = editor.run_in_terminal;
                    rule.hold_open = editor.hold_open;
                    rule.working_dir = editor.working_dir.clone();

                    if let Some(ref orig_id) = editor.original_id {
                        if let Some(idx) = app.config.rules.iter().position(|r| &r.id == orig_id) {
                            app.config.rules[idx] = rule;
                        }
                    } else {
                        app.config.rules.insert(0, rule);
                    }

                    let _ = app.config.save();
                    app.set_status("Rule saved successfully.");
                } else if ui.button("Cancel").clicked() {
                    // cancelled, do not put back into editing_rule
                } else {
                    // Keep editor open
                    app.rules_state.editing_rule = Some(editor);
                }
            });
        });

        ui.separator();
    }

    // Tester modal / card if active
    if let Some(mut tester) = app.rules_state.testing_rule.take() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading(format!("Test Rule: {}", tester.rule.name));
            ui.label(format!("Handles: {}", tester.rule.filetypes_display()));
            ui.add(
                Label::new(RichText::new(format!("Command: {}", tester.rule.command)).monospace())
                    .truncate(),
            )
            .on_hover_text(&tester.rule.command);

            ui.horizontal(|ui| {
                form_label(ui, "Sample File Path:");
                ui.text_edit_singleline(&mut tester.sample_path);
            });

            let test_path = PathBuf::from(&tester.sample_path);
            let matches = tester.rule.matches(&test_path);

            ui.horizontal(|ui| {
                if matches {
                    ui.label(RichText::new("✓ Matches rule filetypes!").color(c.success));
                } else {
                    ui.label(RichText::new("✗ Does not match rule filetypes").color(c.error));
                }
            });

            let formatted = crate::terminal::format_command(&tester.rule.command, &test_path);
            let mode_str = if tester.rule.run_in_terminal {
                "Terminal Execution"
            } else {
                "Direct Background Execution (No Terminal)"
            };
            ui.label(RichText::new(format!("Mode: {}", mode_str)).small().weak());
            ui.add(
                Label::new(RichText::new(format!("Resolved Command: {}", formatted)).monospace())
                    .truncate(),
            )
            .on_hover_text(&formatted);

            ui.horizontal_wrapped(|ui| {
                let btn_text = if tester.rule.run_in_terminal {
                    "Launch Terminal Test"
                } else {
                    "Launch Direct Command Test"
                };
                if ui.button(btn_text).clicked() {
                    let res = crate::terminal::launch_rule(
                        &tester.rule,
                        &test_path,
                        &app.config.preferred_terminal,
                        &app.config.custom_terminal_template,
                    );
                    tester.test_result = Some(res.map(|_| "Launched command successfully".to_string()));
                }

                if ui.button("Close Tester").clicked() {
                    // tester dropped
                } else {
                    app.rules_state.testing_rule = Some(tester);
                }
            });
        });
        ui.separator();
    }

    // Rules List
    let query = app.rules_state.search_query.to_lowercase();
    let total_rules = app.config.rules.len();

    let mut to_delete = None;
    let mut to_edit = None;
    let mut to_test = None;
    let mut to_move_up = None;
    let mut to_move_down = None;

    for (idx, rule) in app.config.rules.iter_mut().enumerate() {
        let matches_query = query.is_empty()
            || rule.name.to_lowercase().contains(&query)
            || rule.command.to_lowercase().contains(&query)
            || rule.filetypes.iter().any(|ft| ft.to_lowercase().contains(&query));

        if !matches_query {
            continue;
        }

        ui.group(|ui| {
            // Row 1: name + actions (actions reserved first so they never get pushed off-screen)
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑").on_hover_text("Delete rule").clicked() {
                        to_delete = Some(idx);
                    }

                    if ui.button("✏").on_hover_text("Edit rule").clicked() {
                        to_edit = Some(rule.clone());
                    }

                    if ui.button("⚡").on_hover_text("Test with sample file").clicked() {
                        to_test = Some(rule.clone());
                    }

                    if idx + 1 < total_rules && ui.button("⬇").on_hover_text("Move down").clicked() {
                        to_move_down = Some(idx);
                    }

                    if idx > 0 && ui.button("⬆").on_hover_text("Move up").clicked() {
                        to_move_up = Some(idx);
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.checkbox(&mut rule.enabled, "");
                        ui.add(Label::new(RichText::new(&rule.name).strong()).truncate())
                            .on_hover_text(&rule.name);
                    });
                });
            });

            // Row 2: filetypes
            ui.horizontal_wrapped(|ui| {
                if rule.filetypes.is_empty() {
                    ui.label(RichText::new("(no filetypes)").italics().weak());
                }
                for ft in &rule.filetypes {
                    ui.label(
                        RichText::new(format!(".{}", ft))
                            .background_color(c.chip_bg)
                            .color(c.chip_fg)
                            .monospace(),
                    );
                }
            });

            // Row 3: mode badge (right) + command (truncates in the remaining space)
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !rule.run_in_terminal {
                        ui.label(
                            RichText::new("[No Terminal / Direct]")
                                .small()
                                .color(c.warning)
                                .background_color(c.warning_bg),
                        );
                    } else if rule.hold_open {
                        ui.label(RichText::new("[Keep Open]").small().weak());
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(RichText::new("➜").strong().weak());
                        ui.add(Label::new(RichText::new(&rule.command).monospace()).truncate())
                            .on_hover_text(&rule.command);
                    });
                });
            });
        });
        ui.add_space(4.0);
    }

    // Handle mutations outside loop
    if let Some(idx) = to_delete {
        app.config.rules.remove(idx);
        let _ = app.config.save();
        app.set_status("Rule deleted.");
    }

    if let Some(rule) = to_edit {
        app.rules_state.editing_rule = Some(RuleEditor {
            original_id: Some(rule.id),
            name: rule.name,
            filetypes_input: rule.filetypes.join(", "),
            command: rule.command,
            run_in_terminal: rule.run_in_terminal,
            hold_open: rule.hold_open,
            working_dir: rule.working_dir,
        });
    }

    if let Some(rule) = to_test {
        let sample_ext = rule.filetypes.first().cloned().unwrap_or_else(|| "txt".to_string());
        app.rules_state.testing_rule = Some(RuleTester {
            sample_path: format!("/tmp/test_file.{}", sample_ext),
            rule,
            test_result: None,
        });
    }

    if let Some(idx) = to_move_up {
        app.config.rules.swap(idx, idx - 1);
        let _ = app.config.save();
    }

    if let Some(idx) = to_move_down {
        app.config.rules.swap(idx, idx + 1);
        let _ = app.config.save();
    }
}
