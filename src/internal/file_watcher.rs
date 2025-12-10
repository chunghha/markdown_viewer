//! File watching module for automatic reload on file changes
//!
//! This module provides file system monitoring using the `notify` crate,
//! with debouncing to handle rapid file changes gracefully.

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_full::{DebouncedEvent, Debouncer, FileIdMap, new_debouncer};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;
use tracing::{debug, error, info};

/// Events emitted by the file watcher
#[derive(Debug, Clone)]
pub enum FileWatcherEvent {
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// Error occurred while watching
    Error(String),
}

/// Start watching a file for changes
///
/// Returns a receiver that will receive `FileWatcherEvent`s when the file changes.
/// The watcher runs in a background thread and will continue until the receiver is dropped.
///
/// # Arguments
/// * `file_path` - Path to the file to watch
/// * `debounce_ms` - Debounce timeout in milliseconds (typically 100ms)
///
/// # Returns
/// A tuple of (event receiver, debouncer handle). The debouncer must be kept alive
/// for watching to continue.
pub fn start_watching(
    file_path: &Path,
    debounce_ms: u64,
) -> Result<(
    Receiver<FileWatcherEvent>,
    Debouncer<notify::RecommendedWatcher, FileIdMap>,
)> {
    let (tx, rx) = channel();
    let file_path = file_path.to_path_buf();

    info!("Starting file watcher for: {:?}", file_path);

    // Create a debouncer with the specified timeout
    let debounce_duration = Duration::from_millis(debounce_ms);

    let tx_clone = tx.clone();
    // Clone file_path for use in the closure
    let file_path_for_closure = file_path.clone();
    let mut debouncer = new_debouncer(
        debounce_duration,
        None,
        move |result: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
            match result {
                Ok(events) => {
                    for event in events {
                        debug!("File watcher event: {:?}", event);

                        // Check if any of the paths match our watched file
                        for path in &event.paths {
                            if path == &file_path_for_closure {
                                match event.kind {
                                    notify::EventKind::Remove(_) => {
                                        info!("File deleted: {:?}", path);
                                        tx_clone.send(FileWatcherEvent::Deleted).ok();
                                    }
                                    notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                                        info!("File modified: {:?}", path);
                                        tx_clone.send(FileWatcherEvent::Modified).ok();
                                    }
                                    _ => {
                                        debug!("Ignoring event kind: {:?}", event.kind);
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        error!("File watcher error: {:?}", error);
                        tx_clone
                            .send(FileWatcherEvent::Error(error.to_string()))
                            .ok();
                    }
                }
            }
        },
    )
    .context("Failed to create file watcher debouncer")?;

    // Watch the file's parent directory (watching individual files isn't supported on all platforms)
    let watch_path = match file_path.is_file() {
        true => file_path
            .parent()
            .context("File has no parent directory")?
            .to_path_buf(),
        false => file_path.clone(),
    };

    // Call watch directly on debouncer (watcher() is deprecated)
    debouncer
        .watch(&watch_path, RecursiveMode::NonRecursive)
        .context("Failed to start watching file")?;

    debug!("File watcher started for: {:?}", watch_path);

    Ok((rx, debouncer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;

    #[test]
    fn test_file_watcher_detects_modification() {
        // Create a temporary file
        let temp_file = "test_watch_modify.txt";
        fs::write(temp_file, "initial content").expect("Failed to create test file");

        // Get absolute path
        let abs_path = fs::canonicalize(temp_file).expect("Failed to canonicalize path");

        // Start watching
        let (rx, _debouncer) = start_watching(&abs_path, 50).expect("Failed to start watcher");

        // Give the watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        // Modify the file
        fs::write(temp_file, "modified content").expect("Failed to modify test file");

        // Wait for the event (with timeout)
        let event = rx.recv_timeout(Duration::from_secs(2));

        // Clean up
        fs::remove_file(temp_file).ok();

        // Verify we got a Modified event
        assert!(event.is_ok(), "Should receive an event");
        match event.unwrap() {
            FileWatcherEvent::Modified => {
                // Success
            }
            other => panic!("Expected Modified event, got {:?}", other),
        }
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "File deletion detection is unreliable on macOS"
    )]
    fn test_file_watcher_detects_deletion() {
        // Create a temporary file
        let temp_file = "test_watch_delete.txt";
        fs::write(temp_file, "content").expect("Failed to create test file");

        // Get absolute path
        let abs_path = fs::canonicalize(temp_file).expect("Failed to canonicalize path");

        // Start watching
        let (rx, _debouncer) = start_watching(&abs_path, 50).expect("Failed to start watcher");

        // Give the watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        // Delete the file
        fs::remove_file(temp_file).expect("Failed to delete test file");

        // Collect events for a reasonable timeout
        // File systems may send multiple events (e.g., modify then delete)
        thread::sleep(Duration::from_millis(500));

        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        // Verify we got at least one Deleted event
        let has_deleted = events
            .iter()
            .any(|e| matches!(e, FileWatcherEvent::Deleted));
        assert!(
            has_deleted,
            "Should receive a Deleted event, got: {:?}",
            events
        );
    }

    #[test]
    fn test_file_watcher_debounces_rapid_changes() {
        // Create a temporary file
        let temp_file = "test_watch_debounce.txt";
        fs::write(temp_file, "initial").expect("Failed to create test file");

        // Get absolute path
        let abs_path = fs::canonicalize(temp_file).expect("Failed to canonicalize path");

        // Start watching with 200ms debounce
        let (rx, _debouncer) = start_watching(&abs_path, 200).expect("Failed to start watcher");

        // Give the watcher time to initialize
        thread::sleep(Duration::from_millis(100));

        // Make 5 rapid changes
        for i in 0..5 {
            fs::write(temp_file, format!("change {}", i)).expect("Failed to modify test file");
            thread::sleep(Duration::from_millis(20)); // 20ms between changes
        }

        // Wait for debounce period plus some margin
        thread::sleep(Duration::from_millis(400));

        // Count events received
        let mut event_count = 0;
        while rx.try_recv().is_ok() {
            event_count += 1;
        }

        // Clean up
        fs::remove_file(temp_file).ok();

        // Should receive fewer events than the number of changes due to debouncing
        // Debouncing should reduce 5 rapid changes to fewer events
        assert!(
            event_count < 5,
            "Expected fewer than 5 events due to debouncing, got {}",
            event_count
        );
        assert!(event_count >= 1, "Should receive at least 1 event");
    }
}
