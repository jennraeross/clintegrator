pub mod os_view;
pub mod presets_view;
pub mod rules_view;
pub mod terminal_view;
pub mod wizard_view;

use crate::config::Config;
use crate::ui::os_view::{render_os_view, OsViewState};
use crate::ui::presets_view::PRESETS;
use crate::ui::rules_view::{render_rules_view, RulesViewState};
use crate::ui::terminal_view::{render_terminal_view, TerminalViewState};
use crate::ui::wizard_view::{render_wizard_view, WizardState};
use eframe::egui::{self, Color32, RichText, Ui};
use std::time::Instant;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ManagerTab {
    Rules,
    Presets,
    Terminal,
    OsIntegration,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AppMode {
    Manager,
    Wizard,
}

pub struct AppState {
    pub config: Config,
    pub mode: AppMode,
    pub wizard_state: Option<WizardState>,
    pub active_tab: ManagerTab,
    pub rules_state: RulesViewState,
    pub terminal_state: TerminalViewState,
    pub os_state: OsViewState,
    pub status_message: Option<(String, Instant)>,
}

impl AppState {
    pub fn new_manager(config: Config) -> Self {
        Self {
            config,
            mode: AppMode::Manager,
            wizard_state: None,
            active_tab: ManagerTab::Rules,
            rules_state: RulesViewState::default(),
            terminal_state: TerminalViewState::default(),
            os_state: OsViewState::default(),
            status_message: None,
        }
    }

    pub fn new_wizard(config: Config, wizard_state: WizardState) -> Self {
        Self {
            config,
            mode: AppMode::Wizard,
            wizard_state: Some(wizard_state),
            active_tab: ManagerTab::Rules,
            rules_state: RulesViewState::default(),
            terminal_state: TerminalViewState::default(),
            os_state: OsViewState::default(),
            status_message: None,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }
}

impl eframe::App for AppState {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // If in Wizard mode, render the walk-through wizard directly!
        if self.mode == AppMode::Wizard {
            render_wizard_view(self, ui);
            return;
        }

        // Otherwise render standard Manager GUI
        egui::CentralPanel::default().show(ui, |ui| {
            // Top Navigation Bar
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("⚙ Clintegrator")
                        .strong()
                        .color(Color32::from_rgb(120, 200, 255)),
                );
                ui.label(RichText::new("Terminal File Opener & Dispatcher").weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(self.active_tab == ManagerTab::OsIntegration, "OS Integration").clicked() {
                        self.active_tab = ManagerTab::OsIntegration;
                    }
                    if ui.selectable_label(self.active_tab == ManagerTab::Terminal, "Terminal").clicked() {
                        self.active_tab = ManagerTab::Terminal;
                    }
                    if ui.selectable_label(self.active_tab == ManagerTab::Presets, "Presets").clicked() {
                        self.active_tab = ManagerTab::Presets;
                    }
                    if ui.selectable_label(self.active_tab == ManagerTab::Rules, "Rules").clicked() {
                        self.active_tab = ManagerTab::Rules;
                    }
                });
            });

            ui.separator();

            // Status notification bar (auto expires after 4 seconds)
            if let Some((msg, time)) = &self.status_message {
                if time.elapsed().as_secs() < 4 {
                    ui.label(
                        RichText::new(format!("ℹ {}", msg))
                            .color(Color32::from_rgb(120, 220, 140))
                            .strong(),
                    );
                    ui.separator();
                }
            }

            // Body content based on active tab
            match self.active_tab {
                ManagerTab::Rules => {
                    render_rules_view(self, ui);
                }
                ManagerTab::Presets => {
                    render_presets_tab(self, ui);
                }
                ManagerTab::Terminal => {
                    render_terminal_view(self, ui);
                }
                ManagerTab::OsIntegration => {
                    render_os_view(self, ui);
                }
            }
        });
    }
}

fn render_presets_tab(app: &mut AppState, ui: &mut Ui) {
    ui.heading("Curated Rule Presets");
    ui.label(RichText::new("Quickly add pre-configured rule pairs for popular CLI tools and workflows.").weak());
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for preset in PRESETS {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(preset.name).strong());
                        ui.label(RichText::new(preset.description).weak().small());

                        ui.horizontal_wrapped(|ui| {
                            for ft in preset.filetypes {
                                ui.label(
                                    RichText::new(format!(".{}", ft))
                                        .background_color(Color32::from_rgb(30, 45, 60))
                                        .color(Color32::from_rgb(140, 200, 255))
                                        .small(),
                                );
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("+ Add Rule").strong()).clicked() {
                            app.config.rules.push(preset.to_rule());
                            let _ = app.config.save();
                            app.set_status(format!("Added preset: {}", preset.name));
                        }

                        ui.label(RichText::new(preset.command).monospace());
                    });
                });
            });
            ui.add_space(4.0);
        }
    });
}
