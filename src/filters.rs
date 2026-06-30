//! Program-aware output filters.
//!
//! Each filter takes a command's raw output and returns a compact form that
//! preserves the signal an agent needs (errors, changed paths, results) while
//! dropping noise (progress bars, decorative borders, repeated frames). All
//! original content remains recoverable via the ledger, so filters can be
//! aggressive. Anything without a dedicated filter falls back to the generic
//! squeeze.

use crate::squeeze;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ANSI: Regex = Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap();
    static ref ERRORLIKE: Regex =
        Regex::new(r"(?i)\b(error|warning|failed|failure|panic|exception|fatal|cannot|undefined|unresolved)\b").unwrap();
    static ref TEST_RESULT: Regex =
        Regex::new(r"(?i)(\d+\s+(passed|failed|ok|error|skipped))|(test result:)|(FAILED|PASSED)").unwrap();
    static ref GIT_PORCELAIN: Regex = Regex::new(r"^[ MADRCU?!]{1,2}\s").unwrap();
}

pub fn apply(prog: &str, args: &[String], raw: &str) -> String {
    let base = prog.rsplit(['/', '\\']).next().unwrap_or(prog);
    match base {
        "git" => git(args, raw),
        "grep" | "rg" | "ag" => grep(raw),
        "cargo" | "go" | "make" | "npm" | "pnpm" | "yarn" | "gradle" | "mvn"
        | "tsc" | "eslint" | "pytest" | "jest" | "vitest" => build_or_test(raw),
        "ls" | "find" | "tree" => listing(raw),
        _ => generic(raw),
    }
}

fn strip_ansi(s: &str) -> String {
    ANSI.replace_all(s, "").into_owned()
}

/// Keep only the lines that carry signal for git subcommands.
fn git(args: &[String], raw: &str) -> String {
    let sub = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    let clean = strip_ansi(raw);
    match sub {
        Some("status") => {
            // prefer porcelain-style change lines; summarize if clean
            let changes: Vec<&str> = clean
                .lines()
                .filter(|l| GIT_PORCELAIN.is_match(l) || l.trim_start().starts_with("modified:")
                    || l.trim_start().starts_with("new file:")
                    || l.trim_start().starts_with("deleted:"))
                .collect();
            if changes.is_empty() {
                "clean — nothing to commit".into()
            } else {
                changes.join("\n")
            }
        }
        Some("log") => {
            // collapse each commit to a single line: hash + subject
            let mut out = Vec::new();
            let mut hash = "";
            for line in clean.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    hash = h.get(..8).unwrap_or(h);
                } else if !line.trim().is_empty()
                    && !line.starts_with("Author:")
                    && !line.starts_with("Date:")
                    && !hash.is_empty()
                {
                    out.push(format!("{hash} {}", line.trim()));
                    hash = "";
                }
            }
            if out.is_empty() {
                generic(&clean)
            } else {
                out.join("\n")
            }
        }
        Some("diff") | Some("show") => {
            // keep file headers and changed lines, drop context lines
            clean
                .lines()
                .filter(|l| {
                    l.starts_with("diff ")
                        || l.starts_with("+++")
                        || l.starts_with("---")
                        || l.starts_with("@@")
                        || l.starts_with('+')
                        || l.starts_with('-')
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => generic(&clean),
    }
}

/// grep/rg: drop blank lines, cap very long matches.
fn grep(raw: &str) -> String {
    strip_ansi(raw)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            if l.len() > 240 {
                format!("{}…", &l[..240])
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// build/test tools: surface errors, warnings, and result summaries only.
fn build_or_test(raw: &str) -> String {
    let clean = strip_ansi(raw);
    let mut keep: Vec<&str> = clean
        .lines()
        .filter(|l| ERRORLIKE.is_match(l) || TEST_RESULT.is_match(l))
        .collect();
    if keep.is_empty() {
        // success with no errors — return a one-line summary plus the tail
        let tail: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
        let n = tail.len();
        return tail
            .into_iter()
            .skip(n.saturating_sub(3))
            .collect::<Vec<_>>()
            .join("\n");
    }
    keep.dedup();
    keep.join("\n")
}

/// directory listings: drop permission/owner columns, keep names.
fn listing(raw: &str) -> String {
    strip_ansi(raw)
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("total "))
        .map(|l| l.split_whitespace().last().unwrap_or(l).to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn generic(raw: &str) -> String {
    // reuse the reversible-free squeeze transform (no ledger side effects here;
    // the engine handles stashing once for the whole command).
    squeeze::squeeze(raw, false).map(|s| s.text).unwrap_or_else(|_| raw.to_string())
}
