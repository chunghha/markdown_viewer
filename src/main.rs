use anyhow::{Context, Result};
use clap::Parser;
use comrak::{Arena, Options, parse_document};
use gpui::{
    App, Application, AsyncWindowContext, Context as GpuiContext, ImageSource, IntoElement,
    KeyDownEvent, Render, RenderImage, ScrollWheelEvent, WeakEntity, Window, WindowOptions, div,
    prelude::*, px,
};
use markdown_viewer::fetch_and_decode_image;
use markdown_viewer::{
    BG_COLOR, IMAGE_MAX_WIDTH, ScrollState, TEXT_COLOR, config::AppConfig, load_markdown_content,
    render_markdown_ast_with_loader, resolve_markdown_file_path,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
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
    fn recompute_max_scroll(&mut self) {
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;

        // Heuristic counts
        let mut heading_lines: usize = 0;
        let mut code_block_lines: usize = 0;
        let mut blockquote_lines: usize = 0;
        let mut list_lines: usize = 0;
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
                // empty line — small spacer, count as half-line
                // We'll fold these into other_lines with lower weight
                other_lines += 1;
            } else {
                other_lines += 1;
            }
        }

        // Weights (multipliers relative to avg_line_height)
        let heading_weight = 1.6; // headings are larger and have extra spacing
        let code_line_weight = 1.25; // fenced code lines take slightly more vertical space
        let blockquote_weight = 1.15;
        let list_line_weight = 1.0;
        let normal_line_weight = 1.0;

        let estimated_text_height = (heading_lines as f32 * avg_line_height * heading_weight)
            + (code_block_lines as f32 * avg_line_height * code_line_weight)
            + (blockquote_lines as f32 * avg_line_height * blockquote_weight)
            + (list_lines as f32 * avg_line_height * list_line_weight)
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
        let arena = Arena::new();
        let mut options = Options::default();
        options.extension.table = true; // Enable GFM tables
        let root = parse_document(&arena, &self.markdown_content, &options);

        let mut missing_images = HashSet::new();
        let element = div()
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
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                let arrow_increment = this.config.scroll.arrow_key_increment;
                let page_percent = this.config.scroll.page_scroll_percentage;
                let space_percent = this.config.scroll.space_scroll_percentage;

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
                        .child(render_markdown_ast_with_loader(
                            root,
                            Some(&self.markdown_file_path),
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
    Application::new().run(move |cx: &mut App| {
        let window_config = config.clone();
        let file_path_buf = PathBuf::from(file_path.clone());
        let bg_rt = bg_rt.clone();
        cx.open_window(WindowOptions::default(), move |_, cx| {
            cx.new(|_| {
                let mut viewer = MarkdownViewer {
                    markdown_content: markdown_input.clone(),
                    markdown_file_path: file_path_buf.clone(),
                    scroll_state: ScrollState::new(),
                    viewport_height: window_config.window.height,
                    config: window_config.clone(),
                    image_cache: HashMap::new(),
                    image_display_heights: HashMap::new(),
                    bg_rt: bg_rt.clone(),
                };
                viewer.recompute_max_scroll(); // Calculate initial scroll bounds
                debug!("MarkdownViewer initialized");
                viewer
            })
        })
        .unwrap();
    });

    Ok(())
}
