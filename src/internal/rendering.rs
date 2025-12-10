//! Markdown rendering functions for the viewer
//!
//! This module handles rendering of the Markdown AST to GPUI elements,
//! including support for headings, lists, code blocks, tables, and more.

use super::style::*;
use super::theme::ThemeColors;
use comrak::nodes::{AstNode, NodeValue};
use gpui::{
    AnyElement, ClipboardItem, Context, FontWeight, ImageSource, InteractiveElement, IntoElement,
    MouseButton, Rgba, SharedString, div, img, prelude::*, px,
};
use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tracing::{debug, error};

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn syntect_color_to_gpui(color: syntect::highlighting::Color) -> Rgba {
    Rgba {
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: color.a as f32 / 255.0,
    }
}

/// Calculate responsive column width for tables
///
/// Returns (column_width, needs_horizontal_scroll)
fn calculate_column_width(num_columns: usize, viewport_width: f32) -> (f32, bool) {
    if num_columns == 0 {
        return (MIN_COLUMN_WIDTH, false);
    }

    let available_width = viewport_width - TABLE_HORIZONTAL_PADDING;
    let equal_width = available_width / num_columns as f32;

    match equal_width < MIN_COLUMN_WIDTH {
        true => {
            // Use minimum width and enable horizontal scrolling
            (MIN_COLUMN_WIDTH, true)
        }
        false => {
            // Use equal distribution, no scrolling needed
            (equal_width, false)
        }
    }
}

fn render_highlighted_code_block<T: 'static>(
    code: String,
    language: String,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
) -> AnyElement {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();

    // Use theme-appropriate syntect theme
    let syntect_theme_name = current_theme.syntect_theme();
    let theme = theme_set
        .themes
        .get(syntect_theme_name)
        .or_else(|| theme_set.themes.values().next())
        .unwrap();

    let syntax = syntax_set
        .find_syntax_by_token(&language)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for (i, line) in code.lines().enumerate() {
        let ranges: Vec<(syntect::highlighting::Style, &str)> = highlighter
            .highlight_line(line, syntax_set)
            .unwrap_or_default();

        let mut line_elements = Vec::new();
        for (style, text) in ranges {
            let color = syntect_color_to_gpui(style.foreground);
            line_elements.push(
                div()
                    .text_color(color)
                    .child(text.to_string())
                    .into_any_element(),
            );
        }

        // Line number
        let line_number = div()
            .w_8()
            .mr_4()
            .text_color(theme_colors.code_line_color)
            .justify_end()
            .flex()
            .child((i + 1).to_string());

        lines.push(
            div()
                .flex()
                .w_full()
                .child(line_number)
                .child(div().flex().children(line_elements)),
        );
    }

    let copy_code = code.clone();
    let copy_button = div()
        .absolute()
        .top_2()
        .right_2()
        .bg(theme_colors.copy_button_bg_color)
        .text_color(theme_colors.copy_button_text_color)
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .child("Copy")
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copy_code.clone()));
            }),
        );

    div()
        .relative()
        .group("code_block")
        .bg(theme_colors.code_bg_color)
        .p_3()
        .rounded_md()
        .font_family(CODE_FONT)
        .flex_col()
        .child(
            div()
                .invisible()
                .group_hover("code_block", |style| style.visible())
                .child(copy_button),
        )
        .children(lines)
        .into_any_element()
}

/// Helper: collect inline text content for wrapping within block containers
fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    match &node.data.borrow().value {
        NodeValue::Text(text) => out.push_str(&String::from_utf8_lossy(text.as_bytes())),
        NodeValue::Code(code) => out.push_str(&String::from_utf8_lossy(code.literal.as_bytes())),
        NodeValue::LineBreak | NodeValue::SoftBreak => out.push('\n'),
        _ => {
            for child in node.children() {
                out.push_str(&collect_text(child));
            }
        }
    }
    out
}

/// Render a Markdown AST node to a GPUI element with context
///
/// This internal function accepts an optional markdown file path for resolving relative image paths.
#[allow(clippy::too_many_arguments)]
fn render_markdown_ast_internal<'a, T: 'static>(
    node: &'a AstNode<'a>,
    markdown_file_path: Option<&Path>,
    search_state: Option<&super::search::SearchState>,
    viewport_width: f32,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
    image_loader: &mut dyn FnMut(&str) -> Option<ImageSource>,
    focused_element: Option<&super::viewer::FocusableElement>,
) -> AnyElement {
    match &node.data.borrow().value {
        NodeValue::Document => div()
            .flex_col()
            .children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element(),

        NodeValue::Paragraph => {
            // Avoid extra spacing inside list items.
            let is_in_list_item = node
                .parent()
                .is_some_and(|p| matches!(p.data.borrow().value, NodeValue::Item(_)));

            let mut p = div().w_full().flex().flex_row().flex_wrap();
            if !is_in_list_item {
                p = p.mb_2();
            }
            p.children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element()
        }

        NodeValue::Heading(heading) => {
            let text_size = match heading.level {
                1 => px(H1_SIZE),
                2 => px(H2_SIZE),
                3 => px(H3_SIZE),
                4 => px(H4_SIZE),
                5 => px(H5_SIZE),
                _ => px(H6_SIZE),
            };
            {
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .text_size(text_size)
                    .font_weight(FontWeight::SEMIBOLD)
                    .mt(px((heading.level == 1) as u8 as f32 * 4.0))
                    .children(node.children().map(|child| {
                        render_markdown_ast_internal(
                            child,
                            markdown_file_path,
                            search_state,
                            viewport_width,
                            theme_colors,
                            current_theme,
                            cx,
                            image_loader,
                            focused_element,
                        )
                    }))
                    .into_any_element()
            }
        }

        NodeValue::Text(text) => {
            let text_str = String::from_utf8_lossy(text.as_bytes()).to_string();

            // Use search highlighting if search is active
            match search_state {
                Some(search_state) => {
                    let elements = super::text_highlight::render_text_with_search(
                        &text_str,
                        Some(search_state),
                    );
                    div()
                        .flex()
                        .flex_row()
                        .children(elements)
                        .into_any_element()
                }
                None => div().child(text_str).into_any_element(),
            }
        }

        NodeValue::Code(code) => div()
            .font_family(CODE_FONT)
            .bg(theme_colors.code_bg_color)
            .text_color(theme_colors.text_color)
            .px_1()
            .rounded_sm()
            .child(String::from_utf8_lossy(code.literal.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::CodeBlock(code_block) => {
            let language = code_block.info.clone();
            let code = code_block.literal.clone();
            render_highlighted_code_block(code, language, theme_colors, current_theme, cx)
        }

        NodeValue::List(list) => {
            let mut items = Vec::new();
            for item in node.children() {
                let marker = match list.list_type {
                    comrak::nodes::ListType::Bullet => "•".to_string(),
                    comrak::nodes::ListType::Ordered => format!("{}.", items.len() + 1),
                };
                let content = div().w_full().children(item.children().map(|child| {
                    render_markdown_ast_internal(
                        child,
                        markdown_file_path,
                        search_state,
                        viewport_width,
                        theme_colors,
                        current_theme,
                        cx,
                        image_loader,
                        focused_element,
                    )
                }));
                items.push(
                    div()
                        .flex()
                        .w_full()
                        .mb_1()
                        .child(div().mr_2().child(marker))
                        .child(content),
                );
            }
            div().flex_col().pl_4().children(items).into_any_element()
        }

        NodeValue::Image(link) => {
            use super::file_handling::resolve_image_path;

            let image_url = link.url.clone();
            let alt_text = collect_text(node);

            debug!("Rendering image '{}' -> '{}'", alt_text, image_url);

            // Resolve image path
            let resolved_path = match markdown_file_path {
                Some(md_path) => resolve_image_path(&image_url, md_path),
                None => image_url.to_string(),
            };

            debug!("Resolved image path: {}", resolved_path);

            match image_loader(&resolved_path) {
                Some(source) => div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .my_2()
                    .child(
                        img(source)
                            .w(px(IMAGE_MAX_WIDTH))
                            .object_fit(gpui::ObjectFit::Contain)
                            .rounded(px(IMAGE_BORDER_RADIUS)),
                    )
                    .into_any_element(),
                None => {
                    // Show placeholder
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .my_2()
                        .p_4()
                        .bg(Rgba {
                            r: 0.95,
                            g: 0.95,
                            b: 0.95,
                            a: 1.0,
                        })
                        .border_1()
                        .border_color(Rgba {
                            r: 0.8,
                            g: 0.8,
                            b: 0.8,
                            a: 1.0,
                        })
                        .rounded(px(IMAGE_BORDER_RADIUS))
                        .child(
                            div()
                                .text_color(Rgba {
                                    r: 0.4,
                                    g: 0.4,
                                    b: 0.4,
                                    a: 1.0,
                                })
                                .font_weight(FontWeight::BOLD)
                                .mb_2()
                                .child("🖼️ Image"),
                        )
                        .child(div().text_color(theme_colors.text_color).child(
                            match alt_text.is_empty() {
                                false => alt_text,
                                true => "Image".to_string(),
                            },
                        ))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(Rgba {
                                    r: 0.5,
                                    g: 0.5,
                                    b: 0.5,
                                    a: 1.0,
                                })
                                .mt_1()
                                .child(resolved_path),
                        )
                        .into_any_element()
                }
            }
        }

        NodeValue::Link(link) => {
            // Convert URL to owned String for capture in closure
            let url = link.url.clone();
            let link_text = collect_text(node);

            debug!("Rendering link '{}' -> '{}'", link_text, url);

            // Check if this link is currently focused
            let is_focused = match focused_element {
                Some(focused_el) => {
                    matches!(focused_el, super::viewer::FocusableElement::Link(focused_url) if focused_url == &url)
                }
                None => false,
            };

            // If URL is empty, render it as plain text (muted) and do not attach
            // a click handler. Otherwise, style it as a link and attach a handler
            // that opens the URL in the system browser.
            match url.trim() {
                "" => div()
                    .text_color(theme_colors.text_color)
                    .child(link_text)
                    .into_any_element(),
                _ => {
                    // clickable
                    let click_url = url.clone();
                    div()
                        .text_color(theme_colors.link_color)
                        .underline()
                        .cursor_pointer()
                        .when(is_focused, |div| div.font_weight(FontWeight::BOLD))
                        .hover(|style| style.text_color(theme_colors.hover_link_color))
                        .id(SharedString::from(url.clone()))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |_, _, _, _| {
                                debug!("Mouse down detected on link: {}", click_url);
                                // Log and open the URL on a background thread.
                                let url_to_open = click_url.clone();
                                std::thread::spawn(move || match open_url(&url_to_open) {
                                    Ok(_) => {
                                        debug!(
                                            "Successfully spawned open command for {}",
                                            url_to_open
                                        )
                                    }
                                    Err(e) => error!("Failed to open URL '{}': {}", url_to_open, e),
                                });
                            }),
                        )
                        .child(link_text)
                        .into_any_element()
                }
            }
        }

        NodeValue::Strong => div()
            .font_weight(FontWeight::BOLD)
            .children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element(),

        NodeValue::Emph => div()
            .italic()
            .children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element(),

        NodeValue::Strikethrough => div()
            .line_through()
            .children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element(),

        NodeValue::BlockQuote => div()
            .border_l_4()
            .border_color(theme_colors.blockquote_border_color)
            .pl_4()
            .italic()
            .children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element(),

        // Table rendering with responsive column widths
        NodeValue::Table(table_data) => {
            // Count columns from first row
            let num_columns = node
                .children()
                .next()
                .map(|row| row.children().count())
                .unwrap_or(0);

            // Use actual viewport width for responsive calculation
            let (column_width, _needs_scroll) = calculate_column_width(num_columns, viewport_width);

            // Create table container with responsive column widths
            div()
                .flex_col()
                .w_full()
                .my_2()
                .border_1()
                .border_color(theme_colors.table_border_color)
                .children(node.children().map(|row| {
                    render_table_row(
                        row,
                        &table_data.alignments,
                        column_width,
                        markdown_file_path,
                        search_state,
                        viewport_width,
                        theme_colors,
                        current_theme,
                        cx,
                        image_loader,
                        focused_element,
                    )
                }))
                .into_any_element()
        }

        NodeValue::TableRow(_) => {
            // Rows should be rendered via render_table_row, but handle gracefully
            div()
                .flex()
                .w_full()
                .children(node.children().map(|child| {
                    render_markdown_ast_internal(
                        child,
                        markdown_file_path,
                        search_state,
                        viewport_width,
                        theme_colors,
                        current_theme,
                        cx,
                        image_loader,
                        focused_element,
                    )
                }))
                .into_any_element()
        }

        NodeValue::TableCell => {
            // Cells should be rendered via render_table_cell, but handle gracefully
            div()
                .p(px(TABLE_CELL_PADDING))
                .children(node.children().map(|child| {
                    render_markdown_ast_internal(
                        child,
                        markdown_file_path,
                        search_state,
                        viewport_width,
                        theme_colors,
                        current_theme,
                        cx,
                        image_loader,
                        focused_element,
                    )
                }))
                .into_any_element()
        }

        // Fallback: walk children
        _ => div()
            .children(node.children().map(|child| {
                render_markdown_ast_internal(
                    child,
                    markdown_file_path,
                    search_state,
                    viewport_width,
                    theme_colors,
                    current_theme,
                    cx,
                    image_loader,
                    focused_element,
                )
            }))
            .into_any_element(),
    }
}

/// Render a Markdown AST node to a GPUI element
///
/// This is the public API that maintains backward compatibility.
/// Render a Markdown AST node to a GPUI element
///
/// This is the simplest rendering function that uses default settings.
pub fn render_markdown_ast<'a, T: 'static>(
    node: &'a AstNode<'a>,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
) -> AnyElement {
    const DEFAULT_VIEWPORT_WIDTH: f32 = 1200.0;
    render_markdown_ast_internal(
        node,
        None,
        None,
        DEFAULT_VIEWPORT_WIDTH,
        theme_colors,
        current_theme,
        cx,
        &mut |_| None,
        None,
    )
}

/// Render a Markdown AST node to a GPUI element with markdown file path context
///
/// This version accepts the markdown file path to enable proper resolution of relative image paths.
pub fn render_markdown_ast_with_loader<'a, T: 'static>(
    node: &'a AstNode<'a>,
    markdown_file_path: Option<&Path>,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
    image_loader: &mut dyn FnMut(&str) -> Option<ImageSource>,
) -> AnyElement {
    const DEFAULT_VIEWPORT_WIDTH: f32 = 1200.0;
    render_markdown_ast_internal(
        node,
        markdown_file_path,
        None,
        DEFAULT_VIEWPORT_WIDTH,
        theme_colors,
        current_theme,
        cx,
        image_loader,
        None,
    )
}

/// Render a Markdown AST node to a GPUI element with search highlighting
///
/// This version accepts search state to highlight matching text.
#[allow(clippy::too_many_arguments)]
pub fn render_markdown_ast_with_search<'a, T: 'static>(
    node: &'a AstNode<'a>,
    markdown_file_path: Option<&Path>,
    search_state: Option<&super::search::SearchState>,
    viewport_width: f32,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
    image_loader: &mut dyn FnMut(&str) -> Option<ImageSource>,
    focused_element: Option<&super::viewer::FocusableElement>,
) -> AnyElement {
    render_markdown_ast_internal(
        node,
        markdown_file_path,
        search_state,
        viewport_width,
        theme_colors,
        current_theme,
        cx,
        image_loader,
        focused_element,
    )
}

/// Render a table row with proper alignment and header styling
#[allow(clippy::too_many_arguments)]
fn render_table_row<'a, T: 'static>(
    row_node: &'a AstNode<'a>,
    alignments: &[comrak::nodes::TableAlignment],
    column_width: f32,
    markdown_file_path: Option<&Path>,
    search_state: Option<&super::search::SearchState>,
    viewport_width: f32,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
    image_loader: &mut dyn FnMut(&str) -> Option<ImageSource>,
    focused_element: Option<&super::viewer::FocusableElement>,
) -> AnyElement {
    let is_header = matches!(row_node.data.borrow().value, NodeValue::TableRow(true));

    let mut row_div = div()
        .flex()
        .w_full()
        .border_b_1()
        .border_color(TABLE_BORDER_COLOR);

    if is_header {
        row_div = row_div
            .bg(theme_colors.table_header_bg)
            .font_weight(FontWeight::BOLD);
    }

    // Render cells with alignment and calculated width
    let cells: Vec<AnyElement> = row_node
        .children()
        .enumerate()
        .map(|(idx, cell)| {
            render_table_cell(
                cell,
                alignments.get(idx),
                column_width,
                markdown_file_path,
                search_state,
                viewport_width,
                theme_colors,
                current_theme,
                cx,
                image_loader,
                focused_element,
            )
        })
        .collect();

    row_div.children(cells).into_any_element()
}

/// Render a table cell with alignment and responsive width
#[allow(clippy::too_many_arguments)]
fn render_table_cell<'a, T: 'static>(
    cell_node: &'a AstNode<'a>,
    alignment: Option<&comrak::nodes::TableAlignment>,
    column_width: f32,
    markdown_file_path: Option<&Path>,
    search_state: Option<&super::search::SearchState>,
    viewport_width: f32,
    theme_colors: &ThemeColors,
    current_theme: super::theme::Theme,
    cx: &mut Context<T>,
    image_loader: &mut dyn FnMut(&str) -> Option<ImageSource>,
    focused_element: Option<&super::viewer::FocusableElement>,
) -> AnyElement {
    use comrak::nodes::TableAlignment;

    let mut cell_div = div()
        .w(px(column_width))
        .min_w(px(MIN_COLUMN_WIDTH))
        .p(px(TABLE_CELL_PADDING))
        .border_r_1()
        .border_color(TABLE_BORDER_COLOR)
        .flex_shrink_0()
        .overflow_hidden();

    // Apply alignment
    cell_div = match alignment {
        Some(TableAlignment::Left) | None => cell_div.justify_start(),
        Some(TableAlignment::Center) => cell_div.justify_center(),
        Some(TableAlignment::Right) => cell_div.justify_end(),
        Some(TableAlignment::None) => cell_div.justify_start(),
    };

    cell_div
        .children(cell_node.children().map(|child| {
            render_markdown_ast_internal(
                child,
                markdown_file_path,
                search_state,
                viewport_width,
                theme_colors,
                current_theme,
                cx,
                image_loader,
                focused_element,
            )
        }))
        .into_any_element()
}

/// Open a URL in the default browser
///
/// Uses platform-specific commands to open URLs in the system's default browser.
///
/// # Arguments
/// * `url` - The URL to open
///
/// # Returns
/// * `Ok(())` if the command was spawned successfully
/// * `Err` if spawning the command failed
pub fn open_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", url])
            .spawn()?;
    }

    Ok(())
}
