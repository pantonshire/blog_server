use std::{
    path::Path,
    sync::mpsc,
    time::Duration,
};

use notify::{
    DebouncedEvent,
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
    watcher,
};
use tracing::info;

use crate::Error;

pub(crate) fn start_watching(
    tx: mpsc::Sender<DebouncedEvent>,
    watch_path: &Path,
    delay: Duration
) -> Result<RecommendedWatcher, Error>
{
    let mut watcher = watcher(tx, delay)
        .map_err(Error::CreateWatcher)?;

    // Watch the path in non-recursive mode, so events are not generated for nodes in
    // sub-directories.
    watcher.watch(watch_path, RecursiveMode::NonRecursive)
        .map_err(|err| Error::WatchDir(watch_path.to_owned(), err))?;

    info!(path = %watch_path.to_string_lossy(), "Watching directory");

    Ok(watcher)
}
