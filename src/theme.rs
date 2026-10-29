//! Omarchy theme integration.
//!
//! Reads the active Omarchy theme palette from
//! `~/.local/state/omarchy/current/theme/colors.toml` (XDG state), maps it onto
//! egui's [`egui::Visuals`], and uses the desktop's fontconfig `monospace`
//! family so the app looks like the rest of the desktop.
//!
//! Everything degrades gracefully: without Omarchy (or without the theme file)
//! egui's default look is kept, and any unreadable file simply leaves the last
//! known palette in place.

use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// One theme file is checked per second; no other filesystem work happens in
/// the paint path.
pub const THEME_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

/// The palette from `colors.toml`, with defaults for anything a theme omits.
#[derive(Debug, Clone)]
pub struct Palette {
    pub dark: bool,
    pub background: Color32,
    pub dark_background: Color32,
    pub darker_background: Color32,
    pub lighter_background: Color32,
    pub foreground: Color32,
    pub bright_foreground: Color32,
    pub muted: Color32,
    pub selection: Color32,
    pub accent: Color32,
    pub red: Color32,
    pub orange: Color32,
    pub yellow: Color32,
    pub green: Color32,
    pub blue: Color32,
}

/// Semantic colors used by the views.
///
/// `Copy` so view code can snapshot it and still mutate `AppState` inside
/// closures.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub accent: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub chip_bg: Color32,
    pub chip_fg: Color32,
    pub warning_bg: Color32,
}

impl Palette {
    fn defaults(dark: bool) -> Self {
        let v = if dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        let foreground = v.widgets.noninteractive.fg_stroke.color;
        Self {
            dark,
            background: v.panel_fill,
            dark_background: v.code_bg_color,
            darker_background: v.extreme_bg_color,
            lighter_background: v.widgets.inactive.bg_fill,
            foreground,
            bright_foreground: foreground,
            muted: v.widgets.inactive.bg_stroke.color,
            selection: v.selection.bg_fill,
            accent: v.hyperlink_color,
            red: v.error_fg_color,
            orange: v.warn_fg_color,
            yellow: v.warn_fg_color,
            green: Color32::from_rgb(100, 220, 100),
            blue: v.hyperlink_color,
        }
    }

    fn from_map(map: &BTreeMap<String, String>) -> Self {
        let dark = map
            .get("mode")
            .map(|mode| !mode.eq_ignore_ascii_case("light"))
            .unwrap_or(true);
        let d = Self::defaults(dark);

        let color = |key: &str| map.get(key).and_then(|hex| parse_hex(hex));

        let background = color("background").unwrap_or(d.background);
        let foreground = color("foreground").unwrap_or(d.foreground);

        Self {
            dark,
            background,
            dark_background: color("dark_background")
                .or_else(|| color("darker_background"))
                .unwrap_or(d.dark_background),
            darker_background: color("darker_background").unwrap_or(d.darker_background),
            lighter_background: color("lighter_background").unwrap_or(d.lighter_background),
            foreground,
            bright_foreground: color("bright_foreground").unwrap_or(foreground),
            muted: color("muted").unwrap_or(d.muted),
            selection: color("selection").unwrap_or(d.selection),
            accent: color("accent").unwrap_or(d.accent),
            red: color("red").unwrap_or(d.red),
            orange: color("orange")
                .or_else(|| color("yellow"))
                .unwrap_or(d.orange),
            yellow: color("yellow")
                .or_else(|| color("orange"))
                .unwrap_or(d.yellow),
            green: color("green").unwrap_or(d.green),
            blue: color("blue").unwrap_or(d.blue),
        }
    }

    pub fn colors(&self) -> ThemeColors {
        ThemeColors {
            accent: self.accent,
            success: self.green,
            warning: self.orange,
            error: self.red,
            chip_bg: mix(self.background, self.accent, 0.18),
            chip_fg: self.bright_foreground,
            warning_bg: mix(self.background, self.orange, 0.18),
        }
    }

    /// Plain inverted background used for the code/preview text.
    fn visuals(&self) -> Visuals {
        let mut v = if self.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        v.dark_mode = self.dark;
        v.override_text_color = Some(self.foreground);
        v.panel_fill = self.background;
        v.window_fill = self.background;
        v.extreme_bg_color = self.darker_background;
        v.code_bg_color = self.dark_background;
        v.faint_bg_color = self.lighter_background;
        v.window_stroke = Stroke::new(1.0, with_alpha(self.muted, 0.6));
        v.hyperlink_color = self.blue;
        v.warn_fg_color = self.yellow;
        v.error_fg_color = self.red;
        v.selection.bg_fill = self.selection;
        v.selection.stroke = Stroke::new(1.0, self.accent);

        let widgets = &mut v.widgets;

        widgets.noninteractive.bg_fill = self.background;
        widgets.noninteractive.weak_bg_fill = self.background;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, with_alpha(self.muted, 0.5));
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.foreground);

        widgets.inactive.bg_fill = self.lighter_background;
        widgets.inactive.weak_bg_fill = self.lighter_background;
        widgets.inactive.bg_stroke = Stroke::new(1.0, mix(self.background, self.muted, 0.55));
        widgets.inactive.fg_stroke = Stroke::new(1.0, self.foreground);

        widgets.hovered.bg_fill = mix(self.lighter_background, self.accent, 0.18);
        widgets.hovered.weak_bg_fill = mix(self.lighter_background, self.accent, 0.18);
        widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent);
        widgets.hovered.fg_stroke = Stroke::new(1.0, self.bright_foreground);

        widgets.active.bg_fill = mix(self.lighter_background, self.accent, 0.32);
        widgets.active.weak_bg_fill = mix(self.lighter_background, self.accent, 0.32);
        widgets.active.bg_stroke = Stroke::new(1.0, self.accent);
        widgets.active.fg_stroke = Stroke::new(1.0, self.bright_foreground);

        widgets.open = widgets.active;

        v
    }
}

/// A font file resolved from fontconfig.
#[derive(Debug, Clone)]
pub struct FontSpec {
    pub path: PathBuf,
}

/// The active theme plus the state needed to notice changes.
pub struct AppTheme {
    pub palette: Palette,
    pub colors: ThemeColors,
    pub base_size: Option<f32>,
    pub font: Option<FontSpec>,
    colors_path: Option<PathBuf>,
    shell_path: Option<PathBuf>,
    colors_mtime: Option<SystemTime>,
    shell_mtime: Option<SystemTime>,
    font_signature: Option<SystemTime>,
}

impl AppTheme {
    /// Loads the current Omarchy theme, or egui's defaults when unavailable.
    pub fn load() -> Self {
        let colors_path = find_colors_path();

        let (palette, colors_mtime) = match &colors_path {
            Some(path) => match fs::read_to_string(path) {
                Ok(text) => (Palette::from_map(&parse_pairs(&text)), mtime(path)),
                Err(_) => (Palette::defaults(true), None),
            },
            None => (Palette::defaults(true), None),
        };

        let shell_path = colors_path
            .as_ref()
            .map(|path| path.with_file_name("shell.toml"))
            .filter(|path| path.is_file());

        let base_size = shell_path
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|text| parse_base_size(&text));

        let font = resolve_font();
        let font_signature = current_font_signature(&font);
        let colors = palette.colors();
        let shell_mtime = shell_path.as_ref().and_then(|path| mtime(path));

        Self {
            palette,
            colors,
            base_size,
            font,
            colors_path,
            shell_path,
            colors_mtime,
            shell_mtime,
            font_signature,
        }
    }

    /// Applies the palette, font and type scale to the egui context.
    pub fn apply(&self, ctx: &egui::Context) {
        ctx.set_theme(if self.palette.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        ctx.set_visuals(self.palette.visuals());

        if let Some(font) = &self.font {
            apply_font(ctx, font);
        }

        if let Some(base) = self.base_size {
            apply_text_styles(ctx, base);
        }
    }

    /// Cheap change check. Returns `true` when anything changed and the caller
    /// should re-apply [`Self::apply`].
    pub fn update(&mut self) -> bool {
        let mut changed = false;

        if let Some(path) = &self.colors_path {
            let modified = mtime(path);
            // `None` during the atomic theme swap: keep the previous palette.
            if modified.is_some() && modified != self.colors_mtime {
                if let Ok(text) = fs::read_to_string(path) {
                    self.palette = Palette::from_map(&parse_pairs(&text));
                    self.colors = self.palette.colors();
                    self.colors_mtime = modified;
                    changed = true;
                }
            }
        }

        if let Some(path) = &self.shell_path {
            let modified = mtime(path);
            if modified != self.shell_mtime {
                if let Ok(text) = fs::read_to_string(path) {
                    self.base_size = parse_base_size(&text);
                    self.shell_mtime = modified;
                    changed = true;
                }
            }
        }

        let signature = current_font_signature(&self.font);
        if signature != self.font_signature {
            self.font = resolve_font();
            self.font_signature = current_font_signature(&self.font);
            changed = true;
        }

        changed
    }
}

/// Resolves the colors.toml path: explicit override, XDG state, then legacy config.
pub fn find_colors_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CLINTEGRATOR_THEME_DIR") {
        if !dir.trim().is_empty() {
            let path = PathBuf::from(dir).join("colors.toml");
            if path.is_file() {
                return Some(path);
            }
        }
    }

    if let Some(state) = dirs::state_dir() {
        let path = state.join("omarchy/current/theme/colors.toml");
        if path.is_file() {
            return Some(path);
        }
    }

    dirs::config_dir()
        .map(|config| config.join("omarchy/current/theme/colors.toml"))
        .filter(|path| path.is_file())
}

/// Parses the flat `key = value` subset of TOML Omarchy uses, including
/// `[section]` prefixes (`font.base-size`).
fn parse_pairs(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut section = String::new();

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        if key.is_empty() {
            continue;
        }

        let value = value.trim().trim_matches('"').trim_matches('\'');
        let full_key = if section.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", section, key)
        };
        map.insert(full_key, value.to_string());
    }

    map
}

fn parse_base_size(text: &str) -> Option<f32> {
    parse_pairs(text)
        .get("font.base-size")
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|size| *size > 0.0)
}

/// Parses `#rgb`, `#rrggbb`, and `#rrggbbaa`.
fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    let digits: Vec<u8> = hex
        .chars()
        .map(|c| c.to_digit(16).map(|d| d as u8))
        .collect::<Option<_>>()?;

    match digits.len() {
        3 => Some(Color32::from_rgb(
            digits[0] * 17,
            digits[1] * 17,
            digits[2] * 17,
        )),
        6 => Some(Color32::from_rgb(
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
        )),
        8 => Some(Color32::from_rgba_unmultiplied(
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
            digits[6] * 16 + digits[7],
        )),
        _ => None,
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |a: u8, b: u8| (a as f32 * (1.0 - t) + b as f32 * t).round() as u8;
    Color32::from_rgb(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
    )
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

/// Resolves Omarchy's canonical UI font (`fc-match monospace`, the same lookup
/// the Omarchy shell uses).
pub fn resolve_font() -> Option<FontSpec> {
    if let Ok(path) = std::env::var("CLINTEGRATOR_FONT_FILE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(FontSpec { path });
        }
    }

    let output = std::process::Command::new("fc-match")
        .args(["-f", "%{file}\n", "monospace"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();

    if path.is_empty() {
        return None;
    }

    let path = PathBuf::from(path);
    path.is_file().then_some(FontSpec { path })
}

fn current_font_signature(font: &Option<FontSpec>) -> Option<SystemTime> {
    let mut signature = None;

    let mut paths = Vec::new();
    if let Some(config) = dirs::config_dir() {
        paths.push(config.join("fontconfig/fonts.conf"));
        paths.push(config.join("fontconfig/conf.d"));
    }
    paths.push(PathBuf::from("/etc/fonts/fonts.conf"));
    paths.push(PathBuf::from("/etc/fonts/conf.d"));

    for path in paths {
        if let Some(modified) = mtime(&path) {
            signature = signature.max(Some(modified));
        }
    }

    if let Some(font) = font {
        if let Some(modified) = mtime(&font.path) {
            signature = signature.max(Some(modified));
        }
    }

    signature
}

fn apply_font(ctx: &egui::Context, font: &FontSpec) {
    let Ok(bytes) = fs::read(&font.path) else {
        return;
    };

    let key = "omarchy-ui".to_owned();
    let mut definitions = FontDefinitions::default();
    definitions
        .font_data
        .insert(key.clone(), Arc::new(FontData::from_owned(bytes)));

    // Keep egui's bundled fonts (including emoji) as fallbacks.
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        definitions
            .families
            .entry(family)
            .or_default()
            .insert(0, key.clone());
    }

    ctx.set_fonts(definitions);
}

fn apply_text_styles(ctx: &egui::Context, base: f32) {
    let small = (base - 2.0).max(8.0);
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(base + 6.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(base, FontFamily::Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(base, FontFamily::Monospace),
            ),
            (
                TextStyle::Button,
                FontId::new(base, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(small, FontFamily::Proportional),
            ),
        ]
        .into();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVERFOREST: &str = r##"
mode = "dark"

accent = "#7fbbb3"
selection = "#3d484d"
muted = "#475258"

background = "#2d353b"
dark_background = "#21272c"
darker_background = "#181d20"
lighter_background = "#343f44"

foreground = "#d3c6aa"
dark_foreground = "#4f585e"
light_foreground = "#9da9a0"
bright_foreground = "#d3c6aa"

red = "#e67e80"
yellow = "#dbbc7f"
orange = "#e09d7f"
green = "#a7c080"
cyan = "#83c092"
blue = "#7fbbb3"
magenta = "#d699b6"
brown = "#704e3f"
"##;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("#fff"), Some(Color32::from_rgb(255, 255, 255)));
        assert_eq!(
            parse_hex("#2d353b"),
            Some(Color32::from_rgb(0x2d, 0x35, 0x3b))
        );
        assert_eq!(
            parse_hex("#7fbbb380"),
            Some(Color32::from_rgba_unmultiplied(0x7f, 0xbb, 0xb3, 0x80))
        );
        assert_eq!(parse_hex("nope"), None);
        assert_eq!(parse_hex("#12345"), None);
    }

    #[test]
    fn test_parse_sections_and_quotes() {
        let map = parse_pairs("[font]\nbase-size = 12\n[controls]\nnormal-fill-alpha = 0.04");
        assert_eq!(map.get("font.base-size").map(String::as_str), Some("12"));
        assert_eq!(
            map.get("controls.normal-fill-alpha").map(String::as_str),
            Some("0.04")
        );

        let map = parse_pairs("mode = \"light\"\n# comment\naccent = '#abcdef'");
        assert_eq!(map.get("mode").map(String::as_str), Some("light"));
        assert_eq!(map.get("accent").map(String::as_str), Some("#abcdef"));
    }

    #[test]
    fn test_palette_from_theme() {
        let palette = Palette::from_map(&parse_pairs(EVERFOREST));
        assert!(palette.dark);
        assert_eq!(palette.accent, Color32::from_rgb(0x7f, 0xbb, 0xb3));
        assert_eq!(palette.background, Color32::from_rgb(0x2d, 0x35, 0x3b));
        assert_eq!(palette.foreground, Color32::from_rgb(0xd3, 0xc6, 0xaa));
        assert_eq!(palette.green, Color32::from_rgb(0xa7, 0xc0, 0x80));
    }

    #[test]
    fn test_light_mode_and_fallbacks() {
        let palette = Palette::from_map(&parse_pairs("mode = \"light\"\naccent = \"#123456\""));
        assert!(!palette.dark);
        assert_eq!(palette.accent, Color32::from_rgb(0x12, 0x34, 0x56));
        // Missing orange falls back to yellow.
        assert_eq!(palette.orange, palette.yellow);
        // Missing bright_foreground falls back to foreground.
        assert_eq!(palette.bright_foreground, palette.foreground);
        assert!(!palette.visuals().dark_mode);
    }

    #[test]
    fn test_visuals_use_theme_colors() {
        let palette = Palette::from_map(&parse_pairs(EVERFOREST));
        let visuals = palette.visuals();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, palette.background);
        assert_eq!(visuals.extreme_bg_color, palette.darker_background);
        assert_eq!(visuals.selection.bg_fill, palette.selection);
        assert_eq!(visuals.widgets.hovered.bg_stroke.color, palette.accent);
        assert_eq!(visuals.override_text_color, Some(palette.foreground));
    }

    #[test]
    fn test_theme_colors_are_derived() {
        let palette = Palette::from_map(&parse_pairs(EVERFOREST));
        let colors = palette.colors();
        assert_eq!(colors.accent, palette.accent);
        assert_eq!(colors.success, palette.green);
        assert_eq!(colors.warning, palette.orange);
        assert_eq!(colors.error, palette.red);
        assert_ne!(colors.chip_bg, palette.background);
        assert_ne!(colors.chip_bg, palette.accent);
    }

    #[test]
    fn test_base_size() {
        assert_eq!(parse_base_size("[font]\nbase-size = 12"), Some(12.0));
        assert_eq!(parse_base_size("[font]\nbase-size = 0"), None);
        assert_eq!(parse_base_size("mode = \"dark\""), None);
    }

    #[test]
    fn test_mix_endpoints() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(255, 255, 255);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
        assert_eq!(mix(a, b, 0.5), Color32::from_rgb(128, 128, 128));
    }
}
