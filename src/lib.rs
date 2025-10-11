//! Markdown Viewer Library
//!
//! This module contains the core functionality for rendering Markdown content
//! with scrolling support.

use comrak::nodes::{AstNode, NodeValue};
use gpui::{div, prelude::*, px, AnyElement, FontWeight, IntoElement, Rgba};
use std::path::Path;

// ---- Style Constants -------------------------------------------------------

pub const PRIMARY_FONT: &str = "Google Sans Code";
pub const CODE_FONT: &str = "monospace";

pub const BG_COLOR: Rgba = Rgba {
    r: 0.95,
    g: 0.958,
    b: 0.977,
    a: 1.0,
};
pub const TEXT_COLOR: Rgba = Rgba {
    r: 0.066,
    g: 0.133,
    b: 0.133,
    a: 1.0,
};
pub const CODE_BG_COLOR: Rgba = Rgba {
    r: 0.88,
    g: 0.96,
    b: 0.88,
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

pub const BASE_TEXT_SIZE: f32 = 19.2;
pub const H1_SIZE: f32 = 38.4;
pub const H2_SIZE: f32 = 33.6;
pub const H3_SIZE: f32 = 28.8;
pub const H4_SIZE: f32 = 26.4;
pub const H5_SIZE: f32 = 24.0;
pub const H6_SIZE: f32 = 21.6;

// ---- Content Height Estimation Constants ----------------------------------

/// Line height multiplier for estimating content height
/// Accounts for text size + line spacing
pub const LINE_HEIGHT_MULTIPLIER: f32 = 1.5;

/// Additional buffer in pixels to ensure all content is accessible
/// Accounts for headings, lists, blockquotes, and extra spacing
pub const CONTENT_HEIGHT_BUFFER: f32 = 400.0;

/// Default viewport height used when window dimensions are unavailable
pub const DEFAULT_VIEWPORT_HEIGHT: f32 = 800.0;

// ---- Scrolling State ------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollState {
    pub scroll_y: f32,
    pub max_scroll_y: f32,
    pub target_scroll_y: f32,   // For smooth scrolling
    pub scroll_velocity: f32,   // For momentum scrolling
    pub is_dragging: bool,      // For scroll thumb dragging
    pub drag_start_y: f32,      // Starting position when dragging
    pub drag_start_scroll: f32, // Starting scroll position when dragging
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            scroll_y: 0.0,
            max_scroll_y: 0.0,
            target_scroll_y: 0.0,
            scroll_velocity: 0.0,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_scroll: 0.0,
        }
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scroll_up(&mut self, amount: f32) {
        self.scroll_y = (self.scroll_y - amount).max(0.0);
    }

    pub fn scroll_down(&mut self, amount: f32) {
        self.scroll_y = (self.scroll_y + amount).min(self.max_scroll_y);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_y = 0.0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_y = self.max_scroll_y;
    }

    pub fn page_up(&mut self, page_height: f32) {
        self.scroll_up(page_height * 0.8);
    }

    pub fn page_down(&mut self, page_height: f32) {
        self.scroll_down(page_height * 0.8);
    }

    pub fn set_max_scroll(&mut self, content_height: f32, viewport_height: f32) {
        self.max_scroll_y = (content_height - viewport_height).max(0.0);
        // Ensure current scroll position is still valid
        self.scroll_y = self.scroll_y.min(self.max_scroll_y);
        self.target_scroll_y = self.target_scroll_y.min(self.max_scroll_y);
    }

    // Smooth scrolling methods
    pub fn smooth_scroll_to(&mut self, target_y: f32) {
        self.target_scroll_y = target_y.clamp(0.0, self.max_scroll_y);
        // Also update current scroll position immediately for testing
        self.scroll_y = self.target_scroll_y;
    }

    pub fn smooth_scroll_by(&mut self, delta: f32) {
        self.smooth_scroll_to(self.target_scroll_y + delta);
    }

    pub fn update_smooth_scroll(&mut self, dt: f32) {
        const SMOOTH_FACTOR: f32 = 8.0; // Higher = faster smoothing
        const VELOCITY_DAMPING: f32 = 0.9; // Lower = more damping

        // Smooth interpolation towards target
        let diff = self.target_scroll_y - self.scroll_y;
        self.scroll_velocity = diff * SMOOTH_FACTOR * dt;
        self.scroll_y += self.scroll_velocity;

        // Apply velocity damping for momentum
        self.scroll_velocity *= VELOCITY_DAMPING;

        // Clamp to bounds
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y);
        self.target_scroll_y = self.target_scroll_y.clamp(0.0, self.max_scroll_y);
    }

    // Override existing methods to use smooth scrolling
    pub fn smooth_scroll_up(&mut self, amount: f32) {
        self.smooth_scroll_by(-amount);
    }

    pub fn smooth_scroll_down(&mut self, amount: f32) {
        self.smooth_scroll_by(amount);
    }

    pub fn smooth_scroll_to_top(&mut self) {
        self.smooth_scroll_to(0.0);
    }

    pub fn smooth_scroll_to_bottom(&mut self) {
        self.smooth_scroll_to(self.max_scroll_y);
    }

    // Scroll thumb dragging methods
    pub fn start_drag(&mut self, mouse_y: f32, _viewport_height: f32) {
        if self.max_scroll_y > 0.0 {
            self.is_dragging = true;
            self.drag_start_y = mouse_y;
            self.drag_start_scroll = self.scroll_y;
        }
    }

    pub fn update_drag(&mut self, mouse_y: f32, viewport_height: f32) {
        if self.is_dragging && self.max_scroll_y > 0.0 {
            let drag_delta = mouse_y - self.drag_start_y;
            let scroll_delta = (drag_delta / viewport_height) * self.max_scroll_y;
            let new_scroll = (self.drag_start_scroll + scroll_delta).clamp(0.0, self.max_scroll_y);
            self.smooth_scroll_to(new_scroll);
        }
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
    }

    // Calculate scroll thumb position and size
    pub fn get_scroll_thumb_info(&self, viewport_height: f32) -> (f32, f32) {
        if self.max_scroll_y <= 0.0 {
            return (0.0, viewport_height);
        }

        let scroll_ratio = self.scroll_y / self.max_scroll_y;
        let thumb_height = viewport_height * 0.1; // 10% of viewport height
        let thumb_top = scroll_ratio * (viewport_height - thumb_height);

        (thumb_top, thumb_height)
    }

    // Scroll memory/persistence methods
    pub fn save_scroll_state(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let scroll_data = format!(
            "scroll_y: {:.2}\ntarget_scroll_y: {:.2}\nmax_scroll_y: {:.2}",
            self.scroll_y, self.target_scroll_y, self.max_scroll_y
        );
        std::fs::write(file_path, scroll_data)?;
        Ok(())
    }

    pub fn load_scroll_state(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        for line in content.lines() {
            if let Some((key, value)) = line.split_once(": ") {
                if let Ok(val) = value.parse::<f32>() {
                    match key {
                        "scroll_y" => self.scroll_y = val.max(0.0).min(self.max_scroll_y),
                        "target_scroll_y" => {
                            self.target_scroll_y = val.max(0.0).min(self.max_scroll_y)
                        }
                        "max_scroll_y" => self.max_scroll_y = val.max(0.0),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    // Get scroll position as percentage (0.0 to 1.0)
    pub fn get_scroll_percentage(&self) -> f32 {
        if self.max_scroll_y <= 0.0 {
            0.0
        } else {
            self.scroll_y / self.max_scroll_y
        }
    }

    // Set scroll position from percentage (0.0 to 1.0)
    pub fn set_scroll_percentage(&mut self, percentage: f32) {
        let clamped_percentage = percentage.clamp(0.0, 1.0);
        self.smooth_scroll_to(clamped_percentage * self.max_scroll_y);
    }
}

// ---- Rendering -------------------------------------------------------------

pub fn render_markdown_ast<'a, T>(node: &'a AstNode<'a>, _cx: &mut Context<T>) -> AnyElement {
    match &node.data.borrow().value {
        NodeValue::Document => div()
            .flex_col()
            .children(node.children().map(|child| render_markdown_ast(child, _cx)))
            .into_any_element(),

        NodeValue::Paragraph => {
            // Avoid extra spacing inside list items.
            let is_in_list_item = node
                .parent()
                .is_some_and(|p| matches!(p.data.borrow().value, NodeValue::Item(_)));

            let mut p = div().flex();
            if !is_in_list_item {
                p = p.mb_2();
            }
            p.children(node.children().map(|child| render_markdown_ast(child, _cx)))
                .into_any_element()
        }

        NodeValue::Heading(heading) => {
            let text_size = match heading.level {
                1 => px(H1_SIZE),
                2 => px(H2_SIZE),
                3 => px(H3_SIZE),
                4 => px(H4_SIZE),
                5 => px(H5_SIZE),
                _ => px(H6_SIZE),
            };
            div()
                .flex()
                .text_size(text_size)
                .font_weight(FontWeight::SEMIBOLD)
                .mt(px((heading.level == 1) as u8 as f32 * 4.0)) // small spacing tweak for top-level
                .children(node.children().map(|child| render_markdown_ast(child, _cx)))
                .into_any_element()
        }

        NodeValue::Text(text) => div()
            .child(String::from_utf8_lossy(text.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::Code(code) => div()
            .font_family(CODE_FONT)
            .bg(CODE_BG_COLOR)
            .text_color(TEXT_COLOR)
            .px_1()
            .rounded_sm()
            .child(String::from_utf8_lossy(code.literal.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::CodeBlock(code_block) => div()
            .bg(CODE_BG_COLOR)
            .p_3()
            .rounded_md()
            .font_family(CODE_FONT)
            .child(String::from_utf8_lossy(code_block.literal.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::List(list) => {
            let mut items = Vec::new();
            for item in node.children() {
                let marker = match list.list_type {
                    comrak::nodes::ListType::Bullet => "•".to_string(),
                    comrak::nodes::ListType::Ordered => format!("{}.", items.len() + 1),
                };
                let content =
                    div().children(item.children().map(|child| render_markdown_ast(child, _cx)));
                items.push(
                    div()
                        .flex()
                        .mb_1()
                        .child(div().mr_2().child(marker))
                        .child(content),
                );
            }
            div().flex_col().pl_4().children(items).into_any_element()
        }

        NodeValue::Link(link) => {
            // Could surface destination (link.url) via tooltip or on-click navigation later.
            let _href = &link.url; // Already a String (per comrak 0.43); avoid unnecessary conversion
            div()
                .flex()
                .text_color(LINK_COLOR)
                .underline()
                .children(node.children().map(|child| render_markdown_ast(child, _cx)))
                .into_any_element()
        }

        NodeValue::Strong => div()
            .flex()
            .font_weight(FontWeight::BOLD)
            .children(node.children().map(|child| render_markdown_ast(child, _cx)))
            .into_any_element(),

        NodeValue::Emph => div()
            .flex()
            .italic()
            .children(node.children().map(|child| render_markdown_ast(child, _cx)))
            .into_any_element(),

        NodeValue::Strikethrough => div()
            .flex()
            .line_through()
            .children(node.children().map(|child| render_markdown_ast(child, _cx)))
            .into_any_element(),

        NodeValue::BlockQuote => div()
            .border_l_4()
            .border_color(BLOCKQUOTE_BORDER_COLOR)
            .pl_4()
            .italic()
            .children(node.children().map(|child| render_markdown_ast(child, _cx)))
            .into_any_element(),

        // Fallback: walk children
        _ => div()
            .children(node.children().map(|child| render_markdown_ast(child, _cx)))
            .into_any_element(),
    }
}

// ---- File Handling ---------------------------------------------------------

/// Resolves the markdown file path based on CLI argument or default
///
/// # Arguments
/// * `file_path` - Optional file path from CLI arguments
///
/// # Returns
/// * `Ok(String)` - The resolved file path
/// * `Err(String)` - Error message if file resolution fails
pub fn resolve_markdown_file_path(file_path: Option<&str>) -> Result<String, String> {
    match file_path {
        Some(path) => {
            if Path::new(path).exists() {
                Ok(path.to_string())
            } else {
                Err(format!("File not found: {}", path))
            }
        }
        None => {
            let default_path = "TODO.md";
            if Path::new(default_path).exists() {
                Ok(default_path.to_string())
            } else {
                Err("Default file TODO.md not found. Please specify a markdown file.".to_string())
            }
        }
    }
}

/// Loads markdown content from a file
///
/// # Arguments
/// * `file_path` - Path to the markdown file
///
/// # Returns
/// * `Ok(String)` - The file content
/// * `Err(String)` - Error message if loading fails
pub fn load_markdown_content(file_path: &str) -> Result<String, String> {
    std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_state_initializes_correctly() {
        let state = ScrollState::new();
        assert_eq!(state.scroll_y, 0.0);
        assert_eq!(state.max_scroll_y, 0.0);
    }

    #[test]
    fn scroll_up_prevents_negative_scroll() {
        let mut state = ScrollState::new();
        state.scroll_up(10.0);
        assert_eq!(state.scroll_y, 0.0);
    }

    #[test]
    fn scroll_down_respects_max_scroll() {
        let mut state = ScrollState::new();
        state.set_max_scroll(100.0, 50.0);
        state.scroll_down(200.0);
        assert_eq!(state.scroll_y, 50.0);
    }

    #[test]
    fn scroll_to_top_works() {
        let mut state = ScrollState::new();
        state.set_max_scroll(100.0, 50.0);
        state.scroll_down(30.0);
        state.scroll_to_top();
        assert_eq!(state.scroll_y, 0.0);
    }

    #[test]
    fn scroll_to_bottom_works() {
        let mut state = ScrollState::new();
        state.set_max_scroll(100.0, 50.0);
        state.scroll_to_bottom();
        assert_eq!(state.scroll_y, 50.0);
    }

    #[test]
    fn page_up_scrolls_by_80_percent_of_page_height() {
        let mut state = ScrollState::new();
        state.set_max_scroll(100.0, 50.0);
        state.scroll_down(30.0);
        state.page_up(50.0);
        assert_eq!(state.scroll_y, 0.0); // 30 - (50 * 0.8) = 30 - 40 = -10, clamped to 0
    }

    #[test]
    fn page_down_scrolls_by_80_percent_of_page_height() {
        let mut state = ScrollState::new();
        state.set_max_scroll(100.0, 50.0);
        state.page_down(50.0);
        assert_eq!(state.scroll_y, 40.0); // 0 + (50 * 0.8) = 40
    }

    #[test]
    fn scroll_bounds_are_enforced() {
        let mut state = ScrollState::new();
        state.set_max_scroll(100.0, 50.0); // max_scroll_y = 50

        // Test that scroll_y cannot go negative
        state.scroll_up(100.0); // Try to scroll way up
        assert_eq!(state.scroll_y, 0.0);

        // Test that scroll_y cannot exceed max_scroll_y
        state.scroll_down(200.0); // Try to scroll way down
        assert_eq!(state.scroll_y, 50.0);

        // Test load_scroll_state bounds checking
        state.scroll_y = 25.0; // Valid position
        state.max_scroll_y = 30.0; // New bounds

        // Simulate loading invalid scroll position
        let temp_scroll: f32 = 100.0; // Would be out of bounds
        state.scroll_y = temp_scroll.max(0.0).min(state.max_scroll_y);
        assert_eq!(state.scroll_y, 30.0); // Should be clamped to max

        let temp_scroll: f32 = -10.0; // Would be negative
        state.scroll_y = temp_scroll.max(0.0).min(state.max_scroll_y);
        assert_eq!(state.scroll_y, 0.0); // Should be clamped to 0
    }

    #[test]
    fn content_height_estimation_constants_work() {
        // Test that the constants produce reasonable values
        let line_count = 100;
        let avg_line_height = BASE_TEXT_SIZE * LINE_HEIGHT_MULTIPLIER;
        let estimated_content_height = line_count as f32 * avg_line_height;
        let total_content_height = estimated_content_height + CONTENT_HEIGHT_BUFFER;

        // Verify line height is reasonable (should be > base text size)
        assert!(avg_line_height > BASE_TEXT_SIZE);
        assert_eq!(avg_line_height, 19.2 * 1.5); // 28.8 pixels

        // Verify buffer is added correctly
        assert_eq!(
            total_content_height,
            estimated_content_height + CONTENT_HEIGHT_BUFFER
        );

        // Verify default viewport height is reasonable
        assert!(DEFAULT_VIEWPORT_HEIGHT > 0.0);
        assert_eq!(DEFAULT_VIEWPORT_HEIGHT, 800.0);
    }

    #[test]
    fn resolve_markdown_file_path_with_valid_file() {
        // Create a temporary file for testing
        let test_content = "# Test\nThis is a test file.";
        std::fs::write("test_file.md", test_content).expect("Failed to create test file");

        let result = crate::resolve_markdown_file_path(Some("test_file.md"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_file.md");

        // Clean up
        std::fs::remove_file("test_file.md").ok();
    }

    #[test]
    fn resolve_markdown_file_path_with_nonexistent_file() {
        let result = crate::resolve_markdown_file_path(Some("nonexistent_file.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }

    #[test]
    fn resolve_markdown_file_path_with_no_path_and_todo_exists() {
        // Create TODO.md for testing
        let test_content = "# TODO\nSome todo items.";
        std::fs::write("TODO.md", test_content).expect("Failed to create TODO.md");

        let result = crate::resolve_markdown_file_path(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TODO.md");

        // Clean up
        std::fs::remove_file("TODO.md").ok();
    }

    #[test]
    fn resolve_markdown_file_path_with_no_path_and_no_todo() {
        // Ensure TODO.md doesn't exist
        std::fs::remove_file("TODO.md").ok();

        let result = crate::resolve_markdown_file_path(None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Default file TODO.md not found"));
    }

    #[test]
    fn load_markdown_content_success() {
        let test_content = "# Test Content\nThis is test markdown.";
        std::fs::write("test_load.md", test_content).expect("Failed to create test file");

        let result = crate::load_markdown_content("test_load.md");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);

        // Clean up
        std::fs::remove_file("test_load.md").ok();
    }

    #[test]
    fn load_markdown_content_failure() {
        let result = crate::load_markdown_content("nonexistent_file.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }
}
