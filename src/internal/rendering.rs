//! Markdown rendering functions for the viewer
//!
//! This module handles rendering of the Markdown AST to GPUI elements,
//! including support for headings, lists, code blocks, tables, and more.

use super::style::*;
use comrak::nodes::{AstNode, NodeValue};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton, SharedString,
    div, prelude::*, px,
};

use tracing::{debug, error};

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

/// Render a Markdown AST node to a GPUI element
pub fn render_markdown_ast<'a, T: 'static>(
    node: &'a AstNode<'a>,
    cx: &mut Context<T>,
) -> AnyElement {
    match &node.data.borrow().value {
        NodeValue::Document => div()
            .flex_col()
            .children(node.children().map(|child| render_markdown_ast(child, cx)))
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
            p.children(node.children().map(|child| render_markdown_ast(child, cx)))
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
                    .children(node.children().map(|child| render_markdown_ast(child, cx)))
                    .into_any_element()
            }
        }

        NodeValue::Text(text) => div()
            .child(String::from_utf8_lossy(text.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::Code(code) => div()
            .font_family(CODE_FONT)
            .bg(CODE_BG_COLOR)
            .text_color(TEXT_COLOR)
            .px_1()
            .rounded_sm()
            .child(String::from_utf8_lossy(code.literal.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::CodeBlock(code_block) => div()
            .bg(CODE_BG_COLOR)
            .p_3()
            .rounded_md()
            .font_family(CODE_FONT)
            .child(String::from_utf8_lossy(code_block.literal.as_bytes()).to_string())
            .into_any_element(),

        NodeValue::List(list) => {
            let mut items = Vec::new();
            for item in node.children() {
                let marker = match list.list_type {
                    comrak::nodes::ListType::Bullet => "•".to_string(),
                    comrak::nodes::ListType::Ordered => format!("{}.", items.len() + 1),
                };
                let content = div()
                    .w_full()
                    .children(item.children().map(|child| render_markdown_ast(child, cx)));
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

        NodeValue::Link(link) => {
            // Convert URL to owned String for capture in closure
            let url = link.url.clone();
            let link_text = collect_text(node);

            debug!("Rendering link '{}' -> '{}'", link_text, url);

            // If URL is empty, render it as plain text (muted) and do not attach
            // a click handler. Otherwise, style it as a link and attach a handler
            // that opens the URL in the system browser.
            if url.trim().is_empty() {
                div()
                    .text_color(TEXT_COLOR)
                    .child(link_text)
                    .into_any_element()
            } else {
                // clickable
                let click_url = url.clone();
                div()
                    .text_color(LINK_COLOR)
                    .underline()
                    .cursor_pointer()
                    .hover(|style| style.text_color(HOVER_LINK_COLOR))
                    .id(SharedString::from(url.clone()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |_, _, _, _| {
                            debug!("Mouse down detected on link: {}", click_url);
                            // Log and open the URL on a background thread.
                            let url_to_open = click_url.clone();
                            std::thread::spawn(move || match open_url(&url_to_open) {
                                Ok(_) => {
                                    debug!("Successfully spawned open command for {}", url_to_open)
                                }
                                Err(e) => error!("Failed to open URL '{}': {}", url_to_open, e),
                            });
                        }),
                    )
                    .child(link_text)
                    .into_any_element()
            }
        }

        NodeValue::Strong => div()
            .font_weight(FontWeight::BOLD)
            .children(node.children().map(|child| render_markdown_ast(child, cx)))
            .into_any_element(),

        NodeValue::Emph => div()
            .italic()
            .children(node.children().map(|child| render_markdown_ast(child, cx)))
            .into_any_element(),

        NodeValue::Strikethrough => div()
            .line_through()
            .children(node.children().map(|child| render_markdown_ast(child, cx)))
            .into_any_element(),

        NodeValue::BlockQuote => div()
            .border_l_4()
            .border_color(BLOCKQUOTE_BORDER_COLOR)
            .pl_4()
            .italic()
            .children(node.children().map(|child| render_markdown_ast(child, cx)))
            .into_any_element(),

        // Table rendering
        NodeValue::Table(table_data) => div()
            .flex_col()
            .w_full()
            .my_2()
            .border_1()
            .border_color(TABLE_BORDER_COLOR)
            .children(
                node.children()
                    .map(|row| render_table_row(row, &table_data.alignments, cx)),
            )
            .into_any_element(),

        NodeValue::TableRow(_) => {
            // Rows should be rendered via render_table_row, but handle gracefully
            div()
                .flex()
                .w_full()
                .children(node.children().map(|child| render_markdown_ast(child, cx)))
                .into_any_element()
        }

        NodeValue::TableCell => {
            // Cells should be rendered via render_table_cell, but handle gracefully
            div()
                .p(px(TABLE_CELL_PADDING))
                .children(node.children().map(|child| render_markdown_ast(child, cx)))
                .into_any_element()
        }

        // Fallback: walk children
        _ => div()
            .children(node.children().map(|child| render_markdown_ast(child, cx)))
            .into_any_element(),
    }
}

/// Render a table row with proper alignment and header styling
fn render_table_row<'a, T: 'static>(
    row_node: &'a AstNode<'a>,
    alignments: &[comrak::nodes::TableAlignment],
    cx: &mut Context<T>,
) -> AnyElement {
    let is_header = matches!(row_node.data.borrow().value, NodeValue::TableRow(true));

    let mut row_div = div()
        .flex()
        .w_full()
        .border_b_1()
        .border_color(TABLE_BORDER_COLOR);

    if is_header {
        row_div = row_div.bg(TABLE_HEADER_BG).font_weight(FontWeight::BOLD);
    }

    // Render cells with alignment
    let cells: Vec<AnyElement> = row_node
        .children()
        .enumerate()
        .map(|(idx, cell)| render_table_cell(cell, alignments.get(idx), cx))
        .collect();

    row_div.children(cells).into_any_element()
}

/// Render a table cell with alignment
fn render_table_cell<'a, T: 'static>(
    cell_node: &'a AstNode<'a>,
    alignment: Option<&comrak::nodes::TableAlignment>,
    cx: &mut Context<T>,
) -> AnyElement {
    use comrak::nodes::TableAlignment;

    let mut cell_div = div()
        .flex_1()
        .p(px(TABLE_CELL_PADDING))
        .border_r_1()
        .border_color(TABLE_BORDER_COLOR);

    // Apply alignment
    cell_div = match alignment {
        Some(TableAlignment::Left) | None => cell_div.justify_start(),
        Some(TableAlignment::Center) => cell_div.justify_center(),
        Some(TableAlignment::Right) => cell_div.justify_end(),
        Some(TableAlignment::None) => cell_div.justify_start(),
    };

    cell_div
        .children(
            cell_node
                .children()
                .map(|child| render_markdown_ast(child, cx)),
        )
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
fn open_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
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
