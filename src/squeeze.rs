//! Reversible boilerplate squeeze: strip ANSI, progress bars, opaque blobs,
//! and runs of identical lines; collapse blank-line runs. Used directly by the
//! `squeeze` command and as the generic fallback inside the engine.

use crate::ledger;
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::Result;

lazy_static! {
    static ref ANSI: Regex = Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap();
    static ref PROGRESS: Regex =
        Regex::new(r"(?m)^\s*[\d.]+%.*$|^[\s#=>\-]*\[[#=>\-. ]+\]\s*$").unwrap();
    static ref BLOB: Regex = Regex::new(r"[A-Za-z0-9+/]{120,}={0,2}").unwrap();
    static ref BLANKRUN: Regex = Regex::new(r"\n{3,}").unwrap();
    static ref TRAILWS: Regex = Regex::new(r"[ \t]+\n").unwrap();
}

pub fn est_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let ws = text.chars().filter(|c| c.is_whitespace()).count() as f64;
    (((text.len() as f64 - ws * 0.5) / 4.0).max(1.0)) as usize
}

fn dedupe_runs(text: &str, threshold: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let mut j = i;
        while j < lines.len() && lines[j] == lines[i] {
            j += 1;
        }
        let run = j - i;
        if run >= threshold {
            out.push(lines[i].to_string());
            out.push(format!("  … [obelisk: ×{run} identical lines collapsed]"));
        } else {
            out.extend(lines[i..j].iter().map(|s| s.to_string()));
        }
        i = j;
    }
    out.join("\n")
}

pub struct Squeezed {
    pub text: String,
    pub before: usize,
    pub after: usize,
}

pub fn squeeze(input: &str, reversible: bool) -> Result<Squeezed> {
    let before = est_tokens(input);
    let mut work = ANSI.replace_all(input, "").into_owned();
    work = PROGRESS.replace_all(&work, "").into_owned();
    work = BLOB
        .replace_all(&work, |c: &regex::Captures| format!("[obelisk:blob {}b]", c[0].len()))
        .into_owned();
    work = dedupe_runs(&work, 4);
    work = TRAILWS.replace_all(&work, "\n").into_owned();
    work = BLANKRUN.replace_all(&work, "\n\n").into_owned();
    let work = work.trim_matches('\n').to_string();

    if est_tokens(&work) >= before {
        let _ = ledger::record_event("squeeze", "squeeze", before, before);
        return Ok(Squeezed { text: input.to_string(), before, after: before });
    }

    let mut out = work;
    if reversible && out != input {
        let h = ledger::stash(input, "squeeze", "boilerplate")?;
        out.push_str(&format!(
            "\n\n[obelisk:restore {h} — full original via `obelisk restore {h}`]"
        ));
    }
    let after = est_tokens(&out);
    let _ = ledger::record_event("squeeze", "squeeze", before, after.min(before));
    Ok(Squeezed { text: out, before, after })
}
