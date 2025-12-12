//! Scroll state management for the markdown viewer
//!
//! This module handles all scrolling logic including smooth scrolling,
//! bounds checking, and scroll state persistence.

use tracing::trace;

/// Scroll state for the markdown viewer
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

    /// Scroll up by the specified amount, ensuring we don't go negative
    pub fn scroll_up(&mut self, amount: f32) {
        trace!("Scrolling up by {}", amount);
        self.scroll_y = (self.scroll_y - amount).max(0.0);
    }

    /// Scroll down by the specified amount, respecting max scroll
    pub fn scroll_down(&mut self, amount: f32) {
        trace!("Scrolling down by {}", amount);
        self.scroll_y = (self.scroll_y + amount).min(self.max_scroll_y);
    }

    /// Scroll up by one page (80% of viewport height)
    pub fn page_up(&mut self, viewport_height: f32) {
        let page_amount = viewport_height * 0.8;
        self.scroll_up(page_amount);
    }

    /// Scroll down by one page (80% of viewport height)
    pub fn page_down(&mut self, viewport_height: f32) {
        let page_amount = viewport_height * 0.8;
        self.scroll_down(page_amount);
    }

    /// Scroll to the top of the document
    pub fn scroll_to_top(&mut self) {
        self.scroll_y = 0.0;
    }

    /// Scroll to the bottom of the document
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_y = self.max_scroll_y;
    }

    /// Set the maximum scroll position based on content and viewport height
    pub fn set_max_scroll(&mut self, content_height: f32, viewport_height: f32) {
        self.max_scroll_y = (content_height - viewport_height).max(0.0);
        // Clamp current scroll to new bounds
        self.scroll_y = self.scroll_y.min(self.max_scroll_y);
    }

    /// Re-clamp the current scroll position to valid bounds
    pub fn reclamp(&mut self) {
        self.scroll_y = self.scroll_y.max(0.0).min(self.max_scroll_y);
    }

    /// Smooth scroll to a target position
    pub fn smooth_scroll_to(&mut self, target: f32) {
        self.target_scroll_y = target.clamp(0.0, self.max_scroll_y);
    }

    /// Update smooth scrolling animation
    pub fn update_smooth_scroll(&mut self, delta_time: f32) {
        let diff = self.target_scroll_y - self.scroll_y;
        match diff.abs() {
            x if x > 0.1 => {
                let lerp_factor = (delta_time * 10.0).min(1.0);
                self.scroll_y += diff * lerp_factor;
            }
            _ => {
                self.scroll_y = self.target_scroll_y;
            }
        }
    }

    /// Start dragging the scroll thumb
    pub fn start_drag(&mut self, mouse_y: f32) {
        self.is_dragging = true;
        self.drag_start_y = mouse_y;
        self.drag_start_scroll = self.scroll_y;
    }

    /// Stop dragging the scroll thumb
    pub fn stop_drag(&mut self) {
        self.is_dragging = false;
    }

    /// Update scroll position based on drag
    pub fn update_drag(&mut self, mouse_y: f32, viewport_height: f32) {
        if self.is_dragging && self.max_scroll_y > 0.0 {
            let drag_delta = mouse_y - self.drag_start_y;
            let scroll_delta = (drag_delta / viewport_height) * self.max_scroll_y;
            let new_scroll = (self.drag_start_scroll + scroll_delta).clamp(0.0, self.max_scroll_y);
            self.smooth_scroll_to(new_scroll);
        }
    }

    /// Save scroll state to a file
    pub fn save_scroll_state(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = format!(
            "scroll_y: {}\ntarget_scroll_y: {}\nmax_scroll_y: {}",
            self.scroll_y, self.target_scroll_y, self.max_scroll_y
        );
        std::fs::write(file_path, content)?;
        Ok(())
    }

    /// Load scroll state from a file
    pub fn load_scroll_state(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        for line in content.lines() {
            if let Some((key, value)) = line.split_once(": ")
                && let Ok(val) = value.parse::<f32>()
            {
                match key {
                    "scroll_y" => self.scroll_y = val.max(0.0).min(self.max_scroll_y),
                    "target_scroll_y" => self.target_scroll_y = val.max(0.0).min(self.max_scroll_y),
                    "max_scroll_y" => self.max_scroll_y = val.max(0.0),
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
