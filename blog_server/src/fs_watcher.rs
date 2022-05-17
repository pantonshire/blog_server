use std::{
    path::Path,
    sync::mpsc,
    time::Duration,
};

use miette::{IntoDiagnostic, WrapErr};
use notify::{
    DebouncedEvent,
    FsEventWatcher,
    RecursiveMode,
    Watcher,
    watcher,
};
use tracing::info;

pub fn start_watching(
    tx: mpsc::Sender<DebouncedEvent>,
    watch_path: &Path
) -> miette::Result<FsEventWatcher>
{
    let mut watcher = watcher(tx, Duration::from_secs(2))
        .into_diagnostic()
        .wrap_err("Failed to create filesystem watcher")?;

    // Watch the path in non-recursive mode, so events are not generated for nodes in
    // sub-directories.
    watcher.watch(watch_path, RecursiveMode::NonRecursive)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to watch directory {}", watch_path.to_string_lossy()))?;

    info!(path = %watch_path.to_string_lossy(), "Watching directory");

    Ok(watcher)
}
