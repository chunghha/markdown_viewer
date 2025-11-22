# Walkthrough - Refactoring main.rs

I have successfully refactored `src/main.rs` to significantly reduce its size and improve maintainability. The logic has been extracted into dedicated modules within `src/internal/`.

## Changes

### 1. Extracted `MarkdownViewer` to `src/internal/viewer.rs`
The core `MarkdownViewer` struct and its logic (scrolling, image loading, state management) have been moved to a new module. This allows it to be shared and tested independently.
- **File**: `src/internal/viewer.rs`
- **Key Changes**:
    - Moved `MarkdownViewer` struct definition.
    - Moved `calculate_y_for_offset`, `recompute_max_scroll`, `load_image` methods.
    - Implemented `Render` trait for `MarkdownViewer` in this module, encapsulating the UI logic.

### 2. Extracted Event Handling to `src/internal/events.rs`
Input event handling (keyboard shortcuts, scrolling) has been moved to a dedicated module.
- **File**: `src/internal/events.rs`
- **Key Changes**:
    - Created `handle_key_down` for keyboard events.
    - Created `handle_scroll_wheel` for mouse wheel events.
    - These functions now accept `&mut MarkdownViewer` and `&mut Window`.

### 3. Extracted UI Overlays to `src/internal/ui.rs`
The rendering logic for UI overlays (Search, Help, Version Badge, File Deleted) has been moved to a separate module.
- **File**: `src/internal/ui.rs`
- **Key Changes**:
    - Created `render_search_overlay`, `render_help_overlay`, `render_version_badge`, `render_file_deleted_overlay`.
    - These functions return `impl IntoElement` and are composed in the main render loop.

### 4. Simplified `src/main.rs`
`src/main.rs` is now a lightweight entry point responsible only for:
- Parsing command-line arguments.
- Loading configuration.
- Setting up the file watcher and background runtime.
- Initializing the `MarkdownViewer` and launching the application.

## Verification Results

### Automated Checks
- `cargo check` passes successfully, confirming that all modules are correctly wired and types are consistent.

### Manual Verification
- The application structure is now much cleaner and easier to navigate.
- Logic is separated by concern (rendering, events, state), making future features easier to implement.
