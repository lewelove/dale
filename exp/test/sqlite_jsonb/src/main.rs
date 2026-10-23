use rusqlite::Connection;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Deserialize)]
struct Config {
    storage: StorageConfig,
}

#[derive(Deserialize)]
struct StorageConfig {
    library_root: String,
}

fn expand_path(path_str: &str) -> PathBuf {
    if path_str.starts_with('~') {
        if let Some(mut home) = dirs::home_dir() {
            let stripped = path_str.strip_prefix("~/").unwrap_or("");
            home.push(stripped);
            return home;
        }
    }
    PathBuf::from(path_str)
}

fn main() {
    let config_path = dirs::home_dir()
        .expect("Failed to find home directory")
        .join(".config/datamile/config.toml");

    let config_content = fs::read_to_string(&config_path).expect("Failed to read config.toml");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse config.toml");
    let library_root = expand_path(&config.storage.library_root);

    println!("Target library root: {}", library_root.display());

    let scan_start = Instant::now();
    let mut lock_files = Vec::new();
    for entry in WalkDir::new(&library_root).into_iter().filter_map(Result::ok) {
        if entry.file_name() == "metadata.lock.json" {
            lock_files.push(entry.path().to_path_buf());
        }
    }
    let scan_duration = scan_start.elapsed();
    println!("Found {} lockfiles in {:?}", lock_files.len(), scan_duration);

    let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");

    conn.execute(
        "CREATE TABLE library (
            id TEXT PRIMARY KEY,
            metadata BLOB
        )",
        [],
    )
    .expect("Failed to create table");

    let ingest_start = Instant::now();
    let mut success_count = 0;
    
    for path in lock_files {
        if let Ok(content) = fs::read_to_string(&path) {
            let id = path
                .parent()
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            let res = conn.execute(
                "INSERT INTO library (id, metadata) VALUES (?1, jsonb(?2))",
                (&id, &content),
            );

            if res.is_ok() {
                success_count += 1;
            }
        }
    }
    let ingest_duration = ingest_start.elapsed();
    println!("Ingested {} files into SQLite JSONB in {:?}", success_count, ingest_duration);

    let q1_start = Instant::now();
    let mut stmt1 = conn
        .prepare(
            "SELECT 
                id,
                metadata ->> '$.album.ALBUM',
                metadata ->> '$.album.ALBUMARTIST',
                metadata ->> '$.album.DATE',
                metadata ->> '$.album.info.cover_hash'
            FROM library"
        )
        .unwrap();

    let rows1 = stmt1.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
        ))
    }).unwrap();
    let count1 = rows1.count();
    let q1_duration = q1_start.elapsed();
    println!("Query 1 (Grid Fetch): Fetched {} lightweight rows in {:?}", count1, q1_duration);

    let q2_start = Instant::now();
    let mut stmt2 = conn
        .prepare(
            "SELECT metadata ->> '$.album.ALBUM'
            FROM library 
            WHERE metadata ->> '$.album.tags.UNIX_ADDED_YOUTUBE' IS NOT NULL"
        )
        .unwrap();

    let rows2 = stmt2.query_map([], |row| row.get::<_, String>(0)).unwrap();
    let count2 = rows2.count();
    let q2_duration = q2_start.elapsed();
    println!("Query 2 (Shelf Filter): Found {} matches in {:?}", count2, q2_duration);

    let q3_start = Instant::now();
    let mut stmt3 = conn
        .prepare(
            "SELECT 
                metadata ->> '$.album.GENRE',
                COUNT(*) as count
            FROM library
            GROUP BY metadata ->> '$.album.GENRE'
            ORDER BY count DESC"
        )
        .unwrap();

    let rows3 = stmt3.query_map([], |row| row.get::<_, Option<String>>(0)).unwrap();
    let count3 = rows3.count();
    let q3_duration = q3_start.elapsed();
    println!("Query 3 (Grouper): Calculated {} genre groups in {:?}", count3, q3_duration);

    let q4_start = Instant::now();
    let mut stmt4 = conn
        .prepare(
            "SELECT metadata ->> '$.album.ALBUM'
            FROM library
            ORDER BY CAST(metadata ->> '$.album.info.unix_added' AS INTEGER) DESC"
        )
        .unwrap();

    let rows4 = stmt4.query_map([], |row| row.get::<_, Option<String>>(0)).unwrap();
    let count4 = rows4.count();
    let q4_duration = q4_start.elapsed();
    println!("Query 4 (Sorter): Sorted {} rows by unix_added in {:?}", count4, q4_duration);
}
