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
use crate::internal::style::{BG_COLOR, IMAGE_MAX_WIDTH, TEXT_COLOR};
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

pub enum ImageState {
    Loading,
    Loaded(ImageSource),
    Error,
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

            let weight = if in_fenced_code {
                1.25 // code_line_weight
            } else if line.starts_with('#') {
                1.6 // heading_weight
            } else if line.starts_with('>') {
                1.15 // blockquote_weight
            } else {
                1.0 // list_line_weight and normal_line_weight
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
        let (height, _) = self.calculate_smart_height(Some(line_number));
        // Add top padding
        height + 32.0 // CONTAINER_PADDING
    }

    /// Calculates the height of the content using smart logic (wrapping, images, etc.)
    /// If stop_at_line is Some(n), returns the height up to the start of line n.
    /// Returns (height, found_image_paths)
    fn calculate_smart_height(
        &self,
        stop_at_line: Option<usize>,
    ) -> (f32, std::collections::HashSet<String>) {
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
        let effective_width = if self.show_toc {
            self.viewport_width - crate::internal::style::TOC_WIDTH - 64.0
        } else {
            self.viewport_width - 64.0
        };
        let char_width = self.config.theme.base_text_size * 0.35;
        let chars_per_line = (effective_width / char_width).max(20.0);

        let mut smart_text_height = 0.0;
        let mut found_image_paths = std::collections::HashSet::new();

        for (idx, raw_line) in self.markdown_content.lines().enumerate() {
            if stop_at_line.is_some_and(|stop_idx| idx >= stop_idx) {
                break;
            }

            let line = raw_line.trim_start();

            // Toggle fenced code block state
            if line.starts_with("```") {
                in_fenced_code = !in_fenced_code;
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

                            if let Some(&height) = self.image_display_heights.get(&resolved_path) {
                                image_height_on_line += height + IMAGE_VERTICAL_PADDING;
                            } else {
                                // Use PLACEHOLDER_HEIGHT for unloaded images
                                image_height_on_line += PLACEHOLDER_HEIGHT + IMAGE_VERTICAL_PADDING;
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
            }

            let weight = if line.starts_with('#') {
                heading_weight
            } else if line.starts_with('>') {
                blockquote_weight
            } else if line.starts_with('|') {
                let col_count = line.chars().filter(|c| *c == '|').count().max(2) - 1;
                // Reduced from 0.5 to 0.15: 10 cols = 2.5x instead of 6.0x
                1.0 + (col_count as f32 * 0.15)
            } else if line.starts_with('-')
                || line.starts_with('*')
                || line.starts_with('+')
                || (line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". "))
            {
                list_line_weight
            } else if line_text.trim().is_empty() {
                if found_image { 0.0 } else { empty_line_weight }
            } else {
                normal_line_weight
            };

            let trimmed_len = line_text.trim().len();
            let visual_lines = if trimmed_len > 0 {
                (trimmed_len as f32 / chars_per_line).ceil()
            } else if found_image {
                0.0
            } else {
                1.0
            };

            smart_text_height += visual_lines * avg_line_height * weight;
        }

        (smart_text_height, found_image_paths)
    }

    pub fn recompute_max_scroll(&mut self) {
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        // --- Smart Logic (Current) ---
        let (smart_text_height, found_image_paths) = self.calculate_smart_height(None);

        let smart_total_height = smart_text_height;

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

            let weight = if line.starts_with('#') {
                legacy_heading_weight
            } else if line.starts_with('>') {
                legacy_quote_weight
            } else if line.starts_with('-')
                || line.starts_with('*')
                || line.starts_with('+')
                || (line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". "))
            {
                legacy_list_weight
            } else if line.is_empty() {
                legacy_empty_weight
            } else {
                legacy_normal_weight
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
            "Scroll calculation: smart={:.1}px, legacy={:.1}px, images={}/{} loaded, unloaded_buffer={:.1}px, using={:.1}px",
            smart_total_height,
            legacy_total_height,
            loaded_images_count,
            total_images_found,
            unloaded_image_buffer,
            f32::max(smart_total_height, legacy_total_height)
        );

        let content_height = f32::max(smart_total_height, legacy_total_height)
            + CONTAINER_PADDING
            + self.config.theme.content_height_buffer
            + unloaded_image_buffer;

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
                            let displayed_w = if orig_w > IMAGE_MAX_WIDTH {
                                IMAGE_MAX_WIDTH
                            } else {
                                orig_w
                            };
                            // Maintain aspect ratio for displayed height
                            let displayed_h = if orig_w > 0.0 {
                                (displayed_w / orig_w) * orig_h
                            } else {
                                orig_h
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
}

impl Render for MarkdownViewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                    if let Some(path_str) = self.markdown_file_path.to_str() {
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
                                self.toc = crate::internal::toc::TableOfContents::from_ast(root);

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
                    } else {
                        warn!(
                            "Failed to convert path to string: {:?}",
                            self.markdown_file_path
                        );
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

        let mut missing_images = HashSet::new();
        let element = div()
            .track_focus(&self.focus_handle)
            .flex()
            .relative() // Ensure absolute children are positioned relative to this container
            .size_full()
            .bg(BG_COLOR)
            .text_color(TEXT_COLOR)
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
                if this.search_state.is_some() {
                    // Exit search mode
                    debug!("Exiting search mode");
                    this.search_state = None;
                    this.search_input.clear();
                } else {
                    // Enter search mode
                    debug!("Entering search mode");
                    this.search_state =
                        Some(SearchState::new(String::new(), &this.markdown_content));
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
                        .pr(if self.show_toc {
                            px(crate::internal::style::TOC_WIDTH + 32.0)
                        } else {
                            px(32.0)
                        })
                        .pb_4()
                        .pl_8()
                        .relative()
                        .top(px(-self.scroll_state.scroll_y))
                        .child(render_markdown_ast_with_search(
                            root,
                            Some(&self.markdown_file_path),
                            self.search_state.as_ref(),
                            if self.show_toc {
                                self.viewport_width - crate::internal::style::TOC_WIDTH - 64.0
                            } else {
                                self.viewport_width - 64.0
                            },
                            cx,
                            &mut |path| {
                                if let Some(ImageState::Loaded(src)) = self.image_cache.get(path) {
                                    Some(src.clone())
                                } else {
                                    if !self.image_cache.contains_key(path) {
                                        missing_images.insert(path.to_string());
                                    }
                                    None
                                }
                            },
                        )),
                ),
            )
            // Version Badge
            .child(ui::render_version_badge());

        // Add search indicator overlay if search is active
        let element = if let Some(overlay) = ui::render_search_overlay(self) {
            element.child(overlay)
        } else {
            element
        };

        // Help Overlay
        let element = if let Some(overlay) = ui::render_help_overlay(self) {
            element.child(overlay)
        } else {
            element
        };

        // File Deleted Overlay
        let element = if let Some(overlay) = ui::render_file_deleted_overlay(self) {
            element.child(overlay)
        } else {
            element
        };

        // TOC Sidebar
        let element = if let Some(sidebar) = ui::render_toc_sidebar(self, cx) {
            element.child(sidebar)
        } else {
            element
        };

        // TOC Toggle Button
        let element = element.child(ui::render_toc_toggle_button(self, cx));

        for path in missing_images {
            self.load_image(path, window, cx);
        }

        element
    }
}
