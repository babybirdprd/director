//! # Watch Module
//!
//! File watching for automatic context regeneration.

use crate::synthesizer;
use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Watch a directory for .ron spec file changes and regenerate context.
pub fn watch_specs(dir: &Path, output: &Path) -> Result<()> {
    let (tx, rx) = channel();

    // Create watcher
    let mut watcher = notify::recommended_watcher(move |result: Result<Event, _>| {
        if let Ok(event) = result {
            let _ = tx.send(event);
        }
    })?;

    // Watch directory
    std::fs::create_dir_all(dir)?;
    std::fs::create_dir_all(output)?;
    watcher.watch(dir, RecursiveMode::Recursive)?;

    tracing::info!("Watching {} for spec changes...", dir.display());
    tracing::info!("Output directory: {}", output.display());
    tracing::info!("Press Ctrl+C to stop.\n");

    // Process on startup - generate for all existing specs
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ron") {
            process_spec_file(&path, output);
        }
    }

    // Main event loop with debouncing
    let debounce_duration = Duration::from_millis(500);
    let mut last_event_time = std::time::Instant::now();
    let mut pending_paths: Vec<std::path::PathBuf> = Vec::new();

    loop {
        match rx.recv_timeout(debounce_duration) {
            Ok(event) => {
                // Collect paths with .ron extension
                for path in event.paths {
                    if path.extension().map_or(false, |e| e == "ron") {
                        if !pending_paths.contains(&path) {
                            pending_paths.push(path);
                        }
                        last_event_time = std::time::Instant::now();
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Process pending paths after debounce
                if !pending_paths.is_empty() && last_event_time.elapsed() >= debounce_duration {
                    for path in pending_paths.drain(..) {
                        process_spec_file(&path, output);
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    Ok(())
}

fn process_spec_file(path: &Path, output_dir: &Path) {
    tracing::info!("Spec changed: {}", path.display());

    let output_file = output_dir
        .join(path.file_stem().unwrap_or_default())
        .with_extension("md");

    match synthesizer::generate_context(path, &output_file) {
        Ok(()) => {
            tracing::info!("  → Generated {}", output_file.display());
        }
        Err(e) => {
            tracing::error!("  → Failed to generate context: {}", e);
        }
    }
}
