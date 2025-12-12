use comrak::{Arena, Options, parse_document};
use gpui::{
    AsyncWindowContext, Context, FocusHandle, ImageSource, IntoElement, Render, RenderImage,
    WeakEntity, Window, actions, div, prelude::*, px,
};
use notify_debouncer_full::Debouncer;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, mpsc::Receiver};
use tokio::runtime::Runtime;
use tracing::{debug, info, warn};

use crate::config::AppConfig;
use crate::internal::events;
use crate::internal::file_handling::{load_markdown_content, resolve_image_path};
use crate::internal::file_watcher::FileWatcherEvent;
use crate::internal::image::rgba_to_bgra;
use crate::internal::image_loader::fetch_and_decode_image;
use crate::internal::rendering::render_markdown_ast_with_search;
use crate::internal::scroll::ScrollState;
use crate::internal::search::SearchState;
use crate::internal::style::{
    BLOCK_ELEMENT_SPACING, BOTTOM_SCROLL_PADDING, CHAR_WIDTH_MULTIPLIER, CONTENT_HEIGHT_SCALE,
    IMAGE_MAX_WIDTH, get_theme_colors,
};
use crate::internal::ui;

// Define search actions
actions!(search, [ToggleSearch, NextMatch, PrevMatch, ExitSearch]);

/// Estimated vertical spacing (margins + padding) applied around images in the renderer.
pub const IMAGE_VERTICAL_PADDING: f32 = 16.0;
/// Height of the placeholder shown when an image is loading or missing
/// Set to IMAGE_MAX_WIDTH (800px) to handle worst-case square images (800×800)
pub const PLACEHOLDER_HEIGHT: f32 = 800.0;
/// Container padding applied by the renderer (.pt_4() + .pb_4() = ~16px * 2)
pub const CONTAINER_PADDING: f32 = 32.0;

/// Represents different types of interactive elements that can receive keyboard focus
#[derive(Debug, Clone, PartialEq)]
pub enum FocusableElement {
    /// A clickable link with its URL
    Link(String),
    /// A TOC item with its line number
    TocItem(usize),
    /// The TOC toggle button
    TocToggleButton,
    /// Copy button for a code block (identified by code content hash)
    CopyButton(String),
    /// A bookmark item with its list index
    BookmarkItem(usize),
    /// Close button for bookmarks overlay
    BookmarksCloseButton,
}

pub enum ImageState {
    Loading,
    Loaded(ImageSource),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarkMode {
    Set,
    Jump,
}

pub struct MarkdownViewer {
    pub markdown_content: String,
    pub markdown_file_path: PathBuf,
    pub scroll_state: ScrollState,
    pub viewport_height: f32,
    pub viewport_width: f32,
    pub config: AppConfig,
    pub image_cache: HashMap<String, ImageState>,
    /// Per-image displayed heights (in pixels) used to compute content height for scrolling.
    pub image_display_heights: HashMap<String, f32>,
    pub bg_rt: Arc<Runtime>,
    /// Search state (None when search is not active)
    pub search_state: Option<SearchState>,
    /// Current search input text
    pub search_input: String,
    /// Focus handle for keyboard events
    pub focus_handle: FocusHandle,
    /// Whether to show the help overlay
    pub show_help: bool,
    /// File watcher event receiver
    pub file_watcher_rx: Option<Receiver<FileWatcherEvent>>,
    /// File watcher debouncer (must be kept alive)
    #[allow(dead_code)]
    pub file_watcher:
        Option<Debouncer<notify::RecommendedWatcher, notify_debouncer_full::FileIdMap>>,
    /// Whether the file has been deleted
    pub file_deleted: bool,
    /// Whether to show the table of contents sidebar
    pub show_toc: bool,
    /// Table of contents extracted from markdown
    pub toc: crate::internal::toc::TableOfContents,
    /// TOC sidebar scroll position
    pub toc_scroll_y: f32,
    /// TOC sidebar maximum scroll position
    pub toc_max_scroll_y: f32,
    /// Whether go-to-line dialog is active
    pub show_goto_line: bool,
    /// Current go-to-line input text
    pub goto_line_input: String,
    /// Whether to trigger PDF export
    pub trigger_pdf_export: bool,
    /// PDF export result message (Some when showing notification)
    pub pdf_export_message: Option<String>,
    /// Whether PDF export was successful (for coloring the notification)
    pub pdf_export_success: bool,
    /// Whether showing PDF overwrite confirmation
    pub show_pdf_overwrite_confirm: bool,
    /// Path of PDF to potentially overwrite
    pub pdf_overwrite_path: Option<std::path::PathBuf>,
    /// Current index in search history (None means not browsing history)
    pub search_history_index: Option<usize>,
    /// List of bookmarked line numbers
    pub bookmarks: Vec<usize>,
    /// Whether to show the bookmarks overlay
    pub show_bookmarks: bool,
    /// Message to show when search history is cleared/saved
    pub search_history_message: Option<String>,
    /// List of focusable elements found during render (for keyboard navigation)
    pub focusable_elements: Vec<FocusableElement>,
    /// Index of the currently focused element (None means no focus)
    pub current_focus_index: Option<usize>,
    /// v0.12.5: Map of marks to scroll positions
    pub marks: HashMap<char, f32>,
    /// v0.12.5: Current mark mode (Set/Jump)
    pub mark_mode: Option<MarkMode>,
    /// v0.12.5: Track if 'z' was pressed for 'zz' command
    pub z_pressed_once: bool,
    /// v0.12.5: Current help overlay page (0 = General, 1 = Navigation)
    pub help_page: usize,
}

impl MarkdownViewer {
    pub fn new(
        markdown_content: String,
        markdown_file_path: PathBuf,
        config: AppConfig,
        bg_rt: Arc<Runtime>,
        focus_handle: FocusHandle,
        file_watcher_rx: Option<Receiver<FileWatcherEvent>>,
        file_watcher: Option<
            Debouncer<notify::RecommendedWatcher, notify_debouncer_full::FileIdMap>,
        >,
    ) -> Self {
        let viewport_height = config.window.height;
        let viewport_width = config.window.width;

        // Parse markdown to generate TOC
        let arena = comrak::Arena::new();
        let mut options = comrak::Options::default();
        options.extension.table = true;
        let root = comrak::parse_document(&arena, &markdown_content, &options);
        let toc = crate::internal::toc::TableOfContents::from_ast(root);

        let mut viewer = Self {
            markdown_content,
            markdown_file_path,
            scroll_state: ScrollState::new(),
            viewport_height,
            viewport_width,
            config,
            image_cache: HashMap::new(),
            image_display_heights: HashMap::new(),
            bg_rt,
            search_state: None,
            search_input: String::new(),
            focus_handle,
            show_help: false,
            file_watcher_rx,
            file_watcher,
            file_deleted: false,
            show_toc: false,
            toc,
            toc_scroll_y: 0.0,
            toc_max_scroll_y: 0.0,
            show_goto_line: false,
            goto_line_input: String::new(),
            trigger_pdf_export: false,
            pdf_export_message: None,
            pdf_export_success: false,
            show_pdf_overwrite_confirm: false,
            pdf_overwrite_path: None,
            search_history_index: None,
            bookmarks: Vec::new(),
            show_bookmarks: false,
            search_history_message: None,
            focusable_elements: Vec::new(),
            current_focus_index: None,
            marks: HashMap::new(),
            mark_mode: None,
            z_pressed_once: false,
            help_page: 0,
        };

        viewer.recompute_max_scroll();
        viewer.compute_toc_max_scroll();
        viewer
    }

    /// Compute the maximum scroll position for the TOC sidebar
    pub fn compute_toc_max_scroll(&mut self) {
        if self.toc.entries.is_empty() {
            self.toc_max_scroll_y = 0.0;
            return;
        }

        // Each TOC entry has: 8px horizontal padding + text + 4px vertical padding (py_1)
        // Plus gap_1 (4px) between entries, and pt_4/pb_4 (16px each) for the container
        const ENTRY_HEIGHT: f32 = 30.0; // Approximate height per entry
        const CONTAINER_PADDING: f32 = 32.0; // pt_4 + pb_4

        let toc_content_height = (self.toc.entries.len() as f32) * ENTRY_HEIGHT + CONTAINER_PADDING;
        let toc_viewport_height = self.viewport_height;

        self.toc_max_scroll_y = (toc_content_height - toc_viewport_height).max(0.0);
    }

    // Calculate the estimated Y scroll position for a given byte offset
    pub fn calculate_y_for_offset(&self, target_offset: usize) -> f32 {
        if target_offset >= self.markdown_content.len() {
            return self.scroll_state.max_scroll_y;
        }

        let pre_text = &self.markdown_content[..target_offset];
        // Count newlines to know how many full lines are before the target
        let lines_to_sum = pre_text.chars().filter(|&c| c == '\n').count();

        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        let mut y = 0.0;
        let mut in_fenced_code = false;

        for (i, raw_line) in self.markdown_content.lines().enumerate() {
            if i >= lines_to_sum {
                break;
            }

            let line = raw_line.trim_start();

            // Toggle fenced code block state
            if line.starts_with("```") {
                in_fenced_code = !in_fenced_code;
            }

            let weight = match () {
                _ if in_fenced_code => 1.25,        // code_line_weight
                _ if line.starts_with('#') => 1.6,  // heading_weight
                _ if line.starts_with('>') => 1.15, // blockquote_weight
                _ => 1.0,                           // list_line_weight and normal_line_weight
            };

            y += avg_line_height * weight;

            // Rough image height estimation
            // If the line contains an image link that we have loaded, add its height
            if line.contains("![") && line.contains("](") {
                // Try to extract path (very rough)
                if let Some(start) = line.find("](")
                    && let Some(end) = line[start..].find(')')
                {
                    let path = &line[start + 2..start + end];
                    if let Some(height) = self.image_display_heights.get(path) {
                        y += height + IMAGE_VERTICAL_PADDING;
                    }
                }
            }
        }

        y
    }

    pub fn scroll_to_current_match(&mut self) {
        if let Some(m) = self.search_state.as_ref().and_then(|s| s.current_match()) {
            let y = self.calculate_y_for_offset(m.start);
            // Center the match
            let target_y = (y - self.viewport_height / 2.0).max(0.0);
            self.scroll_state.scroll_y = target_y.min(self.scroll_state.max_scroll_y);
        }
    }

    /// Calculate the Y position for a specific line number
    pub fn calculate_y_for_line(&self, line_number: usize) -> f32 {
        let (height, _, _) = self.calculate_smart_height(Some(line_number));
        // Add top padding
        height + 32.0 // CONTAINER_PADDING
    }

    /// Parse a line number from input string
    /// Returns None if the input is invalid (empty, non-numeric, zero, or negative)
    pub fn parse_line_number(input: &str) -> Option<usize> {
        input.trim().parse::<usize>().ok().filter(|&n| n > 0)
    }

    /// Validate that a line number is within bounds
    /// Returns an error message if the line number is invalid
    pub fn validate_line_number(&self, line_number: usize) -> Result<(), String> {
        let total_lines = self.markdown_content.lines().count();
        if line_number == 0 {
            return Err("Line number must be greater than 0".to_string());
        }
        if line_number > total_lines {
            return Err(format!(
                "Line number {} exceeds total lines ({})",
                line_number, total_lines
            ));
        }
        Ok(())
    }

    /// Scroll to a specific line number
    /// Centers the line in the viewport if possible
    pub fn scroll_to_line(&mut self, line_number: usize) -> Result<(), String> {
        self.validate_line_number(line_number)?;

        let target_y = self.calculate_y_for_line(line_number - 1); // Convert to 0-based
        // Center the line in the viewport
        let centered_y = (target_y - self.viewport_height / 2.0).max(0.0);
        // Directly set scroll_y for immediate scrolling (like scroll_to_top/bottom)
        self.scroll_state.scroll_y = centered_y.min(self.scroll_state.max_scroll_y);

        Ok(())
    }

    /// Get the line number corresponding to the current scroll position
    pub fn get_current_line_number(&self) -> usize {
        let scroll_y = self.scroll_state.scroll_y;
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        // Simple estimation: scroll_y / avg_line_height
        // This is rough but sufficient for bookmarks in this MVP
        // A more accurate approach would reverse calculate_y_for_line
        let line_index = (scroll_y / avg_line_height).floor() as usize;
        let total_lines = self.markdown_content.lines().count();
        (line_index + 1).min(total_lines).max(1)
    }

    /// Move focus to the next focusable element (Tab key)
    pub fn focus_next(&mut self) {
        if self.focusable_elements.is_empty() {
            self.current_focus_index = None;
            return;
        }

        self.current_focus_index = Some(match self.current_focus_index {
            None => 0,
            Some(idx) => {
                match idx.checked_add(1) {
                    Some(next) if next < self.focusable_elements.len() => next,
                    _ => 0, // Wrap around to first element
                }
            }
        });
        debug!(
            "Focus next: index {:?}/{}",
            self.current_focus_index,
            self.focusable_elements.len()
        );
    }

    /// Move focus to the previous focusable element (Shift+Tab key)
    pub fn focus_previous(&mut self) {
        if self.focusable_elements.is_empty() {
            self.current_focus_index = None;
            return;
        }

        self.current_focus_index = Some(match self.current_focus_index {
            None => self.focusable_elements.len() - 1,
            Some(idx) => {
                match idx {
                    0 => self.focusable_elements.len() - 1, // Wrap around to last element
                    _ => idx - 1,
                }
            }
        });
        debug!(
            "Focus previous: index {:?}/{}",
            self.current_focus_index,
            self.focusable_elements.len()
        );
    }

    /// Clear keyboard focus
    pub fn clear_focus(&mut self) {
        self.current_focus_index = None;
        debug!("Cleared keyboard focus");
    }

    /// Activate the currently focused element (Enter key)
    /// Returns true if an action was performed
    pub fn activate_focused_element(&mut self) -> bool {
        if let Some(idx) = self.current_focus_index
            && let Some(element) = self.focusable_elements.get(idx).cloned()
        {
            match element {
                FocusableElement::Link(url) => {
                    debug!("Activating focused link: {}", url);
                    // Open URL in browser
                    let url_clone = url.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = crate::internal::rendering::open_url(&url_clone) {
                            warn!("Failed to open URL '{}': {}", url_clone, e);
                        }
                    });
                    return true;
                }
                FocusableElement::TocItem(line_number) => {
                    debug!("Activating focused TOC item: line {}", line_number);
                    // Navigate to the line
                    let target_y = self.calculate_y_for_line(line_number);
                    self.scroll_state.scroll_y = target_y.min(self.scroll_state.max_scroll_y);
                    return true;
                }
                FocusableElement::TocToggleButton => {
                    debug!("Activating TOC toggle button");
                    self.show_toc = !self.show_toc;
                    self.recompute_max_scroll();
                    return true;
                }
                FocusableElement::CopyButton(code) => {
                    debug!("Activating copy button");
                    // Note: We can't copy to clipboard here without WindowContext
                    // This will be handled in the render method via a message
                    info!("Copy button activated for code: {} bytes", code.len());
                    return true;
                }
                FocusableElement::BookmarkItem(line_number) => {
                    debug!("Activating bookmark item: line {}", line_number);
                    let _ = self.scroll_to_line(line_number);
                    self.show_bookmarks = false;
                    return true;
                }
                FocusableElement::BookmarksCloseButton => {
                    debug!("Activating bookmarks close button");
                    self.show_bookmarks = false;
                    return true;
                }
            }
        }
        false
    }

    /// Perform PDF export and set notification message
    fn perform_pdf_export(&mut self, pdf_path: &std::path::Path) {
        debug!("PDF export triggered, output path: {:?}", pdf_path);

        // Perform export using pdf_export module with configuration
        match crate::internal::pdf_export::export_to_pdf(
            &self.markdown_content,
            pdf_path,
            &self.config.pdf_export,
        ) {
            Ok(()) => {
                info!("Successfully exported PDF to {:?}", pdf_path);
                // Show success notification
                let filename = pdf_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("output.pdf");
                self.pdf_export_message = Some(format!("✓ PDF exported: {}", filename));
                self.pdf_export_success = true;
            }
            Err(e) => {
                warn!("Failed to export PDF: {}", e);
                // Show error notification
                self.pdf_export_message = Some(format!("✗ PDF export failed: {}", e));
                self.pdf_export_success = false;
            }
        }
    }

    /// Calculates the height of the content using smart logic (wrapping, images, etc.)
    /// If stop_at_line is Some(n), returns the height up to the start of line n.
    /// Returns (height, found_image_paths, block_element_count)
    fn calculate_smart_height(
        &self,
        stop_at_line: Option<usize>,
    ) -> (f32, std::collections::HashSet<String>, usize) {
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        // Weights (multipliers relative to avg_line_height)
        let heading_weight = 1.4;
        let code_line_weight = 1.2;
        let blockquote_weight = 1.1;
        let list_line_weight = 1.0;
        let empty_line_weight = 0.25;
        let normal_line_weight = 1.0;
        let mut in_fenced_code = false;

        // Estimate wrapping for text lines
        let effective_width = match self.show_toc {
            true => self.viewport_width - crate::internal::style::TOC_WIDTH - 64.0,
            false => self.viewport_width - 64.0,
        };
        // Use conservative multiplier for variable-width fonts
        let char_width = self.config.theme.base_text_size * CHAR_WIDTH_MULTIPLIER;
        let chars_per_line = (effective_width / char_width).max(20.0);

        let mut smart_text_height = 0.0;
        let mut found_image_paths = std::collections::HashSet::new();
        let mut block_element_count: usize = 0;
        let mut prev_line_empty = true; // Track paragraph boundaries

        for (idx, raw_line) in self.markdown_content.lines().enumerate() {
            if stop_at_line.is_some_and(|stop_idx| idx >= stop_idx) {
                break;
            }

            let line = raw_line.trim_start();

            // Toggle fenced code block state
            if line.starts_with("```") {
                in_fenced_code = !in_fenced_code;
                if !in_fenced_code {
                    // End of code block = one block element
                    block_element_count += 1;
                }
                smart_text_height += avg_line_height * code_line_weight;
                continue;
            }

            if in_fenced_code {
                smart_text_height += avg_line_height * code_line_weight;
                continue;
            }

            // Robust Image Detection & Mixed Content Handling
            let mut line_text = line.to_string();
            let mut image_height_on_line = 0.0;
            let mut found_image = false;

            while let Some(start_idx) = line_text.find("![") {
                if let Some(alt_end) = line_text[start_idx..].find("](") {
                    let alt_end_idx = start_idx + alt_end;
                    if let Some(url_end) = line_text[alt_end_idx..].find(')') {
                        let url_end_idx = alt_end_idx + url_end;

                        let url_part = &line_text[alt_end_idx + 2..url_end_idx];
                        let url = url_part.split_whitespace().next().unwrap_or("").trim();

                        if !url.is_empty() {
                            let resolved_path = resolve_image_path(url, &self.markdown_file_path);

                            // Track this image path
                            found_image_paths.insert(resolved_path.clone());

                            match self.image_display_heights.get(&resolved_path) {
                                Some(&height) => {
                                    image_height_on_line += height + IMAGE_VERTICAL_PADDING;
                                }
                                None => {
                                    // Use PLACEHOLDER_HEIGHT for unloaded images
                                    image_height_on_line +=
                                        PLACEHOLDER_HEIGHT + IMAGE_VERTICAL_PADDING;
                                }
                            }
                            found_image = true;
                        }

                        line_text.replace_range(start_idx..=url_end_idx, " ");
                        continue;
                    }
                }
                break;
            }

            if found_image {
                smart_text_height += image_height_on_line;
                block_element_count += 1; // Images are block elements
            }

            // Count block elements: paragraphs (text after empty line), tables, lists
            let is_table_line = line.starts_with('|');
            let is_list_line = line.starts_with('-')
                || line.starts_with('*')
                || line.starts_with('+')
                || (line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". "));
            let is_heading = line.starts_with('#');

            if is_table_line || is_heading {
                block_element_count += 1;
            } else if !line.is_empty() && prev_line_empty && !is_list_line {
                // New paragraph (non-empty line after empty line)
                block_element_count += 1;
            }

            prev_line_empty = line.is_empty();

            let weight = match () {
                _ if line.starts_with('#') => heading_weight,
                _ if line.starts_with('>') => blockquote_weight,
                _ if line.starts_with('|') => {
                    let col_count = line.chars().filter(|c| *c == '|').count().max(2) - 1;
                    // Reduced from 0.5 to 0.15: 10 cols = 2.5x instead of 6.0x
                    1.0 + (col_count as f32 * 0.15)
                }
                _ if line.starts_with('-')
                    || line.starts_with('*')
                    || line.starts_with('+')
                    || (line.chars().next().is_some_and(|c| c.is_ascii_digit())
                        && line.contains(". ")) =>
                {
                    list_line_weight
                }
                _ if line_text.trim().is_empty() => match found_image {
                    true => 0.0,
                    false => empty_line_weight,
                },
                _ => normal_line_weight,
            };

            let trimmed_len = line_text.trim().len();
            let visual_lines = match (trimmed_len, found_image) {
                (n, _) if n > 0 => (n as f32 / chars_per_line).ceil(),
                (0, true) => 0.0,
                _ => 1.0,
            };

            smart_text_height += visual_lines * avg_line_height * weight;
        }

        (smart_text_height, found_image_paths, block_element_count)
    }

    pub fn recompute_max_scroll(&mut self) {
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        // --- Smart Logic (Current) ---
        let (smart_text_height, found_image_paths, block_count) = self.calculate_smart_height(None);

        // Apply percentage-based scaling + block element spacing
        let smart_total_height = (smart_text_height * CONTENT_HEIGHT_SCALE)
            + (block_count as f32 * BLOCK_ELEMENT_SPACING);

        // --- Legacy Logic (Old) ---
        // Simple line counts + sum of all loaded images
        let legacy_heading_weight = 1.6;
        let legacy_code_weight = 1.25;
        let legacy_quote_weight = 1.15;
        let legacy_list_weight = 1.0;
        let legacy_empty_weight = 0.25;
        let legacy_normal_weight = 1.0;

        let mut legacy_text_height = 0.0;
        let mut in_fenced_code_legacy = false;

        for raw_line in self.markdown_content.lines() {
            let line = raw_line.trim_start();

            if line.starts_with("```") {
                in_fenced_code_legacy = !in_fenced_code_legacy;
                // Old logic: fence lines had 0 height (continue without adding)
                continue;
            }

            if in_fenced_code_legacy {
                legacy_text_height += avg_line_height * legacy_code_weight;
                continue;
            }

            let weight = match () {
                _ if line.starts_with('#') => legacy_heading_weight,
                _ if line.starts_with('>') => legacy_quote_weight,
                _ if line.starts_with('-')
                    || line.starts_with('*')
                    || line.starts_with('+')
                    || (line.chars().next().is_some_and(|c| c.is_ascii_digit())
                        && line.contains(". ")) =>
                {
                    legacy_list_weight
                }
                _ if line.is_empty() => legacy_empty_weight,
                _ => legacy_normal_weight,
            };

            legacy_text_height += avg_line_height * weight;
        }

        let legacy_images_height: f32 = self
            .image_display_heights
            .values()
            .copied()
            .map(|h| h + IMAGE_VERTICAL_PADDING)
            .sum();

        let legacy_total_height = legacy_text_height + legacy_images_height;

        // --- Dynamic Image Buffer ---
        // Count how many images we found vs how many have loaded heights
        let total_images_found = found_image_paths.len();
        let loaded_images_count = found_image_paths
            .iter()
            .filter(|path| self.image_display_heights.contains_key(*path))
            .count();
        let unloaded_images_count = total_images_found.saturating_sub(loaded_images_count);

        // Add 500px per unloaded image (reasonable average height for typical web images)
        // This is more conservative than PLACEHOLDER_HEIGHT but less than IMAGE_MAX_WIDTH
        let unloaded_image_buffer = (unloaded_images_count as f32) * 500.0;

        // --- Hybrid Result ---
        // Use the maximum of the two estimates + container padding + base buffer + unloaded image buffer
        // TODO: Revisit this hybrid calculation logic. It's a temporary fix to ensure
        // both text-heavy (wrapping) and image-heavy (legacy) files scroll correctly.
        // Ideally, we should have a single unified logic that handles all cases perfectly.

        debug!(
            "Scroll calculation: smart={:.1}px (scaled), legacy={:.1}px, blocks={}, images={}/{} loaded, unloaded_buffer={:.1}px",
            smart_total_height,
            legacy_total_height,
            block_count,
            loaded_images_count,
            total_images_found,
            unloaded_image_buffer
        );

        // Content height = max(smart, legacy) + container padding + image buffer + safety margin
        // Note: smart_total_height already includes scaling (8%) and block element spacing
        let content_height = f32::max(smart_total_height, legacy_total_height)
            + CONTAINER_PADDING
            + unloaded_image_buffer
            + BOTTOM_SCROLL_PADDING;

        self.scroll_state
            .set_max_scroll(content_height, self.viewport_height);
    }

    pub fn load_image(&mut self, path: String, window: &Window, cx: &mut Context<Self>) {
        if self.image_cache.contains_key(&path) {
            return;
        }

        self.image_cache.insert(path.clone(), ImageState::Loading);
        let path_for_load = path.clone();
        let path_for_update = path.clone();
        let bg_rt = self.bg_rt.clone();

        // Spawn a gpui background task which delegatesthe network + decode work to the dedicated Tokio runtime.
        cx.spawn_in(
            window,
            move |this: WeakEntity<MarkdownViewer>, cx: &mut AsyncWindowContext| {
                let mut cx = cx.clone();
                let bg_rt = bg_rt.clone();
                async move {
                    // Spawn the network+decode job on the background runtime.
                    // The background job returns Result<image::DynamicImage, anyhow::Error>.
                    let join_handle = bg_rt.spawn(async move {
                        // Delegate fetching + decoding to the centralized image_loader helper.
                        // This keeps main UI code small and moves network/fallback logic into an internal module.
                        fetch_and_decode_image(&path_for_load).await
                    });

                    // Await the join handle produced by the background runtime.
                    let join_result = join_handle.await;

                    // Update gpui state on the UI context thread.
                    this.update(&mut cx, |this, cx| match join_result {
                        Ok(Ok(dyn_img)) => {
                            // Successfully decoded image into DynamicImage. Convert to RGBA and create RenderImage.
                            let mut rgba = dyn_img.into_rgba8();

                            // GPUI on macOS expects BGRA format, but image crate produces RGBA.
                            // Convert RGBA -> BGRA before passing to GPUI
                            rgba_to_bgra(&mut rgba);

                            let orig_w = rgba.width() as f32;
                            let orig_h = rgba.height() as f32;
                            // Compute displayed width constrained by IMAGE_MAX_WIDTH (same as rendering).
                            let displayed_w = match orig_w > IMAGE_MAX_WIDTH {
                                true => IMAGE_MAX_WIDTH,
                                false => orig_w,
                            };
                            // Maintain aspect ratio for displayed height
                            let displayed_h = match orig_w {
                                w if w > 0.0 => (displayed_w / w) * orig_h,
                                _ => orig_h,
                            };

                            let frame = image::Frame::new(rgba);
                            let render_image = RenderImage::new(vec![frame]);
                            let arc_img = Arc::new(render_image);

                            debug!("Successfully loaded image: {}", path_for_update);
                            this.image_cache.insert(
                                path_for_update.clone(),
                                ImageState::Loaded(ImageSource::Render(arc_img.clone())),
                            );
                            this.image_display_heights
                                .insert(path_for_update.clone(), displayed_h);
                            // Recompute scroll bounds now that an image height is known
                            this.recompute_max_scroll();
                            cx.notify();
                        }
                        Ok(Err(e)) => {
                            debug!("Failed to load image '{}': {}", path_for_update, e);
                            this.image_cache
                                .insert(path_for_update.clone(), ImageState::Error);
                            this.image_display_heights.remove(&path_for_update);
                        }
                        Err(join_err) => {
                            debug!(
                                "Image task join error for '{}': {}",
                                path_for_update, join_err
                            );
                            this.image_cache
                                .insert(path_for_update.clone(), ImageState::Error);
                            this.image_display_heights.remove(&path_for_update);
                        }
                    })
                    .ok();
                }
            },
        )
        .detach();
    }

    /// Collect all links from a markdown AST node and add them to focusable_elements
    fn collect_links_from_ast<'a>(&mut self, node: &'a comrak::nodes::AstNode<'a>) {
        use comrak::nodes::NodeValue;

        if let NodeValue::Link(link) = &node.data.borrow().value
            && !link.url.trim().is_empty()
        {
            self.focusable_elements
                .push(FocusableElement::Link(link.url.clone()));
        }

        for child in node.children() {
            self.collect_links_from_ast(child);
        }
    }
}

impl Render for MarkdownViewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Clear focusable elements list - will be rebuilt during this render pass
        self.focusable_elements.clear();

        // Poll file watcher for events (non-blocking)
        // Collect events first to avoid borrow checker issues
        let mut events = Vec::new();
        if let Some(rx) = &self.file_watcher_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }

        // Process collected events
        for event in events {
            match event {
                FileWatcherEvent::Modified => {
                    info!("File modified, reloading: {:?}", self.markdown_file_path);
                    // Save current scroll position
                    let saved_scroll_y = self.scroll_state.scroll_y;

                    // Reload file content
                    match self.markdown_file_path.to_str() {
                        Some(path_str) => {
                            match load_markdown_content(path_str) {
                                Ok(new_content) => {
                                    self.markdown_content = new_content;

                                    // Regenerate TOC
                                    let arena = comrak::Arena::new();
                                    let mut options = comrak::Options::default();
                                    options.extension.table = true;
                                    let root = comrak::parse_document(
                                        &arena,
                                        &self.markdown_content,
                                        &options,
                                    );
                                    self.toc =
                                        crate::internal::toc::TableOfContents::from_ast(root);

                                    // Clear image cache as images may have changed
                                    self.image_cache.clear();
                                    self.image_display_heights.clear();
                                    // Restore scroll position
                                    self.scroll_state.scroll_y = saved_scroll_y;
                                    self.recompute_max_scroll();
                                    self.compute_toc_max_scroll();
                                    // Clear file deleted flag if it was set
                                    self.file_deleted = false;
                                    info!("File reloaded successfully");
                                }
                                Err(e) => {
                                    warn!("Failed to reload file: {}", e);
                                }
                            }
                        }
                        None => {
                            warn!(
                                "Failed to convert path to string: {:?}",
                                self.markdown_file_path
                            );
                        }
                    }
                    cx.notify();
                }
                FileWatcherEvent::Deleted => {
                    info!("File deleted: {:?}", self.markdown_file_path);
                    self.file_deleted = true;
                    cx.notify();
                }
                FileWatcherEvent::Error(err) => {
                    warn!("File watcher error: {}", err);
                }
            }
        }

        // Update viewport dimensions if changed
        let viewport_size = window.viewport_size();
        let current_height_f32 = f32::from(viewport_size.height);
        let current_width_f32 = f32::from(viewport_size.width);

        if (current_height_f32 - self.viewport_height).abs() > 1.0 {
            self.viewport_height = current_height_f32;
            self.recompute_max_scroll();
        }

        if (current_width_f32 - self.viewport_width).abs() > 1.0 {
            self.viewport_width = current_width_f32;
            self.recompute_max_scroll();
        }

        let arena = Arena::new();
        let mut options = Options::default();
        options.extension.table = true; // Enable GFM tables
        let root = parse_document(&arena, &self.markdown_content, &options);

        // Collect all links from the markdown AST for keyboard navigation
        self.collect_links_from_ast(root);

        debug!("AST parsing complete");
        let mut missing_images = HashSet::new();
        let theme_colors = get_theme_colors(&self.config.theme.theme);
        let element = div()
            .track_focus(&self.focus_handle)
            .flex()
            .relative() // Ensure absolute children are positioned relative to this container
            .size_full()
            .bg(theme_colors.bg_color)
            .text_color(theme_colors.text_color)
            .font_family(self.config.theme.primary_font.clone())
            .text_size(px(self.config.theme.base_text_size))
            // New: Event handlers for scrolling
            .on_mouse_move(cx.listener(|this, _, _, cx| {
                // Use viewport height from config
                if this.viewport_height == 0.0 {
                    this.viewport_height = this.config.window.height;
                    this.recompute_max_scroll();
                }
                cx.notify();
            }))
            // Search action handlers
            .on_action(cx.listener(|this, _: &ToggleSearch, _, cx| {
                debug!("ToggleSearch action triggered");
                match this.search_state.take() {
                    Some(_) => {
                        // Exit search mode
                        debug!("Exiting search mode");
                        this.search_input.clear();
                    }
                    None => {
                        // Enter search mode
                        debug!("Entering search mode");
                        this.search_state =
                            Some(SearchState::new(String::new(), &this.markdown_content));
                    }
                }
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NextMatch, _, cx| {
                debug!("NextMatch action triggered");
                if let Some(state) = &mut this.search_state {
                    state.next_match();
                    debug!("Next match: {:?}", state.current_match_number());
                    this.scroll_to_current_match();
                }
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PrevMatch, _, cx| {
                debug!("PrevMatch action triggered");
                if let Some(state) = &mut this.search_state {
                    state.prev_match();
                    debug!("Previous match: {:?}", state.current_match_number());
                    this.scroll_to_current_match();
                }
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ExitSearch, _, cx| {
                debug!("ExitSearch action triggered");
                this.search_state = None;
                this.search_input.clear();
                cx.notify();
            }))
            .on_key_down(cx.listener(events::handle_key_down))
            .on_scroll_wheel(cx.listener(events::handle_scroll_wheel))
            .child(
                div().flex().size_full().overflow_hidden().child(
                    div()
                        .flex_col()
                        .w_full()
                        .pt_4()
                        .pr(match self.show_toc {
                            true => px(crate::internal::style::TOC_WIDTH + 32.0),
                            false => px(32.0),
                        })
                        .pb_4()
                        .pl_8()
                        .relative()
                        .top(px(-self.scroll_state.scroll_y))
                        .child(render_markdown_ast_with_search(
                            root,
                            Some(&self.markdown_file_path),
                            self.search_state.as_ref(),
                            match self.show_toc {
                                true => {
                                    self.viewport_width - crate::internal::style::TOC_WIDTH - 64.0
                                }
                                false => self.viewport_width - 64.0,
                            },
                            theme_colors,
                            cx,
                            &mut |path: &str| match self.image_cache.get(path) {
                                Some(ImageState::Loaded(src)) => Some(src.clone()),
                                None => {
                                    missing_images.insert(path.to_string());
                                    None
                                }
                                _ => None,
                            },
                            self.current_focus_index
                                .and_then(|idx| self.focusable_elements.get(idx)),
                        )),
                ),
            )
            // Interactive Status Bar
            .child(ui::render_status_bar(self, theme_colors, cx));

        // Add search indicator overlay if search is active
        let element = match ui::render_search_overlay(self) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // Add go-to-line overlay if active
        let element = match ui::render_goto_line_overlay(self) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // Bookmarks Overlay
        let element = match ui::render_bookmarks_overlay(self, theme_colors, cx) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // Help Overlay
        let element = match ui::render_help_overlay(self, theme_colors) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // File Deleted Overlay
        let element = match ui::render_file_deleted_overlay(self) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // PDF Export Notification Overlay
        let element = match ui::render_pdf_export_overlay(self, theme_colors) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // Search History Notification Overlay
        let element = match ui::render_search_history_notification(self, theme_colors, cx) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // PDF Overwrite Confirmation Overlay
        let element = match ui::render_pdf_overwrite_confirm(self, theme_colors) {
            Some(overlay) => element.child(overlay),
            None => element,
        };

        // TOC Sidebar
        let element = match ui::render_toc_sidebar(self, theme_colors, cx) {
            Some(sidebar) => element.child(sidebar),
            None => element,
        };

        // TOC Toggle Button
        let element = element.child(ui::render_toc_toggle_button(self, cx));

        for path in missing_images {
            self.load_image(path, window, cx);
        }

        // Handle PDF export trigger
        if self.trigger_pdf_export {
            self.trigger_pdf_export = false;

            // Generate output path from markdown file path
            let pdf_path = self.markdown_file_path.with_extension("pdf");

            // Check if file already exists
            match pdf_path.exists() {
                true => {
                    // Show confirmation prompt
                    debug!(
                        "PDF file already exists, prompting for confirmation: {:?}",
                        pdf_path
                    );
                    self.show_pdf_overwrite_confirm = true;
                    self.pdf_overwrite_path = Some(pdf_path);
                    cx.notify();
                }
                false => {
                    // File doesn't exist, export directly
                    self.perform_pdf_export(&pdf_path);
                    cx.notify();
                }
            }
        }

        // Handle PDF overwrite confirmation
        if let Some(pdf_path) = self.pdf_overwrite_path.clone()
            && !self.show_pdf_overwrite_confirm
        {
            // User confirmed, perform export
            self.perform_pdf_export(&pdf_path);
            self.pdf_overwrite_path = None;
            cx.notify();
        }

        element
    }
}
