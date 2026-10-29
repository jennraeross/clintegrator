use crate::config::Rule;

pub struct PresetItem {
    pub name: &'static str,
    pub description: &'static str,
    pub filetypes: &'static [&'static str],
    pub command: &'static str,
    pub run_in_terminal: bool,
    pub hold_open: bool,
}

pub const PRESETS: &[PresetItem] = &[
    PresetItem {
        name: "Neovim - Code & Config",
        description: "Open code and config files in Neovim",
        filetypes: &["rs", "py", "js", "ts", "json", "toml", "yaml", "yml", "c", "cpp", "h", "go", "lua"],
        command: "nvim \"{file}\"",
        run_in_terminal: true,
        hold_open: false,
    },
    PresetItem {
        name: "Neovim - Markdown & Docs",
        description: "Edit documentation and markdown notes in Neovim",
        filetypes: &["md", "markdown", "rst", "txt"],
        command: "nvim \"{file}\"",
        run_in_terminal: true,
        hold_open: false,
    },
    PresetItem {
        name: "Bat - Syntax Highlighted Pager",
        description: "View logs, diffs, and text files with syntax highlighting",
        filetypes: &["log", "diff", "patch", "txt", "conf", "ini"],
        command: "bat --paging=always \"{file}\"",
        run_in_terminal: true,
        hold_open: false,
    },
    PresetItem {
        name: "Helix - Modern Modal Editor",
        description: "Edit code and text using the Helix terminal editor",
        filetypes: &["rs", "toml", "py", "md", "txt", "json", "yaml"],
        command: "hx \"{file}\"",
        run_in_terminal: true,
        hold_open: false,
    },
    PresetItem {
        name: "VisiData - Interactive CSV/TSV Explorer",
        description: "Explore tabular data and spreadsheets in terminal",
        filetypes: &["csv", "tsv", "parquet"],
        command: "vd \"{file}\"",
        run_in_terminal: true,
        hold_open: false,
    },
    PresetItem {
        name: "Glow - Terminal Markdown Renderer",
        description: "Render markdown documents beautifully in terminal",
        filetypes: &["md", "markdown"],
        command: "glow -p \"{file}\"",
        run_in_terminal: true,
        hold_open: false,
    },
    PresetItem {
        name: "Python Runner",
        description: "Execute Python script in a terminal and hold open",
        filetypes: &["py"],
        command: "python3 \"{file}\"",
        run_in_terminal: true,
        hold_open: true,
    },
    PresetItem {
        name: "Bash Runner",
        description: "Execute shell script in a terminal and hold open",
        filetypes: &["sh", "bash"],
        command: "bash \"{file}\"",
        run_in_terminal: true,
        hold_open: true,
    },
    PresetItem {
        name: "MPV Media Player (Direct / No Terminal)",
        description: "Play video and audio files directly in background without opening terminal",
        filetypes: &["mp4", "mkv", "webm", "avi", "mov", "mp3", "flac", "wav", "ogg"],
        command: "mpv \"{file}\"",
        run_in_terminal: false,
        hold_open: false,
    },
    PresetItem {
        name: "Zathura PDF Reader (Direct / No Terminal)",
        description: "Open PDF, PostScript, and e-books directly without opening terminal",
        filetypes: &["pdf", "djvu", "epub", "ps"],
        command: "zathura \"{file}\"",
        run_in_terminal: false,
        hold_open: false,
    },
    PresetItem {
        name: "Terminal Image Viewer (Chafa)",
        description: "View images as high-res terminal graphics or sixel",
        filetypes: &["png", "jpg", "jpeg", "webp", "gif", "svg"],
        command: "chafa \"{file}\"",
        run_in_terminal: true,
        hold_open: true,
    },
    PresetItem {
        name: "Hex Viewer (xxd)",
        description: "Inspect binary and executable files in hex mode",
        filetypes: &["bin", "dat", "so", "dll", "exe", "o"],
        command: "xxd \"{file}\" | less",
        run_in_terminal: true,
        hold_open: false,
    },
];

impl PresetItem {
    pub fn to_rule(&self) -> Rule {
        let fts: Vec<String> = self.filetypes.iter().map(|s| s.to_string()).collect();
        let mut rule = Rule::new(self.name, fts, self.command);
        rule.run_in_terminal = self.run_in_terminal;
        rule.hold_open = self.hold_open;
        rule
    }
}

