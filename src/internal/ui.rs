use gpui::{FontWeight, IntoElement, Rgba, div, prelude::*, px};

use crate::internal::help_overlay::help_panel;
use crate::internal::style::{
    GOTO_LINE_OVERLAY_BG_COLOR, GOTO_LINE_OVERLAY_TEXT_COLOR, VERSION_BADGE_BG_COLOR,
    VERSION_BADGE_TEXT_COLOR,
};
use crate::internal::viewer::MarkdownViewer;

pub fn render_version_badge() -> impl IntoElement {
    div()
        .absolute()
        .bottom_3()
        .right_4()
        .bg(VERSION_BADGE_BG_COLOR)
        .text_color(VERSION_BADGE_TEXT_COLOR)
        .rounded_md()
        .px_2()
        .py_1()
        .text_xs()
        .child(format!("v{}", env!("CARGO_PKG_VERSION")))
}

pub fn render_search_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    if let Some(search_state) = &viewer.search_state {
        let match_info = if search_state.match_count() > 0 {
            format!(
                "Search: \"{}\" ({} of {} matches)",
                viewer.search_input,
                search_state.current_match_number().unwrap_or(0),
                search_state.match_count()
            )
        } else if viewer.search_input.is_empty() {
            "Search: (type to search)".to_string()
        } else {
            format!("Search: \"{}\" (no matches)", viewer.search_input)
        };

        Some(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bg(Rgba {
                    r: 1.0,
                    g: 0.95,
                    b: 0.6,
                    a: 0.95,
                })
                .text_color(Rgba {
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
        None
    }
}

pub fn render_goto_line_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    if viewer.show_goto_line {
        let total_lines = viewer.markdown_content.lines().count();
        let display_text = if viewer.goto_line_input.is_empty() {
            format!("Go to line: (1-{})", total_lines)
        } else {
            // Validate the input
            if let Some(line_number) = MarkdownViewer::parse_line_number(&viewer.goto_line_input) {
                if line_number > total_lines {
                    format!(
                        "Go to line: \"{}\" (exceeds max: {})",
                        viewer.goto_line_input, total_lines
                    )
                } else {
                    format!("Go to line: \"{}\"", viewer.goto_line_input)
                }
            } else {
                format!("Go to line: \"{}\" (invalid)", viewer.goto_line_input)
            }
        };

        Some(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bg(GOTO_LINE_OVERLAY_BG_COLOR)
                .text_color(GOTO_LINE_OVERLAY_TEXT_COLOR)
                .px_4()
                .py_2()
                .text_size(px(14.0))
                .child(display_text),
        )
    } else {
        None
    }
}

pub fn render_help_overlay(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
) -> Option<impl IntoElement> {
    if viewer.show_help {
        Some(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .bg(Rgba {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.8,
                })
                .flex()
                .items_center()
                .justify_center()
                .child(help_panel(theme_colors)),
        )
    } else {
        None
    }
}

pub fn render_file_deleted_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    if viewer.file_deleted {
        Some(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bg(Rgba {
                    r: 1.0,
                    g: 0.4,
                    b: 0.4,
                    a: 0.95,
                })
                .text_color(Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                })
                .px_4()
                .py_2()
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .child("⚠ File deleted - monitoring for recreation"),
        )
    } else {
        None
    }
}

pub fn render_toc_sidebar(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> Option<impl IntoElement> {
    if !viewer.show_toc || viewer.toc.entries.is_empty() {
        return None;
    }

    use crate::internal::style::{TOC_INDENT_PER_LEVEL, TOC_WIDTH};

    let avg_line_height =
        viewer.config.theme.base_text_size * viewer.config.theme.line_height_multiplier;
    let current_section_idx = viewer
        .toc
        .find_current_section(viewer.scroll_state.scroll_y, avg_line_height);

    let toc_entries = viewer
        .toc
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_active = current_section_idx == Some(idx);
            let indent = (entry.level as f32 - 1.0) * TOC_INDENT_PER_LEVEL;
            let line_number = entry.line_number;

            div()
                .px(px(8.0 + indent))
                .py_1()
                .text_size(px(13.0))
                .text_color(theme_colors.toc_text_color)
                .cursor_pointer()
                .when(is_active, |div| div.bg(theme_colors.toc_active_color))
                .hover(|div| div.bg(theme_colors.toc_hover_color))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _event, _, cx| {
                        // Calculate target scroll position based on line number using smart logic
                        let target_y = this.calculate_y_for_line(line_number);
                        this.scroll_state.scroll_y = target_y.min(this.scroll_state.max_scroll_y);
                        cx.notify();
                    }),
                )
                .child(entry.text.clone())
        })
        .collect::<Vec<_>>();

    Some(
        div()
            .absolute()
            .top_0()
            .right_0()
            .bottom_0()
            .w(px(TOC_WIDTH))
            .bg(theme_colors.toc_bg_color)
            .border_l_1()
            .border_color(theme_colors.toc_border_color)
            .overflow_hidden()
            .on_scroll_wheel(cx.listener(|this, event: &gpui::ScrollWheelEvent, _, cx| {
                let delta = event
                    .delta
                    .pixel_delta(px(this.config.theme.base_text_size))
                    .y;
                let delta_f32: f32 = delta.into();

                // Scroll TOC
                if delta_f32 > 0.0 {
                    // Scroll down
                    this.toc_scroll_y = (this.toc_scroll_y + delta_f32).min(this.toc_max_scroll_y);
                } else {
                    // Scroll up
                    this.toc_scroll_y = (this.toc_scroll_y + delta_f32).max(0.0);
                }
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pt_4()
                    .pb_4()
                    .relative()
                    .top(px(-viewer.toc_scroll_y))
                    .children(toc_entries),
            ),
    )
}

pub fn render_toc_toggle_button(
    viewer: &MarkdownViewer,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> impl IntoElement {
    use crate::internal::style::{
        TOC_TOGGLE_BG_COLOR, TOC_TOGGLE_HOVER_COLOR, TOC_TOGGLE_TEXT_COLOR,
    };

    div()
        .absolute()
        .top_4()
        .right_4()
        .bg(TOC_TOGGLE_BG_COLOR)
        .text_color(TOC_TOGGLE_TEXT_COLOR)
        .rounded_md()
        .px_3()
        .py_2()
        .text_size(px(18.0))
        .font_weight(FontWeight::BOLD)
        .cursor_pointer()
        .hover(|div| div.bg(TOC_TOGGLE_HOVER_COLOR))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|this, _event, _, cx| {
                this.show_toc = !this.show_toc;
                this.recompute_max_scroll();
                cx.notify();
            }),
        )
        .child(if viewer.show_toc { "✕" } else { "☰" })
}
