/*
help_overlay.rs

Provides reusable UI building blocks for the help overlay used by the
application UI.

This module exposes:
- `shortcut_row(key, desc)` - small two-column row with a bold shortcut and a description.
- `help_panel()` - a self-contained panel element that shows the "Keyboard Shortcuts" title
  and the canonical list of shortcut rows used by the app.

The functions return `impl IntoElement` so they can be composed directly into other gpui
elements (for example: `element.child(help_overlay::help_panel())`).

Note: styling (colors, fonts, sizes) is intentionally kept minimal here so the caller
can further wrap or style the returned element as needed. This keeps the module focused
on layout and content.
*/

use gpui::{FontWeight, IntoElement, Rgba, div, prelude::*};

/// Render a single shortcut row: bold key on the left, description on the right.
///
/// Example:
///     help_overlay::shortcut_row("Cmd + F", "Search")
pub fn shortcut_row(key: &str, desc: &str) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .gap_8()
        .child(div().font_weight(FontWeight::BOLD).child(key.to_string()))
        .child(div().child(desc.to_string()))
}

/// Build the help panel body which lists keyboard shortcuts.
///
/// The returned element is meant to be placed inside a styled container by the caller,
/// for example wrapped with background, shadow, padding, etc.
pub fn help_panel(
    theme_colors: &crate::internal::theme::ThemeColors,
    page_index: usize,
) -> impl IntoElement {
    let content = match page_index {
        0 => div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Keyboard Shortcuts (1/2)"),
            )
            .child(
                div()
                    .flex_col()
                    .gap_2()
                    .child(shortcut_row("Cmd + H", "Toggle Help"))
                    .child(shortcut_row("Cmd + Z", "Toggle TOC"))
                    .child(shortcut_row("Cmd + F", "Search (Up/Down for History)"))
                    .child(shortcut_row("Cmd + P", "Go to File"))
                    .child(shortcut_row("Cmd + Shift + O", "Open Recent"))
                    .child(shortcut_row("Cmd + Shift + H", "Clear Search History"))
                    .child(shortcut_row("Cmd + G", "Go to Line"))
                    .child(shortcut_row("Cmd + E", "Export to PDF"))
                    .child(shortcut_row("Cmd + Shift + T", "Toggle Theme"))
                    .child(shortcut_row("Cmd + Shift + N", "Cycle Theme Family"))
                    .child(shortcut_row("Cmd + D", "Toggle Bookmark"))
                    .child(shortcut_row("Cmd + Shift + B", "View Bookmarks"))
                    .child(shortcut_row("Cmd + + / -", "Zoom In / Out"))
                    .child(shortcut_row("Esc", "Close Overlay / Search")),
            )
            .child(
                div()
                    .text_color(theme_colors.text_color)
                    .opacity(0.7)
                    .text_sm()
                    .child("Use Right Arrow for Navigation shortcuts →"),
            ),
        _ => div()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Navigation Shortcuts (2/2)"),
            )
            .child(
                div()
                    .flex_col()
                    .gap_2()
                    .child(shortcut_row("j / k", "Scroll Down / Up"))
                    .child(shortcut_row("Arrow keys", "Scroll limit"))
                    .child(shortcut_row("Ctrl + D / U", "Half-Page Down / Up"))
                    .child(shortcut_row("PageUp / Down", "Page Scroll"))
                    .child(shortcut_row("Space (+Shift)", "Page Scroll"))
                    .child(shortcut_row("g", "Scroll to Top"))
                    .child(shortcut_row("G (Shift+g)", "Scroll to Bottom"))
                    .child(shortcut_row("Cmd + T / B", "Scroll to Top / Bottom"))
                    .child(shortcut_row("zz", "Center View"))
                    .child(shortcut_row("m + <char>", "Set Mark"))
                    .child(shortcut_row("' + <char>", "Jump to Mark")),
            )
            .child(
                div()
                    .text_color(theme_colors.text_color)
                    .opacity(0.7)
                    .text_sm()
                    .child("← Use Left Arrow for General shortcuts"),
            ),
    };

    div()
        .bg(theme_colors.bg_color)
        .text_color(theme_colors.text_color)
        .rounded_xl()
        .p_8()
        .shadow_lg()
        .border_1()
        .border_color(Rgba {
            r: 0.8,
            g: 0.8,
            b: 0.8,
            a: 1.0,
        })
        .child(content)
}
