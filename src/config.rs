use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub id: String,
    pub name: String,
    /// List of file extensions (e.g. "md", "txt") or full filenames (e.g. "Makefile", "Dockerfile")
    pub filetypes: Vec<String>,
    /// Terminal command template with {file} or %f placeholder
    pub command: String,
    /// Specific terminal override (None = global default)
    #[serde(default)]
    pub terminal: Option<String>,
    /// Keep terminal window open after command exits
    #[serde(default)]
    pub hold_open: bool,
    /// Working directory: "file_dir" or "current"
    #[serde(default = "default_working_dir")]
    pub working_dir: String,
    /// Enabled / disabled toggle
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Whether to run command in terminal (true) or directly in background without a terminal (false)
    #[serde(default = "default_true")]
    pub run_in_terminal: bool,
}

fn default_working_dir() -> String {
    "file_dir".to_string()
}

fn default_true() -> bool {
    true
}

impl Rule {
    pub fn new(name: impl Into<String>, filetypes: Vec<String>, command: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            filetypes: normalize_filetypes(filetypes),
            command: command.into(),
            terminal: None,
            hold_open: false,
            working_dir: default_working_dir(),
            enabled: true,
            run_in_terminal: true,
        }
    }

    /// Formats filetypes for display (comma separated)
    pub fn filetypes_display(&self) -> String {
        self.filetypes.join(", ")
    }

    /// Tests if this rule matches a given path
    pub fn matches(&self, path: &Path) -> bool {
        if !self.enabled {
            return false;
        }

        // 1. Check full filename
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            let lower_name = file_name.to_lowercase();
            for ft in &self.filetypes {
                if ft == "*" || ft.to_lowercase() == lower_name {
                    return true;
                }
            }
        }

        // 2. Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let lower_ext = ext.to_lowercase();
            for ft in &self.filetypes {
                let clean_ft = ft.trim_start_matches('.').to_lowercase();
                if clean_ft == lower_ext || clean_ft == "*" {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rules: Vec<Rule>,
    /// "auto", "xdg-terminal-exec", "foot", "alacritty", "kitty", "custom", etc.
    #[serde(default = "default_terminal")]
    pub preferred_terminal: String,
    /// Custom template when preferred_terminal == "custom": e.g. "{terminal} -e {command}"
    #[serde(default = "default_custom_template")]
    pub custom_terminal_template: String,
    /// Default hold open behavior for newly created rules
    #[serde(default)]
    pub default_hold_open: bool,
}

fn default_terminal() -> String {
    "auto".to_string()
}

fn default_custom_template() -> String {
    "{terminal} -e {command}".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rules: vec![
                Rule::new(
                    "Code & Text Files",
                    vec![
                        "rs".into(), "py".into(), "js".into(), "ts".into(), "c".into(), "cpp".into(),
                        "h".into(), "go".into(), "toml".into(), "json".into(), "yaml".into(), "yml".into(),
                    ],
                    "nvim \"{file}\"",
                ),
                Rule::new(
                    "Markdown & Documentation",
                    vec!["md".into(), "markdown".into(), "rst".into()],
                    "nvim \"{file}\"",
                ),
                Rule::new(
                    "Logs & Text View",
                    vec!["txt".into(), "log".into(), "diff".into(), "patch".into()],
                    "bat --paging=always \"{file}\"",
                ),
                Rule::new(
                    "CSV & Tabular Data",
                    vec!["csv".into(), "tsv".into()],
                    "vd \"{file}\"",
                ),
                Rule::new(
                    "Shell Scripts",
                    vec!["sh".into(), "bash".into(), "zsh".into()],
                    "bash \"{file}\"",
                ),
            ],
            preferred_terminal: default_terminal(),
            custom_terminal_template: default_custom_template(),
            default_hold_open: false,
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clintegrator")
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<Config>(&content) {
                    return config;
                }
            }
        }

        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let path = Self::config_file_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Finds the first matching rule for a given file path
    pub fn find_matching_rule<'a>(&'a self, path: &Path) -> Option<&'a Rule> {
        self.rules.iter().find(|r| r.matches(path))
    }
}

/// Normalizes and cleans a list of filetypes: trims whitespace, removes leading dots, removes empty strings
pub fn normalize_filetypes(types: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for t in types {
        let cleaned = t.trim().trim_start_matches('.').to_string();
        if !cleaned.is_empty() && !result.iter().any(|existing: &String| existing.eq_ignore_ascii_case(&cleaned)) {
            result.push(cleaned);
        }
    }
    result
}

/// Parses comma or space-separated filetypes from text input
pub fn parse_filetypes_input(input: &str) -> Vec<String> {
    let raw: Vec<String> = input
        .split([',', ';', ' '])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    normalize_filetypes(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_filetypes() {
        let input = vec![
            "  .rs ".to_string(),
            "TOML".to_string(),
            ".toml".to_string(),
            "".to_string(),
            "md".to_string(),
        ];
        let normalized = normalize_filetypes(input);
        assert_eq!(normalized, vec!["rs", "TOML", "md"]);
    }

    #[test]
    fn test_parse_filetypes_input() {
        let input = ".rs, toml; md  json .YAML";
        let parsed = parse_filetypes_input(input);
        assert_eq!(parsed, vec!["rs", "toml", "md", "json", "YAML"]);
    }

    #[test]
    fn test_rule_matching_extension() {
        let rule = Rule::new("Docs", vec!["md".into(), "txt".into()], "nvim {file}");
        assert!(rule.matches(Path::new("readme.md")));
        assert!(rule.matches(Path::new("/path/to/notes.TXT")));
        assert!(!rule.matches(Path::new("script.py")));
    }

    #[test]
    fn test_rule_matching_filename() {
        let rule = Rule::new("Docker", vec!["dockerfile".into()], "nvim {file}");
        assert!(rule.matches(Path::new("Dockerfile")));
        assert!(rule.matches(Path::new("/nested/dir/dockerfile")));
        assert!(!rule.matches(Path::new("Cargo.toml")));
    }

    #[test]
    fn test_rule_disabled() {
        let mut rule = Rule::new("Disabled", vec!["md".into()], "nvim {file}");
        rule.enabled = false;
        assert!(!rule.matches(Path::new("readme.md")));
    }

    #[test]
    fn test_wildcard_matching() {
        let rule = Rule::new("Fallback", vec!["*".into()], "less {file}");
        assert!(rule.matches(Path::new("any_unknown_file.xyz")));
    }

    #[test]
    fn test_config_first_match_priority() {
        let config = Config {
            rules: vec![
                Rule::new("Specific MD", vec!["md".into()], "glow {file}"),
                Rule::new("Generic Text", vec!["md".into(), "txt".into()], "bat {file}"),
            ],
            ..Default::default()
        };
        let matched = config.find_matching_rule(Path::new("test.md"));
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "Specific MD");
    }

    #[test]
    fn test_rule_run_in_terminal_default() {
        let json = r#"{
            "id": "1",
            "name": "Test",
            "filetypes": ["txt"],
            "command": "cat {file}"
        }"#;
        let rule: Rule = serde_json::from_str(json).unwrap();
        assert!(rule.run_in_terminal);

        let json_direct = r#"{
            "id": "2",
            "name": "Direct Test",
            "filetypes": ["mp4"],
            "command": "mpv {file}",
            "run_in_terminal": false
        }"#;
        let direct_rule: Rule = serde_json::from_str(json_direct).unwrap();
        assert!(!direct_rule.run_in_terminal);
    }
}

