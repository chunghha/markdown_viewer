use gpui::{FontWeight, IntoElement, Rgba, div, prelude::*, px};

use crate::internal::help_overlay::help_panel;
use crate::internal::style::{VERSION_BADGE_BG_COLOR, VERSION_BADGE_TEXT_COLOR};
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

pub fn render_help_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
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
                .child(help_panel()),
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
