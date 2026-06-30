//! The shared ledger — the keystone of "minimize tokens, lose no context".
//!
//! One SQLite store holds four things:
//!   * blobs       — originals any layer compressed, keyed by a short handle
//!   * checkpoints — full session snapshots that survive context compaction
//!   * markers     — compact, named, hand-curated resume points
//!   * events      — per-layer token accounting for the dashboard
//!
//! Because every compression stashes its original here, any output is one
//! `obelisk restore <handle>` away from being whole again.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn db_path() -> PathBuf {
    let dir = dirs::data_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("obelisk");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("ledger.db")
}

fn open() -> Result<Connection> {
    let conn = Connection::open(db_path()).context("open ledger")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS blobs (
            handle TEXT PRIMARY KEY, layer TEXT NOT NULL, original BLOB NOT NULL,
            meta TEXT, created_at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS checkpoints (
            handle TEXT PRIMARY KEY, label TEXT NOT NULL, state TEXT NOT NULL,
            created_at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS markers (
            name TEXT PRIMARY KEY, content TEXT NOT NULL, updated_at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT, layer TEXT NOT NULL, command TEXT,
            tokens_before INTEGER NOT NULL, tokens_after INTEGER NOT NULL,
            created_at INTEGER NOT NULL);",
    )?;
    Ok(conn)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn short_hash(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().take(6).map(|b| format!("{b:02x}")).collect()
}

pub fn stash(original: &str, layer: &str, meta: &str) -> Result<String> {
    let handle = short_hash(original.as_bytes());
    open()?.execute(
        "INSERT OR REPLACE INTO blobs VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![handle, layer, original.as_bytes(), meta, now()],
    )?;
    Ok(handle)
}

pub fn restore(handle: &str) -> Result<Option<String>> {
    let conn = open()?;
    {
        let mut stmt = conn.prepare("SELECT original FROM blobs WHERE handle=?1")?;
        let mut rows = stmt.query([handle])?;
        if let Some(row) = rows.next()? {
            let b: Vec<u8> = row.get(0)?;
            return Ok(Some(String::from_utf8_lossy(&b).into_owned()));
        }
    }
    {
        let mut stmt = conn.prepare("SELECT state FROM checkpoints WHERE handle=?1")?;
        let mut rows = stmt.query([handle])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(row.get(0)?));
        }
    }
    Ok(None)
}

pub fn checkpoint(state: &str, label: &str) -> Result<String> {
    let handle = short_hash(format!("{label}{}", now()).as_bytes());
    open()?.execute(
        "INSERT OR REPLACE INTO checkpoints VALUES (?1,?2,?3,?4)",
        rusqlite::params![handle, label, state, now()],
    )?;
    Ok(handle)
}

pub fn record_event(layer: &str, command: &str, before: usize, after: usize) -> Result<()> {
    open()?.execute(
        "INSERT INTO events (layer,command,tokens_before,tokens_after,created_at)
         VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![layer, command, before as i64, after as i64, now()],
    )?;
    Ok(())
}

pub fn event_rollup() -> Result<Vec<(String, i64, i64, i64)>> {
    let conn = open()?;
    let mut stmt = conn.prepare(
        "SELECT layer, COALESCE(SUM(tokens_before),0), COALESCE(SUM(tokens_after),0),
                COUNT(*) FROM events GROUP BY layer ORDER BY 2 DESC",
    )?;
    let rows: Vec<(String, i64, i64, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn store_counts() -> Result<(i64, i64, i64)> {
    let conn = open()?;
    Ok((
        conn.query_row("SELECT COUNT(*) FROM blobs", [], |r| r.get(0))?,
        conn.query_row("SELECT COUNT(*) FROM checkpoints", [], |r| r.get(0))?,
        conn.query_row("SELECT COUNT(*) FROM markers", [], |r| r.get(0))?,
    ))
}

// --- markers ---------------------------------------------------------------
pub fn marker_save(name: &str, content: &str) -> Result<()> {
    open()?.execute(
        "INSERT OR REPLACE INTO markers VALUES (?1,?2,?3)",
        rusqlite::params![name, content, now()],
    )?;
    Ok(())
}

pub fn marker_get(name: &str) -> Result<Option<String>> {
    let conn = open()?;
    let mut stmt = conn.prepare("SELECT content FROM markers WHERE name=?1")?;
    let mut rows = stmt.query([name])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row.get(0)?));
    }
    Ok(None)
}

pub fn marker_list() -> Result<Vec<(String, i64, i64)>> {
    let conn = open()?;
    let now = now();
    let mut stmt =
        conn.prepare("SELECT name, LENGTH(content), updated_at FROM markers ORDER BY updated_at DESC")?;
    let rows: Vec<(String, i64, i64)> = stmt
        .query_map([], |r| {
            let upd: i64 = r.get(2)?;
            Ok((r.get(0)?, r.get(1)?, now - upd))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn marker_delete(name: &str) -> Result<bool> {
    Ok(open()?.execute("DELETE FROM markers WHERE name=?1", [name])? > 0)
}

pub fn gc(days: i64) -> Result<usize> {
    Ok(open()?.execute("DELETE FROM blobs WHERE created_at < ?1", [now() - days * 86400])?)
}
