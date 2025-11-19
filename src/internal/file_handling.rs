//! File handling utilities for the markdown viewer
//!
//! This module provides functions for resolving file paths and loading
//! markdown content with proper error handling.

use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info};

/// Check if a file has a supported extension
///
/// # Arguments
/// * `file_path` - Path to the file
/// * `supported_extensions` - List of supported extensions (without dots)
///
/// # Returns
/// * `true` if the file has a supported extension, `false` otherwise
pub fn is_supported_extension(file_path: &str, supported_extensions: &[String]) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            supported_extensions
                .iter()
                .any(|supported| supported.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}

/// Resolves the markdown file path based on CLI argument or default
///
/// # Arguments
/// * `file_path` - Optional file path from CLI arguments
/// * `supported_extensions` - List of supported file extensions (without dots)
///
/// # Returns
/// * `Ok(String)` - The resolved file path
/// * `Err` - Error if file resolution fails
pub fn resolve_markdown_file_path(
    file_path: Option<&str>,
    supported_extensions: &[String],
) -> Result<String> {
    match file_path {
        Some(path) => {
            debug!("Resolving file path: {}", path);
            if !Path::new(path).exists() {
                anyhow::bail!("File not found: {}", path);
            }

            if !is_supported_extension(path, supported_extensions) {
                let supported_list = supported_extensions.join(", ");
                anyhow::bail!(
                    "Unsupported file format. File '{}' does not have a supported extension.\nSupported formats: {}",
                    path,
                    supported_list
                );
            }

            info!("File found: {}", path);
            Ok(path.to_string())
        }
        None => {
            debug!("No file specified, trying default files");
            // Try README.md first, then TODO.md as fallback
            let readme_path = "README.md";
            let todo_path = "TODO.md";

            if Path::new(readme_path).exists() {
                info!("Using default file: {}", readme_path);
                Ok(readme_path.to_string())
            } else if Path::new(todo_path).exists() {
                info!("Using fallback file: {}", todo_path);
                Ok(todo_path.to_string())
            } else {
                anyhow::bail!(
                    "Default files README.md and TODO.md not found. Please specify a markdown file."
                );
            }
        }
    }
}

/// Loads markdown content from a file
///
/// # Arguments
/// * `file_path` - Path to the markdown file
///
/// # Returns
/// * `Ok(String)` - The file content
/// * `Err` - Error if loading fails
pub fn load_markdown_content(file_path: &str) -> Result<String> {
    debug!("Loading markdown content from: {}", file_path);
    let content = std::fs::read_to_string(file_path)
        .context(format!("Failed to read file '{}'", file_path))?;
    info!(
        "Successfully loaded {} bytes from {}",
        content.len(),
        file_path
    );
    Ok(content)
}
