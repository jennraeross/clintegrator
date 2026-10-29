# Clintegrator 🚀

**Cross-Platform GUI & CLI Terminal File Opener & Dispatcher written in Rust.**

Clintegrator bridges graphical desktop environments (file managers, desktop search, "Open With" dialogs) to your favorite command-line tools (Neovim, Helix, Bat, VisiData, Python, Bash, Glow, etc.).

---

## ✨ Key Features

- **Filetype-to-Command Pairs**:
  - Map file extensions (e.g. `rs`, `toml`, `md`, `py`, `csv`, `log`) or full filenames (e.g. `Dockerfile`, `Makefile`) to any terminal command.
  - Command placeholders support `{file}`, `%f`, or automatic quoting.
  - **Terminal or Direct Execution**: Choose whether to run the command inside a terminal emulator or run directly in the background without opening a terminal window (ideal for GUI applications, media players like MPV, or silent automation scripts).
  - Per-rule options: "Keep terminal open after exit", working directory selection, and custom terminal emulator overrides.

- **Background Dispatcher Mode**:
  - When opening a **listed filetype** (via CLI or OS file manager), Clintegrator stays completely in the background: zero GUI overhead, instantaneous launch of your terminal emulator running the command, and immediate clean detachment.

- **Interactive New Rule Wizard**:
  - When opening an **unlisted filetype**, Clintegrator opens a dedicated Walk-Through Wizard that prompts you to create a rule for that filetype on the fly.
  - Includes instant presets (`nvim`, `bat`, `hx`, `less`, `python3`, `bash`), live preview of the generated command, and 1-click **"Save Rule & Open File"** or **"Open Once"**.

- **Full GUI Rule Manager**:
  - When launched without arguments (or with `-g` / `--gui`), Clintegrator opens a rich management interface:
    - **Rules Tab**: Add, edit, test, delete, reorder, or toggle enable/disable for any rule.
    - **Presets Tab**: 1-click additions for popular CLI workflows (Code, Markdown, Tabular Data, Images, Hex inspection).
    - **Terminal Tab**: Auto-detects installed terminals (`xdg-terminal-exec`, `foot`, `alacritty`, `kitty`, `wezterm`, `ghostty`, `gnome-terminal`, `konsole`, `Windows Terminal`, `Terminal.app`), lets you set preferred terminal or custom command template, and includes a live test button.
    - **OS Integration Tab**: View registration status, install or remove FreeDesktop `.desktop` integration, export Windows Registry `.reg` files, and view macOS bundle configurations.

- **Seen by Operating System for ALL Filetypes**:
  - **Linux / FreeDesktop**: Installs `~/.local/share/applications/clintegrator.desktop` declaring `MimeType=application/octet-stream;text/plain;*/*;all/all;all/allfiles;`. Desktop environments (GNOME, KDE, XFCE, Sway) and file managers (Nautilus, Dolphin, Thunar, PCManFM) will offer Clintegrator in the "Open With" list and as a default handler for any file.
  - **Windows**: Exports a `.reg` file registering Clintegrator for all files (`*`) in Windows Explorer context menus and `SupportedTypes: *`.
  - **macOS**: Provides Info.plist declaring `CFBundleDocumentTypes` for `public.item`, `public.data`, and `public.content`.

---

## 🎨 Omarchy Integration

On [Omarchy](https://omarchy.org/), Clintegrator automatically follows the active desktop theme — no configuration needed.

- **Colors**: Reads the active palette from `~/.local/state/omarchy/current/theme/colors.toml` (XDG state) and maps it onto the whole UI, including success/warning/error accents, chips, selections, and hover states. Light themes (`catppuccin-latte`, `flexoki-light`, `white`, …) are detected via the palette's `mode` key.
- **Font**: Uses the desktop's fontconfig `monospace` family (the same lookup the Omarchy shell uses, e.g. `fc-match monospace` → Go Mono) and matches the shell's `[font] base-size` from `shell.toml`. egui's bundled emoji fonts remain as fallbacks, so icons like ⚙ 🚀 🗑 still render.
- **Live reload**: Theme and font files are checked once per second, so switching with `omarchy theme set <name>` or `omarchy font set <name>` updates an open window within a second — no restart required.

Outside Omarchy (or when the theme file is unavailable), egui's default appearance is used. Two environment variables are available for overrides and testing:

```bash
# Use a specific theme directory (must contain colors.toml, optionally shell.toml)
CLINTEGRATOR_THEME_DIR=/path/to/theme clintegrator --gui

# Force a specific font file instead of the fontconfig monospace font
CLINTEGRATOR_FONT_FILE=/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf clintegrator --gui
```

---

## 🛠 Installation & Build

### Install from crates.io
```bash
cargo install clintegrator
```
This builds an optimized release binary and installs it to `~/.cargo/bin/clintegrator`.

### Prerequisites
- Rust & Cargo (1.85+, required by the Rust 2024 edition)
- Standard X11/Wayland libraries on Linux (e.g., `libxkbcommon`, `wayland-client`, `mesa`)

### Build
```bash
# Debug build
cargo build

# Optimized release build
cargo build --release
```

Binary is produced at `target/release/clintegrator`.

---

## 💻 CLI Usage

```bash
# Open a file (opens in background if listed, or opens Wizard if unlisted)
clintegrator /path/to/notes.md

# Open an unlisted file to trigger the New Rule Wizard
clintegrator /path/to/unknown_document.xyz

# Open the full GUI Rule Manager
clintegrator
# or
clintegrator --gui

# Install Linux desktop integration (.desktop file)
clintegrator --install-desktop

# Remove Linux desktop integration
clintegrator --uninstall-desktop

# Display configuration and active terminal status
clintegrator --status

# Show help
clintegrator --help
```

---

## ⚙ Configuration

Configuration is stored in standard OS user config directories:
- **Linux**: `~/.config/clintegrator/config.json`
- **macOS**: `~/Library/Application Support/clintegrator/config.json`
- **Windows**: `%APPDATA%\clintegrator\config.json`

### Example `config.json`
```json
{
  "rules": [
    {
      "id": "c1f7b093-...",
      "name": "Code & Text Files",
      "filetypes": ["rs", "py", "js", "ts", "json", "toml", "yaml"],
      "command": "nvim \"{file}\"",
      "terminal": null,
      "hold_open": false,
      "working_dir": "file_dir",
      "enabled": true
    },
    {
      "id": "e4a29c11-...",
      "name": "CSV & Tabular Data",
      "filetypes": ["csv", "tsv"],
      "command": "vd \"{file}\"",
      "terminal": null,
      "run_in_terminal": true,
      "hold_open": false,
      "working_dir": "file_dir",
      "enabled": true
    },
    {
      "id": "f5b38d22-...",
      "name": "MPV Media Player (Direct / No Terminal)",
      "filetypes": ["mp4", "mkv", "webm", "mp3", "wav"],
      "command": "mpv \"{file}\"",
      "terminal": null,
      "run_in_terminal": false,
      "hold_open": false,
      "working_dir": "file_dir",
      "enabled": true
    }
  ],
  "preferred_terminal": "auto",
  "custom_terminal_template": "{terminal} -e {command}",
  "default_hold_open": false
}
```

---

## 🧪 Testing

Run automated unit and integration tests:
```bash
cargo test
```
All tests verify rule matching, case-insensitivity, wildcard matching, placeholder substitution, and OS registration generators. Theme tests cover hex parsing, palette fallbacks, dark/light visuals, and derived semantic colors.

