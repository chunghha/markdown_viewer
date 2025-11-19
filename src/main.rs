use anyhow::{Context, Result};
use clap::Parser;
use comrak::{Arena, Options, parse_document};
use gpui::{
    App, Application, Context as GpuiContext, IntoElement, KeyDownEvent, Render, ScrollWheelEvent,
    Window, WindowOptions, div, prelude::*, px,
};
use markdown_viewer::{
    BG_COLOR, ScrollState, TEXT_COLOR, config::AppConfig, load_markdown_content,
    render_markdown_ast, resolve_markdown_file_path,
};
use tracing::{debug, info, warn};

#[derive(Parser)]
#[command(name = "markdown_viewer")]
#[command(about = "A simple markdown viewer with scrolling support")]
struct Args {
    /// Path to the markdown file to view
    file: Option<String>,
}

struct MarkdownViewer {
    markdown_content: String,
    scroll_state: ScrollState,
    viewport_height: f32,
    config: AppConfig,
}

impl MarkdownViewer {
    // New: Method to recompute max scroll based on content and viewport
    fn recompute_max_scroll(&mut self) {
        // Better estimation based on actual line count with proper spacing
        let line_count = self.markdown_content.lines().count();
        let avg_line_height =
            self.config.theme.base_text_size * self.config.theme.line_height_multiplier;
        let estimated_content_height = line_count as f32 * avg_line_height;

        // Add buffer for headings, lists, and spacing
        let content_height = estimated_content_height + self.config.theme.content_height_buffer;

        self.scroll_state
            .set_max_scroll(content_height, self.viewport_height);
    }
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut GpuiContext<Self>) -> impl IntoElement {
        let arena = Arena::new();
        let mut options = Options::default();
        options.extension.table = true; // Enable GFM tables
        let root = parse_document(&arena, &self.markdown_content, &options);

        div()
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
                        .p_4()
                        .relative()
                        .top(px(-self.scroll_state.scroll_y))
                        .child(render_markdown_ast(root, cx)),
                ),
            )
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

    Application::new().run(move |cx: &mut App| {
        let window_config = config.clone();
        cx.open_window(WindowOptions::default(), move |_, cx| {
            cx.new(|_| {
                let mut viewer = MarkdownViewer {
                    markdown_content: markdown_input.clone(),
                    scroll_state: ScrollState::new(),
                    viewport_height: window_config.window.height,
                    config: window_config.clone(),
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
