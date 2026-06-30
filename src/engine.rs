//! Command-output compression engine.
//!
//! `obelisk run <cmd…>` executes a command once, then compresses its output
//! before it reaches the model. A program-aware filter is chosen by the
//! command name (git, grep, build tools, …); anything without a dedicated
//! filter falls back to the generic boilerplate squeeze. The full original is
//! always stashed in the ledger and a restore pointer is appended.

use crate::filters;
use crate::ledger;
use crate::squeeze::est_tokens;
use anyhow::{anyhow, Result};
use std::process::Command;

pub fn run(cmd: &[String]) -> Result<i32> {
    let prog = cmd.first().ok_or_else(|| anyhow!("run: no command given"))?;
    let args = &cmd[1..];

    let out = Command::new(prog).args(args).output()?;
    let mut raw = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.trim().is_empty() {
        if !raw.is_empty() {
            raw.push('\n');
        }
        raw.push_str(&stderr);
    }
    let rc = out.status.code().unwrap_or(0);

    let before = est_tokens(&raw);
    let compressed = filters::apply(prog, args, &raw);
    let after_compressed = est_tokens(&compressed);

    // Only stash + append a restore pointer when it actually pays off: the
    // compression must save a meaningful amount, and the pointer (~20 tokens)
    // must not cost more than it saves. Otherwise emit the compressed output
    // clean, so we never make small output bigger.
    let saved = before.saturating_sub(after_compressed);
    let worth_restoring = compressed != raw && before > 60 && saved > 25;

    let (body, after) = if worth_restoring {
        let handle = ledger::stash(&raw, "run", &cmd.join(" "))?;
        let b = format!(
            "{compressed}\n[obelisk:restore {handle} — raw via `obelisk restore {handle}`]"
        );
        let a = est_tokens(&b);
        (b, a)
    } else {
        (compressed, after_compressed)
    };

    ledger::record_event("run", &cmd.join(" "), before, after.min(before))?;

    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    Ok(rc)
}
