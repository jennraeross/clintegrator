pub mod os_view;
pub mod presets_view;
pub mod rules_view;
pub mod terminal_view;
pub mod wizard_view;

use crate::config::Config;
use crate::terminal::{detect_available_terminals, resolve_terminal_from, TerminalInfo};
use crate::theme::{AppTheme, ThemeColors, THEME_POLL_INTERVAL};
use crate::ui::os_view::{render_os_view, OsViewState};
use crate::ui::presets_view::PRESETS;
use crate::ui::rules_view::{render_rules_view, RulesViewState};
use crate::ui::terminal_view::{render_terminal_view, TerminalViewState};
use crate::ui::wizard_view::{render_wizard_view, WizardState};
use eframe::egui::{self, Label, RichText, Ui};
use std::time::{Duration, Instant};

/// Width reserved for the label column in form rows.
const FORM_LABEL_WIDTH: f32 = 140.0;

/// Adds a fixed-width, truncating label for a form row.
pub fn form_label(ui: &mut Ui, text: &str) {
    let height = ui.spacing().interact_size.y;
    ui.add_sized(
        [FORM_LABEL_WIDTH, height],
        Label::new(text).truncate(),
    )
    .on_hover_text(text);
}

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
    /// Cached terminal detection, so repaints never scan the filesystem.
    pub terminals: Vec<TerminalInfo>,
    pub resolved_terminal: (String, String),
    /// Omarchy palette/font state, refreshed at most once per second.
    pub theme: AppTheme,
    /// Snapshot of `theme.colors` for the views (`Copy`, so closures can use it).
    pub colors: ThemeColors,
    theme_applied: bool,
    last_theme_check: Instant,
}

impl AppState {
    pub fn new_manager(config: Config) -> Self {
        let (terminals, resolved_terminal) = Self::detect_terminals(&config);
        let theme = AppTheme::load();
        let colors = theme.colors;
        Self {
            config,
            mode: AppMode::Manager,
            wizard_state: None,
            active_tab: ManagerTab::Rules,
            rules_state: RulesViewState::default(),
            terminal_state: TerminalViewState::default(),
            os_state: OsViewState::default(),
            status_message: None,
            terminals,
            resolved_terminal,
            theme,
            colors,
            theme_applied: false,
            last_theme_check: Instant::now(),
        }
    }

    pub fn new_wizard(config: Config, wizard_state: WizardState) -> Self {
        let (terminals, resolved_terminal) = Self::detect_terminals(&config);
        let theme = AppTheme::load();
        let colors = theme.colors;
        Self {
            config,
            mode: AppMode::Wizard,
            wizard_state: Some(wizard_state),
            active_tab: ManagerTab::Rules,
            rules_state: RulesViewState::default(),
            terminal_state: TerminalViewState::default(),
            os_state: OsViewState::default(),
            status_message: None,
            terminals,
            resolved_terminal,
            theme,
            colors,
            theme_applied: false,
            last_theme_check: Instant::now(),
        }
    }

    fn detect_terminals(config: &Config) -> (Vec<TerminalInfo>, (String, String)) {
        let terminals = detect_available_terminals();
        let resolved = resolve_terminal_from(&terminals, &config.preferred_terminal);
        (terminals, resolved)
    }

    /// Re-scans for installed terminals (only on demand, never per frame).
    pub fn refresh_terminal_cache(&mut self) {
        let (terminals, resolved) = Self::detect_terminals(&self.config);
        self.terminals = terminals;
        self.resolved_terminal = resolved;
    }

    /// Re-resolves the active terminal from the cached list after a preference change.
    pub fn refresh_resolved_terminal(&mut self) {
        self.resolved_terminal =
            resolve_terminal_from(&self.terminals, &self.config.preferred_terminal);
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    /// Applies the theme on the first frame, then polls for Omarchy theme/font
    /// changes at most once per second.
    fn sync_theme(&mut self, ctx: &egui::Context) {
        if !self.theme_applied {
            self.theme.apply(ctx);
            self.colors = self.theme.colors;
            self.theme_applied = true;
            self.last_theme_check = Instant::now();
        } else if self.last_theme_check.elapsed() >= THEME_POLL_INTERVAL {
            self.last_theme_check = Instant::now();
            if self.theme.update() {
                self.theme.apply(ctx);
                self.colors = self.theme.colors;
                ctx.request_repaint();
            }
        }

        ctx.request_repaint_after(THEME_POLL_INTERVAL);
    }
}

impl eframe::App for AppState {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_theme(ui.ctx());

        // If in Wizard mode, render the walk-through wizard directly!
        if self.mode == AppMode::Wizard {
            render_wizard_view(self, ui);
            return;
        }

        let c = self.colors;

        // Fixed top navigation (brand + tabs reflow instead of overlapping)
        egui::Panel::top("app_nav").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading(
                    RichText::new("⚙ Clintegrator")
                        .strong()
                        .color(c.accent),
                );
                ui.label(RichText::new("Terminal File Opener & Dispatcher").weak());
            });

            ui.horizontal_wrapped(|ui| {
                let tabs = [
                    (ManagerTab::Rules, "Rules"),
                    (ManagerTab::Presets, "Presets"),
                    (ManagerTab::Terminal, "Terminal"),
                    (ManagerTab::OsIntegration, "OS Integration"),
                ];
                for (tab, label) in tabs {
                    if ui.selectable_label(self.active_tab == tab, label).clicked() {
                        self.active_tab = tab;
                    }
                }
            });
            ui.add_space(4.0);
        });

        // Status notification (auto expires after 4 seconds)
        if let Some((msg, time)) = &self.status_message {
            let elapsed = time.elapsed();
            if elapsed < Duration::from_secs(4) {
                egui::Panel::bottom("app_status").show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("ℹ {}", msg))
                            .color(c.success)
                            .strong(),
                    );
                });
                ui.ctx()
                    .request_repaint_after(Duration::from_secs(4).saturating_sub(elapsed));
            }
        }

        // Body content based on active tab, always scrollable
        egui::CentralPanel::default().show(ui, |ui| {
            let scroll_id = match self.active_tab {
                ManagerTab::Rules => "scroll_rules",
                ManagerTab::Presets => "scroll_presets",
                ManagerTab::Terminal => "scroll_terminal",
                ManagerTab::OsIntegration => "scroll_os",
            };

            egui::ScrollArea::vertical()
                .id_salt(scroll_id)
                .auto_shrink([false, false])
                .show(ui, |ui| match self.active_tab {
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
                });
        });
    }
}

fn render_presets_tab(app: &mut AppState, ui: &mut Ui) {
    let c = app.colors;
    ui.heading("Curated Rule Presets");
    ui.label(RichText::new("Quickly add pre-configured rule pairs for popular CLI tools and workflows.").weak());
    ui.separator();

    for preset in PRESETS {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            // Name + action on one reflowing row
            ui.horizontal_wrapped(|ui| {
                ui.add(Label::new(RichText::new(preset.name).strong()).truncate())
                    .on_hover_text(preset.name);

                if ui.button(RichText::new("+ Add Rule").strong()).clicked() {
                    app.config.rules.push(preset.to_rule());
                    let _ = app.config.save();
                    app.set_status(format!("Added preset: {}", preset.name));
                }
            });

            ui.add(
                Label::new(RichText::new(preset.description).weak().small())
                    .truncate(),
            )
            .on_hover_text(preset.description);

            ui.horizontal_wrapped(|ui| {
                for ft in preset.filetypes {
                    ui.label(
                        RichText::new(format!(".{}", ft))
                            .background_color(c.chip_bg)
                            .color(c.chip_fg)
                            .small(),
                    );
                }
            });

            ui.add(
                Label::new(RichText::new(preset.command).monospace().small())
                    .truncate(),
            )
            .on_hover_text(preset.command);
        });
        ui.add_space(4.0);
    }
}
