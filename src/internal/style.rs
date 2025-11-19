//! Style constants for the markdown viewer
//!
//! This module contains all visual styling constants including colors,
//! fonts, sizes, and spacing values.

use gpui::Rgba;

// ---- Fonts -----------------------------------------------------------------

pub const PRIMARY_FONT: &str = "Google Sans Code";
pub const CODE_FONT: &str = "monospace";

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
    r: 0.980,
    g: 0.953,
    b: 0.910,
    a: 1.0,
};

pub const BLOCKQUOTE_BORDER_COLOR: Rgba = Rgba {
    r: 0.8,
    g: 0.8,
    b: 0.8,
    a: 1.0,
};

pub const LINK_COLOR: Rgba = Rgba {
    r: 0.173,
    g: 0.627,
    b: 0.627,
    a: 1.0,
};

// ---- Table Styling ---------------------------------------------------------

pub const TABLE_BORDER_COLOR: Rgba = Rgba {
    r: 0.7,
    g: 0.7,
    b: 0.7,
    a: 1.0,
};

pub const TABLE_HEADER_BG: Rgba = Rgba {
    r: 0.15,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};

pub const TABLE_CELL_PADDING: f32 = 8.0;

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

/// Default viewport height used when window dimensions are unavailable
pub const DEFAULT_VIEWPORT_HEIGHT: f32 = 800.0;
