//! Wire Obelisk into an AI coding agent. Each agent calls `obelisk hook <agent>`
//! on tool use; we patch the agent's config to add that hook. Idempotent and
//! reversible (configs are backed up before patching).

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn self_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "obelisk".into())
}

pub fn run(agent: &str) -> Result<i32> {
    match agent {
        "claude" => claude(),
        "hermes" => hermes(),
        "opencode" => opencode(),
        "openclaw" => openclaw(),
        other => Err(anyhow!("unknown agent '{other}' (claude|hermes|opencode|openclaw)")),
    }
}

fn backup_and_write(path: &PathBuf, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if path.exists() {
        let _ = std::fs::copy(path, path.with_extension("bak"));
    }
    std::fs::write(path, content)?;
    Ok(())
}

fn claude() -> Result<i32> {
    let settings = home().join(".claude").join("settings.json");
    let mut root: serde_json::Value = if settings.exists() {
        serde_json::from_str(&std::fs::read_to_string(&settings)?).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let entry = serde_json::json!({
        "matcher": "Bash",
        "hooks": [ { "type": "command", "command": "obelisk hook claude" } ]
    });

    let hooks = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("settings.json is not an object"))?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    let pre = hooks
        .as_object_mut()
        .unwrap()
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]));
    let arr = pre.as_array_mut().ok_or_else(|| anyhow!("PreToolUse is not an array"))?;

    let already = arr.iter().any(|e| {
        e.get("hooks")
            .and_then(|h| h.as_array())
            .map(|h| h.iter().any(|x| x.get("command").and_then(|c| c.as_str()) == Some("obelisk hook claude")))
            .unwrap_or(false)
    });
    if !already {
        arr.push(entry);
    }

    backup_and_write(&settings, &serde_json::to_string_pretty(&root)?)
        .context("write ~/.claude/settings.json")?;
    println!("✓ Claude Code: PreToolUse hook installed -> {}", settings.display());
    println!("  backup: {}", settings.with_extension("bak").display());
    println!("  restart Claude Code, then run: git status");
    Ok(0)
}

fn hermes() -> Result<i32> {
    // Hermes reads a rules/awareness file; drop one that tells it to route
    // output-heavy commands through `obelisk run`.
    let rules = home().join(".hermes").join("OBELISK.md");
    backup_and_write(&rules, &awareness())?;
    println!("✓ Hermes: awareness rule installed -> {}", rules.display());
    Ok(0)
}

fn opencode() -> Result<i32> {
    let plugin = home().join(".config").join("opencode").join("plugins").join("obelisk.ts");
    backup_and_write(&plugin, OPENCODE_PLUGIN)?;
    println!("✓ OpenCode: plugin installed -> {}", plugin.display());
    println!("  restart OpenCode.");
    Ok(0)
}

fn openclaw() -> Result<i32> {
    let rules = home().join(".openclaw").join("OBELISK.md");
    backup_and_write(&rules, &awareness())?;
    println!("✓ OpenClaw: awareness rule installed -> {}", rules.display());
    Ok(0)
}

fn awareness() -> String {
    "# Obelisk\n\nRoute output-heavy, read-only shell commands through `obelisk run` so their\noutput is compressed before entering context. Examples:\n\n- `obelisk run git status`\n- `obelisk run cargo build`\n- `obelisk run grep -rn TODO src`\n\nFetch a single function instead of a whole file with `obelisk symbol <file> <name>`.\nCompressed output ends with a `[obelisk:restore <handle>]` pointer — run\n`obelisk restore <handle>` if you need the full original.\n".into()
}

const OPENCODE_PLUGIN: &str = r#"// Obelisk plugin for OpenCode — routes shell output through `obelisk run`.
export const obelisk = async ({ $ }) => ({
  "tool.execute.before": async (input, output) => {
    if (input.tool !== "bash") return;
    const cmd = (output.args?.command ?? "").trim();
    const prog = cmd.split(/\s+/)[0];
    const ok = ["git","grep","rg","ls","find","cargo","go","npm","pnpm","make","pytest","jest","vitest"];
    if (ok.includes(prog) && !/[|<>;]|&&/.test(cmd) && !cmd.startsWith("obelisk ")) {
      output.args.command = `obelisk run ${cmd}`;
    }
  },
});
"#;

pub fn doctor() -> Result<i32> {
    println!("obelisk {}", env!("CARGO_PKG_VERSION"));
    println!("binary    : {}", self_path());
    let h = crate::ledger::stash("doctor-probe", "doctor", "")?;
    let ok = crate::ledger::restore(&h)? == Some("doctor-probe".into());
    println!("ledger    : {}", if ok { "OK (reversible)" } else { "FAIL" });
    let (b, c, m) = crate::ledger::store_counts().unwrap_or((0, 0, 0));
    println!("store     : {b} blobs · {c} checkpoints · {m} markers");
    println!("agents    : install with `obelisk install <claude|hermes|opencode|openclaw>`");
    Ok(if ok { 0 } else { 1 })
}
