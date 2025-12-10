use gpui::{Context, KeyDownEvent, ScrollWheelEvent, px};
use tracing::{debug, info};

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
        match viewer.search_state.take() {
            Some(_) => {
                // Exit search mode
                debug!("Exiting search mode");
                viewer.search_input.clear();
            }
            None => {
                // Enter search mode
                debug!("Entering search mode");
                viewer.search_state =
                    Some(SearchState::new(String::new(), &viewer.markdown_content));
            }
        }
        cx.notify();
        return;
    }

    // Check for Cmd+G (macOS) or Ctrl+G (other platforms) to toggle go-to-line
    if event.keystroke.key.as_str() == "g"
        && (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
    {
        debug!("Go-to-line shortcut triggered (Cmd/Ctrl+G)");
        match viewer.show_goto_line {
            true => {
                // Exit go-to-line mode
                debug!("Exiting go-to-line mode");
                viewer.show_goto_line = false;
                viewer.goto_line_input.clear();
            }
            false => {
                // Enter go-to-line mode
                debug!("Entering go-to-line mode");
                viewer.show_goto_line = true;
                viewer.goto_line_input.clear();
            }
        }
        cx.notify();
        return;
    }

    // Check for Cmd+Shift+H (macOS) or Ctrl+Shift+H (other platforms) to clear search history
    if (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
        && event.keystroke.modifiers.shift
        && event.keystroke.key.as_str() == "h"
    {
        debug!("Clear search history shortcut triggered (Cmd/Ctrl+Shift+H)");
        viewer.config.search_history.clear();
        viewer.search_history_index = None;
        // Save config
        match viewer.config.save_to_file("config.ron") {
            Err(e) => {
                debug!("Failed to save cleared search history: {}", e);
                viewer.search_history_message = Some(format!("Failed to save: {}", e));
            }
            Ok(_) => {
                info!("Search history cleared");
                viewer.search_history_message = Some("Search history cleared".to_string());
            }
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

    // Check for Cmd+Shift+B (macOS) or Ctrl+Shift+B (other platforms) to toggle bookmarks list
    if (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
        && event.keystroke.modifiers.shift
        && event.keystroke.key.as_str() == "b"
    {
        debug!("Toggle bookmarks list shortcut triggered (Cmd/Ctrl+Shift+B)");
        viewer.show_bookmarks = !viewer.show_bookmarks;
        cx.notify();
        return;
    }

    // Check for Cmd+D (macOS) or Ctrl+D (other platforms) to toggle bookmark
    if (event.keystroke.modifiers.platform || event.keystroke.modifiers.control)
        && event.keystroke.key.as_str() == "d"
    {
        debug!("Toggle bookmark shortcut triggered (Cmd/Ctrl+D)");
        let current_line = viewer.get_current_line_number();

        match viewer.bookmarks.iter().position(|&l| l == current_line) {
            Some(pos) => {
                // Remove existing bookmark
                viewer.bookmarks.remove(pos);
                debug!("Removed bookmark at line {}", current_line);
            }
            None => {
                // Add new bookmark
                viewer.bookmarks.push(current_line);
                viewer.bookmarks.sort(); // Keep sorted
                debug!("Added bookmark at line {}", current_line);
            }
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
            "e" => {
                debug!("Export to PDF (Cmd+E)");
                // Trigger PDF export
                // We'll handle the actual export in the viewer's render method via an action
                viewer.trigger_pdf_export = true;
                cx.notify();
                return;
            }
            _ => {}
        }
    }

    // Also handle Ctrl+E on non-Mac platforms
    if event.keystroke.modifiers.control && event.keystroke.key.as_str() == "e" {
        debug!("Export to PDF (Ctrl+E)");
        viewer.trigger_pdf_export = true;
        cx.notify();
        return;
    }

    // Handle Escape to close help overlay
    if viewer.show_help && event.keystroke.key.as_str() == "escape" {
        viewer.show_help = false;
        cx.notify();
        return;
    }

    // Handle Escape to close PDF export notification
    if viewer.pdf_export_message.is_some() && event.keystroke.key.as_str() == "escape" {
        viewer.pdf_export_message = None;
        cx.notify();
        return;
    }

    // Handle PDF overwrite confirmation (Y/N)
    if viewer.show_pdf_overwrite_confirm {
        match event.keystroke.key.as_str() {
            "y" | "Y" => {
                debug!("User confirmed PDF overwrite");
                viewer.show_pdf_overwrite_confirm = false;
                // Export will happen in render() when show_pdf_overwrite_confirm is false
                cx.notify();
                return;
            }
            "n" | "N" | "escape" => {
                debug!("User cancelled PDF overwrite");
                viewer.show_pdf_overwrite_confirm = false;
                viewer.pdf_overwrite_path = None;
                cx.notify();
                return;
            }
            _ => {}
        }
    }

    // ========== KEYBOARD-ONLY NAVIGATION ==========
    // Handle Tab/Shift-Tab for focus cycling (only when not in input modes)
    if viewer.search_state.is_none() && !viewer.show_goto_line {
        if event.keystroke.key.as_str() == "tab" {
            match event.keystroke.modifiers.shift {
                true => {
                    // Shift+Tab: focus previous
                    debug!("Shift+Tab: focus previous element");
                    viewer.focus_previous();
                }
                false => {
                    // Tab: focus next
                    debug!("Tab: focus next element");
                    viewer.focus_next();
                }
            }
            cx.notify();
            return;
        }

        // Handle Enter key to activate focused element (when not in input modes)
        if event.keystroke.key.as_str() == "enter" && viewer.current_focus_index.is_some() {
            debug!("Enter: activating focused element");
            if viewer.activate_focused_element() {
                cx.notify();
            }
            return;
        }
    }

    // Vi-style navigation (j/k for down/up) - only when not in input modes
    if viewer.search_state.is_none() && !viewer.show_goto_line {
        match event.keystroke.key.as_str() {
            "j" => {
                debug!("Vi-style: j (scroll down)");
                viewer.scroll_state.scroll_down(arrow_increment);
                cx.notify();
                return;
            }
            "k" => {
                debug!("Vi-style: k (scroll up)");
                viewer.scroll_state.scroll_up(arrow_increment);
                cx.notify();
                return;
            }
            _ => {}
        }
    }

    // Handle search mode input
    if viewer.search_state.is_some() {
        match event.keystroke.key.as_str() {
            "escape" => {
                // Exit search mode
                debug!("Exiting search mode (Escape)");
                viewer.search_state = None;
                viewer.search_input.clear();
                viewer.search_history_index = None;
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
                // Next match AND save to history
                if !viewer.search_input.trim().is_empty() {
                    let input = viewer.search_input.clone();
                    let history = &mut viewer.config.search_history;

                    // Add to history if it's different from the last item
                    if history.last() != Some(&input) {
                        history.push(input.clone());
                        // Enforce max items
                        if history.len() > viewer.config.max_history_items {
                            history.remove(0);
                        }
                        // Save config
                        match viewer.config.save_to_file("config.ron") {
                            Err(e) => {
                                debug!("Failed to save search history: {}", e);
                            }
                            Ok(_) => {
                                info!("Saved to search history: '{}'", input);
                            }
                        }
                    }
                }

                if let Some(state) = &mut viewer.search_state {
                    state.next_match();
                    debug!("Next match (key_down): {:?}", state.current_match_number());
                    viewer.scroll_to_current_match();
                }
                cx.notify();
                return;
            }
            "up" => {
                // Navigate history back
                let history_len = viewer.config.search_history.len();
                if history_len > 0 {
                    let new_index = match viewer.search_history_index {
                        None => history_len - 1,
                        Some(i) if i > 0 => i - 1,
                        Some(_) => 0, // Stay at start
                    };

                    viewer.search_history_index = Some(new_index);
                    if let Some(item) = viewer.config.search_history.get(new_index) {
                        viewer.search_input = item.clone();
                        viewer.search_state = Some(SearchState::new(
                            viewer.search_input.clone(),
                            &viewer.markdown_content,
                        ));
                        viewer.scroll_to_current_match();
                    }
                }
                cx.notify();
                return;
            }
            "down" => {
                // Navigate history forward
                if let Some(i) = viewer.search_history_index {
                    let history_len = viewer.config.search_history.len();
                    match i.checked_add(1) {
                        Some(new_index) if new_index < history_len => {
                            viewer.search_history_index = Some(new_index);
                            if let Some(item) = viewer.config.search_history.get(new_index) {
                                viewer.search_input = item.clone();
                                viewer.search_state = Some(SearchState::new(
                                    viewer.search_input.clone(),
                                    &viewer.markdown_content,
                                ));
                                viewer.scroll_to_current_match();
                            }
                        }
                        _ => {
                            // End of history, clear input
                            viewer.search_history_index = None;
                            viewer.search_input.clear();
                            viewer.search_state =
                                Some(SearchState::new(String::new(), &viewer.markdown_content));
                        }
                    }
                    cx.notify();
                }
                return;
            }
            "backspace" => {
                // Remove last character
                viewer.search_input.pop();
                viewer.search_history_index = None; // Reset history index on manual edit
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
                viewer.search_history_index = None; // Reset history index on manual edit
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
                match MarkdownViewer::parse_line_number(&viewer.goto_line_input) {
                    Some(line_number) => match viewer.scroll_to_line(line_number) {
                        Ok(()) => {
                            debug!("Scrolled to line {}", line_number);
                            viewer.show_goto_line = false;
                            viewer.goto_line_input.clear();
                        }
                        Err(e) => {
                            debug!("Failed to scroll to line {}: {}", line_number, e);
                            // Keep dialog open to show error (could add error message display later)
                        }
                    },
                    None => {
                        debug!("Invalid line number: '{}'", viewer.goto_line_input);
                        // Keep dialog open for invalid input
                    }
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
    match delta_f32 {
        d if d > 0.0 => viewer.scroll_state.scroll_up(d),
        d => viewer.scroll_state.scroll_down(-d),
    }
    cx.notify();
}
