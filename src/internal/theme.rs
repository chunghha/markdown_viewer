//! Theme system for the markdown viewer
//!
//! This module provides theme support with Light and Dark variants,
//! each containing a complete set of color values for all UI elements.

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
    /// Create color set for Light theme
    fn light() -> Self {
        Self {
            bg_color: Rgba {
                r: 0.992,
                g: 0.980,
                b: 0.965,
                a: 1.0,
            },
            text_color: Rgba {
                r: 0.239,
                g: 0.114,
                b: 0.114,
                a: 1.0,
            },
            code_bg_color: Rgba {
                r: 0.972,
                g: 0.960,
                b: 0.945,
                a: 1.0,
            },
            code_line_color: Rgba {
                r: 0.45,
                g: 0.45,
                b: 0.45,
                a: 1.0,
            },
            copy_button_bg_color: Rgba {
                r: 0.2,
                g: 0.5,
                b: 0.8,
                a: 0.8,
            },
            copy_button_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            search_bg_color: Rgba {
                r: 1.0,
                g: 0.92,
                b: 0.23,
                a: 0.5,
            },
            current_match_bg_color: Rgba {
                r: 1.0,
                g: 0.6,
                b: 0.0,
                a: 0.6,
            },
            blockquote_border_color: Rgba {
                r: 0.8,
                g: 0.8,
                b: 0.8,
                a: 1.0,
            },
            link_color: Rgba {
                r: 0.05,
                g: 0.1,
                b: 0.35,
                a: 1.0,
            },
            hover_link_color: Rgba {
                r: 0.1,
                g: 0.2,
                b: 0.5,
                a: 1.0,
            },
            version_badge_bg_color: Rgba {
                r: 0.529,
                g: 0.808,
                b: 0.922,
                a: 1.0,
            },
            version_badge_text_color: Rgba {
                r: 0.0,
                g: 0.122,
                b: 0.247,
                a: 1.0,
            },
            table_border_color: Rgba {
                r: 0.7,
                g: 0.7,
                b: 0.7,
                a: 1.0,
            },
            table_header_bg: Rgba {
                r: 0.96,
                g: 0.94,
                b: 0.90,
                a: 1.0,
            },
            toc_bg_color: Rgba {
                r: 0.969,
                g: 0.957,
                b: 0.941,
                a: 1.0,
            },
            toc_text_color: Rgba {
                r: 0.3,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
            toc_hover_color: Rgba {
                r: 0.859,
                g: 0.847,
                b: 0.831,
                a: 1.0,
            },
            toc_active_color: Rgba {
                r: 0.502,
                g: 0.686,
                b: 0.878,
                a: 0.3,
            },
            toc_toggle_bg_color: Rgba {
                r: 0.502,
                g: 0.686,
                b: 0.878,
                a: 0.9,
            },
            toc_toggle_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            toc_border_color: Rgba {
                r: 0.8,
                g: 0.8,
                b: 0.8,
                a: 1.0,
            },
            toc_toggle_hover_color: Rgba {
                r: 0.502,
                g: 0.686,
                b: 0.878,
                a: 1.0,
            },
            goto_line_overlay_bg_color: Rgba {
                r: 0.6,
                g: 0.95,
                b: 1.0,
                a: 0.95,
            },
            goto_line_overlay_text_color: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            pdf_success_bg_color: Rgba {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 0.95,
            },
            pdf_error_bg_color: Rgba {
                r: 1.0,
                g: 0.4,
                b: 0.4,
                a: 0.95,
            },
            pdf_warning_bg_color: Rgba {
                r: 1.0,
                g: 0.8,
                b: 0.2,
                a: 0.95,
            },
            pdf_notification_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        }
    }

    /// Create color set for Dark theme
    fn dark() -> Self {
        Self {
            bg_color: Rgba {
                r: 0.11,
                g: 0.11,
                b: 0.12,
                a: 1.0,
            },
            text_color: Rgba {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
            code_bg_color: Rgba {
                r: 0.15,
                g: 0.15,
                b: 0.16,
                a: 1.0,
            },
            code_line_color: Rgba {
                r: 0.6,
                g: 0.6,
                b: 0.6,
                a: 1.0,
            },
            copy_button_bg_color: Rgba {
                r: 0.2,
                g: 0.5,
                b: 0.8,
                a: 0.8,
            },
            copy_button_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            search_bg_color: Rgba {
                r: 0.8,
                g: 0.6,
                b: 0.0,
                a: 0.5,
            },
            current_match_bg_color: Rgba {
                r: 1.0,
                g: 0.7,
                b: 0.0,
                a: 0.6,
            },
            blockquote_border_color: Rgba {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            link_color: Rgba {
                r: 0.4,
                g: 0.6,
                b: 0.9,
                a: 1.0,
            },
            hover_link_color: Rgba {
                r: 0.5,
                g: 0.7,
                b: 1.0,
                a: 1.0,
            },
            version_badge_bg_color: Rgba {
                r: 0.2,
                g: 0.4,
                b: 0.6,
                a: 1.0,
            },
            version_badge_text_color: Rgba {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
            table_border_color: Rgba {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 1.0,
            },
            table_header_bg: Rgba {
                r: 0.2,
                g: 0.2,
                b: 0.22,
                a: 1.0,
            },
            toc_bg_color: Rgba {
                r: 0.13,
                g: 0.13,
                b: 0.14,
                a: 1.0,
            },
            toc_text_color: Rgba {
                r: 0.8,
                g: 0.8,
                b: 0.8,
                a: 1.0,
            },
            toc_hover_color: Rgba {
                r: 0.2,
                g: 0.2,
                b: 0.22,
                a: 1.0,
            },
            toc_active_color: Rgba {
                r: 0.3,
                g: 0.5,
                b: 0.7,
                a: 0.3,
            },
            toc_toggle_bg_color: Rgba {
                r: 0.3,
                g: 0.5,
                b: 0.7,
                a: 0.9,
            },
            toc_toggle_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            toc_border_color: Rgba {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 1.0,
            },
            toc_toggle_hover_color: Rgba {
                r: 0.4,
                g: 0.6,
                b: 0.8,
                a: 1.0,
            },
            goto_line_overlay_bg_color: Rgba {
                r: 0.2,
                g: 0.3,
                b: 0.4,
                a: 0.95,
            },
            goto_line_overlay_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            pdf_success_bg_color: Rgba {
                r: 0.15,
                g: 0.6,
                b: 0.3,
                a: 0.95,
            },
            pdf_error_bg_color: Rgba {
                r: 0.8,
                g: 0.2,
                b: 0.2,
                a: 0.95,
            },
            pdf_warning_bg_color: Rgba {
                r: 0.8,
                g: 0.6,
                b: 0.1,
                a: 0.95,
            },
            pdf_notification_text_color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        }
    }
}
