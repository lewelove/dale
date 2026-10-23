mod classifier;
pub mod handler;
#[cfg(test)]
mod tests;
mod watcher;

use crate::server::state::AppState;
use std::sync::Arc;

pub fn start(state: Arc<AppState>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut fs_watcher = watcher::setup_watcher(tx);

    tokio::spawn(async move {
        watcher::run_loop(rx, &mut fs_watcher, state).await;
    });
}
