//! Agent hook processors and the rewrite rule they all share.
//!
//! Every integration — Claude Code's `PreToolUse` hook, Codex's `PreToolUse`
//! hook, Hermes's `pre_tool_call` plugin, anything added later — ultimately
//! asks the same yes/no question: "does this command benefit from being
//! routed through `obelisk run`?" `rewrite()` is the single place that
//! answers it. Hook processors are thin adapters from an agent's payload
//! shape to that one function; they must not duplicate the eligibility logic.

use anyhow::Result;
use std::io::Read;

/// Commands worth wrapping (read-heavy, output-heavy, idempotent). We avoid
/// rewriting anything interactive, mutating, or piped/redirected, since those
/// shouldn't have their output transformed.
fn eligible(cmd: &str) -> bool {
    let c = cmd.trim();
    if c.is_empty()
        || c.contains('|')
        || c.contains('>')
        || c.contains('<')
        || c.contains("&&")
        || c.contains(';')
        || c.starts_with("obelisk ")
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

/// The one rewrite rule. `None` means "leave the command alone" — not
/// eligible, already wrapped, or otherwise out of scope.
pub fn rewrite(cmd: &str) -> Option<String> {
    if eligible(cmd) {
        Some(format!("obelisk run {cmd}"))
    } else {
        None
    }
}

fn claude_codex_response(input: &serde_json::Value, rewritten: &str) -> serde_json::Value {
    let mut updated_input = input
        .as_object()
        .cloned()
        .map(serde_json::Value::Object)
        .unwrap_or_else(|| serde_json::json!({}));

    if let serde_json::Value::Object(ref mut obj) = updated_input {
        obj.insert("command".to_string(), serde_json::Value::String(rewritten.to_string()));
    }

    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": "Obelisk output compression",
            "updatedInput": updated_input
        }
    })
}

pub fn claude() -> Result<i32> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let v: serde_json::Value = serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null);
    let tool = v.get("tool_name").and_then(|t| t.as_str()).unwrap_or("");
    let input = v.get("tool_input").cloned().unwrap_or(serde_json::Value::Null);
    let cmd = input.get("command").and_then(|c| c.as_str()).unwrap_or("");

    if tool == "Bash" {
        if let Some(rewritten) = rewrite(cmd) {
            println!("{}", claude_codex_response(&input, &rewritten));
        }
    }
    Ok(0)
}

/// Codex CLI's hooks.json uses the same PreToolUse contract as Claude Code
/// (matcher/hooks/command, hookSpecificOutput.updatedInput on stdout), but its
/// shell tool's name and argument shape vary by version, so we accept a few.
pub fn codex() -> Result<i32> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let v: serde_json::Value = serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null);
    let tool = v.get("tool_name").and_then(|t| t.as_str()).unwrap_or("");
    if !matches!(tool, "Bash" | "shell" | "local_shell" | "exec_command") {
        return Ok(0);
    }

    let input = v.get("tool_input").cloned().unwrap_or(serde_json::Value::Null);
    let cmd = if let Some(s) = input.get("command").and_then(|c| c.as_str()) {
        s.to_string()
    } else if let Some(arr) = input.get("command").and_then(|c| c.as_array()) {
        arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" ")
    } else if let Some(arr) = input.get("argv").and_then(|c| c.as_array()) {
        arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" ")
    } else {
        String::new()
    };

    if let Some(rewritten) = rewrite(&cmd) {
        println!("{}", claude_codex_response(&input, &rewritten));
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_noisy_read_only_commands() {
        assert_eq!(rewrite("cargo build"), Some("obelisk run cargo build".to_string()));
        assert_eq!(rewrite("git status"), Some("obelisk run git status".to_string()));
    }

    #[test]
    fn refuses_mutating_or_complex_commands() {
        assert_eq!(rewrite("git push"), None);
        assert_eq!(rewrite("cargo build | tee build.log"), None);
        assert_eq!(rewrite("obelisk run git status"), None);
    }

    #[test]
    fn pre_tool_response_preserves_existing_bash_fields() {
        let input = serde_json::json!({
            "command": "cargo build",
            "description": "Build the project",
            "timeout": 120000,
            "run_in_background": false
        });
        let response = claude_codex_response(&input, "obelisk run cargo build");
        let updated = response
            .pointer("/hookSpecificOutput/updatedInput")
            .expect("updated input");

        assert_eq!(updated.get("command").and_then(|v| v.as_str()), Some("obelisk run cargo build"));
        assert_eq!(updated.get("description").and_then(|v| v.as_str()), Some("Build the project"));
        assert_eq!(updated.get("timeout").and_then(|v| v.as_i64()), Some(120000));
        assert_eq!(
            response.pointer("/hookSpecificOutput/permissionDecision").and_then(|v| v.as_str()),
            Some("allow")
        );
    }
}
