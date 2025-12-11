/*! Theme system for the markdown viewer
 *
 * This module provides theme support with Light and Dark variants,
 * each containing a complete set of color values for all UI elements.
 */

use gpui::Rgba;

/// Theme variant (Light or Dark)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Theme {
    /// Light theme (default)
    #[default]
    Light,
    /// Dark theme
    Dark,
}

impl Theme {
    /// Toggle between Light and Dark themes
    pub fn toggle(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    /// Get the syntect theme name for code syntax highlighting
    pub fn syntect_theme(self) -> &'static str {
        match self {
            Theme::Light => "base16-ocean.light",
            Theme::Dark => "Solarized (dark)",
        }
    }
}

/// Complete set of colors for a theme
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
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

impl From<Theme> for ThemeColors {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => ThemeColors::light(),
            Theme::Dark => ThemeColors::dark(),
        }
    }
}

impl ThemeColors {
    /// Create color set for Light theme (Zoegi Light hex values)
    fn light() -> Self {
        Self {
            // colors.background: "#ffffffff"
            bg_color: rgba_from_hex("#ffffffff"),

            // colors.foreground: "#333333ff"
            text_color: rgba_from_hex("#333333ff"),

            // highlight.editor.active_line.background: "#f7f7f7ff"
            code_bg_color: rgba_from_hex("#f7f7f7ff"),

            // highlight.editor.line_number: "#aaaaaaff"
            code_line_color: rgba_from_hex("#aaaaaaff"),

            // colors.primary.background: "#377961ff"
            copy_button_bg_color: rgba_from_hex("#377961ff"),

            // colors.primary.foreground: "#ffffffff"
            copy_button_text_color: rgba_from_hex("#ffffffff"),

            // highlight.hint.background: "#deeaedff"
            search_bg_color: rgba_from_hex("#deeaedff"),

            // colors.selection.background: "#568b9926"
            current_match_bg_color: rgba_from_hex("#568b9926"),

            // colors.border: "#0000001a"
            blockquote_border_color: rgba_from_hex("#0000001a"),

            // highlight.syntax.link_text: "#377961ff"
            link_color: rgba_from_hex("#377961ff"),

            // highlight.syntax.link_uri: "#568b99ff"
            hover_link_color: rgba_from_hex("#568b99ff"),

            // colors.info.background: "#568b99ff"
            version_badge_bg_color: rgba_from_hex("#568b99ff"),

            // colors.accent.foreground: "#333333ff"
            version_badge_text_color: rgba_from_hex("#333333ff"),

            // colors.border: "#0000001a"
            table_border_color: rgba_from_hex("#0000001a"),

            // colors.list.active.background: "#ebebebff"
            table_header_bg: rgba_from_hex("#ebebebff"),

            // colors.accent.background / tab_bar.background: "#fafafaff"
            toc_bg_color: rgba_from_hex("#fafafaff"),

            // colors.tab.foreground: "#595959ff"
            toc_text_color: rgba_from_hex("#595959ff"),

            // colors.secondary.hover.background: "#f0f0f0ff"
            toc_hover_color: rgba_from_hex("#f0f0f0ff"),

            // colors.primary.foreground: "#ffffffff"
            toc_active_color: rgba_from_hex("#ffffffff"),

            // colors.primary.background: "#377961ff"
            toc_toggle_bg_color: rgba_from_hex("#000000ff"),

            // colors.primary.foreground: "#ffffffff"
            toc_toggle_text_color: rgba_from_hex("#ffffffff"),

            // colors.title_bar.border / colors.border: "#0000001a"
            toc_border_color: rgba_from_hex("#0000001a"),

            // colors.list.active.border: "#408068ff"
            toc_toggle_hover_color: rgba_from_hex("#408068ff"),

            // highlight.info.background: "#deeaedff"
            goto_line_overlay_bg_color: rgba_from_hex("#deeaedff"),

            // highlight.editor.foreground: "#333333ff"
            goto_line_overlay_text_color: rgba_from_hex("#333333ff"),

            // highlight.created.background: "#dfeadbff"
            pdf_success_bg_color: rgba_from_hex("#dfeadbff"),

            // highlight.error.background: "#fadfdbff"
            pdf_error_bg_color: rgba_from_hex("#fadfdbff"),

            // highlight.warning.background: "#fbedcbff"
            pdf_warning_bg_color: rgba_from_hex("#fbedcbff"),

            // colors.primary.foreground: "#ffffffff"
            pdf_notification_text_color: rgba_from_hex("#ffffffff"),
        }
    }

    /// Create color set for Dark theme (Zoegi Dark hex values)
    fn dark() -> Self {
        Self {
            // colors.background: "#2b2b2bff"
            bg_color: rgba_from_hex("#2b2b2bff"),

            // colors.foreground: "#ddddddff"
            text_color: rgba_from_hex("#ddddddff"),

            // highlight.editor.background: "#181818ff"
            code_bg_color: rgba_from_hex("#181818ff"),

            // highlight.editor.line_number: "#666666ff"
            code_line_color: rgba_from_hex("#666666ff"),

            // colors.primary.background: "#66b395ff"
            copy_button_bg_color: rgba_from_hex("#66b395ff"),

            // colors.primary.foreground: "#2b2b2bff"
            copy_button_text_color: rgba_from_hex("#2b2b2bff"),

            // highlight.hint.background: "#3d595cff"
            search_bg_color: rgba_from_hex("#3d595cff"),

            // colors.selection.background: "#77b9c026"
            current_match_bg_color: rgba_from_hex("#77b9c026"),

            // colors.border: "#333333ff"
            blockquote_border_color: rgba_from_hex("#333333ff"),

            // highlight.syntax.link_text: "#74ccaaff"
            link_color: rgba_from_hex("#74ccaaff"),

            // highlight.syntax.link_uri: "#77b9c0ff"
            hover_link_color: rgba_from_hex("#77b9c0ff"),

            // colors.info.background: "#70b7beff"
            version_badge_bg_color: rgba_from_hex("#70b7beff"),

            // popover.foreground / editor.foreground: "#ddddddff"
            version_badge_text_color: rgba_from_hex("#ddddddff"),

            // colors.border: "#333333ff"
            table_border_color: rgba_from_hex("#333333ff"),

            // colors.list.active.background: "#2e2e2eff"
            table_header_bg: rgba_from_hex("#2e2e2eff"),

            // colors.tab_bar.background: "#1f1f1fff"
            toc_bg_color: rgba_from_hex("#1f1f1fff"),

            // colors.tab.foreground: "#999999ff"
            toc_text_color: rgba_from_hex("#999999ff"),

            // colors.secondary.hover.background: "#262626ff"
            toc_hover_color: rgba_from_hex("#262626ff"),

            // colors.primary.foreground: "#2b2b2bff"
            toc_active_color: rgba_from_hex("#2b2b2bff"),

            // colors.primary.background: "#66b395ff"
            toc_toggle_bg_color: rgba_from_hex("#66b395ff"),

            // colors.primary.foreground: "#2b2b2bff"
            toc_toggle_text_color: rgba_from_hex("#2b2b2bff"),

            // colors.border: "#333333ff"
            toc_border_color: rgba_from_hex("#333333ff"),

            // colors.list.active.border: "#66b395ff"
            toc_toggle_hover_color: rgba_from_hex("#66b395ff"),

            // highlight.info.background: "#3d595cff"
            goto_line_overlay_bg_color: rgba_from_hex("#3d595cff"),

            // highlight.editor.foreground: "#ddddddff"
            goto_line_overlay_text_color: rgba_from_hex("#ddddddff"),

            // highlight.created.background: "#a1c1811a"
            pdf_success_bg_color: rgba_from_hex("#a1c1811a"),

            // highlight.error.background: "#693b35ff"
            pdf_error_bg_color: rgba_from_hex("#693b35ff"),

            // highlight.warning.background: "#564b2eff"
            pdf_warning_bg_color: rgba_from_hex("#564b2eff"),

            // popover.foreground / editor.foreground: "#ddddddff"
            pdf_notification_text_color: rgba_from_hex("#ddddddff"),
        }
    }
}

/// Parse a hex color like `#RRGGBB` or `#RRGGBBAA` into an `Rgba`.
/// Returns opaque black on parse failure.
fn rgba_from_hex(s: &str) -> Rgba {
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

    #[test]
    fn test_invalid_length_returns_opaque_black() {
        let c = rgba_from_hex("zzz");
        assert!(approx_eq(c.r, 0.0));
        assert!(approx_eq(c.g, 0.0));
        assert!(approx_eq(c.b, 0.0));
        assert!(approx_eq(c.a, 1.0));
    }

    #[test]
    fn test_partial_parse_failure_in_eight_chars() {
        // Invalid red component ("GG") should fall back to 0, alpha parse fails? here alpha valid
        let c = rgba_from_hex("#GG0000FF");
        assert!(approx_eq(c.r, 0.0)); // "GG" is invalid -> unwrap_or(0)
        assert!(approx_eq(c.g, 0.0));
        assert!(approx_eq(c.b, 0.0));
        assert!(approx_eq(c.a, 255.0 / 255.0));
    }
}
