use gpui::{Context, KeyDownEvent, ScrollWheelEvent, px};
use tracing::debug;

use crate::internal::search::SearchState;
use crate::internal::viewer::MarkdownViewer;

pub fn handle_key_down(
    viewer: &mut MarkdownViewer,
    event: &KeyDownEvent,
    _window: &mut gpui::Window,
    cx: &mut Context<MarkdownViewer>,
) {
    let arrow_increment = viewer.config.scroll.arrow_key_increment;
    let page_percent = viewer.config.scroll.page_scroll_percentage;
    let space_percent = viewer.config.scroll.space_scroll_percentage;

    // Debug: log all key events
    debug!(
        "Key pressed: '{}', platform: {}, control: {}, shift: {}, alt: {}",
        event.keystroke.key,
        event.keystroke.modifiers.platform,
        event.keystroke.modifiers.control,
        event.keystroke.modifiers.shift,
        event.keystroke.modifiers.alt
    );

    // Check for Cmd+F (macOS) or Ctrl+F (other platforms) to toggle search
    if event.keystroke.key.as_str() == "f"
        && (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
    {
        debug!("Search shortcut triggered (Cmd/Ctrl+F)");
        if viewer.search_state.is_some() {
            // Exit search mode
            debug!("Exiting search mode");
            viewer.search_state = None;
            viewer.search_input.clear();
        } else {
            // Enter search mode
            debug!("Entering search mode");
            viewer.search_state = Some(SearchState::new(String::new(), &viewer.markdown_content));
        }
        cx.notify();
        return;
    }

    // Check for Cmd+G (macOS) or Ctrl+G (other platforms) to toggle go-to-line
    if event.keystroke.key.as_str() == "g"
        && (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
    {
        debug!("Go-to-line shortcut triggered (Cmd/Ctrl+G)");
        if viewer.show_goto_line {
            // Exit go-to-line mode
            debug!("Exiting go-to-line mode");
            viewer.show_goto_line = false;
            viewer.goto_line_input.clear();
        } else {
            // Enter go-to-line mode
            debug!("Entering go-to-line mode");
            viewer.show_goto_line = true;
            viewer.goto_line_input.clear();
        }
        cx.notify();
        return;
    }

    // Check for Cmd+Shift+T (macOS) or Ctrl+Shift+T (other platforms) to toggle theme
    // This must come BEFORE the platform modifier checks to avoid conflicts with Cmd+T
    if (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
        && event.keystroke.modifiers.shift
        && event.keystroke.key.as_str() == "t"
    {
        debug!("Theme toggle shortcut triggered (Cmd/Ctrl+Shift+T)");
        viewer.config.theme.theme = viewer.config.theme.theme.toggle();
        // Save config to persist theme preference
        if let Err(e) = viewer.config.save_to_file("config.ron") {
            debug!("Failed to save theme preference: {}", e);
        }
        cx.notify();
        return;
    }

    // Handle global shortcuts (Cmd+T, Cmd+B, Cmd+Q, Cmd+=, Cmd+-, Cmd+H)
    if event.keystroke.modifiers.platform {
        match event.keystroke.key.as_str() {
            "t" => {
                debug!("Scroll to top (Cmd+T)");
                viewer.scroll_state.scroll_to_top();
                cx.notify();
                return;
            }
            "b" => {
                debug!("Scroll to bottom (Cmd+B)");
                viewer.scroll_state.scroll_to_bottom();
                cx.notify();
                return;
            }
            "q" => {
                debug!("Quit application (Cmd+Q)");
                cx.quit();
                return;
            }
            "=" | "+" => {
                debug!("Increase font size (Cmd+=)");
                let new_size = (viewer.config.theme.base_text_size + 2.0).min(64.0);
                if (new_size - viewer.config.theme.base_text_size).abs() > 0.01 {
                    viewer.config.theme.base_text_size = new_size;
                    viewer.recompute_max_scroll();
                    cx.notify();
                }
                return;
            }
            "-" => {
                debug!("Decrease font size (Cmd+-)");
                let new_size = (viewer.config.theme.base_text_size - 2.0).max(8.0);
                if (new_size - viewer.config.theme.base_text_size).abs() > 0.01 {
                    viewer.config.theme.base_text_size = new_size;
                    viewer.recompute_max_scroll();
                    cx.notify();
                }
                return;
            }
            "h" => {
                debug!("Toggle help overlay (Cmd+H)");
                viewer.show_help = !viewer.show_help;
                cx.notify();
                return;
            }
            "z" => {
                debug!("Toggle TOC sidebar (Cmd+Z)");
                viewer.show_toc = !viewer.show_toc;
                viewer.recompute_max_scroll();
                cx.notify();
                return;
            }
            _ => {}
        }
    }

    // Handle Escape to close help overlay
    if viewer.show_help && event.keystroke.key.as_str() == "escape" {
        viewer.show_help = false;
        cx.notify();
        return;
    }

    // Handle search mode input
    if viewer.search_state.is_some() {
        match event.keystroke.key.as_str() {
            "escape" => {
                // Exit search mode
                debug!("Exiting search mode (Escape)");
                viewer.search_state = None;
                viewer.search_input.clear();
                cx.notify();
                return;
            }
            "enter" if event.keystroke.modifiers.shift => {
                // Previous match
                if let Some(state) = &mut viewer.search_state {
                    state.prev_match();
                    debug!(
                        "Previous match (key_down): {:?}",
                        state.current_match_number()
                    );
                    viewer.scroll_to_current_match();
                }
                cx.notify();
                return;
            }
            "enter" => {
                // Next match
                if let Some(state) = &mut viewer.search_state {
                    state.next_match();
                    debug!("Next match (key_down): {:?}", state.current_match_number());
                    viewer.scroll_to_current_match();
                }
                cx.notify();
                return;
            }
            "backspace" => {
                // Remove last character
                viewer.search_input.pop();
                viewer.search_state = Some(SearchState::new(
                    viewer.search_input.clone(),
                    &viewer.markdown_content,
                ));
                debug!("Search query: '{}'", viewer.search_input);
                viewer.scroll_to_current_match();
                cx.notify();
                return;
            }
            key if key.len() == 1
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.platform =>
            {
                // Add character to search
                viewer.search_input.push_str(key);
                viewer.search_state = Some(SearchState::new(
                    viewer.search_input.clone(),
                    &viewer.markdown_content,
                ));
                debug!("Search query: '{}'", viewer.search_input);
                viewer.scroll_to_current_match();
                cx.notify();
                return;
            }
            _ => {}
        }
    }

    // Handle go-to-line mode input
    if viewer.show_goto_line {
        match event.keystroke.key.as_str() {
            "escape" => {
                // Exit go-to-line mode
                debug!("Exiting go-to-line mode (Escape)");
                viewer.show_goto_line = false;
                viewer.goto_line_input.clear();
                cx.notify();
                return;
            }
            "enter" => {
                // Execute go-to-line
                debug!("Go-to-line execute: '{}'", viewer.goto_line_input);
                if let Some(line_number) =
                    MarkdownViewer::parse_line_number(&viewer.goto_line_input)
                {
                    match viewer.scroll_to_line(line_number) {
                        Ok(()) => {
                            debug!("Scrolled to line {}", line_number);
                            viewer.show_goto_line = false;
                            viewer.goto_line_input.clear();
                        }
                        Err(e) => {
                            debug!("Failed to scroll to line {}: {}", line_number, e);
                            // Keep dialog open to show error (could add error message display later)
                        }
                    }
                } else {
                    debug!("Invalid line number: '{}'", viewer.goto_line_input);
                    // Keep dialog open for invalid input
                }
                cx.notify();
                return;
            }
            "backspace" => {
                // Remove last character
                viewer.goto_line_input.pop();
                debug!("Go-to-line input: '{}'", viewer.goto_line_input);
                cx.notify();
                return;
            }
            key if key.len() == 1
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.platform =>
            {
                // Add character to input (only digits)
                if key.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    viewer.goto_line_input.push_str(key);
                    debug!("Go-to-line input: '{}'", viewer.goto_line_input);
                    cx.notify();
                }
                return;
            }
            _ => {}
        }
    }

    match event.keystroke.key.as_str() {
        "up" => viewer.scroll_state.scroll_up(arrow_increment),
        "down" => viewer.scroll_state.scroll_down(arrow_increment),
        "pageup" => viewer
            .scroll_state
            .page_up(viewer.viewport_height * page_percent),
        "pagedown" => viewer
            .scroll_state
            .page_down(viewer.viewport_height * page_percent),
        "home" => viewer.scroll_state.scroll_to_top(),
        "end" => viewer.scroll_state.scroll_to_bottom(),
        "space" if event.keystroke.modifiers.shift => viewer
            .scroll_state
            .page_up(viewer.viewport_height * space_percent),
        "space" => viewer
            .scroll_state
            .page_down(viewer.viewport_height * space_percent),
        _ => {}
    }
    cx.notify();
}

pub fn handle_scroll_wheel(
    viewer: &mut MarkdownViewer,
    event: &ScrollWheelEvent,
    _window: &mut gpui::Window,
    cx: &mut Context<MarkdownViewer>,
) {
    let delta = event
        .delta
        .pixel_delta(px(viewer.config.theme.base_text_size))
        .y;
    let delta_f32: f32 = delta.into();
    if delta_f32 > 0.0 {
        viewer.scroll_state.scroll_up(delta_f32);
    } else {
        viewer.scroll_state.scroll_down(-delta_f32);
    }
    cx.notify();
}
