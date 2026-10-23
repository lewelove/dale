use crate::compile;
use crate::update::cache::FileStat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbumReloadPayload {
    pub id: String,
    pub lock_json: String,
}

pub struct NotificationTaskArgs {
    pub notify_rx: mpsc::Receiver<compile::stream::AlbumUpdateSignal>,
    pub cache_for_task: Arc<Mutex<HashMap<String, FileStat>>>,
    pub lib_root_for_task: Arc<PathBuf>,
    pub missing_paths: Vec<PathBuf>,
    pub start_time: std::time::Instant,
    pub silent: bool,
}

pub fn start_notification_task(args: NotificationTaskArgs) -> tokio::task::JoinHandle<()> {
    let mut rx = args.notify_rx;
    let cache_for_task = args.cache_for_task;
    let lib_root_for_task = args.lib_root_for_task;
    let missing_paths = args.missing_paths;
    let start_time = args.start_time;
    let silent = args.silent;

    tokio::spawn(async move {
        let mut updated_albums = Vec::new();
        while let Some(signal) = rx.recv().await {
            if !silent {
                log::info!("Updated: {} - {}", signal.artist, signal.album);
            }
            updated_albums.push((signal.path, signal.lock_json));
        }

        let mut payloads_for_server = Vec::new();

        {
            let mut c = cache_for_task.lock().await;
            for (album_root, lock_json) in &updated_albums {
                let album_id = libdatamile::resolvers::rel_path(album_root, &lib_root_for_task);
                let prefix = if album_id.is_empty() {
                    String::new()
                } else {
                    format!("{album_id}/")
                };

                c.retain(|k, _| {
                    if prefix.is_empty() {
                        k.contains('/')
                    } else {
                        !k.starts_with(&prefix)
                    }
                });

                for entry in walkdir::WalkDir::new(album_root)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    let p = entry.path();
                    if p.is_file() {
                        let rel = libdatamile::resolvers::rel_path(p, &lib_root_for_task);
                        if let Ok(m) = entry.metadata() {
                            c.insert(rel, FileStat::from(&m));
                        }
                    }
                }
                payloads_for_server.push(AlbumReloadPayload {
                    id: album_id,
                    lock_json: lock_json.clone(),
                });
            }

            for missing in missing_paths {
                let album_id = libdatamile::resolvers::rel_path(&missing, &lib_root_for_task);
                let prefix = format!("{album_id}/");

                if !silent {
                    let display_path = missing.strip_prefix(&*lib_root_for_task).unwrap_or(&missing);
                    log::info!("Removed: {}", display_path.display());
                }

                c.retain(|k, _| !k.starts_with(&prefix));
                payloads_for_server.push(AlbumReloadPayload {
                    id: album_id,
                    lock_json: String::new(),
                });
            }
            drop(c);
        }

        if payloads_for_server.is_empty() {
            return;
        }

        let elapsed = start_time.elapsed().as_millis();
        let client = reqwest::Client::new();
        let _ = client
            .post("http://127.0.0.1:8000/api/internal/batch_reload")
            .query(&[("time", elapsed.to_string())])
            .json(&payloads_for_server)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;
    })
}
