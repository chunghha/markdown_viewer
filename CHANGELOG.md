# Changelog

All notable changes to the Markdown Viewer project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0] - 2025-11-22

### Added
- **PDF Export**: Export markdown documents to PDF with `Cmd+E` / `Ctrl+E` keyboard shortcut
- **Automatic Naming**: PDF files are automatically saved with the same name as the source file
- **Styled Output**: Uses `markdown2pdf` library for professional PDF generation with proper formatting
- **Success Notifications**: Green notification overlay shows exported filename
- **Error Handling**: Red notification overlay displays error messages if export fails
- **Overwrite Confirmation**: Yellow warning prompt when PDF already exists, press Y/N to confirm/cancel
- **Theme Support**: All PDF notification colors adapt to Light/Dark theme
- **Help Integration**: PDF export shortcut added to help overlay
- **Dismissible Notifications**: Press Escape to close notification overlays

## [0.9.0] - 2025-11-22

### Added
- **Light/Dark Themes**: Complete theme system with Light (default) and Dark variants
- **Theme Toggle**: Press `Cmd+Shift+T` to toggle between themes instantly
- **Theme Persistence**: Selected theme is saved to config and restored on app launch
- **Comprehensive Colors**: All UI elements (text, code, TOC, overlays) adapt to theme
- **Code Quality**: Enhanced with derive macros and comprehensive theme test coverage

## [0.8.1] - 2025-11-22

### Added
- **Go-to-Line Dialog**: Press `Cmd+G` (macOS) or `Ctrl+G` to open go-to-line dialog
- **Line Number Input**: Type a line number and press `Enter` to jump to that line
- **Input Validation**: Only accepts numeric input with bounds checking against total line count
- **Visual Feedback**: Blue overlay shows current input and validation status
- **Immediate Scrolling**: Instantly scrolls to target line, centered in viewport
- **Error Handling**: Helpful error messages for invalid line numbers
- **Help Integration**: Added go-to-line shortcut to help overlay

## [0.8.0] - 2025-11-22

### Added
- **TOC Sidebar**: Right-side hierarchical navigation for headings (levels 2-4)
- **Smart Navigation**: Precise click-to-scroll accounting for text wrapping and images
- **Toggle Controls**: `Cmd+Z` keyboard shortcut and top-right toggle button
- **Auto-Highlighting**: Current section automatically highlighted based on scroll position
- **Dynamic Layout**: Content area adjusts automatically when TOC is visible
- **Live Updates**: TOC regenerates when markdown file is modified

## [0.7.2] - 2025-11-20

### Changed
- **Modular Architecture**: Refactored `main.rs` into dedicated internal modules (`viewer`, `events`, `ui`)
- **Clean Code**: Significantly reduced `main.rs` size and complexity
- **Component Extraction**: Separated event handling, UI overlays, and core viewer logic
- **Maintainability**: Improved code organization for easier future development

### Fixed
- Verified all features and tests work seamlessly with new structure

## [0.7.1] - 2025-11-20

### Added
- **Responsive Tables**: Dynamic column width calculation based on column count
- **Minimum Width**: 150px minimum column width ensures readability

### Changed
- **Improved Layout**: Fixed-width columns with better text wrapping and warm beige headers
- **Smart Sizing**: Tables adapt to content while maintaining consistency

### Fixed
- Corrected viewport width calculation for dynamic resizing
- Resolved clippy warnings and optimized rendering performance

## [0.7.0] - 2025-11-20

### Added
- **Auto-Reload**: Automatically reloads when markdown files change on disk
- **Scroll Preservation**: Maintains scroll position after reload
- **Deletion Handling**: Gracefully handles file deletion with visual feedback
- **Configurable**: Debouncing and enable/disable options in config
- **Tested**: Comprehensive test coverage for all file watching scenarios

## [0.6.0] - 2025-11-19

### Added
- **Version Badge**: Sky blue badge with version number in top-right corner
- **Global Shortcuts**: Cmd+T (top), Cmd+B (bottom), Cmd+Q (quit)
- **Font Size Controls**: Cmd+= to increase, Cmd+- to decrease font size
- **Dynamic Scroll**: Scroll bounds recompute when font size changes
- **Search Functionality**: Full text search with highlighting and navigation
- **CLI Arguments**: Accept file path as command-line argument with fallback to README.md/TODO.md
- **File Validation**: Proper error handling for missing or invalid files
- **Usage Help**: Built-in help system with `--help` flag
- **Configuration System**: RON-based configuration for customizable settings
- **Structured Logging**: Tracing integration for debugging and monitoring
- **Error Handling**: Professional error messages with anyhow context

### Changed
- **Meaningful Constants**: Extracted magic numbers to named constants
- **Enhanced Documentation**: Updated all project documentation
- **Test Coverage**: 71 tests covering scrolling, file handling, configuration, validation, file watching, and theme system

### Fixed
- **Mouse Wheel Direction**: Proper handling of scroll up/down events
- **Bounds Enforcement**: Eliminated scrolling beyond document boundaries
- **Content Height**: Accurate estimation prevents content cutoff
- **Safe Tests**: File-manipulating tests now preserve project files
