use comrak::{parse_document, Arena, ComrakOptions};
use gpui::{
    div, prelude::*, px, App, Application, Context, IntoElement, KeyDownEvent, Render,
    ScrollWheelEvent, Window, WindowOptions,
};
use markdown_viewer::{
    render_markdown_ast, ScrollState, BASE_TEXT_SIZE, BG_COLOR, CONTENT_HEIGHT_BUFFER,
    DEFAULT_VIEWPORT_HEIGHT, LINE_HEIGHT_MULTIPLIER, PRIMARY_FONT, TEXT_COLOR,
};
use std::fs;

struct MarkdownViewer {
    markdown_content: String,
    scroll_state: ScrollState,
    viewport_height: f32,
}

impl MarkdownViewer {
    // New: Method to recompute max scroll based on content and viewport
    fn recompute_max_scroll(&mut self) {
        // Better estimation based on actual line count with proper spacing
        let line_count = self.markdown_content.lines().count();
        let avg_line_height = BASE_TEXT_SIZE * LINE_HEIGHT_MULTIPLIER;
        let estimated_content_height = line_count as f32 * avg_line_height;

        // Add buffer for headings, lists, and spacing
        let content_height = estimated_content_height + CONTENT_HEIGHT_BUFFER;

        self.scroll_state
            .set_max_scroll(content_height, self.viewport_height);
    }
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let arena = Arena::new();
        let root = parse_document(&arena, &self.markdown_content, &ComrakOptions::default());

        div()
            .flex()
            .size_full()
            .bg(BG_COLOR)
            .text_color(TEXT_COLOR)
            .font_family(PRIMARY_FONT)
            .text_size(px(BASE_TEXT_SIZE))
            // New: Event handlers for scrolling
            .on_mouse_move(cx.listener(|this, _, _, cx| {
                // Use more reasonable default viewport height
                if this.viewport_height == 0.0 {
                    this.viewport_height = DEFAULT_VIEWPORT_HEIGHT;
                    this.recompute_max_scroll();
                }
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "up" => this.scroll_state.scroll_up(20.0),
                    "down" => this.scroll_state.scroll_down(20.0),
                    "pageup" => this.scroll_state.page_up(this.viewport_height),
                    "pagedown" => this.scroll_state.page_down(this.viewport_height),
                    "home" => this.scroll_state.scroll_to_top(),
                    "end" => this.scroll_state.scroll_to_bottom(),
                    "space" if event.keystroke.modifiers.shift => {
                        this.scroll_state.page_up(this.viewport_height * 0.2)
                    }
                    "space" => this.scroll_state.page_down(this.viewport_height * 0.2),
                    _ => {}
                }
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                let delta = event.delta.pixel_delta(px(BASE_TEXT_SIZE)).y;
                let delta_f32: f32 = delta.into();
                if delta_f32 > 0.0 {
                    this.scroll_state.scroll_down(delta_f32);
                } else {
                    this.scroll_state.scroll_up(-delta_f32);
                }
                cx.notify();
            }))
            .child(
                div().flex().size_full().overflow_hidden().child(
                    div()
                        .flex_col()
                        .p_4()
                        .relative()
                        .top(px(-self.scroll_state.scroll_y))
                        .child(render_markdown_ast(root, cx)),
                ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let markdown_input = fs::read_to_string("TODO.md").expect("Failed to read TODO.md file");

        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| {
                let mut viewer = MarkdownViewer {
                    markdown_content: markdown_input,
                    scroll_state: ScrollState::new(),
                    viewport_height: DEFAULT_VIEWPORT_HEIGHT,
                };
                viewer.recompute_max_scroll(); // Calculate initial scroll bounds
                viewer
            })
        })
        .unwrap();
    });
}
