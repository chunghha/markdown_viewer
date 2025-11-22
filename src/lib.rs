//! Markdown Viewer Library
//!
//! This module provides the public API for the markdown viewer,
//! including scrolling, rendering, and file handling functionality.

pub mod config;
mod internal;

// Re-export public types and functions
pub use internal::events;
pub use internal::file_handling::{
    is_supported_extension, load_markdown_content, resolve_image_path, resolve_markdown_file_path,
};
pub use internal::file_watcher::{FileWatcherEvent, start_watching};
pub use internal::rendering::{
    render_markdown_ast, render_markdown_ast_with_loader, render_markdown_ast_with_search,
};
pub use internal::scroll::ScrollState;
pub use internal::search::SearchState;
pub use internal::style::*;
pub use internal::ui;
pub use internal::viewer::{ImageState, MarkdownViewer};

// Re-export internal helpers that are useful to binary targets (controlled exposure)
pub use internal::image::{rasterize_svg_to_dynamic_image, rgba_to_bgra};
// Expose high-level image loading helper so binary targets can call it
// without reaching into private `internal` modules.
pub use internal::image_loader::fetch_and_decode_image;

// Re-export help overlay builders so binary / integration code can compose the
// help UI without reaching into the private `internal` module tree.
pub use internal::help_overlay::{help_panel, shortcut_row};

#[cfg(test)]
mod tests {
    use super::*;
    use internal::scroll::ScrollState;
    use std::sync::Mutex;

    // Mutex to serialize tests that manipulate files
    static FILE_TEST_LOCK: Mutex<()> = Mutex::new(());

    // ---- Scroll State Tests ------------------------------------------------

    #[test]
    fn scroll_state_initializes_correctly() {
        let state = ScrollState::new();
        assert_eq!(state.scroll_y, 0.0);
        assert_eq!(state.max_scroll_y, 0.0);
    }

    #[test]
    fn scroll_up_prevents_negative_scroll() {
        let mut state = ScrollState::new();
        state.scroll_up(100.0);
        assert_eq!(state.scroll_y, 0.0);
    }

    #[test]
    fn scroll_down_respects_max_scroll() {
        let mut state = ScrollState::new();
        state.set_max_scroll(1000.0, 500.0);
        state.scroll_down(1000.0);
        assert_eq!(state.scroll_y, state.max_scroll_y);
    }

    #[test]
    fn scroll_bounds_are_enforced() {
        let mut state = ScrollState::new();
        state.set_max_scroll(1000.0, 500.0);

        // Try to scroll beyond bounds
        state.scroll_down(2000.0);
        assert!(state.scroll_y <= state.max_scroll_y);

        state.scroll_to_top();
        state.scroll_up(100.0);
        assert!(state.scroll_y >= 0.0);
    }

    #[test]
    fn page_down_scrolls_by_80_percent_of_page_height() {
        let mut state = ScrollState::new();
        state.set_max_scroll(2000.0, 800.0);
        state.page_down(800.0);
        assert_eq!(state.scroll_y, 640.0); // 80% of 800
    }

    #[test]
    fn page_up_scrolls_by_80_percent_of_page_height() {
        let mut state = ScrollState::new();
        state.set_max_scroll(2000.0, 800.0);
        state.scroll_y = 1000.0;
        state.page_up(800.0);
        assert_eq!(state.scroll_y, 360.0); // 1000 - 640
    }

    #[test]
    fn scroll_to_top_works() {
        let mut state = ScrollState::new();
        state.scroll_y = 500.0;
        state.scroll_to_top();
        assert_eq!(state.scroll_y, 0.0);
    }

    #[test]
    fn scroll_to_bottom_works() {
        let mut state = ScrollState::new();
        state.set_max_scroll(1000.0, 500.0);
        state.scroll_to_bottom();
        assert_eq!(state.scroll_y, 500.0);
    }

    #[test]
    fn content_height_estimation_constants_work() {
        // Test that the constants produce reasonable values
        let line_count = 100;
        let avg_line_height = BASE_TEXT_SIZE * LINE_HEIGHT_MULTIPLIER;
        let estimated_content_height = line_count as f32 * avg_line_height;
        let total_content_height = estimated_content_height + CONTENT_HEIGHT_BUFFER;

        // Verify line height is reasonable (should be > base text size)
        assert!(avg_line_height > BASE_TEXT_SIZE);
        assert_eq!(avg_line_height, 19.2 * 1.5); // 28.8 pixels

        // Verify buffer is added correctly
        assert_eq!(
            total_content_height,
            estimated_content_height + CONTENT_HEIGHT_BUFFER
        );

        // Verify default viewport height is reasonable
        assert!(DEFAULT_VIEWPORT_HEIGHT > 0.0);
        assert_eq!(DEFAULT_VIEWPORT_HEIGHT, 800.0);
    }

    // ---- File Extension Tests ----------------------------------------------

    #[test]
    fn is_supported_extension_with_md() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        assert!(is_supported_extension("file.md", &supported));
    }

    #[test]
    fn is_supported_extension_with_markdown() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        assert!(is_supported_extension("file.markdown", &supported));
    }

    #[test]
    fn is_supported_extension_with_txt() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        assert!(is_supported_extension("notes.txt", &supported));
    }

    #[test]
    fn is_supported_extension_case_insensitive() {
        let supported = vec!["md".to_string(), "markdown".to_string()];
        assert!(is_supported_extension("file.MD", &supported));
        assert!(is_supported_extension("file.Md", &supported));
        assert!(is_supported_extension("file.MARKDOWN", &supported));
    }

    #[test]
    fn is_supported_extension_with_unsupported() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        assert!(!is_supported_extension("file.pdf", &supported));
        assert!(!is_supported_extension("file.docx", &supported));
        assert!(!is_supported_extension("file", &supported)); // No extension
    }

    #[test]
    fn resolve_with_unsupported_extension() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];

        // Create a file with unsupported extension
        std::fs::write("test.pdf", "content").expect("Failed to create test file");

        let result = resolve_markdown_file_path(Some("test.pdf"), &supported);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unsupported file format"));
        assert!(err_msg.contains("md, markdown, txt"));

        // Clean up
        std::fs::remove_file("test.pdf").ok();
    }

    #[test]
    fn resolve_with_markdown_extension() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];

        // Create a .markdown file
        std::fs::write("test.markdown", "# Test").expect("Failed to create test file");

        let result = resolve_markdown_file_path(Some("test.markdown"), &supported);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test.markdown");

        // Clean up
        std::fs::remove_file("test.markdown").ok();
    }

    #[test]
    fn resolve_with_txt_extension() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];

        // Create a .txt file
        std::fs::write("notes.txt", "# Notes").expect("Failed to create test file");

        let result = resolve_markdown_file_path(Some("notes.txt"), &supported);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "notes.txt");

        // Clean up
        std::fs::remove_file("notes.txt").ok();
    }

    #[test]
    fn resolve_markdown_file_path_with_valid_file() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];

        // Create a temporary file for testing
        let test_content = "# Test\nThis is a test file.";
        std::fs::write("test_file.md", test_content).expect("Failed to create test file");

        let result = resolve_markdown_file_path(Some("test_file.md"), &supported);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_file.md");

        // Clean up
        std::fs::remove_file("test_file.md").ok();
    }

    #[test]
    fn resolve_markdown_file_path_with_nonexistent_file() {
        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        let result = resolve_markdown_file_path(Some("nonexistent_file.md"), &supported);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn resolve_markdown_file_path_with_no_path_and_readme_exists() {
        let _lock = FILE_TEST_LOCK.lock().unwrap();

        // Backup existing files if they exist
        let readme_backup = std::fs::read_to_string("README.md").ok();
        let todo_backup = std::fs::read_to_string("TODO.md").ok();

        // Create test files
        let readme_content = "# README\nProject documentation.";
        std::fs::write("README.md", readme_content).expect("Failed to create test README");

        let todo_content = "# TODO\nSome todo items.";
        std::fs::write("TODO.md", todo_content).expect("Failed to create test TODO");

        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        let result = resolve_markdown_file_path(None, &supported);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "README.md");

        // Restore original files or clean up
        if let Some(content) = readme_backup {
            std::fs::write("README.md", content).ok();
        } else {
            std::fs::remove_file("README.md").ok();
        }
        if let Some(content) = todo_backup {
            std::fs::write("TODO.md", content).ok();
        } else {
            std::fs::remove_file("TODO.md").ok();
        }
    }

    #[test]
    fn resolve_markdown_file_path_with_no_path_and_todo_fallback() {
        let _lock = FILE_TEST_LOCK.lock().unwrap();

        // Backup existing files if they exist
        let readme_backup = std::fs::read_to_string("README.md").ok();
        let todo_backup = std::fs::read_to_string("TODO.md").ok();

        // Ensure README.md doesn't exist for this test
        std::fs::remove_file("README.md").ok();

        let todo_content = "# TODO\nSome todo items.";
        std::fs::write("TODO.md", todo_content).expect("Failed to create test TODO");

        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        let result = resolve_markdown_file_path(None, &supported);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TODO.md");

        // Restore original files or clean up
        if let Some(content) = readme_backup {
            std::fs::write("README.md", content).ok();
        }
        if let Some(content) = todo_backup {
            std::fs::write("TODO.md", content).ok();
        } else {
            std::fs::remove_file("TODO.md").ok();
        }
    }

    #[test]
    fn resolve_markdown_file_path_with_no_path_and_no_defaults() {
        let _lock = FILE_TEST_LOCK.lock().unwrap();

        // Backup existing files
        let readme_backup = std::fs::read_to_string("README.md").ok();
        let todo_backup = std::fs::read_to_string("TODO.md").ok();

        // Ensure both README.md and TODO.md don't exist
        std::fs::remove_file("README.md").ok();
        std::fs::remove_file("TODO.md").ok();

        let supported = vec!["md".to_string(), "markdown".to_string(), "txt".to_string()];
        let result = resolve_markdown_file_path(None, &supported);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Default files README.md and TODO.md not found")
        );

        // Restore original files
        if let Some(content) = readme_backup {
            std::fs::write("README.md", content).ok();
        }
        if let Some(content) = todo_backup {
            std::fs::write("TODO.md", content).ok();
        }
    }

    #[test]
    fn load_markdown_content_success() {
        let test_content = "# Test Content\nThis is test markdown.";
        std::fs::write("test_load.md", test_content).expect("Failed to create test file");

        let result = load_markdown_content("test_load.md");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);

        // Clean up
        std::fs::remove_file("test_load.md").ok();
    }

    #[test]
    fn load_markdown_content_failure() {
        let result = load_markdown_content("nonexistent_file_xyz.md");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read file")
        );
    }

    // ---- Table Rendering Tests ------------------------------------------------

    #[test]
    fn table_node_parses_from_markdown() {
        use comrak::nodes::NodeValue;
        use comrak::{Arena, Options, parse_document};

        let arena = Arena::new();
        let markdown = "| A | B |\n|---|---|\n| 1 | 2 |";
        let mut options = Options::default();
        options.extension.table = true;
        let root = parse_document(&arena, markdown, &options);

        // Verify it parses and has a table
        let has_table = root
            .descendants()
            .any(|node| matches!(node.data.borrow().value, NodeValue::Table(_)));
        assert!(has_table, "Markdown should contain a table node");
    }

    #[test]
    fn table_with_alignment_parses() {
        use comrak::nodes::NodeValue;
        use comrak::{Arena, Options, parse_document};

        let arena = Arena::new();
        let markdown = "| Left | Center | Right |\n|:-----|:------:|------:|\n| A | B | C |";
        let mut options = Options::default();
        options.extension.table = true;
        let root = parse_document(&arena, markdown, &options);

        let has_table = root
            .descendants()
            .any(|node| matches!(node.data.borrow().value, NodeValue::Table(_)));
        assert!(has_table, "Markdown with aligned table should parse");
    }

    #[test]
    fn table_has_header_and_body_rows() {
        use comrak::nodes::NodeValue;
        use comrak::{Arena, Options, parse_document};

        let arena = Arena::new();
        let markdown = "| Header |\n|--------|\n| Data |";
        let mut options = Options::default();
        options.extension.table = true;
        let root = parse_document(&arena, markdown, &options);

        let mut has_header = false;
        let mut has_body = false;

        for node in root.descendants() {
            if let NodeValue::TableRow(is_header) = node.data.borrow().value {
                if is_header {
                    has_header = true;
                } else {
                    has_body = true;
                }
            }
        }

        assert!(has_header, "Table should have header row");
        assert!(has_body, "Table should have body row");
    }

    // ---- Image Path Resolution Tests ---------------------------------------

    #[test]
    fn resolve_image_path_with_url() {
        use std::path::Path;
        let result = resolve_image_path(
            "https://example.com/image.png",
            Path::new("/docs/README.md"),
        );
        assert_eq!(result, "https://example.com/image.png");
    }

    #[test]
    fn resolve_image_path_with_http_url() {
        use std::path::Path;
        let result =
            resolve_image_path("http://example.com/photo.jpg", Path::new("/docs/README.md"));
        assert_eq!(result, "http://example.com/photo.jpg");
    }

    #[test]
    fn resolve_image_path_with_absolute_path() {
        use std::path::Path;
        let result = resolve_image_path("/assets/icon.png", Path::new("/docs/README.md"));
        assert_eq!(result, "/assets/icon.png");
    }

    #[test]
    fn resolve_image_path_with_relative_path() {
        use std::path::Path;
        let result = resolve_image_path("./images/logo.png", Path::new("/docs/README.md"));
        // On Unix-like systems
        #[cfg(unix)]
        assert_eq!(result, "/docs/images/logo.png");
    }

    #[test]
    fn resolve_image_path_with_parent_relative_path() {
        use std::path::Path;
        let result = resolve_image_path("../assets/icon.png", Path::new("/docs/sub/README.md"));
        // On Unix-like systems
        #[cfg(unix)]
        assert_eq!(result, "/docs/assets/icon.png");
    }

    #[test]
    fn resolve_image_path_with_nested_relative_path() {
        use std::path::Path;
        let result = resolve_image_path("images/icons/logo.png", Path::new("/docs/README.md"));
        // On Unix-like systems
        #[cfg(unix)]
        assert_eq!(result, "/docs/images/icons/logo.png");
    }

    #[test]
    fn resolve_image_path_with_no_parent_directory() {
        use std::path::Path;
        // Path with no parent (e.g., root or relative path without parent)
        let result = resolve_image_path("image.png", Path::new("README.md"));
        // Should return the image path as-is when no parent can be determined
        assert_eq!(result, "image.png");
    }

    // ---- Go-to-Line Tests ------------------------------------------------

    #[test]
    fn parse_line_number_with_valid_input() {
        use internal::viewer::MarkdownViewer;
        // Test parsing valid line numbers
        assert_eq!(MarkdownViewer::parse_line_number("1"), Some(1));
        assert_eq!(MarkdownViewer::parse_line_number("42"), Some(42));
        assert_eq!(MarkdownViewer::parse_line_number("100"), Some(100));
        assert_eq!(MarkdownViewer::parse_line_number(" 42 "), Some(42)); // With whitespace
    }

    #[test]
    fn parse_line_number_with_invalid_input() {
        use internal::viewer::MarkdownViewer;
        // Test parsing invalid inputs
        assert_eq!(MarkdownViewer::parse_line_number(""), None);
        assert_eq!(MarkdownViewer::parse_line_number("abc"), None);
        assert_eq!(MarkdownViewer::parse_line_number("0"), None); // Line numbers start at 1
        assert_eq!(MarkdownViewer::parse_line_number("-5"), None); // Negative not allowed
        assert_eq!(MarkdownViewer::parse_line_number("1.5"), None); // Decimals not allowed
    }

    #[test]
    fn validate_line_number_logic() {
        // Test the line counting logic that validate_line_number uses
        let content_100_lines = (0..100)
            .map(|i| format!("Line {}\n", i))
            .collect::<String>();
        let total_lines = content_100_lines.lines().count();
        assert_eq!(total_lines, 100);

        // Test bounds checking logic
        assert!(1 <= total_lines);
        assert!(50 <= total_lines);
        assert!(100 <= total_lines);
        assert!(101 > total_lines);
    }

    #[test]
    fn scroll_to_line_bounds_checking() {
        use internal::scroll::ScrollState;
        // Test that smooth_scroll_to respects bounds (used by scroll_to_line)
        let mut state = ScrollState::new();
        state.set_max_scroll(2000.0, 500.0);

        // Test that scrolling beyond max scroll is clamped
        let target_y = 3000.0;
        state.smooth_scroll_to(target_y);
        assert_eq!(state.target_scroll_y, state.max_scroll_y);

        // Test that scrolling below 0 is clamped
        let target_y = -100.0;
        state.smooth_scroll_to(target_y);
        assert_eq!(state.target_scroll_y, 0.0);
    }
}
