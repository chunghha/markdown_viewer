use anyhow::{Context, Result};
use clap::Parser;
use comrak::{Arena, Options, parse_document};
use gpui::{
    App, Application, AsyncWindowContext, Context as GpuiContext, ImageSource, IntoElement,
    KeyDownEvent, Render, RenderImage, ScrollWheelEvent, WeakEntity, Window, WindowOptions,
    actions, div, prelude::*, px,
};
use markdown_viewer::fetch_and_decode_image;
use markdown_viewer::{
    BG_COLOR, IMAGE_MAX_WIDTH, ScrollState, TEXT_COLOR, config::AppConfig, load_markdown_content,
    resolve_markdown_file_path,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Define search actions
actions!(search, [ToggleSearch, NextMatch, PrevMatch, ExitSearch]);
use tracing::{debug, info, warn};

/// Estimated vertical spacing (margins + padding) applied around images in the renderer.
/// Tunable constant to better match gpui layout spacing. Reduced from earlier 24.0 to 16.0.
const IMAGE_VERTICAL_PADDING: f32 = 16.0;

enum ImageState {
    Loading,
    Loaded(ImageSource),
    Error,
}

#[derive(Parser)]
#[command(name = "markdown_viewer")]
#[command(about = "A simple markdown viewer")]
struct Args {
    /// Path to the markdown file to view
    file: Option<String>,
}

struct MarkdownViewer {
    markdown_content: String,
    markdown_file_path: PathBuf,
    scroll_state: ScrollState,
    viewport_height: f32,
    config: AppConfig,
    image_cache: HashMap<String, ImageState>,
    /// Per-image displayed heights (in pixels) used to compute content height for scrolling.
    image_display_heights: HashMap<String, f32>,
    bg_rt: Arc<Runtime>,
    /// Search state (None when search is not active)
    #[allow(dead_code)]
    search_state: Option<markdown_viewer::SearchState>,
    /// Current search input text
    #[allow(dead_code)]
    search_input: String,
    /// Focus handle for keyboard events
    focus_handle: gpui::FocusHandle,
}

impl MarkdownViewer {
    // New: Method to recompute max scroll based on content, viewport and loaded image heights
    //
    // Improved text-height estimation:
    // - Headings lines (lines that start with '#') are heavier (larger font / spacing).
    // - Fenced code blocks count as code lines and have a slightly larger per-line height.
    // - Blockquotes ('>') are slightly larger.
    // - Lists are treated similar to normal lines but could be adjusted independently.
    // This remains a fast heuristic (no full layout pass) but reduces large under/over estimates.
    // Calculate the estimated Y scroll position for a given byte offset
    fn calculate_y_for_offset(&self, target_offset: usize) -> f32 {
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

    fn scroll_to_current_match(&mut self) {
        if let Some(m) = self.search_state.as_ref().and_then(|s| s.current_match()) {
            let y = self.calculate_y_for_offset(m.start);
            // Center the match
            let target_y = (y - self.viewport_height / 2.0).max(0.0);
            self.scroll_state.scroll_y = target_y.min(self.scroll_state.max_scroll_y);
        }
    }

    fn recompute_max_scroll(&mut self) {
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        // Heuristic counts
        let mut heading_lines: usize = 0;
        let mut code_block_lines: usize = 0;
        let mut blockquote_lines: usize = 0;
        let mut list_lines: usize = 0;
        let mut empty_lines: usize = 0;
        let mut other_lines: usize = 0;

        let mut in_fenced_code = false;

        for raw_line in self.markdown_content.lines() {
            let line = raw_line.trim_start();

            // Toggle fenced code block state on lines starting with ```
            if line.starts_with("```") {
                in_fenced_code = !in_fenced_code;
                // Count the fence line as part of code block header (small impact)
                continue;
            }

            if in_fenced_code {
                code_block_lines += 1;
                continue;
            }

            if line.starts_with('#') {
                heading_lines += 1;
            } else if line.starts_with('>') {
                blockquote_lines += 1;
            } else if line.starts_with('-')
                || line.starts_with('*')
                || line.starts_with('+')
                || line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                    && line.contains(". ")
            {
                list_lines += 1;
            } else if line.is_empty() {
                empty_lines += 1;
            } else {
                other_lines += 1;
            }
        }

        // Weights (multipliers relative to avg_line_height)
        let heading_weight = 1.6; // headings are larger and have extra spacing
        let code_line_weight = 1.25; // fenced code lines take slightly more vertical space
        let blockquote_weight = 1.15;
        let list_line_weight = 1.0;
        let empty_line_weight = 0.25; // Empty lines are usually just small gaps
        let normal_line_weight = 1.0;

        let estimated_text_height = (heading_lines as f32 * avg_line_height * heading_weight)
            + (code_block_lines as f32 * avg_line_height * code_line_weight)
            + (blockquote_lines as f32 * avg_line_height * blockquote_weight)
            + (list_lines as f32 * avg_line_height * list_line_weight)
            + (empty_lines as f32 * avg_line_height * empty_line_weight)
            + (other_lines as f32 * avg_line_height * normal_line_weight);

        // Sum the displayed heights of all loaded images (if known),
        // including a small per-image padding to match visual spacing.
        let images_total_height: f32 = self
            .image_display_heights
            .values()
            .copied()
            .map(|h| h + IMAGE_VERTICAL_PADDING)
            .sum();

        // Combine text + images + buffer
        let content_height =
            estimated_text_height + images_total_height + self.config.theme.content_height_buffer;

        self.scroll_state
            .set_max_scroll(content_height, self.viewport_height);
    }

    fn load_image(&mut self, path: String, window: &Window, cx: &mut GpuiContext<Self>) {
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
                            markdown_viewer::rgba_to_bgra(&mut rgba);

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
    fn render(&mut self, window: &mut Window, cx: &mut GpuiContext<Self>) -> impl IntoElement {
        // Update viewport height if changed
        let current_height = window.viewport_size().height;
        let current_height_f32 = f32::from(current_height);

        if (current_height_f32 - self.viewport_height).abs() > 1.0 {
            self.viewport_height = current_height_f32;
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
                    this.search_state = Some(markdown_viewer::SearchState::new(
                        String::new(),
                        &this.markdown_content,
                    ));
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
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                let arrow_increment = this.config.scroll.arrow_key_increment;
                let page_percent = this.config.scroll.page_scroll_percentage;
                let space_percent = this.config.scroll.space_scroll_percentage;

                // Debug: log all key events
                debug!(
                    "Key pressed: '{}', platform: {}, control: {}, shift: {}, alt: {}",
                    event.keystroke.key,
                    event.keystroke.modifiers.platform,
                    event.keystroke.modifiers.control,
                    event.keystroke.modifiers.shift,
                    event.keystroke.modifiers.alt
                );

                // Check for Cmd+F (macOS) or Ctrl+F (other platforms) to toggle search
                if event.keystroke.key.as_str() == "f"
                    && (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
                {
                    debug!("Search shortcut triggered (Cmd/Ctrl+F)");
                    if this.search_state.is_some() {
                        // Exit search mode
                        debug!("Exiting search mode");
                        this.search_state = None;
                        this.search_input.clear();
                    } else {
                        // Enter search mode
                        debug!("Entering search mode");
                        this.search_state = Some(markdown_viewer::SearchState::new(
                            String::new(),
                            &this.markdown_content,
                        ));
                    }
                    cx.notify();
                    return;
                }

                // Handle global shortcuts (Cmd+T, Cmd+B, Cmd+Q)
                if event.keystroke.modifiers.platform {
                    match event.keystroke.key.as_str() {
                        "t" => {
                            debug!("Scroll to top (Cmd+T)");
                            this.scroll_state.scroll_to_top();
                            cx.notify();
                            return;
                        }
                        "b" => {
                            debug!("Scroll to bottom (Cmd+B)");
                            this.scroll_state.scroll_to_bottom();
                            cx.notify();
                            return;
                        }
                        "q" => {
                            debug!("Quit application (Cmd+Q)");
                            cx.quit();
                            return;
                        }
                        _ => {}
                    }
                }

                // Handle search mode input
                if this.search_state.is_some() {
                    match event.keystroke.key.as_str() {
                        "escape" => {
                            // Exit search mode
                            debug!("Exiting search mode (Escape)");
                            this.search_state = None;
                            this.search_input.clear();
                            cx.notify();
                            return;
                        }
                        "enter" if event.keystroke.modifiers.shift => {
                            // Previous match
                            if let Some(state) = &mut this.search_state {
                                state.prev_match();
                                debug!(
                                    "Previous match (key_down): {:?}",
                                    state.current_match_number()
                                );
                                this.scroll_to_current_match();
                            }
                            cx.notify();
                            return;
                        }
                        "enter" => {
                            // Next match
                            if let Some(state) = &mut this.search_state {
                                state.next_match();
                                debug!("Next match (key_down): {:?}", state.current_match_number());
                                this.scroll_to_current_match();
                            }
                            cx.notify();
                            return;
                        }
                        "backspace" => {
                            // Remove last character
                            this.search_input.pop();
                            this.search_state = Some(markdown_viewer::SearchState::new(
                                this.search_input.clone(),
                                &this.markdown_content,
                            ));
                            debug!("Search query: '{}'", this.search_input);
                            this.scroll_to_current_match();
                            cx.notify();
                            return;
                        }
                        key if key.len() == 1
                            && !event.keystroke.modifiers.control
                            && !event.keystroke.modifiers.platform =>
                        {
                            // Add character to search
                            this.search_input.push_str(key);
                            this.search_state = Some(markdown_viewer::SearchState::new(
                                this.search_input.clone(),
                                &this.markdown_content,
                            ));
                            debug!("Search query: '{}'", this.search_input);
                            this.scroll_to_current_match();
                            cx.notify();
                            return;
                        }
                        _ => {}
                    }
                }

                match event.keystroke.key.as_str() {
                    "up" => this.scroll_state.scroll_up(arrow_increment),
                    "down" => this.scroll_state.scroll_down(arrow_increment),
                    "pageup" => this
                        .scroll_state
                        .page_up(this.viewport_height * page_percent),
                    "pagedown" => this
                        .scroll_state
                        .page_down(this.viewport_height * page_percent),
                    "home" => this.scroll_state.scroll_to_top(),
                    "end" => this.scroll_state.scroll_to_bottom(),
                    "space" if event.keystroke.modifiers.shift => this
                        .scroll_state
                        .page_up(this.viewport_height * space_percent),
                    "space" => this
                        .scroll_state
                        .page_down(this.viewport_height * space_percent),
                    _ => {}
                }
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                let delta = event
                    .delta
                    .pixel_delta(px(this.config.theme.base_text_size))
                    .y;
                let delta_f32: f32 = delta.into();
                if delta_f32 > 0.0 {
                    this.scroll_state.scroll_up(delta_f32);
                } else {
                    this.scroll_state.scroll_down(-delta_f32);
                }
                cx.notify();
            }))
            .child(
                div().flex().size_full().overflow_hidden().child(
                    div()
                        .flex_col()
                        .w_full()
                        .pt_4()
                        .pr_4()
                        .pb_4()
                        .pl_8()
                        .relative()
                        .top(px(-self.scroll_state.scroll_y))
                        .child(markdown_viewer::render_markdown_ast_with_search(
                            root,
                            Some(&self.markdown_file_path),
                            self.search_state.as_ref(),
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
            );

        // Add search indicator overlay if search is active
        let element = if let Some(search_state) = &self.search_state {
            let match_info = if search_state.match_count() > 0 {
                format!(
                    "Search: \"{}\" ({} of {} matches)",
                    self.search_input,
                    search_state.current_match_number().unwrap_or(0),
                    search_state.match_count()
                )
            } else if self.search_input.is_empty() {
                "Search: (type to search)".to_string()
            } else {
                format!("Search: \"{}\" (no matches)", self.search_input)
            };

            element.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bg(gpui::Rgba {
                        r: 1.0,
                        g: 0.95,
                        b: 0.6,
                        a: 0.95,
                    })
                    .text_color(gpui::Rgba {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    })
                    .px_4()
                    .py_2()
                    .text_size(px(14.0))
                    .child(match_info),
            )
        } else {
            element
        };

        for path in missing_images {
            self.load_image(path, window, cx);
        }

        element
    }
}

fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Markdown Viewer");

    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}. Using defaults.", e);
        AppConfig::default()
    });

    debug!("Configuration loaded: {:?}", config);

    let args = Args::parse();

    // Resolve the file path using our new function
    let file_path =
        resolve_markdown_file_path(args.file.as_deref(), &config.files.supported_extensions)
            .context("Failed to resolve markdown file path")?;

    // Load the markdown content
    let markdown_input =
        load_markdown_content(&file_path).context("Failed to load markdown content")?;

    info!(
        "Loaded file: {} ({} bytes)",
        file_path,
        markdown_input.len()
    );

    // Create a dedicated background Tokio runtime for async tasks (image downloads, etc.)
    let bg_rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("Failed to build background Tokio runtime")?,
    );

    // Run the GUI on the main thread (required by gpui). Background async work will use `bg_rt`.
    Application::new().run(move |app: &mut App| {
        let window_config = config.clone();
        let file_path_buf = PathBuf::from(file_path.clone());
        let bg_rt = bg_rt.clone();
        let window = app
            .open_window(WindowOptions::default(), move |_, cx| {
                // We can't focus here because we don't have &mut Window
                cx.new(|cx| {
                    let focus_handle = cx.focus_handle();
                    let mut viewer = MarkdownViewer {
                        markdown_content: markdown_input.clone(),
                        markdown_file_path: file_path_buf.clone(),
                        scroll_state: ScrollState::new(),
                        viewport_height: window_config.window.height,
                        config: window_config.clone(),
                        image_cache: HashMap::new(),
                        image_display_heights: HashMap::new(),
                        bg_rt: bg_rt.clone(),
                        search_state: None,
                        search_input: String::new(),
                        focus_handle,
                    };
                    viewer.recompute_max_scroll(); // Calculate initial scroll bounds
                    debug!("MarkdownViewer initialized");
                    viewer
                })
            })
            .unwrap();

        window
            .update(app, |view, cx, _| {
                view.focus_handle.focus(cx);
            })
            .ok();
    });

    Ok(())
}
