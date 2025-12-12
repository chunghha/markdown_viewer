//! Style constants for the markdown viewer
//!
//! This module contains all visual styling constants including colors,
//! fonts, sizes, and spacing values.

use super::theme::ThemeColors;
use gpui::Rgba;

// ---- Fonts -----------------------------------------------------------------

pub const PRIMARY_FONT: &str = "Google Sans Code";
pub const CODE_FONT: &str = "GeistMono Nerd Font";

// ---- Colors ----------------------------------------------------------------

pub const BG_COLOR: Rgba = Rgba {
    r: 0.992,
    g: 0.980,
    b: 0.965,
    a: 1.0,
};

pub const TEXT_COLOR: Rgba = Rgba {
    r: 0.239,
    g: 0.114,
    b: 0.114,
    a: 1.0,
};

pub const CODE_BG_COLOR: Rgba = Rgba {
    r: 0.972,
    g: 0.960,
    b: 0.945,
    a: 1.0,
};

pub const CODE_LINE_COLOR: Rgba = Rgba {
    r: 0.45,
    g: 0.45,
    b: 0.45,
    a: 1.0,
};

pub const COPY_BUTTON_BG_COLOR: Rgba = Rgba {
    r: 0.2,
    g: 0.5,
    b: 0.8,
    a: 0.8,
};

pub const COPY_BUTTON_TEXT_COLOR: Rgba = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// Search match highlight color (yellow)
pub const SEARCH_BG_COLOR: Rgba = Rgba {
    r: 1.0,
    g: 0.92,
    b: 0.23,
    a: 0.5,
};

pub const CURRENT_MATCH_BG_COLOR: Rgba = Rgba {
    r: 1.0,
    g: 0.6,
    b: 0.0,
    a: 0.6,
};

pub const BLOCKQUOTE_BORDER_COLOR: Rgba = Rgba {
    r: 0.8,
    g: 0.8,
    b: 0.8,
    a: 1.0,
};

pub const LINK_COLOR: Rgba = Rgba {
    r: 0.05,
    g: 0.1,
    b: 0.35,
    a: 1.0,
};

pub const HOVER_LINK_COLOR: Rgba = Rgba {
    r: 0.1,
    g: 0.2,
    b: 0.5,
    a: 1.0,
};

pub const VERSION_BADGE_BG_COLOR: Rgba = Rgba {
    r: 0.529,
    g: 0.808,
    b: 0.922,
    a: 1.0,
};

pub const VERSION_BADGE_TEXT_COLOR: Rgba = Rgba {
    r: 0.0,
    g: 0.122,
    b: 0.247,
    a: 1.0,
};

// ---- Image Styling ---------------------------------------------------------

/// Maximum width for inline images to prevent overflow
pub const IMAGE_MAX_WIDTH: f32 = 800.0;

/// Default image border radius
pub const IMAGE_BORDER_RADIUS: f32 = 4.0;

// ---- Table Styling ---------------------------------------------------------

pub const TABLE_BORDER_COLOR: Rgba = Rgba {
    r: 0.7,
    g: 0.7,
    b: 0.7,
    a: 1.0,
};

pub const TABLE_HEADER_BG: Rgba = Rgba {
    r: 0.96,
    g: 0.94,
    b: 0.90,
    a: 1.0,
};

pub const TABLE_CELL_PADDING: f32 = 8.0;

/// Minimum width for table columns to ensure readability
pub const MIN_COLUMN_WIDTH: f32 = 150.0;

/// Total horizontal padding for tables (left + right margins/padding)
pub const TABLE_HORIZONTAL_PADDING: f32 = 32.0;

// ---- Text Sizes ------------------------------------------------------------

pub const BASE_TEXT_SIZE: f32 = 19.2;
pub const H1_SIZE: f32 = 38.4;
pub const H2_SIZE: f32 = 33.6;
pub const H3_SIZE: f32 = 28.8;
pub const H4_SIZE: f32 = 26.4;
pub const H5_SIZE: f32 = 24.0;
pub const H6_SIZE: f32 = 21.6;

// ---- Content Height Estimation ---------------------------------------------

/// Line height multiplier for estimating content height
/// Accounts for text size + line spacing
pub const LINE_HEIGHT_MULTIPLIER: f32 = 1.5;

/// Additional buffer in pixels to ensure all content is accessible
/// Accounts for headings, lists, blockquotes, and extra spacing
pub const CONTENT_HEIGHT_BUFFER: f32 = 400.0;

/// Character width multiplier for estimating text wrapping
/// Used with base_text_size to approximate average character width
/// Higher values = more conservative (more wrapped lines estimated)
pub const CHAR_WIDTH_MULTIPLIER: f32 = 0.4;

/// Minimal safety margin at bottom to account for any rendering variance
pub const BOTTOM_SCROLL_PADDING: f32 = 120.0;

/// Scaling multiplier applied to estimated content height
/// Accounts for cumulative inter-element spacing not captured in per-line estimation
/// 1.02 = 2% extra height to account for margins between paragraphs, lists, etc.
pub const CONTENT_HEIGHT_SCALE: f32 = 1.02;

/// Average vertical spacing per block element (paragraph, list, table, code block)
/// Accounts for .mb_2(), .my_2(), .p_3() margins in rendering
pub const BLOCK_ELEMENT_SPACING: f32 = 4.0;

/// Default viewport height used when window dimensions are unavailable
/// Default viewport height used when window dimensions are unavailable
pub const DEFAULT_VIEWPORT_HEIGHT: f32 = 800.0;

// ---- Table of Contents Styling ---------------------------------------------

/// Width of the TOC sidebar when visible
pub const TOC_WIDTH: f32 = 300.0;

/// Background color for TOC sidebar
pub const TOC_BG_COLOR: Rgba = Rgba {
    r: 0.969,
    g: 0.957,
    b: 0.941,
    a: 1.0,
};

/// Text color for TOC entries
pub const TOC_TEXT_COLOR: Rgba = Rgba {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};

/// Background color for TOC entry on hover
pub const TOC_HOVER_COLOR: Rgba = Rgba {
    r: 0.859,
    g: 0.847,
    b: 0.831,
    a: 1.0,
};

/// Background color for active TOC entry (current section)
pub const TOC_ACTIVE_COLOR: Rgba = Rgba {
    r: 0.502,
    g: 0.686,
    b: 0.878,
    a: 0.3,
};

/// Indentation per heading level in TOC
pub const TOC_INDENT_PER_LEVEL: f32 = 12.0;

/// TOC toggle button background color
pub const TOC_TOGGLE_BG_COLOR: Rgba = Rgba {
    r: 0.502,
    g: 0.686,
    b: 0.878,
    a: 0.9,
};

/// TOC toggle button text color
pub const TOC_TOGGLE_TEXT_COLOR: Rgba = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// TOC sidebar border color
pub const TOC_BORDER_COLOR: Rgba = Rgba {
    r: 0.8,
    g: 0.8,
    b: 0.8,
    a: 1.0,
};

/// TOC toggle button hover color
pub const TOC_TOGGLE_HOVER_COLOR: Rgba = Rgba {
    r: 0.502,
    g: 0.686,
    b: 0.878,
    a: 1.0,
};

// ---- Go-to-Line Overlay Styling -----------------------------------------

/// Background color for go-to-line overlay (light cyan/blue)
pub const GOTO_LINE_OVERLAY_BG_COLOR: Rgba = Rgba {
    r: 0.6,
    g: 0.95,
    b: 1.0,
    a: 0.95,
};

/// Text color for go-to-line overlay
pub const GOTO_LINE_OVERLAY_TEXT_COLOR: Rgba = Rgba {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

// ---- Keyboard Focus Indicators -----------------------------------------

/// Focus ring color for keyboard navigation (blue)
pub const FOCUS_RING_COLOR: Rgba = Rgba {
    r: 0.0,
    g: 0.4,
    b: 0.8,
    a: 1.0,
};

/// Background color for focused elements
pub const FOCUS_BG_COLOR: Rgba = Rgba {
    r: 0.678,
    g: 0.847,
    b: 1.0,
    a: 0.25,
};

/// Focus ring width in pixels
pub const FOCUS_RING_WIDTH: f32 = 2.0;

// ---- Theme-based Color Access -----------------------------------------

/// Get theme colors for the given theme name
///
/// This function provides access to all colors based on the active theme.
/// Prefer using this over the individual color constants when theme support is needed.
pub fn get_theme_colors(theme_name: &str) -> &'static ThemeColors {
    super::theme::registry().get(theme_name).unwrap_or_else(|| {
        tracing::warn!("Theme '{}' not found, falling back to default", theme_name);
        // Fallback to the first available theme or a specific default if initialized
        super::theme::registry()
            .get("Zoegi Light")
            .or_else(|| {
                // If completely empty (shouldn't happen if initialized), return a static default?
                // Accessing internal default via registry might be hard if registry is empty.
                // But we can construct one temporarily? No, return type is static ref.
                // Best effort: panic if no themes (user configuration error), or return whatever is there.
                super::theme::registry()
                    .list_names()
                    .first()
                    .and_then(|n| super::theme::registry().get(n))
            })
            .expect("No themes loaded! Ensure ‘themes/’ directory exists.")
    })
}
