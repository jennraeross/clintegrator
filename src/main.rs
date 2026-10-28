mod config;
mod os_integration;
mod terminal;
mod ui;

use config::Config;
use os_integration::OsIntegration;
use std::env;
use std::path::PathBuf;
use terminal::launch_in_terminal;
use ui::wizard_view::WizardState;
use ui::AppState;

fn print_help() {
    println!(
        r#"Clintegrator - Cross-Platform Terminal File Opener & Dispatcher

USAGE:
    clintegrator [OPTIONS] [FILE]

ARGS:
    <FILE>    File to open with matching terminal rule

OPTIONS:
    -h, --help                 Print help information
    -g, --gui                  Force opening the GUI Rule Manager
    --install-desktop          Install Linux desktop integration (.desktop file)
    --uninstall-desktop        Remove Linux desktop integration
    --status                   Check current status and configuration

BEHAVIOR:
    - When passed a listed filetype, Clintegrator stays in the background and launches
      your preferred terminal running the configured command for that filetype.
    - When passed an unlisted filetype, Clintegrator launches a walk-through wizard
      to create a new value pair for that filetype.
    - When launched without arguments, Clintegrator opens the full GUI Manager.
"#
    );
}

fn print_status(config: &Config) {
    println!("=== Clintegrator Status ===");
    println!("Config file:      {}", Config::config_file_path().display());
    println!("Configured rules: {}", config.rules.len());
    let (term_id, term_exe) = terminal::resolve_terminal(&config.preferred_terminal);
    println!("Active terminal:  {} ({})", term_exe, term_id);
    let os_status = OsIntegration::status();
    println!("OS integration:   {:?}", os_status);
}

fn main() -> eframe::Result {
    let args: Vec<String> = env::args().collect();
    let config = Config::load();

    // Check for CLI flags
    let mut file_arg: Option<PathBuf> = None;
    let mut force_gui = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-g" | "--gui" => {
                force_gui = true;
            }
            "--install-desktop" => match OsIntegration::install_linux_desktop() {
                Ok(path) => {
                    println!("Successfully installed desktop integration to {}", path.display());
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error installing desktop integration: {}", e);
                    std::process::exit(1);
                }
            },
            "--uninstall-desktop" => match OsIntegration::uninstall_linux_desktop() {
                Ok(_) => {
                    println!("Desktop integration removed.");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error removing desktop integration: {}", e);
                    std::process::exit(1);
                }
            },
            "--status" => {
                print_status(&config);
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                if file_arg.is_none() {
                    file_arg = Some(PathBuf::from(arg));
                }
            }
            _ => {}
        }
        i += 1;
    }

    // If a file argument was provided and --gui was not forced:
    if let (Some(target_file), false) = (file_arg, force_gui) {
        // Resolve absolute path if exists
        let resolved_file = if let Ok(abs) = std::fs::canonicalize(&target_file) {
            abs
        } else {
            target_file
        };

        // Check if there is a matching rule
        if let Some(rule) = config.find_matching_rule(&resolved_file) {
            // MATCH FOUND: Stay in background! Launch terminal immediately and exit.
            match launch_in_terminal(
                &config.preferred_terminal,
                &config.custom_terminal_template,
                &rule.command,
                &resolved_file,
                rule.hold_open,
                &rule.working_dir,
            ) {
                Ok(_) => {
                    // Success! Process spawned detached, exit cleanly and instantly.
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error launching terminal for rule '{}': {}", rule.name, e);
                    // Fall through to show GUI wizard so the user can diagnose or adjust
                }
            }
        }

        // NO MATCH FOUND: Walk the user through making a value pair in Wizard mode!
        let wizard_state = WizardState::new(resolved_file);
        let app_state = AppState::new_wizard(config, wizard_state);

        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title("Clintegrator - New Filetype Detected")
                .with_inner_size([720.0, 540.0])
                .with_min_inner_size([500.0, 400.0]),
            ..Default::default()
        };

        return eframe::run_native(
            "Clintegrator",
            options,
            Box::new(|_cc| Ok(Box::new(app_state))),
        );
    }

    // Launch Full Manager GUI
    let app_state = AppState::new_manager(config);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Clintegrator - Terminal File Opener & Dispatcher")
            .with_inner_size([920.0, 680.0])
            .with_min_inner_size([650.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clintegrator",
        options,
        Box::new(|_cc| Ok(Box::new(app_state))),
    )
}
