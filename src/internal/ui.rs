use gpui::{FontWeight, IntoElement, Rgba, div, prelude::*, px};

use crate::internal::help_overlay::help_panel;
use crate::internal::style::{GOTO_LINE_OVERLAY_BG_COLOR, GOTO_LINE_OVERLAY_TEXT_COLOR};
use crate::internal::viewer::MarkdownViewer;

pub fn render_status_bar(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> impl IntoElement {
    let filename = viewer
        .markdown_file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let total_lines = viewer.markdown_content.lines().count().max(1);
    let current_line = viewer.get_current_line_number();
    let percentage = (current_line as f32 / total_lines as f32 * 100.0) as usize;

    div()
        .absolute()
        .bottom_0()
        .left_0()
        .right_0()
        .h(px(30.0))
        .bg(theme_colors.toc_bg_color)
        .border_t_1()
        .border_color(theme_colors.toc_border_color)
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .text_size(px(12.0))
        .text_color(theme_colors.text_color)
        .child(
            div()
                .flex()
                .gap_4()
                .child(div().font_weight(FontWeight::BOLD).child(filename))
                .child(format!("{} lines", total_lines)),
        )
        .child(
            div()
                .flex()
                .gap_4()
                .child(format!("Ln {}, Col 1", current_line)) // Col is always 1 for now
                .child(format!("{}%", percentage)),
        )
        .child(
            div()
                .flex()
                .gap_4()
                .child(viewer.config.theme.theme.clone())
                .child(
                    div()
                        .cursor_pointer()
                        .font_weight(FontWeight::BOLD)
                        .child("Help")
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.show_help = !this.show_help;
                                cx.notify();
                            }),
                        ),
                )
                .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
        )
}

pub fn render_search_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    match &viewer.search_state {
        Some(search_state) => {
            let match_info = match (search_state.match_count(), viewer.search_input.is_empty()) {
                (n, _) if n > 0 => format!(
                    "Search: \"{}\" ({} of {} matches)",
                    viewer.search_input,
                    search_state.current_match_number().unwrap_or(0),
                    search_state.match_count()
                ),
                (0, true) => "Search: (type to search)".to_string(),
                (0, false) => format!("Search: \"{}\" (no matches)", viewer.search_input),
                // Fallback arm, though all cases are covered above
                _ => "Search: (type to search)".to_string(),
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
        }
        None => None,
    }
}

pub fn render_goto_line_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    match viewer.show_goto_line {
        true => {
            let total_lines = viewer.markdown_content.lines().count();
            let display_text = match viewer.goto_line_input.as_str() {
                "" => format!("Go to line: (1-{})", total_lines),
                input => match MarkdownViewer::parse_line_number(input) {
                    Some(line_number) if line_number > total_lines => format!(
                        "Go to line: \"{}\" (exceeds max: {})",
                        viewer.goto_line_input, total_lines
                    ),
                    Some(_) => format!("Go to line: \"{}\"", viewer.goto_line_input),
                    None => format!("Go to line: \"{}\" (invalid)", viewer.goto_line_input),
                },
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
        }
        false => None,
    }
}

pub fn render_help_overlay(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
) -> Option<impl IntoElement> {
    match viewer.show_help {
        true => Some(
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
                .child(help_panel(theme_colors, viewer.help_page)),
        ),
        false => None,
    }
}

pub fn render_file_deleted_overlay(viewer: &MarkdownViewer) -> Option<impl IntoElement> {
    match viewer.file_deleted {
        true => Some(
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
        ),
        false => None,
    }
}

pub fn render_pdf_export_overlay(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
) -> Option<impl IntoElement> {
    match &viewer.pdf_export_message {
        Some(message) => {
            let (bg_color, icon) = match viewer.pdf_export_success {
                true => (theme_colors.pdf_success_bg_color, "✓"),
                false => (theme_colors.pdf_error_bg_color, "✗"),
            };

            Some(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bg(bg_color)
                    .text_color(theme_colors.pdf_notification_text_color)
                    .px_4()
                    .py_2()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::BOLD)
                    .child(format!("{} {}", icon, message)),
            )
        }
        None => None,
    }
}

pub fn render_pdf_overwrite_confirm(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
) -> Option<impl IntoElement> {
    match viewer.show_pdf_overwrite_confirm {
        true => {
            let filename = viewer
                .pdf_overwrite_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("output.pdf");

            Some(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bg(theme_colors.pdf_warning_bg_color)
                    .text_color(theme_colors.text_color)
                    .px_4()
                    .py_2()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::BOLD)
                    .child(format!("⚠ {} already exists. Overwrite? (Y/N)", filename)),
            )
        }
        false => None,
    }
}

pub fn render_toc_sidebar(
    viewer: &mut MarkdownViewer,
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

            // Note: TOC items are NOT tracked as focusable (excluded from tab navigation)

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
                // On macOS with natural scrolling, delta is already in the correct direction
                // Positive delta = scroll down (content moves up)
                // Negative delta = scroll up (content moves down)
                // Simply apply the delta directly
                this.toc_scroll_y =
                    (this.toc_scroll_y - delta_f32).clamp(0.0, this.toc_max_scroll_y);
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
    viewer: &mut MarkdownViewer,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> impl IntoElement {
    use crate::internal::style::{
        TOC_TOGGLE_BG_COLOR, TOC_TOGGLE_HOVER_COLOR, TOC_TOGGLE_TEXT_COLOR,
    };

    // Note: TOC toggle is NOT tracked as focusable (excluded from tab navigation)

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
        .child(match viewer.show_toc {
            true => "✕",
            false => "☰",
        })
}

pub fn render_bookmarks_overlay(
    viewer: &mut MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> Option<impl IntoElement> {
    if !viewer.show_bookmarks {
        return None;
    }

    use crate::internal::style::FOCUS_BG_COLOR;
    use crate::internal::viewer::FocusableElement;

    let bookmarks_list = match viewer.bookmarks.as_slice() {
        [] => div()
            .flex()
            .items_center()
            .justify_center()
            .py_4()
            .text_color(theme_colors.text_color)
            .child("No bookmarks yet. Press Cmd+D to add one."),
        entries => div().flex().flex_col().gap_1().children(
            entries
                .iter()
                .enumerate()
                .map(|(idx, &line_number)| {
                    // Track this bookmark item as focusable
                    let element_index = viewer.focusable_elements.len();
                    viewer
                        .focusable_elements
                        .push(FocusableElement::BookmarkItem(line_number));

                    let is_focused = viewer.current_focus_index == Some(element_index);

                    div()
                        .px_4()
                        .py_2()
                        .cursor_pointer()
                        .when(is_focused, |div| div.bg(FOCUS_BG_COLOR))
                        .hover(|div| div.bg(theme_colors.toc_hover_color))
                        .text_color(theme_colors.text_color)
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                let _ = this.scroll_to_line(line_number);
                                this.show_bookmarks = false;
                                cx.notify();
                            }),
                        )
                        .child(format!("Bookmark {}: Line {}", idx + 1, line_number))
                })
                .collect::<Vec<_>>(),
        ),
    };

    // Track close button as focusable
    let close_button_index = viewer.focusable_elements.len();
    viewer
        .focusable_elements
        .push(FocusableElement::BookmarksCloseButton);
    let close_button_focused = viewer.current_focus_index == Some(close_button_index);

    Some(
        div()
            .absolute()
            .top_12()
            .right_12()
            .w(px(300.0))
            .bg(theme_colors.bg_color)
            .border_1()
            .border_color(theme_colors.toc_border_color)
            .shadow_lg()
            .rounded_md()
            .p_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .pb_2()
                            .border_b_1()
                            .border_color(theme_colors.toc_border_color)
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme_colors.text_color)
                                    .child("Bookmarks"),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .text_color(theme_colors.text_color)
                                    .when(close_button_focused, |div| div.bg(FOCUS_BG_COLOR).px_1())
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_bookmarks = false;
                                            cx.notify();
                                        }),
                                    )
                                    .child("✕"),
                            ),
                    )
                    .child(bookmarks_list),
            ),
    )
}

pub fn render_search_history_notification(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> Option<impl IntoElement> {
    viewer.search_history_message.as_ref().map(|message| {
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bg(theme_colors.pdf_success_bg_color)
            .text_color(theme_colors.pdf_notification_text_color)
            .px_4()
            .py_2()
            .text_size(px(14.0))
            .font_weight(FontWeight::BOLD)
            .cursor_pointer()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.search_history_message = None;
                    cx.notify();
                }),
            )
            .child(format!("ℹ {} (Click to dismiss)", message))
    })
}

pub fn render_file_finder(
    viewer: &MarkdownViewer,
    theme_colors: &crate::internal::theme::ThemeColors,
    cx: &mut gpui::Context<MarkdownViewer>,
) -> Option<impl IntoElement> {
    if !viewer.show_file_finder {
        return None;
    }

    let query = viewer.finder_query.clone();

    let list_items = viewer
        .finder_matches
        .iter()
        .enumerate()
        .map(|(idx, (_, path))| {
            let is_selected = idx == viewer.finder_selected_index;
            let path_str = path.to_string_lossy().to_string();
            let path_clone = path.clone();

            div()
                .flex()
                .px_2()
                .py_1()
                .w_full()
                .rounded_sm()
                .cursor_pointer()
                .bg(match is_selected {
                    true => theme_colors.toc_active_color,
                    false => gpui::transparent_black().into(),
                })
                .hover(|style| style.bg(theme_colors.toc_hover_color))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.load_file(path_clone.clone(), cx);
                    }),
                )
                .child(div().text_color(theme_colors.text_color).child(path_str))
        })
        .collect::<Vec<_>>();

    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(gpui::rgba(0x00000080)) // Dim background
            .flex()
            .items_start()
            .justify_center()
            .pt(px(100.0))
            .child(
                div()
                    .w(px(600.0))
                    .bg(theme_colors.bg_color)
                    .border_1()
                    .border_color(theme_colors.toc_border_color)
                    .shadow_xl()
                    .rounded_xl()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_col()
                            .child(
                                // Input area
                                div()
                                    .p_4()
                                    .border_b_1()
                                    .border_color(theme_colors.toc_border_color)
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_color(theme_colors.text_color)
                                                    .opacity(0.7)
                                                    .child("Go to file:"),
                                            )
                                            .child(
                                                div()
                                                    .text_color(theme_colors.text_color)
                                                    .font_weight(FontWeight::BOLD)
                                                    .child(format!("{}█", query)),
                                            ),
                                    ),
                            )
                            .child(
                                // Results list
                                div()
                                    .flex_col()
                                    // Use overflow_hidden instead of missing overflow_y_scroll
                                    // since we capped results at 20, we can just show them all for now
                                    // or wraps in a scrollable container if GPUI supports it differently.
                                    // Given the error, we'll avoid the method call and rely on max height.
                                    .max_h(px(400.0))
                                    .overflow_hidden()
                                    .p_2()
                                    .children(list_items),
                            )
                            .child(
                                // Footer
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(theme_colors.toc_bg_color)
                                    .border_t_1()
                                    .border_color(theme_colors.toc_border_color)
                                    .flex()
                                    .justify_between()
                                    .text_xs()
                                    .text_color(theme_colors.text_color)
                                    .opacity(0.7)
                                    .child("Use Up/Down to navigate, Enter to select, Esc to close")
                                    .child(format!("{} files", viewer.all_files.len())),
                            ),
                    ),
            ),
    )
}
