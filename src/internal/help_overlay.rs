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

use crate::{BG_COLOR, TEXT_COLOR};
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
pub fn help_panel() -> impl IntoElement {
    // Panel with background and text color applied so callers can place this directly
    // into the centered overlay container without re-styling.
    div()
        .bg(BG_COLOR)
        .text_color(TEXT_COLOR)
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
        .child(
            div()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .child("Keyboard Shortcuts"),
                )
                .child(
                    div()
                        .flex_col()
                        .gap_2()
                        .child(shortcut_row("Cmd + H", "Toggle Help"))
                        .child(shortcut_row("Cmd + Z", "Toggle TOC"))
                        .child(shortcut_row("Cmd + F", "Search"))
                        .child(shortcut_row("Cmd + T", "Scroll to Top"))
                        .child(shortcut_row("Cmd + B", "Scroll to Bottom"))
                        .child(shortcut_row("Cmd + +", "Increase Font Size"))
                        .child(shortcut_row("Cmd + -", "Decrease Font Size"))
                        .child(shortcut_row("Cmd + Q", "Quit"))
                        .child(shortcut_row("Esc", "Close Overlay / Search")),
                ),
        )
}
