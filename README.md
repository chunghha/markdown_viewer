# Markdown Viewer

A desktop Markdown viewer built with Rust and GPUI, featuring advanced scrolling capabilities and a clean, readable interface.

## Features

### Core Functionality
- **Markdown Rendering**: Full support for CommonMark-compliant Markdown using `comrak`
- **Rich Text Display**: Styled headings, lists, **syntax-highlighted code blocks with line numbers**, **clickable links** with hover effects, emphasis, blockquotes, and **responsive tables**
- **Responsive Tables**: Dynamic column widths with 150px minimum to ensure readability
- **CLI Interface**: Command-line argument support for loading any Markdown file
- **File Watching**: Automatic reload when files change on disk with scroll position preservation
- **Clean Interface**: Minimalist design focused on readability
- **Configuration System**: Customizable settings via RON configuration files
- **Structured Logging**: Debug and trace logging with `tracing`
- **Professional Error Handling**: Contextual error messages with `anyhow`

### Advanced Scrolling
- **Mouse Wheel Scrolling**: Smooth pixel-perfect scrolling with proper direction handling
- **Keyboard Navigation**: Complete keyboard shortcuts for efficient document navigation
- **Bounds Protection**: Prevents scrolling beyond document boundaries
- **Content-Aware**: Automatic content height estimation for accurate scroll limits

### Search Functionality
- **Full Text Search**: Case-insensitive search across the entire document
- **Real-time Highlighting**: Matches are highlighted as you type
- **Navigation**: Jump between matches with keyboard shortcuts
- **Auto-Scroll**: Automatically scrolls to the current match
- **Visual Feedback**: Search overlay with match count and status

### Visual Enhancements
- **Interactive Status Bar**: Persistent footer with file info, scroll position, theme indicator, and **Help button**
- **Styled Interface**: Custom colors and fonts for a polished look
- **Table of Contents**: Right-side sidebar with hierarchical navigation (levels 2-4)
  - Toggle with `Cmd+Z` or top-right button
  - Click headings to jump to sections
  - Auto-highlights current section
  - Precise navigation accounting for text wrapping and images
- **v0.13.0: Fuzzy File Finder**
  - **Quick Open**: Fuzzy search files in the current directory with `Cmd+P`
  - **Fast Navigation**: Instant results using efficient fuzzy matching
  - **Seamless Context Switching**: Opens files while preserving application state

### Keyboard Shortcuts
- **Search**: `Cmd+F` (macOS) or `Ctrl+F` to toggle search
- **Search Navigation**: `Enter` (next), `Shift+Enter` (previous)
- **Exit Search**: `Escape` to clear search and return to document
- **Go to Line**: `Cmd+G` (macOS) or `Ctrl+G` to open go-to-line dialog
- **Quick Open**: `Cmd+P` (macOS) or `Ctrl+P` to fuzzy find and open files
- **Quick Navigation**: `Cmd+T` / `g` (Top), `Cmd+B` / `G` (Bottom)
- **Advanced Navigation**:
  - `Ctrl+d` / `Ctrl+u`: Half-page scroll down/up
  - `zz`: Center current view
  - `m<char>`: Set mark (e.g., `ma`)
  - `'<char>`: Jump to mark (e.g., `'a`)
- **Application**: `Cmd+Q` / `q` / `Ctrl+C` to quit
- **Toggle Help Overlay**: `Cmd+H` to toggle help overlay for showing shortcuts (Arrow keys for multiple pages)
- **Toggle TOC**: `Cmd+Z` to toggle Table of Contents sidebar
- **Toggle Theme**: `Cmd+Shift+T` to toggle between Light and Dark themes
- **Cycle Theme Family**: `Cmd+Shift+N` to cycle through available theme families
- **Arrow Keys**: `↑`/`↓` for 20px incremental scrolling
- **Page Navigation**: `Page Up`/`Page Down` for 80% viewport scrolling
- **Document Navigation**: `Home`/`End` for jumping to top/bottom
- **Space Navigation**: `Space`/`Shift+Space` for page scrolling
- **Font Size**: `Cmd+=` (Increase), `Cmd+-` (Decrease)
- **Reset**: `Escape` to return to document top (when not searching)

## Architecture

### Design Principles
- **Test-Driven Development**: All features developed with comprehensive test coverage
- **Clean Architecture**: Separation of concerns with modular design
- **Rust Best Practices**: Memory safety, performance, and idiomatic code
- **GPUI Integration**: Native desktop UI with efficient rendering

### Core Components
- **ScrollState**: Manages scroll position, bounds, and navigation logic
- **MarkdownViewer**: Main application component with event handling
- **File Handling**: CLI argument parsing and file loading with error handling
- **Rendering Engine**: Efficient Markdown-to-UI transformation
- **Style System**: Centralized styling with meaningful constants

## Getting Started

### Prerequisites
- Rust 1.70+ installed via [Rustup](https://rustup.rs/)
- Compatible operating system (macOS, Linux, Windows)
- [Task](https://taskfile.dev/#/installation) (optional, for enhanced development workflow)

### Installation
```bash
git clone <repository-url>
cd markdown_viewer
cargo build --release
```

### Usage
```bash
# Run with default file (e.g., README.md or TODO.md)
cargo run

# Run with a specific Markdown file
cargo run -- README.md
cargo run -- document.markdown
cargo run -- notes.txt
cargo run -- path/to/your/file.md

# Supported formats: .md, .markdown, .txt

# Show help and usage information
cargo run -- --help

# OR with Task
# Run with default file
task run

# Run with a specific file
task run -- README.md
task run-release -- path/to/your/file.md

# The application will load and display the Markdown file with full scrolling support
```

### Configuration

Customize the viewer by creating a `config.ron` file:

```bash
# Copy the example configuration
cp config.example.ron config.ron

# Edit config.ron to customize settings
# - Window dimensions
# - Scroll behavior
# - Theme and fonts
# - Logging level
```

**Example configuration:**
```ron
(
    window: (width: 1280.0, height: 900.0, title: "My Markdown Viewer"),
    scroll: (page_scroll_percentage: 0.9, arrow_key_increment: 30.0),
    theme: (base_text_size: 20.0, primary_font: "Arial"),
    logging: (default_level: "debug"),
    file_watcher: (enabled: true, debounce_ms: 100),
)
```

### Logging

Control logging output with the `RUST_LOG` environment variable:

```bash
# Info level (default)
cargo run

# Debug level - shows configuration and detailed operations
RUST_LOG=debug cargo run

# Trace level - shows all scroll events
RUST_LOG=trace cargo run
```

### Development Workflow

This project includes a comprehensive `Taskfile.yml` for streamlined development:

```bash
# Show all available tasks
task

# TDD workflow - continuous testing
task tdd

# Pre-commit checks (format, lint, test)
task pre-commit

# Development mode with auto-reload
task dev

# Full CI pipeline
task ci
```

#### Core Tasks
```bash
# Code quality
task fmt              # Format code
task lint             # Run clippy with warnings as errors
task test             # Run all tests

# Building and running
task build            # Build project
task run              # Run the application (e.g., 'task run -- file.md')
task run-release      # Run the application in release mode
task clean            # Clean build artifacts

# Documentation and dependencies
task doc              # Generate documentation
task update           # Update dependencies
task install-tools    # Install helpful development tools
```

#### Traditional Cargo Commands
```bash
# Run tests
cargo test

# Run with clippy linting
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## Technical Details

### Dependencies
- **comrak**: CommonMark-compliant Markdown parsing
- **gpui**: Modern Rust GUI framework for desktop applications
- **clap**: Command-line argument parsing for file specification
- **notify**: Cross-platform file system event monitoring
- **notify-debouncer-full**: Debouncing for file system events
- **anyhow**: Ergonomic error handling with context
- **tracing**: Structured logging and diagnostics
- **ron**: Rusty Object Notation for configuration files
- **serde**: Serialization/deserialization framework

### Performance
- **Efficient Rendering**: Transform-based scrolling without content re-rendering
- **Memory Safe**: Zero-copy string handling where possible
- **Responsive**: 60 FPS scrolling with large documents

### Code Quality
- **82 Unit Tests**: Comprehensive test coverage for scrolling, file handling, configuration, format validation, table rendering, file watching, and JSON theme system
- **Clippy Clean**: Passes all Rust linting checks
- **Well Documented**: Inline documentation and implementation guides

## Project Structure

```
markdown_viewer/
├── src/
│   ├── main.rs           # Application entry point and MarkdownViewer
│   ├── lib.rs            # ScrollState and rendering logic
│   └── config.rs         # Configuration management
├── docs/
│   └── implementations/
│       └── 0001_scrolling.md  # Detailed scrolling implementation
├── config.example.ron    # Example configuration file
├── TODO.md               # Development roadmap
├── AGENTS.md             # Development guidelines and TDD practices
├── Taskfile.yml          # Development task automation
└── README.md             # This file
```

## Recent Improvements

See [CHANGELOG.md](CHANGELOG.md) for a complete history of improvements and version releases.

## Development Philosophy

This project follows Kent Beck's Test-Driven Development (TDD) and "Tidy First" principles:

- **Red → Green → Refactor**: All features developed with failing tests first
- **Structural vs Behavioral**: Clean separation of code organization and feature changes
- **Incremental Improvement**: Small, focused commits with clear intent
- **Test-First**: Comprehensive test suite guides development

## Contributing

### Development Workflow
1. **TDD Cycle**: Use `task tdd` for continuous test-driven development
2. **Pre-commit Checks**: Run `task pre-commit` before committing changes
3. **Code Quality**: Ensure `task ci` passes (format, lint, test, build)

### Guidelines
1. Follow TDD practices with tests for all new features
2. Separate structural (code organization) from behavioral (feature) changes
3. Use meaningful commit prefixes: `feat:`, `fix:`, `struct:`, `refactor:`, `chore:`
4. Ensure all tests pass and clippy is clean before committing

### Quick Start for Contributors
```bash
# Install development tools
task install-tools

# Start TDD workflow
task tdd

# In another terminal, make changes and run pre-commit checks
task pre-commit
```

## Future Enhancements

- **Configuration UI**: In-app settings panel
- **Custom Themes**: Create your own themes by adding JSON files to `themes/` directory
- **Export Options**: HTML export functionality (PDF already available)
- **Performance**: Lazy loading for very large files

## Resources

- [GPUI Documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui/docs)
- [GPUI Examples](https://github.com/zed-industries/zed/tree/main/crates/gpui/examples)
- [CommonMark Specification](https://commonmark.org/)
- [Project Development Guidelines](AGENTS.md)

## License

This project is created with Create GPUI App and follows Rust community standards.
