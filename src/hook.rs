//! Agent hook processors. A coding agent calls `obelisk hook <agent>` on each
//! tool use, passing the tool payload as JSON on stdin; we rewrite eligible
//! shell commands to route their output through `obelisk run`, which compresses
//! it before it lands in the model's context.

use anyhow::Result;
use std::io::Read;

/// Commands worth wrapping (read-heavy, output-heavy, idempotent). We avoid
/// rewriting anything interactive, mutating, or piped/redirected, since those
/// shouldn't have their output transformed.
fn eligible(cmd: &str) -> bool {
    let c = cmd.trim();
    if c.is_empty() || c.contains('|') || c.contains('>') || c.contains('<')
        || c.contains("&&") || c.contains(';') || c.starts_with("obelisk ")
    {
        return false;
    }
    let prog = c.split_whitespace().next().unwrap_or("");
    matches!(
        prog,
        "git" | "grep" | "rg" | "ls" | "find" | "tree" | "cargo" | "go" | "make"
            | "npm" | "pnpm" | "yarn" | "gradle" | "mvn" | "tsc" | "eslint"
            | "pytest" | "jest" | "vitest" | "cat" | "du" | "df" | "ps"
    ) && !(prog == "git" && is_mutating_git(c))
}

fn is_mutating_git(c: &str) -> bool {
    ["push", "pull", "commit", "merge", "rebase", "reset", "checkout", "clean", "fetch", "stash"]
        .iter()
        .any(|m| c.contains(&format!("git {m}")) || c.contains(&format!(" {m} ")))
}

pub fn claude() -> Result<i32> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let v: serde_json::Value = serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null);
    let tool = v.get("tool_name").and_then(|t| t.as_str()).unwrap_or("");
    let cmd = v
        .get("tool_input")
        .and_then(|i| i.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    if tool == "Bash" && eligible(cmd) {
        let rewritten = format!("obelisk run {cmd}");
        let out = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecisionReason": "Obelisk output compression",
                "updatedInput": { "command": rewritten }
            }
        });
        println!("{out}");
    }
    Ok(0)
}
