//! Table of Contents (TOC) module
//!
//! Extracts headings from Markdown documents and provides hierarchical navigation structure.

use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeValue};

/// A single entry in the table of contents
#[derive(Debug, Clone)]
pub struct TocEntry {
    /// The text content of the heading
    pub text: String,
    /// Heading level (1-6 for H1-H6)
    pub level: u8,
    /// Approximate vertical position in the document (line-based)
    pub line_number: usize,
}

/// Table of Contents for a Markdown document
#[derive(Debug, Clone)]
pub struct TableOfContents {
    /// List of heading entries in document order
    pub entries: Vec<TocEntry>,
}

impl TableOfContents {
    /// Create a new empty table of contents
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Build table of contents from a comrak AST
    pub fn from_ast<'a>(root: &'a Node<'a, std::cell::RefCell<Ast>>) -> Self {
        let mut toc = Self::new();
        toc.extract_headings(root);
        toc
    }

    /// Recursively extract headings from the AST
    fn extract_headings<'a>(&mut self, node: &'a Node<'a, std::cell::RefCell<Ast>>) {
        let ast = node.data.borrow();

        // Check if this node is a heading
        if let NodeValue::Heading(heading) = &ast.value {
            let level = heading.level;
            // Only include levels 2, 3, and 4 as requested
            if (2..=4).contains(&level) {
                let text = extract_text_from_node(node);
                // sourcepos.start.line is 1-based, convert to 0-based
                let line_number = ast.sourcepos.start.line.saturating_sub(1);

                self.entries.push(TocEntry {
                    text,
                    level,
                    line_number,
                });
            }
        }

        // Recursively process children
        for child in node.children() {
            self.extract_headings(child);
        }
    }

    /// Find the current active section based on scroll position
    /// Returns the index of the TocEntry, or None if no entries
    pub fn find_current_section(&self, scroll_y: f32, line_height: f32) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }

        // Add offset so highlighting doesn't trigger too early
        // This ensures the section is well into view before highlighting
        const HIGHLIGHT_OFFSET: f32 = 100.0;
        let adjusted_scroll_y = (scroll_y + HIGHLIGHT_OFFSET).max(0.0);

        // Convert scroll position to approximate line number
        let current_line = (adjusted_scroll_y / line_height) as usize;

        // Find the last entry whose line_number is <= current_line
        let mut current_idx = None;
        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.line_number <= current_line {
                current_idx = Some(idx);
            } else {
                break;
            }
        }

        current_idx
    }
}

/// Extract plain text content from a node and its children
fn extract_text_from_node<'a>(node: &'a Node<'a, std::cell::RefCell<Ast>>) -> String {
    let mut text = String::new();

    for child in node.children() {
        let child_ast = child.data.borrow();
        if let NodeValue::Text(ref t) = child_ast.value {
            text.push_str(t);
        } else {
            // Recursively extract from nested nodes
            text.push_str(&extract_text_from_node(child));
        }
    }

    text
}

impl Default for TableOfContents {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{Arena, Options, parse_document};

    #[test]
    fn test_empty_document() {
        let arena = Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "", &options);
        let toc = TableOfContents::from_ast(root);
        assert_eq!(toc.entries.len(), 0);
    }

    #[test]
    fn test_simple_toc() {
        let arena = Arena::new();
        let options = Options::default();
        let markdown = "## Hello World";
        let root = parse_document(&arena, markdown, &options);
        let toc = TableOfContents::from_ast(root);

        assert_eq!(toc.entries.len(), 1);
        assert_eq!(toc.entries[0].text, "Hello World");
        assert_eq!(toc.entries[0].level, 2);
        // Line 0 (1-based line 1)
        assert_eq!(toc.entries[0].line_number, 0);
    }

    #[test]
    fn test_multiple_headings() {
        let arena = Arena::new();
        let options = Options::default();
        let markdown = r#"
# Title (Ignored)
## Subtitle
### Section
## Another Subtitle
##### Too Deep (Ignored)
"#;
        let root = parse_document(&arena, markdown, &options);
        let toc = TableOfContents::from_ast(root);

        assert_eq!(toc.entries.len(), 3);
        assert_eq!(toc.entries[0].level, 2);
        assert_eq!(toc.entries[1].level, 3);
        assert_eq!(toc.entries[2].level, 2);
    }

    #[test]
    fn test_find_current_section() {
        let mut toc = TableOfContents::new();
        toc.entries.push(TocEntry {
            text: "Section 1".to_string(),
            level: 2,
            line_number: 0,
        });
        toc.entries.push(TocEntry {
            text: "Section 2".to_string(),
            level: 2,
            line_number: 10,
        });
        toc.entries.push(TocEntry {
            text: "Section 3".to_string(),
            level: 2,
            line_number: 20,
        });

        // At line 5 (scroll_y = 5 * 20 = 100), should be in Section 1 (index 0)
        assert_eq!(toc.find_current_section(100.0, 20.0), Some(0));

        // At line 15 (scroll_y = 15 * 20 = 300), should be in Section 2 (index 1)
        assert_eq!(toc.find_current_section(300.0, 20.0), Some(1));

        // At line 25 (scroll_y = 25 * 20 = 500), should be in Section 3 (index 2)
        assert_eq!(toc.find_current_section(500.0, 20.0), Some(2));
    }
}
