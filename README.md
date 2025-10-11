# Markdown Viewer

A desktop Markdown viewer built with Rust and GPUI, featuring advanced scrolling capabilities and a clean, readable interface.

## Features

### Core Functionality
- **Markdown Rendering**: Full support for CommonMark-compliant Markdown using `comrak`
- **Rich Text Display**: Styled headings, lists, code blocks, links, emphasis, and blockquotes
- **CLI Interface**: Command-line argument support for loading any Markdown file
- **Clean Interface**: Minimalist design focused on readability

### Advanced Scrolling
- **Mouse Wheel Scrolling**: Smooth pixel-perfect scrolling with proper direction handling
- **Keyboard Navigation**: Complete keyboard shortcuts for efficient document navigation
- **Bounds Protection**: Prevents scrolling beyond document boundaries
- **Content-Aware**: Automatic content height estimation for accurate scroll limits

### Keyboard Shortcuts
- **Arrow Keys**: `↑`/`↓` for 20px incremental scrolling
- **Page Navigation**: `Page Up`/`Page Down` for 80% viewport scrolling
- **Document Navigation**: `Home`/`End` for jumping to top/bottom
- **Space Navigation**: `Space`/`Shift+Space` for page scrolling
- **Reset**: `Escape` to return to document top

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
# Run with default TODO.md file
cargo run

# Run with a specific Markdown file
cargo run -- README.md
cargo run -- path/to/your/file.md

# Show help and usage information
cargo run -- --help

# OR with Task
task run
task run -- README.md

# The application will load and display the Markdown file with full scrolling support
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
task run              # Run the application
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

### Performance
- **Efficient Rendering**: Transform-based scrolling without content re-rendering
- **Memory Safe**: Zero-copy string handling where possible
- **Responsive**: 60 FPS scrolling with large documents

### Code Quality
- **15 Unit Tests**: Comprehensive test coverage for scrolling and file handling logic
- **Clippy Clean**: Passes all Rust linting checks
- **Well Documented**: Inline documentation and implementation guides

## Project Structure

```
markdown_viewer/
├── src/
│   ├── main.rs           # Application entry point and MarkdownViewer
│   └── lib.rs            # ScrollState and rendering logic
├── docs/
│   └── implementations/
│       └── 0001_scrolling.md  # Detailed scrolling implementation
├── TODO.md               # Example Markdown content
├── AGENTS.md             # Development guidelines and TDD practices
├── Taskfile.yml          # Development task automation
└── README.md             # This file
```

## Recent Improvements

### Bug Fixes ✅
- **Fixed Mouse Wheel Direction**: Proper handling of scroll up/down events
- **Bounds Enforcement**: Eliminated scrolling beyond document boundaries
- **Content Height**: Accurate estimation prevents content cutoff

### New Features ✅
- **CLI Arguments**: Accept file path as command-line argument with fallback to TODO.md
- **File Validation**: Proper error handling for missing or invalid files
- **Usage Help**: Built-in help system with `--help` flag

### Code Quality ✅
- **Meaningful Constants**: Extracted magic numbers to named constants
- **Enhanced Documentation**: Updated all project documentation
- **Test Coverage**: Added bounds checking, file handling, and validation tests

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

- **Multiple File Formats**: Support for .markdown, .txt extensions
- **File Watching**: Auto-reload when files change
- **Syntax Highlighting**: Code block syntax highlighting
- **Table Support**: Enhanced table rendering
- **Image Display**: Inline image support
- **Export Options**: PDF/HTML export functionality

## Resources

- [GPUI Documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui/docs)
- [GPUI Examples](https://github.com/zed-industries/zed/tree/main/crates/gpui/examples)
- [CommonMark Specification](https://commonmark.org/)
- [Project Development Guidelines](AGENTS.md)

## License

This project is created with Create GPUI App and follows Rust community standards.