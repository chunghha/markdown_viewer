//! Search functionality for the markdown viewer
//!
//! This module provides text search capabilities including:
//! - Case-insensitive search
//! - Match tracking and navigation
//! - Position information for highlighting

/// Represents a single match position in the text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchPosition {
    /// Byte offset where the match starts
    pub start: usize,
    /// Byte offset where the match ends (exclusive)
    pub end: usize,
}

/// Search state tracking query, matches, and current position
#[derive(Debug, Clone)]
pub struct SearchState {
    /// Current search query
    query: String,
    /// All match positions in the text
    matches: Vec<MatchPosition>,
    /// Index of the currently selected match (if any)
    current_index: Option<usize>,
}

impl SearchState {
    /// Create a new search state with the given query and text
    pub fn new(query: String, text: &str) -> Self {
        let matches = match query.as_str() {
            "" => Vec::new(),
            _ => find_matches(&query, text),
        };

        let current_index = match matches.as_slice() {
            [] => None,
            _ => Some(0),
        };

        Self {
            query,
            matches,
            current_index,
        }
    }

    /// Get the current search query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get all match positions
    pub fn matches(&self) -> &[MatchPosition] {
        &self.matches
    }

    /// Get the total number of matches
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get the current match index (1-based for display)
    pub fn current_match_number(&self) -> Option<usize> {
        self.current_index.map(|i| i + 1)
    }

    /// Get the current match position
    pub fn current_match(&self) -> Option<&MatchPosition> {
        self.current_index.and_then(|i| self.matches.get(i))
    }

    /// Move to the next match (wraps around)
    pub fn next_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        self.current_index = Some(match self.current_index {
            Some(i) if i + 1 < self.matches.len() => i + 1,
            _ => 0, // Wrap to first
        });
    }

    /// Move to the previous match (wraps around)
    pub fn prev_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        self.current_index = Some(match self.current_index {
            Some(0) | None => self.matches.len() - 1, // Wrap to last
            Some(i) => i - 1,
        });
    }

    /// Check if a byte position is within any match
    pub fn is_match_at(&self, pos: usize) -> bool {
        self.matches.iter().any(|m| pos >= m.start && pos < m.end)
    }

    /// Check if a byte position is within the current match
    pub fn is_current_match_at(&self, pos: usize) -> bool {
        self.current_match()
            .map(|m| pos >= m.start && pos < m.end)
            .unwrap_or(false)
    }
}

/// Find all case-insensitive matches of the query in the text
fn find_matches(query: &str, text: &str) -> Vec<MatchPosition> {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut matches = Vec::new();
    let mut start = 0;

    while let Some(pos) = text_lower[start..].find(&query_lower) {
        let match_start = start + pos;
        let match_end = match_start + query.len();

        matches.push(MatchPosition {
            start: match_start,
            end: match_end,
        });

        start = match_end;
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_single_match() {
        let text = "Hello world";
        let state = SearchState::new("world".to_string(), text);

        assert_eq!(state.match_count(), 1);
        assert_eq!(state.current_match_number(), Some(1));

        let m = state.current_match().unwrap();
        assert_eq!(m.start, 6);
        assert_eq!(m.end, 11);
    }

    #[test]
    fn test_find_multiple_matches() {
        let text = "foo bar foo baz foo";
        let state = SearchState::new("foo".to_string(), text);

        assert_eq!(state.match_count(), 3);
        assert_eq!(state.matches()[0].start, 0);
        assert_eq!(state.matches()[1].start, 8);
        assert_eq!(state.matches()[2].start, 16);
    }

    #[test]
    fn test_case_insensitive() {
        let text = "Markdown MARKDOWN markdown";
        let state = SearchState::new("markdown".to_string(), text);

        assert_eq!(state.match_count(), 3);
    }

    #[test]
    fn test_no_matches() {
        let text = "Hello world";
        let state = SearchState::new("xyz".to_string(), text);

        assert_eq!(state.match_count(), 0);
        assert_eq!(state.current_match_number(), None);
    }

    #[test]
    fn test_empty_query() {
        let text = "Hello world";
        let state = SearchState::new("".to_string(), text);

        assert_eq!(state.match_count(), 0);
    }

    #[test]
    fn test_navigation_forward() {
        let text = "a b a b a";
        let mut state = SearchState::new("a".to_string(), text);

        assert_eq!(state.current_match_number(), Some(1));

        state.next_match();
        assert_eq!(state.current_match_number(), Some(2));

        state.next_match();
        assert_eq!(state.current_match_number(), Some(3));

        // Wrap around
        state.next_match();
        assert_eq!(state.current_match_number(), Some(1));
    }

    #[test]
    fn test_navigation_backward() {
        let text = "a b a b a";
        let mut state = SearchState::new("a".to_string(), text);

        // Start at first, go back wraps to last
        state.prev_match();
        assert_eq!(state.current_match_number(), Some(3));

        state.prev_match();
        assert_eq!(state.current_match_number(), Some(2));

        state.prev_match();
        assert_eq!(state.current_match_number(), Some(1));
    }

    #[test]
    fn test_is_match_at() {
        let text = "Hello world";
        let state = SearchState::new("world".to_string(), text);

        assert!(!state.is_match_at(0));
        assert!(!state.is_match_at(5));
        assert!(state.is_match_at(6));
        assert!(state.is_match_at(10));
        assert!(!state.is_match_at(11));
    }

    #[test]
    fn test_is_current_match_at() {
        let text = "foo bar foo";
        let mut state = SearchState::new("foo".to_string(), text);

        // First match is current
        assert!(state.is_current_match_at(0));
        assert!(!state.is_current_match_at(8));

        // Move to second match
        state.next_match();
        assert!(!state.is_current_match_at(0));
        assert!(state.is_current_match_at(8));
    }
}
