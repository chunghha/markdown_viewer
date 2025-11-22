# TODO: Markdown Viewer Development Roadmap

## Current Status ✅ INFRASTRUCTURE COMPLETE

The core infrastructure has been **fully implemented and tested**. The application now has professional-grade error handling, logging, and configuration management.

### Recently Completed ✅
- [x] **Infrastructure Enhancements**: Added anyhow, tracing, and ron
- [x] **Configuration System**: RON-based configuration with validation
- [x] **Error Handling**: Contextual errors with anyhow throughout
- [x] **Structured Logging**: Tracing integration at all levels
- [x] **Test Safety**: Fixed destructive tests to preserve project files
- [x] **Mouse Wheel Scrolling Bug Fix**: Fixed direction handling to prevent negative scroll positions
- [x] **Bounds Enforcement**: Eliminated scrolling beyond document start/end
- [x] **Content Height Accuracy**: Improved estimation to prevent content cutoff
- [x] **Code Quality**: Extracted magic numbers to meaningful constants
- [x] **Search Functionality**: Implemented full text search with highlighting and navigation
- [x] **Global Shortcuts**: Added Cmd+T/B/Q and font size controls
- [x] **Visual Polish**: Added version badge and refined styling
- [x] **Comprehensive Testing**: 65 tests covering all functionality
- [x] **Documentation Update**: Updated all project documentation
- [x] **File Watching**: Auto-reload on file changes with scroll position preservation
- [x] **Responsive Table Layout**: Dynamic column widths with minimum width enforcement
- [x] **Table Header Polish**: Improved styling with warmer background color
- [x] **Layout Bug Fixes**: Corrected viewport width propagation for accurate resizing
- [x] **Code Refactoring**: Extracted `main.rs` logic into modular components (`viewer`, `events`, `ui`)

## Next Development Phases 🚀

### Phase 1: File Loading Enhancement (Medium Priority)
- [x] **Command Line Arguments**
  - [x] Accept file path as CLI argument
  - [x] Support multiple file formats (.md, .markdown, .txt)
  - [x] Add file validation and error handling
  - [x] Default to README.md if no argument provided
  - [x] Professional error messages with context

- [x] **File Watching**
  - [x] Auto-reload when file changes
  - [x] Preserve scroll position on reload
  - [x] Handle file deletion gracefully
  - [x] Configurable debouncing
  - [x] Cross-platform compatibility

### Phase 2: Enhanced Markdown Support (Medium Priority)
- [x] **Tables**
  - [x] Basic table rendering
  - [x] Column alignment support
  - [x] Responsive table layout

- [x] **Code Blocks**
  - [x] Syntax highlighting for common languages
  - [x] Line numbers for code blocks
  - [x] Copy-to-clipboard functionality

- [x] **Links and Images**
  - [x] Clickable links (open in browser)
  - [x] Inline image display
  - [x] Image scaling and positioning

### Phase 3: User Experience Improvements (Medium Priority)
- [x] **Search Functionality**
  - [x] Text search with highlighting
  - [x] Regex search support (Case-insensitive text search implemented)
  - [x] Jump to search results
  - [ ] Search history

- [ ] **Navigation Enhancements**
  - [ ] Table of contents sidebar
  - [ ] Heading-based navigation
  - [ ] Bookmark specific positions
  - [ ] Go-to-line functionality

- [ ] **Visual Improvements**
  - [ ] Theme selection (light/dark)
  - [x] Font size adjustment
  - [ ] Custom CSS styling support
  - [ ] Print preview mode

### Phase 4: Advanced Features (Low Priority)
- [ ] **Export Options**
  - [ ] Export to PDF
  - [ ] Export to HTML
  - [ ] Export to plain text
  - [ ] Print functionality

- [ ] **Plugin System**
  - [ ] Custom renderer plugins
  - [ ] Markdown extension support
  - [ ] Third-party integration

- [ ] **Performance Optimizations**
  - [ ] Lazy loading for very large files
  - [ ] Virtual scrolling for massive documents
  - [ ] Background file parsing

### Phase 5: Platform Integration (Low Priority)
- [ ] **Cross-Platform Polish**
  - [ ] Native file dialogs
  - [ ] System integration (file associations)
  - [ ] Platform-specific optimizations

- [ ] **Accessibility**
  - [ ] Screen reader support
  - [ ] High contrast mode
  - [ ] Keyboard-only navigation
  - [ ] Zoom functionality

## Technical Debt and Maintenance

### Code Organization
- [x] **Module Structure**
  - [x] Extract rendering logic to separate module
  - [x] Create dedicated file handling module
  - [ ] Implement plugin architecture foundation

- [x] **Error Handling**
  - [x] Replace string errors with anyhow
  - [x] Add contextual error messages
  - [x] User-friendly error reporting

### Testing Improvements
- [ ] **Integration Tests**
  - [ ] End-to-end application testing
  - [ ] File loading test scenarios
  - [ ] Cross-platform testing

- [ ] **Performance Tests**
  - [ ] Benchmark scrolling performance
  - [ ] Memory usage profiling
  - [ ] Large file handling tests

## Implementation Guidelines

### Development Process
1. **Follow TDD**: Write failing tests before implementing features
2. **Structural First**: Organize code structure before adding behavior
3. **Small Commits**: Keep changes focused and well-documented
4. **Test Coverage**: Maintain high test coverage for all new features

### Code Quality Standards
- All code must pass `cargo clippy -- -D warnings`
- Maintain comprehensive test suite
- Document all public APIs with rustdoc
- Use meaningful constants for configuration values

### Commit Conventions
- `feat:` - New user-facing features
- `fix:` - Bug fixes
- `struct:` - Code organization improvements (no behavior change)
- `refactor:` - Behavior-preserving code improvements
- `chore:` - Tooling, documentation, or maintenance

## Success Metrics

### Phase 1 Goals
- [ ] Load any Markdown file via CLI
- [ ] Maintain current scrolling quality
- [ ] Zero regressions in existing functionality

### Phase 2 Goals
- [ ] Support 90% of CommonMark specification
- [ ] Render complex documents correctly
- [ ] Maintain smooth performance

### Phase 3 Goals
- [ ] Professional-grade user experience
- [ ] Feature parity with popular Markdown viewers
- [ ] Positive user feedback

## Known Issues

### Current Limitations
- **Fixed File**: Currently only loads TODO.md (Fixed: now accepts CLI args)
- **Basic Rendering**: Limited Markdown feature support
- **No Themes**: Single color scheme

### Technical Considerations
- **Window Bounds**: Need proper GPUI window dimension access
- **Font Loading**: Custom font handling for better typography
- **File System**: Robust file watching and error handling

## Resources and References

### Documentation
- [Current Implementation Guide](docs/implementations/0001_scrolling.md)
- [Development Guidelines](AGENTS.md)
- [GPUI Documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui/docs)

### External Dependencies
- [comrak](https://docs.rs/comrak/) - Markdown parsing
- [gpui](https://www.gpui.rs/) - GUI framework
- [syntect](https://docs.rs/syntect/) - Syntax highlighting (future)

## Decision Log

### Completed Decisions
- ✅ **Scrolling Architecture**: Transform-based scrolling with bounds checking
- ✅ **Testing Strategy**: Comprehensive unit tests with TDD approach
- ✅ **Code Organization**: Separated ScrollState from main application logic
- ✅ **Constants Management**: Meaningful named constants in lib.rs
- ✅ **Error Handling**: anyhow for ergonomic error propagation
- ✅ **Logging**: tracing for structured diagnostics
- ✅ **Configuration**: RON-based configuration system

### Pending Decisions
- **File Loading Strategy**: CLI args vs file dialog vs drag-and-drop
- **Theme System**: CSS-like vs programmatic styling
- **Plugin Architecture**: Trait-based vs script-based extensions

## 🎉 Milestone: Infrastructure Complete

The markdown viewer now has **production-ready infrastructure** with:
- Professional error handling with anyhow
- Structured logging with tracing
- Customizable configuration with ron
- Perfect bounds checking (no over-scrolling)
- Smooth mouse wheel support
- Complete keyboard navigation
- Robust error handling
- Comprehensive test coverage (31 tests)

**Ready for advanced feature development!**
