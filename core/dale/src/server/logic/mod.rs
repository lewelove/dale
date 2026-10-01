pub mod cache;
pub mod query;
pub mod sort;

pub use sort::{SortKey, SortOrder, value_to_sort_key};

use anyhow::Result;
use libdale::error::DaleError;
use libdale::images::{FilterType, FitMode, parse_filter, parse_fit_mode};
use libdale::lua::{LogicManifest, with_evaluated_lua_vm};
use roaring::RoaringBitmap;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct CoverLookupItem {
    pub album_dir: PathBuf,
    pub path: String,
    pub fit_mode: FitMode,
    pub target_width: u32,
    pub target_height: u32,
    pub filter: FilterType,
}

pub struct LogicEngine {
    pub config_path: PathBuf,
    pub manifest: LogicManifest,
    pub(crate) libraries_cache: HashMap<String, RoaringBitmap>,
    pub(crate) filters_cache: HashMap<String, RoaringBitmap>,
    pub(crate) groupers_cache: HashMap<String, HashMap<String, RoaringBitmap>>,
    pub(crate) precomputed_groups: HashMap<(String, Option<String>, String), Vec<Value>>,
    pub(crate) orders_cache: HashMap<String, Vec<u32>>,
    pub(crate) shelves_cache: HashMap<String, Vec<u32>>,
    pub(crate) uid_to_id: HashMap<u32, String>,
    pub(crate) id_to_uid: HashMap<String, u32>,
    pub(crate) album_metrics: HashMap<u32, (u64, u64, u64)>,
    pub(crate) next_uid: u32,
    pub albums_by_path: HashMap<PathBuf, String>,
    pub path_by_id: HashMap<String, PathBuf>,
    pub cover_lookup: HashMap<String, CoverLookupItem>,
    pub dict: HashMap<String, Value>,
    pub track_lookup: HashMap<String, Value>,
    pub path_lookup: HashMap<String, String>,
    pub(crate) evaluated_logic: HashMap<u32, Value>,
    pub(crate) lock_cache: HashMap<String, String>,
}

impl LogicEngine {
    pub fn new() -> Result<Self> {
        let config_path = libdale::lua::resolve_config_path().unwrap_or_default();
        let manifest = with_evaluated_lua_vm(&config_path, |_, eval| Ok(eval.manifest))?;

        Ok(Self {
            config_path,
            manifest,
            libraries_cache: HashMap::new(),
            filters_cache: HashMap::new(),
            groupers_cache: HashMap::new(),
            precomputed_groups: HashMap::new(),
            orders_cache: HashMap::new(),
            shelves_cache: HashMap::new(),
            uid_to_id: HashMap::new(),
            id_to_uid: HashMap::new(),
            album_metrics: HashMap::new(),
            next_uid: 1,
            albums_by_path: HashMap::new(),
            path_by_id: HashMap::new(),
            cover_lookup: HashMap::new(),
            dict: HashMap::new(),
            track_lookup: HashMap::new(),
            path_lookup: HashMap::new(),
            evaluated_logic: HashMap::new(),
            lock_cache: HashMap::new(),
        })
    }

    pub fn reload_manifest(&mut self, config_path: &Path) -> Result<()> {
        let manifest = with_evaluated_lua_vm(config_path, |engine, eval| {
            let mut new_evaluated = HashMap::new();
            for (&uid, id) in &self.uid_to_id {
                if let Some(json_str) = self.lock_cache.get(id)
                    && let Ok(parsed) = serde_json::from_str::<Value>(json_str)
                    && let Ok(res) = engine.evaluate_album_logic(&parsed)
                {
                    new_evaluated.insert(uid, res);
                }
            }
            self.evaluated_logic = new_evaluated;
            Ok(eval.manifest)
        })?;
        self.config_path = config_path.to_path_buf();
        self.manifest = manifest;
        self.build_cache();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.libraries_cache.clear();
        self.filters_cache.clear();
        self.groupers_cache.clear();
        self.precomputed_groups.clear();
        self.orders_cache.clear();
        self.shelves_cache.clear();
        self.uid_to_id.clear();
        self.id_to_uid.clear();
        self.album_metrics.clear();
        self.albums_by_path.clear();
        self.path_by_id.clear();
        self.cover_lookup.clear();
        self.next_uid = 1;
        self.dict.clear();
        self.track_lookup.clear();
        self.path_lookup.clear();
        self.evaluated_logic.clear();
        self.lock_cache.clear();
    }

    pub fn remove_album_by_path(&mut self, path: &Path) -> Option<String> {
        let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.albums_by_path.remove(&canon).inspect(|id| {
            self.remove_album_by_id(id);
        })
    }

    pub fn remove_album_by_id(&mut self, id: &str) {
        if let Some(uid) = self.id_to_uid.remove(id) {
            self.uid_to_id.remove(&uid);
            self.evaluated_logic.remove(&uid);
            self.album_metrics.remove(&uid);
        }
        if let Some(path) = self.path_by_id.remove(id) {
            self.albums_by_path.remove(&path);
            self.cover_lookup.retain(|_, item| item.album_dir != path);
        }
        self.dict.remove(id);
        self.lock_cache.remove(id);
        self.path_lookup.retain(|_, v| v != id);
        self.track_lookup
            .retain(|_, v| v.get("albumId").and_then(|a| a.as_str()) != Some(id));
    }

    fn validate_and_cleanup_paths(
        &mut self,
        album_dir_canon: &Path,
        id: &str,
    ) -> Result<()> {
        if let Some(existing_path) = self.path_by_id.get(id)
            && existing_path != album_dir_canon
        {
            if existing_path.exists() {
                return Err(DaleError::DuplicateAlbumId {
                    id: id.to_string(),
                    path_a: existing_path.clone(),
                    path_b: album_dir_canon.to_path_buf(),
                }
                .into());
            }
            let existing_path_clone = existing_path.clone();
            self.remove_album_by_path(&existing_path_clone);
        }

        if let Some(old_id) = self.albums_by_path.get(album_dir_canon)
            && old_id != id
        {
            let old_id_clone = old_id.clone();
            self.remove_album_by_id(&old_id_clone);
        }

        self.remove_album_by_id(id);

        Ok(())
    }

    fn ingest_tracks_and_lookup(
        &mut self,
        tracks: &[Value],
        album_dir_canon: &Path,
        id: &str,
        music_directory: &Path,
    ) {
        for (track_idx, track) in tracks.iter().enumerate() {
            if let Some(tinfo) = track.get("info") {
                let rel = track
                    .get("file")
                    .and_then(|f| f.get("path"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !rel.is_empty() {
                    let abs_track_path = album_dir_canon.join(rel);
                    let mpd_uri_raw =
                        abs_track_path.strip_prefix(music_directory).map_or_else(
                            |_| abs_track_path.to_string_lossy().to_string(),
                            |p| p.to_string_lossy().to_string(),
                        );
                    let mpd_uri = mpd_uri_raw.trim_start_matches('/').to_string();

                    let track_no = track
                        .get("tracknumber")
                        .cloned()
                        .unwrap_or_else(|| json!(0));
                    let disc_no =
                        track.get("discnumber").cloned().unwrap_or_else(|| json!(1));
                    let title = track
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| json!("Unknown"));
                    let artist = track
                        .get("artist")
                        .cloned()
                        .unwrap_or_else(|| json!("Unknown"));
                    let duration = tinfo
                        .get("duration_formatted")
                        .cloned()
                        .unwrap_or_else(|| json!("0:00"));
                    let duration_ms = tinfo
                        .get("duration_milliseconds")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);

                    let track_light = json!({
                        "path": mpd_uri,
                        "trackIndex": track_idx,
                        "trackNo": track_no,
                        "discNo": disc_no,
                        "title": title,
                        "artist": artist,
                        "duration": duration,
                        "durationMs": duration_ms,
                        "albumId": id
                    });
                    self.track_lookup.insert(mpd_uri.clone(), track_light);
                    self.path_lookup.insert(mpd_uri, id.to_string());
                }
            }
        }
    }

    fn ingest_covers(
        &mut self,
        album: &Value,
        album_dir_canon: &Path,
    ) -> (Option<Value>, Option<Value>) {
        let main_cover = album
            .get("covers")
            .and_then(|c| c.get("main"))
            .and_then(Value::as_array)
            .and_then(|arr| arr.first());

        let cover_path_val = main_cover
            .and_then(|m| m.pointer("/source/file/path"))
            .cloned();
        let cover_hash_val = main_cover.and_then(|m| m.get("address")).cloned();

        let Some(covers) = album.get("covers").and_then(Value::as_object) else {
            return (cover_path_val, cover_hash_val);
        };

        for entries in covers.values().filter_map(Value::as_array) {
            for entry in entries {
                let Some(addr) = entry.get("address").and_then(Value::as_str) else {
                    continue;
                };
                let Some(p_str) =
                    entry.pointer("/source/file/path").and_then(Value::as_str)
                else {
                    continue;
                };
                let fit_str =
                    entry.get("fit").and_then(Value::as_str).unwrap_or("inside");
                let filter_str = entry
                    .get("filter")
                    .and_then(Value::as_str)
                    .unwrap_or("catmullrom");
                let width = entry
                    .get("width")
                    .and_then(Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok())
                    .unwrap_or(1080);
                let height = entry
                    .get("height")
                    .and_then(Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok())
                    .unwrap_or(1080);

                let fit_mode = parse_fit_mode(fit_str).unwrap_or(FitMode::Inside);
                let filter = parse_filter(filter_str).unwrap_or(FilterType::CatmullRom);

                self.cover_lookup.insert(
                    addr.to_string(),
                    CoverLookupItem {
                        album_dir: album_dir_canon.to_path_buf(),
                        path: p_str.to_string(),
                        fit_mode,
                        target_width: width,
                        target_height: height,
                        filter,
                    },
                );
            }
        }

        (cover_path_val, cover_hash_val)
    }

    pub fn ingest_pre_evaluated(
        &mut self,
        album_dir: &Path,
        id: &str,
        metadata_json: &str,
        eval_res: Value,
        music_directory: &Path,
    ) -> Result<()> {
        let album_dir_canon = album_dir
            .canonicalize()
            .unwrap_or_else(|_| album_dir.to_path_buf());
        self.validate_and_cleanup_paths(&album_dir_canon, id)?;

        let uid = self.next_uid;
        self.next_uid += 1;

        let parsed: Value = serde_json::from_str(metadata_json)?;

        self.uid_to_id.insert(uid, id.to_string());
        self.id_to_uid.insert(id.to_string(), uid);
        self.albums_by_path
            .insert(album_dir_canon.clone(), id.to_string());
        self.path_by_id
            .insert(id.to_string(), album_dir_canon.clone());
        self.lock_cache
            .insert(id.to_string(), metadata_json.to_string());

        if let Some(album) = parsed.get("album")
            && let Some(info) = album.get("info")
        {
            let duration_ms = info
                .get("duration_milliseconds")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let total_tracks = info
                .get("total_tracks")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let total_discs =
                info.get("total_discs").and_then(Value::as_u64).unwrap_or(1);
            self.album_metrics
                .insert(uid, (duration_ms, total_tracks, total_discs));

            if let Some(tracks) = parsed.get("tracks").and_then(Value::as_array) {
                self.ingest_tracks_and_lookup(
                    tracks,
                    &album_dir_canon,
                    id,
                    music_directory,
                );
            }

            let (cover_path_val, cover_hash_val) =
                self.ingest_covers(album, &album_dir_canon);

            let entry = json!({
                "id": id,
                "album": album.get("album"),
                "albumartist": album.get("albumartist"),
                "date": album.get("date"),
                "cover_path": cover_path_val,
                "cover_hash": cover_hash_val,
                "covers": album.get("covers"),
                "duration_formatted": info.get("duration_formatted"),
                "total_discs": info.get("total_discs"),
                "total_tracks": info.get("total_tracks"),
                "virtual": info.get("virtual"),
                "keys": album.get("keys"),
                "colors": album.get("colors")
            });
            self.dict.insert(id.to_string(), entry);
        }

        self.evaluated_logic.insert(uid, eval_res);
        Ok(())
    }

    #[must_use]
    pub fn get_album_json(&self, id: &str) -> Option<String> {
        self.lock_cache.get(id).cloned()
    }
}
