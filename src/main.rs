use anyhow::{Context, Result};
use clap::Parser;
use gpui::{App, AppContext, Application, WindowOptions};
use markdown_viewer::{
    MarkdownViewer, config::AppConfig, load_markdown_content, resolve_markdown_file_path,
    start_watching,
};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[derive(Parser)]
#[command(name = "markdown_viewer")]
#[command(about = "A simple markdown viewer")]
struct Args {
    /// Path to the markdown file to view
    file: Option<String>,
}

fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Markdown Viewer");

    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}. Using defaults.", e);
        AppConfig::default()
    });

    debug!("Configuration loaded: {:?}", config);

    let args = Args::parse();

    // Resolve the file path using our new function
    let file_path =
        resolve_markdown_file_path(args.file.as_deref(), &config.files.supported_extensions)
            .context("Failed to resolve markdown file path")?;

    // Load the markdown content
    let markdown_input =
        load_markdown_content(&file_path).context("Failed to load markdown content")?;

    info!(
        "Loaded file: {} ({} bytes)",
        file_path,
        markdown_input.len()
    );

    // Create a dedicated background Tokio runtime for async tasks (image downloads, etc.)
    let bg_rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .context("Failed to build background Tokio runtime")?,
    );

    // Start file watcher if enabled
    let (file_watcher_rx, file_watcher) = if config.file_watcher.enabled {
        // Convert to absolute path for file watcher
        let abs_file_path = std::fs::canonicalize(&file_path)
            .unwrap_or_else(|_| std::path::PathBuf::from(&file_path));

        match start_watching(&abs_file_path, config.file_watcher.debounce_ms) {
            Ok((rx, debouncer)) => {
                info!("File watcher started for: {}", file_path);
                (Some(rx), Some(debouncer))
            }
            Err(e) => {
                warn!(
                    "Failed to start file watcher for '{}': {:?}. Continuing without auto-reload.",
                    file_path, e
                );
                (None, None)
            }
        }
    } else {
        info!("File watcher disabled in configuration");
        (None, None)
    };

    // Run the GUI on the main thread (required by gpui). Background async work will use `bg_rt`.
    Application::new().run(move |app: &mut App| {
        let window_config = config.clone();
        let file_path_buf = PathBuf::from(file_path.clone());
        let bg_rt = bg_rt.clone();
        let window = app
            .open_window(WindowOptions::default(), move |_, cx| {
                // We can't focus here because we don't have &mut Window
                cx.new(|cx| {
                    let focus_handle = cx.focus_handle();
                    let viewer = MarkdownViewer::new(
                        markdown_input.clone(),
                        file_path_buf,
                        window_config,
                        bg_rt.clone(),
                        focus_handle,
                        file_watcher_rx,
                        file_watcher,
                    );
                    debug!("MarkdownViewer initialized");
                    viewer
                })
            })
            .unwrap();

        window
            .update(app, |view, cx, _| {
                view.focus_handle.focus(cx);
            })
            .ok();
    });

    Ok(())
}
