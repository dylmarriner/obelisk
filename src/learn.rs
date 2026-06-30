//! Usage-triggered self-improvement.
//!
//! Obelisk logs coverage/correctness gaps (see `ledger::record_gap`) as it's
//! used. Once enough gaps pile up, this module hands them to a detached
//! `self-improve.sh` script that runs the agent to draft a fix, gates the
//! result on `cargo build && cargo test`, and only commits+pushes to `main`
//! if both pass. Nothing here calls an LLM or touches git directly — that's
//! all in the script, kept out of the binary so it's auditable and editable
//! without a rebuild.
//!
//! Disabled until `obelisk learn enable <repo-path>` is run once, so a fresh
//! install never pushes to a repo the user hasn't pointed it at.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
struct Config {
    repo_path: Option<String>,
    threshold: Option<i64>,
}

fn config_path() -> PathBuf {
    let dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.join("obelisk").join("learn.json")
}

fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

fn lock_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("obelisk")
        .join("self-improve.lock")
}

const DEFAULT_THRESHOLD: i64 = 15;

pub fn enable(repo_path: &str, threshold: Option<i64>) -> Result<i32> {
    let repo = std::fs::canonicalize(repo_path).context("repo path")?;
    let script = repo.join("scripts").join("self-improve.sh");
    if !script.exists() {
        eprintln!(
            "obelisk: no scripts/self-improve.sh in {} — nothing to enable",
            repo.display()
        );
        return Ok(1);
    }
    let cfg = Config {
        repo_path: Some(repo.to_string_lossy().into_owned()),
        threshold,
    };
    save_config(&cfg)?;
    println!(
        "✓ self-improvement enabled for {} (threshold: {} gaps)",
        repo.display(),
        threshold.unwrap_or(DEFAULT_THRESHOLD)
    );
    println!("  every obelisk-routed command now counts toward the next trigger.");
    println!("  disable any time with: obelisk learn disable");
    Ok(0)
}

pub fn disable() -> Result<i32> {
    let _ = std::fs::remove_file(config_path());
    println!("✓ self-improvement disabled");
    Ok(0)
}

pub fn status() -> Result<i32> {
    let cfg = load_config();
    match &cfg.repo_path {
        Some(repo) => {
            let pending = crate::ledger::pending_gap_count().unwrap_or(0);
            let threshold = cfg.threshold.unwrap_or(DEFAULT_THRESHOLD);
            println!("repo      : {repo}");
            println!("threshold : {threshold} gaps");
            println!("pending   : {pending} gaps since last trigger");
            for (kind, prog, n) in crate::ledger::pending_gap_summary().unwrap_or_default() {
                println!("  {n:>4}  {kind:<14} {prog}");
            }
        }
        None => println!("self-improvement is disabled (run `obelisk learn enable <repo-path>`)"),
    }
    Ok(0)
}

pub fn gaps_json() -> Result<i32> {
    let rows = crate::ledger::pending_gap_summary()?;
    let json: Vec<_> = rows
        .into_iter()
        .map(|(kind, prog, n)| serde_json::json!({"kind": kind, "prog": prog, "count": n}))
        .collect();
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(0)
}

/// Called after every recorded gap. Fires the self-improve script in the
/// background if enabled, over threshold, and not already running. Never
/// blocks the caller and never fails the calling command on error.
pub fn maybe_trigger() {
    let _ = try_trigger();
}

fn try_trigger() -> Result<()> {
    let cfg = load_config();
    let repo = match cfg.repo_path {
        Some(r) => r,
        None => return Ok(()), // not enabled
    };
    let threshold = cfg.threshold.unwrap_or(DEFAULT_THRESHOLD);
    let pending = crate::ledger::pending_gap_count()?;
    if pending < threshold {
        return Ok(());
    }

    // A simple file lock: if a previous run is still going, skip. The script
    // itself removes the lock when it finishes (success or failure).
    let lock = lock_path();
    if lock.exists() {
        return Ok(());
    }
    if let Some(parent) = lock.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&lock, std::process::id().to_string())?;

    crate::ledger::mark_gaps_triggered()?;

    let script = PathBuf::from(&repo).join("scripts").join("self-improve.sh");
    std::process::Command::new("sh")
        .arg(&script)
        .arg(&repo)
        .arg(lock.to_string_lossy().into_owned())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("spawn self-improve.sh")?;

    eprintln!("[obelisk] {pending} gaps logged — self-improvement loop launched in background");
    Ok(())
}
