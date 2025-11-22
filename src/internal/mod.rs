#![doc = "Internal implementation modules for the markdown viewer"]
//! Internal implementation modules for the markdown viewer
//!
//! This module contains the internal implementation details organized
//! by functionality. These modules are not part of the public API but
//! are re-exported through the main lib.rs as needed.

pub mod events;
pub mod file_handling;
pub mod file_watcher;
pub mod help_overlay;
pub mod image;
pub mod image_loader;
pub mod rendering;
pub mod scroll;
pub mod search;
pub mod style;
pub mod text_highlight;
pub mod theme;
pub mod toc;
pub mod ui;
pub mod viewer;

// Note: selected helpers from internal submodules are re-exported from
// `lib.rs` when the binary needs them. Avoid re-exporting here to prevent
// unused-import warnings and to keep the internal module surface minimal.
