//! Helper function to render text with search highlighting
use super::search::SearchState;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div};

/// Render text with search match highlighting
pub fn render_text_with_search(text: &str, search_state: Option<&SearchState>) -> Vec<AnyElement> {
    let Some(search_state) = search_state else {
        // No search active, render plain text
        return vec![div().child(text.to_string()).into_any_element()];
    };

    if search_state.match_count() == 0 {
        // No matches, render plain text
        return vec![div().child(text.to_string()).into_any_element()];
    }

    // Find matches in this specific text segment
    let text_lower = text.to_lowercase();
    let query = search_state.query();

    if query.is_empty() {
        return vec![div().child(text.to_string()).into_any_element()];
    }

    let query_lower = query.to_lowercase();
    let mut elements = Vec::new();
    let mut last_end = 0;

    // Find all matches in this text
    let mut start = 0;
    while let Some(pos) = text_lower[start..].find(&query_lower) {
        let match_start = start + pos;
        let match_end = match_start + query.len();

        // Add text before match
        if match_start > last_end {
            elements.push(
                div()
                    .child(text[last_end..match_start].to_string())
                    .into_any_element(),
            );
        }

        // Highlight match in yellow
        elements.push(
            div()
                .bg(super::style::SEARCH_BG_COLOR)
                .child(text[match_start..match_end].to_string())
                .into_any_element(),
        );

        last_end = match_end;
        start = match_end;
    }

    // Add remaining text
    if last_end < text.len() {
        elements.push(div().child(text[last_end..].to_string()).into_any_element());
    }

    elements
}
