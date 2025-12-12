/*! Theme system for the markdown viewer
 *
 * This module provides theme support with Light and Dark variants,
 * loading theme definitions from JSON files.
 */

use anyhow::Result;
use gpui::Rgba;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use tracing::{error, info, warn};

/// Global theme registry
static THEME_REGISTRY: OnceLock<ThemeRegistry> = OnceLock::new();

/// Initialize the theme registry
pub fn init(themes_dir: impl AsRef<Path>) -> Result<()> {
    let registry = ThemeRegistry::load_from_dir(themes_dir)?;
    THEME_REGISTRY
        .set(registry)
        .map_err(|_| anyhow::anyhow!("Theme registry already initialized"))
}

/// Get the global theme registry
pub fn registry() -> &'static ThemeRegistry {
    THEME_REGISTRY.get_or_init(|| {
        warn!("Theme registry not initialized, using default empty registry");
        ThemeRegistry::default()
    })
}

/// Theme variant (Light or Dark) - Deprecated enum usage in config, but useful for logic
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ThemeMode {
    /// Light theme (default)
    #[default]
    #[serde(rename = "light")]
    Light,
    /// Dark theme
    #[serde(rename = "dark")]
    Dark,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    /// Get the syntect theme name for code syntax highlighting
    pub fn syntect_theme(self) -> &'static str {
        match self {
            ThemeMode::Light => "base16-ocean.light",
            ThemeMode::Dark => "Solarized (dark)",
        }
    }
}

/// JSON Schema for top-level theme file
#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: String,
    themes: Vec<ThemeVariantJson>,
}

/// JSON Schema for a theme variant within the file
#[derive(Debug, Deserialize)]
struct ThemeVariantJson {
    name: String,
    mode: ThemeMode,
    colors: HashMap<String, String>,
    highlight: serde_json::Value,
}

/// Complete set of colors for a theme
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    pub name: String,
    pub mode: ThemeMode,
    pub bg_color: Rgba,
    pub text_color: Rgba,
    pub code_bg_color: Rgba,
    pub code_line_color: Rgba,
    pub copy_button_bg_color: Rgba,
    pub copy_button_text_color: Rgba,
    pub search_bg_color: Rgba,
    pub current_match_bg_color: Rgba,
    pub blockquote_border_color: Rgba,
    pub link_color: Rgba,
    pub hover_link_color: Rgba,
    pub version_badge_bg_color: Rgba,
    pub version_badge_text_color: Rgba,
    pub table_border_color: Rgba,
    pub table_header_bg: Rgba,
    pub toc_bg_color: Rgba,
    pub toc_text_color: Rgba,
    pub toc_hover_color: Rgba,
    pub toc_active_color: Rgba,
    pub toc_toggle_bg_color: Rgba,
    pub toc_toggle_text_color: Rgba,
    pub toc_border_color: Rgba,
    pub toc_toggle_hover_color: Rgba,
    pub goto_line_overlay_bg_color: Rgba,
    pub goto_line_overlay_text_color: Rgba,
    pub pdf_success_bg_color: Rgba,
    pub pdf_error_bg_color: Rgba,
    pub pdf_warning_bg_color: Rgba,
    pub pdf_notification_text_color: Rgba,
}

impl Default for ThemeColors {
    fn default() -> Self {
        ThemeColors::from_json(
            "Zoegi Light",
            ThemeMode::Light,
            &HashMap::new(),
            &serde_json::Value::Null,
        )
    }
}

impl ThemeColors {
    /// Create ThemeColors from JSON data
    fn from_json(
        name: &str,
        mode: ThemeMode,
        colors: &HashMap<String, String>,
        highlight: &serde_json::Value,
    ) -> Self {
        let get_color = |key: &str, default: &str| -> Rgba {
            // Try explicit map first
            if let Some(hex) = colors.get(key) {
                return rgba_from_hex(hex);
            }

            // Try looking into highlight object if it's a simple string
            // highlight.key
            if let Some(val) = highlight.get(key)
                && let Some(s) = val.as_str()
            {
                return rgba_from_hex(s);
            }

            // Try nested keys in highlight e.g. "editor.background"
            // The zoegi.json has keys with dots in highlight object, so highlight.get("editor.background") works

            rgba_from_hex(default)
        };

        // Helper for highlight lookups which might be separate or flat
        let get_hl = |key: &str, default: &str| -> Rgba {
            // Check in colors first just in case
            if let Some(hex) = colors.get(key) {
                return rgba_from_hex(hex);
            }

            if let Some(val) = highlight.get(key)
                && let Some(s) = val.as_str()
            {
                return rgba_from_hex(s);
            }
            rgba_from_hex(default)
        };

        Self {
            name: name.to_string(),
            mode,
            // colors.background: "#ffffffff"
            bg_color: get_color("background", "#ffffffff"),

            // colors.foreground: "#333333ff"
            text_color: get_color("foreground", "#333333ff"),

            // highlight.editor.active_line.background (Light) / highlight.editor.background (Dark) ??
            // Using logic from original code:
            // Light: #f7f7f7ff (active_line)
            // Dark: #181818ff (background)
            // We'll try to follow the intent. Code background usually means the editor bg or active line?
            // "code_bg_color" seems to be used for block background.
            // Let's use editor.background if available, or fallback.
            code_bg_color: get_hl("editor.active_line.background", "#f7f7f7ff"),

            // highlight.editor.line_number: "#aaaaaaff"
            code_line_color: get_hl("editor.line_number", "#aaaaaaff"),

            // colors.primary.background: "#377961ff"
            copy_button_bg_color: get_color("primary.background", "#377961ff"),

            // colors.primary.foreground: "#ffffffff"
            copy_button_text_color: get_color("primary.foreground", "#ffffffff"),

            // highlight.hint.background: "#deeaedff"
            search_bg_color: get_hl("hint.background", "#deeaedff"),

            // colors.selection.background: "#568b9926"
            current_match_bg_color: get_color("selection.background", "#568b9926"),

            // colors.border: "#0000001a"
            blockquote_border_color: get_color("border", "#0000001a"),

            // highlight.syntax.link_text: "#377961ff"
            // Access nested syntax object
            link_color: highlight
                .get("syntax")
                .and_then(|s| s.get("link_text"))
                .and_then(|l| l.get("color"))
                .and_then(|c| c.as_str())
                .map(rgba_from_hex)
                .unwrap_or_else(|| rgba_from_hex("#377961ff")),

            // highlight.syntax.link_uri: "#568b99ff"
            hover_link_color: highlight
                .get("syntax")
                .and_then(|s| s.get("link_uri"))
                .and_then(|l| l.get("color"))
                .and_then(|c| c.as_str())
                .map(rgba_from_hex)
                .unwrap_or_else(|| rgba_from_hex("#568b99ff")),

            // colors.info.background: "#568b99ff"
            version_badge_bg_color: get_color("info.background", "#568b99ff"),

            // colors.accent.foreground: "#333333ff"
            version_badge_text_color: get_color("accent.foreground", "#333333ff"),

            // colors.border: "#0000001a"
            table_border_color: get_color("border", "#0000001a"),

            // colors.list.active.background: "#ebebebff"
            table_header_bg: get_color("list.active.background", "#ebebebff"),

            // colors.accent.background / tab_bar.background: "#fafafaff"
            toc_bg_color: get_color("tab_bar.background", "#fafafaff"),

            // colors.tab.foreground: "#595959ff"
            toc_text_color: get_color("tab.foreground", "#595959ff"),

            // colors.secondary.hover.background: "#f0f0f0ff"
            toc_hover_color: get_color("secondary.hover.background", "#f0f0f0ff"),

            // colors.primary.foreground: "#ffffffff"
            toc_active_color: get_color("primary.foreground", "#ffffffff"),

            // colors.primary.background: "#377961ff" (Light) / #66b395ff (Dark)
            // But light hardcoded used #000000ff?
            // Original code: toc_toggle_bg_color: rgba_from_hex("#000000ff"), in Light
            // dark code: toc_toggle_bg_color: rgba_from_hex("#66b395ff"),
            // JSON Light primary.background is #377961ff.
            // I will bind to primary.background or something similar?
            // "toc_toggle_bg_color" seems custom. Let's use primary.background as fallback.
            toc_toggle_bg_color: get_color("primary.background", "#000000ff"),

            // colors.primary.foreground: "#ffffffff"
            toc_toggle_text_color: get_color("primary.foreground", "#ffffffff"),

            // colors.title_bar.border / colors.border: "#0000001a"
            toc_border_color: get_color("title_bar.border", "#0000001a"),

            // colors.list.active.border: "#408068ff"
            toc_toggle_hover_color: get_color("list.active.border", "#408068ff"),

            // highlight.info.background: "#deeaedff"
            goto_line_overlay_bg_color: get_hl("info.background", "#deeaedff"),

            // highlight.editor.foreground: "#333333ff"
            goto_line_overlay_text_color: get_hl("editor.foreground", "#333333ff"),

            // highlight.created.background: "#dfeadbff"
            pdf_success_bg_color: get_hl("created.background", "#dfeadbff"),

            // highlight.error.background: "#fadfdbff"
            pdf_error_bg_color: get_hl("error.background", "#fadfdbff"),

            // highlight.warning.background: "#fbedcbff"
            pdf_warning_bg_color: get_hl("warning.background", "#fbedcbff"),

            // colors.primary.foreground: "#ffffffff"
            pdf_notification_text_color: get_color("primary.foreground", "#ffffffff"),
        }
    }
}

/// Registry storing all loaded themes
#[derive(Default, Debug)]
pub struct ThemeRegistry {
    themes: HashMap<String, ThemeColors>,
    families: HashMap<String, Vec<String>>, // Family name -> List of variant names
}

impl ThemeRegistry {
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let mut themes = HashMap::new();
        let mut families = HashMap::new();

        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(Self::default());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                match Self::load_file(&path) {
                    Ok((family_name, variants)) => {
                        let mut variant_names = Vec::new();
                        for variant in variants {
                            let name = variant.name.clone();
                            variant_names.push(name.clone());
                            themes.insert(name, variant);
                        }
                        families.insert(family_name, variant_names);
                    }
                    Err(e) => {
                        error!("Failed to load theme from {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} themes from {:?}", themes.len(), dir);
        Ok(Self { themes, families })
    }

    fn load_file(path: &Path) -> Result<(String, Vec<ThemeColors>)> {
        let content = std::fs::read_to_string(path)?;
        let theme_file: ThemeFile = serde_json::from_str(&content)?;

        let variants = theme_file
            .themes
            .into_iter()
            .map(|v| ThemeColors::from_json(&v.name, v.mode, &v.colors, &v.highlight))
            .collect();

        Ok((theme_file.name, variants))
    }

    pub fn get(&self, name: &str) -> Option<&ThemeColors> {
        self.themes.get(name)
    }

    pub fn list_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Toggle theme: switch from Light to Dark or vice-versa within the same family
    pub fn toggle_theme(&self, current_name: &str) -> Option<String> {
        let current = self.themes.get(current_name)?;

        // Find the family this theme belongs to
        for variants in self.families.values() {
            if variants.contains(&current_name.to_string()) {
                // Look for a variant with the opposite mode
                let target_mode = match current.mode {
                    ThemeMode::Light => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                };

                for sibling_name in variants {
                    if let Some(sibling) = self.themes.get(sibling_name)
                        && sibling.mode == target_mode
                    {
                        return Some(sibling_name.clone());
                    }
                }
            }
        }

        // Fallback: if no family found or no counterpart, try simple string replacement if standard naming used
        match (
            current_name.contains(" Light"),
            current_name.contains(" Dark"),
        ) {
            (true, _) => {
                let new_name = current_name.replace(" Light", " Dark");
                if self.themes.contains_key(&new_name) {
                    return Some(new_name);
                }
            }
            (false, true) => {
                let new_name = current_name.replace(" Dark", " Light");
                if self.themes.contains_key(&new_name) {
                    return Some(new_name);
                }
            }
            _ => {}
        }

        None
    }

    /// Cycle to next theme family while preserving the current mode (Light/Dark)
    pub fn cycle_theme(&self, current_name: &str) -> Option<String> {
        let current = self.themes.get(current_name)?;
        let current_mode = current.mode;

        // Get all themes with the same mode, sorted by name
        let mut same_mode_themes: Vec<&String> = self
            .themes
            .iter()
            .filter(|(_, colors)| colors.mode == current_mode)
            .map(|(name, _)| name)
            .collect();
        same_mode_themes.sort();

        if same_mode_themes.is_empty() {
            return None;
        }

        // Find current theme's position and get the next one
        let current_pos = same_mode_themes
            .iter()
            .position(|name| *name == current_name)?;
        let next_pos = (current_pos + 1) % same_mode_themes.len();

        Some(same_mode_themes[next_pos].clone())
    }
}

/// Parse a hex color like `#RRGGBB` or `#RRGGBBAA` into an `Rgba`.
/// Returns opaque black on parse failure.
pub fn rgba_from_hex(s: &str) -> Rgba {
    let s = s.trim_start_matches('#');
    match s.len() {
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
            Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }
        }
        _ => Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn test_full_hex_with_hash() {
        // #11223344 -> r=17,g=34,b=51,a=68
        let c = rgba_from_hex("#11223344");
        assert!(approx_eq(c.r, 17.0 / 255.0));
        assert!(approx_eq(c.g, 34.0 / 255.0));
        assert!(approx_eq(c.b, 51.0 / 255.0));
        assert!(approx_eq(c.a, 68.0 / 255.0));
    }

    #[test]
    fn test_six_digit_hex_no_alpha() {
        // FFa500 -> r=255,g=165,b=0, a=1.0
        let c = rgba_from_hex("FFa500");
        assert!(approx_eq(c.r, 255.0 / 255.0));
        assert!(approx_eq(c.g, 165.0 / 255.0));
        assert!(approx_eq(c.b, 0.0));
        assert!(approx_eq(c.a, 1.0));
    }
}
