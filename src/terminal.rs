use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq)]
pub struct TerminalInfo {
    pub id: String,
    pub name: String,
    pub executable: String,
    pub is_available: bool,
}

/// Detects terminal emulators available on the system
pub fn detect_available_terminals() -> Vec<TerminalInfo> {
    let mut list = Vec::new();

    // Check $TERMINAL environment variable
    if let Ok(env_term) = std::env::var("TERMINAL") {
        if !env_term.trim().is_empty() {
            let available = is_in_path(&env_term);
            list.push(TerminalInfo {
                id: "env_terminal".to_string(),
                name: format!("$TERMINAL ({})", env_term),
                executable: env_term,
                is_available: available,
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            ("xdg-terminal-exec", "xdg-terminal-exec (System Default)", "xdg-terminal-exec"),
            ("foot", "Foot Terminal", "foot"),
            ("alacritty", "Alacritty", "alacritty"),
            ("kitty", "Kitty", "kitty"),
            ("wezterm", "WezTerm", "wezterm"),
            ("ghostty", "Ghostty", "ghostty"),
            ("gnome-terminal", "GNOME Terminal", "gnome-terminal"),
            ("konsole", "Konsole", "konsole"),
            ("xfce4-terminal", "XFCE Terminal", "xfce4-terminal"),
            ("x-terminal-emulator", "x-terminal-emulator", "x-terminal-emulator"),
            ("xterm", "XTerm", "xterm"),
        ];

        for (id, name, exe) in candidates {
            list.push(TerminalInfo {
                id: id.to_string(),
                name: name.to_string(),
                executable: exe.to_string(),
                is_available: is_in_path(exe),
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            ("terminal_app", "Apple Terminal.app", "Terminal"),
            ("iterm", "iTerm2", "iTerm"),
            ("alacritty", "Alacritty", "alacritty"),
            ("kitty", "Kitty", "kitty"),
            ("wezterm", "WezTerm", "wezterm"),
        ];

        for (id, name, exe) in candidates {
            list.push(TerminalInfo {
                id: id.to_string(),
                name: name.to_string(),
                executable: exe.to_string(),
                is_available: is_in_path(exe) || Path::new(&format!("/Applications/{}.app", exe)).exists(),
            });
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            ("wt", "Windows Terminal", "wt.exe"),
            ("powershell", "PowerShell", "powershell.exe"),
            ("cmd", "Command Prompt", "cmd.exe"),
        ];

        for (id, name, exe) in candidates {
            list.push(TerminalInfo {
                id: id.to_string(),
                name: name.to_string(),
                executable: exe.to_string(),
                is_available: is_in_path(exe),
            });
        }
    }

    list
}

/// Checks if an executable is found in PATH or as an absolute path
pub fn is_in_path(exe: &str) -> bool {
    let p = Path::new(exe);
    if p.is_absolute() {
        return p.is_file();
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let full = dir.join(exe);
            if full.is_file() {
                return true;
            }
            #[cfg(target_os = "windows")]
            {
                if dir.join(format!("{}.exe", exe)).is_file() {
                    return true;
                }
            }
        }
    }
    false
}

/// Resolves the terminal executable to use
pub fn resolve_terminal(preferred: &str) -> (String, String) {
    let available = detect_available_terminals();

    if preferred != "auto" && !preferred.is_empty() {
        if let Some(t) = available.iter().find(|t| t.id == preferred || t.executable == preferred) {
            if t.is_available {
                return (t.id.clone(), t.executable.clone());
            }
        }
    }

    // Auto-detect first available terminal
    for t in &available {
        if t.is_available {
            return (t.id.clone(), t.executable.clone());
        }
    }

    // Default fallback
    #[cfg(target_os = "windows")]
    {
        ("cmd".to_string(), "cmd.exe".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        ("terminal_app".to_string(), "Terminal".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        ("sh".to_string(), "sh".to_string())
    }
}

/// Formats the command string by substituting {file} and %f
pub fn format_command(command_template: &str, file_path: &Path) -> String {
    let path_str = file_path.to_string_lossy();
    let quoted_path = format!("\"{}\"", path_str);

    let mut result = command_template.trim().to_string();
    if result.contains("{file}") {
        result = result.replace("{file}", &path_str);
    } else if result.contains("%f") {
        result = result.replace("%f", &path_str);
    } else if result.contains("{}") {
        result = result.replace("{}", &path_str);
    } else {
        // Append quoted file path
        result = format!("{} {}", result, quoted_path);
    }

    result
}

/// Builds and spawns the terminal command detached in background
pub fn launch_in_terminal(
    preferred_terminal: &str,
    custom_template: &str,
    command_str: &str,
    file_path: &Path,
    hold_open: bool,
    working_dir: &str,
) -> Result<(), String> {
    let (term_id, term_exe) = resolve_terminal(preferred_terminal);
    let formatted_cmd = format_command(command_str, file_path);

    // Determine working directory
    let cwd: PathBuf = if working_dir == "file_dir" {
        file_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    let mut cmd = build_command(
        &term_id,
        &term_exe,
        custom_template,
        &formatted_cmd,
        hold_open,
    )?;

    cmd.current_dir(cwd);

    // Detach and isolate process
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    #[cfg(windows)]
    {
        // 0x00000200 = CREATE_NEW_PROCESS_GROUP
        // 0x00000008 = DETACHED_PROCESS
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }

    cmd.spawn().map_err(|e| format!("Failed to spawn terminal '{}': {}", term_exe, e))?;

    Ok(())
}

/// Helper that builds the `Command` struct based on terminal emulator peculiarities
fn build_command(
    term_id: &str,
    term_exe: &str,
    custom_template: &str,
    inner_cmd: &str,
    hold_open: bool,
) -> Result<Command, String> {
    // If hold_open is enabled, wrap the command with a prompt or pause
    let final_inner_cmd = if hold_open {
        #[cfg(target_os = "windows")]
        {
            format!("{} & pause", inner_cmd)
        }
        #[cfg(not(target_os = "windows"))]
        {
            format!("{}; echo; read -p 'Press Enter to exit...' -n 1 -r", inner_cmd)
        }
    } else {
        inner_cmd.to_string()
    };

    if term_id == "custom" {
        // e.g. "{terminal} -e {command}"
        let replaced = custom_template
            .replace("{terminal}", term_exe)
            .replace("{command}", &final_inner_cmd);
        let parts: Vec<&str> = replaced.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Custom terminal template is empty".to_string());
        }
        let mut cmd = Command::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }
        return Ok(cmd);
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = Command::new(term_exe);
        match term_id {
            "xdg-terminal-exec" => {
                cmd.arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "foot" => {
                cmd.arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "alacritty" => {
                cmd.arg("-e").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "kitty" => {
                cmd.arg("--").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "wezterm" => {
                cmd.arg("start").arg("--").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "ghostty" => {
                cmd.arg("-e").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "gnome-terminal" => {
                cmd.arg("--").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "konsole" => {
                cmd.arg("-e").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "xfce4-terminal" => {
                cmd.arg("-x").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            "x-terminal-emulator" | "xterm" => {
                cmd.arg("-e").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
            _ => {
                // Generic fallback
                cmd.arg("-e").arg("sh").arg("-c").arg(&final_inner_cmd);
            }
        }
        Ok(cmd)
    }

    #[cfg(target_os = "macos")]
    {
        match term_id {
            "terminal_app" => {
                let mut cmd = Command::new("osascript");
                let script = format!(
                    "tell application \"Terminal\" to do script \"{}\"",
                    final_inner_cmd.replace('"', "\\\"")
                );
                cmd.arg("-e").arg(script);
                Ok(cmd)
            }
            "iterm" => {
                let mut cmd = Command::new("osascript");
                let script = format!(
                    "tell application \"iTerm\" to create window with default profile command \"{}\"",
                    final_inner_cmd.replace('"', "\\\"")
                );
                cmd.arg("-e").arg(script);
                Ok(cmd)
            }
            _ => {
                let mut cmd = Command::new(term_exe);
                cmd.arg("-e").arg("sh").arg("-c").arg(&final_inner_cmd);
                Ok(cmd)
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        match term_id {
            "wt" => {
                let mut cmd = Command::new("wt.exe");
                cmd.arg("cmd.exe").arg("/c").arg(&final_inner_cmd);
                Ok(cmd)
            }
            "powershell" => {
                let mut cmd = Command::new("powershell.exe");
                cmd.arg("-NoExit").arg("-Command").arg(&final_inner_cmd);
                Ok(cmd)
            }
            _ => {
                let mut cmd = Command::new("cmd.exe");
                cmd.arg("/c").arg(&final_inner_cmd);
                Ok(cmd)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_command_file_placeholder() {
        let path = Path::new("/workspace/src/main.rs");
        let formatted = format_command("nvim \"{file}\"", path);
        assert_eq!(formatted, "nvim \"/workspace/src/main.rs\"");
    }

    #[test]
    fn test_format_command_percent_f_placeholder() {
        let path = Path::new("/workspace/readme.md");
        let formatted = format_command("bat %f", path);
        assert_eq!(formatted, "bat /workspace/readme.md");
    }

    #[test]
    fn test_format_command_auto_append() {
        let path = Path::new("/workspace/data.csv");
        let formatted = format_command("vd", path);
        assert_eq!(formatted, "vd \"/workspace/data.csv\"");
    }
}

