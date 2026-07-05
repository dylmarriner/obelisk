//! Model-agnostic context packing.
//!
//! `obelisk pack` builds a compact, token-budgeted context bundle before it is
//! handed to any model/provider. It deliberately avoids model-specific prompt
//! formats: budget in, compact context out. Provider-specific token counters can
//! sit around this later without infecting the core command surface.

use crate::ledger;
use crate::squeeze::{self, est_tokens};
use crate::symbols;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_SECTION_LIMIT: usize = 2_000;
const RESTORE_HINT_TOKENS: usize = 30;

#[derive(Debug, Clone)]
struct Section {
    title: String,
    priority: u8,
    content: String,
    reversible: bool,
}

impl Section {
    fn new(title: impl Into<String>, priority: u8, content: impl Into<String>) -> Self {
        Self { title: title.into(), priority, content: content.into(), reversible: true }
    }

    fn advisory(title: impl Into<String>, priority: u8, content: impl Into<String>) -> Self {
        Self { title: title.into(), priority, content: content.into(), reversible: false }
    }
}

pub fn run(
    budget: usize,
    system: &[String],
    history: &[String],
    files: &[String],
    dirs: &[String],
    diff: bool,
    tools: Option<&String>,
    out: Option<&String>,
) -> Result<i32> {
    let budget = budget.max(500);
    let mut sections = Vec::new();

    sections.push(Section::advisory(
        "packing policy",
        250,
        "Model-agnostic Obelisk context pack. Prefer outlines, diffs, errors, selected source, and restore handles over bulk context. Restore any omitted full content with `obelisk restore <handle>`.",
    ));

    for path in system {
        sections.push(Section::new(
            format!("system: {path}"),
            240,
            read_text(path).with_context(|| format!("read system file {path}"))?,
        ));
    }

    if diff {
        if let Some(d) = git_diff()? {
            sections.push(Section::new("git diff", 230, d));
        }
    }

    for path in history {
        let raw = read_text(path).with_context(|| format!("read history file {path}"))?;
        let squeezed = squeeze::squeeze(&raw, true)?.text;
        sections.push(Section::new(format!("history: {path}"), 170, squeezed));
    }

    for path in files {
        sections.extend(pack_file(Path::new(path), true)?);
    }

    for dir in dirs {
        sections.push(pack_dir(Path::new(dir))?);
    }

    if let Some(path) = tools {
        let raw = read_text(path).with_context(|| format!("read tools file {path}"))?;
        sections.push(Section::new(format!("tools: {path}"), 130, compact_tools(&raw)));
    }

    let packed = render_budgeted(sections, budget)?;

    if let Some(path) = out {
        std::fs::write(path, &packed).with_context(|| format!("write pack output {path}"))?;
    } else {
        print!("{packed}");
    }

    Ok(0)
}

fn render_budgeted(mut sections: Vec<Section>, budget: usize) -> Result<String> {
    sections.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.title.cmp(&b.title)));

    let mut body = String::new();
    body.push_str("# Obelisk Context Pack\n\n");
    body.push_str(&format!("Budget: ~{budget} tokens\n"));
    body.push_str("Counter: provider-neutral estimate. Use provider CountTokens outside this command when exact billing matters.\n\n");

    let mut remaining = budget.saturating_sub(est_tokens(&body));
    let mut included = 0usize;
    let mut omitted = Vec::new();

    for section in sections {
        let header = format!("## {}\n\n", section.title);
        let full = format!("{header}{}\n\n", section.content.trim());
        let need = est_tokens(&full);

        if need <= remaining {
            body.push_str(&full);
            remaining = remaining.saturating_sub(need);
            included += 1;
            continue;
        }

        if remaining > 120 {
            let allowance = remaining.saturating_sub(RESTORE_HINT_TOKENS).max(80);
            let mut clipped = approx_truncate(&section.content, allowance);
            if section.reversible && clipped != section.content {
                let handle = ledger::stash(&section.content, "pack", &section.title)?;
                clipped.push_str(&format!(
                    "\n\n[obelisk:restore {handle} — omitted full section via `obelisk restore {handle}`]"
                ));
            }
            let candidate = format!("{header}{}\n\n", clipped.trim());
            if est_tokens(&candidate) <= remaining.saturating_add(25) {
                body.push_str(&candidate);
                included += 1;
                remaining = remaining.saturating_sub(est_tokens(&candidate));
            } else {
                omitted.push(section.title);
            }
        } else {
            omitted.push(section.title);
        }
    }

    let used = est_tokens(&body);
    body.push_str("## pack stats\n\n");
    body.push_str(&format!("Estimated tokens: ~{used}\n"));
    body.push_str(&format!("Sections included: {included}\n"));
    if !omitted.is_empty() {
        body.push_str("Sections omitted due to budget:\n");
        for title in omitted {
            body.push_str(&format!("- {title}\n"));
        }
    }

    ledger::record_event("pack", "context pack", budget, used.min(budget))?;
    Ok(body)
}

fn approx_truncate(text: &str, token_budget: usize) -> String {
    if est_tokens(text) <= token_budget {
        return text.to_string();
    }

    let char_budget = token_budget.saturating_mul(4).max(200);
    let mut out = String::new();
    for ch in text.chars().take(char_budget) {
        out.push(ch);
    }
    out.push_str("\n… [obelisk: section truncated to fit token budget]");
    out
}

fn read_text(path: impl AsRef<Path>) -> Result<String> {
    std::fs::read_to_string(path.as_ref()).with_context(|| format!("read {}", path.as_ref().display()))
}

fn git_diff() -> Result<Option<String>> {
    let stat = run_git(&["diff", "--stat"])?;
    let names = run_git(&["diff", "--name-only"])?;
    let body = run_git(&["diff", "--unified=40"])?;

    if stat.trim().is_empty() && body.trim().is_empty() {
        return Ok(None);
    }

    let squeezed = squeeze::squeeze(&body, true)?.text;
    let mut out = String::new();
    if !stat.trim().is_empty() {
        out.push_str("### stat\n");
        out.push_str(stat.trim());
        out.push_str("\n\n");
    }
    if !names.trim().is_empty() {
        out.push_str("### changed files\n");
        out.push_str(names.trim());
        out.push_str("\n\n");
    }
    out.push_str("### patch\n");
    out.push_str(squeezed.trim());
    Ok(Some(out))
}

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output();
    match output {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        Ok(out) => Ok(String::from_utf8_lossy(&out.stderr).into_owned()),
        Err(_) => Ok(String::new()),
    }
}

fn pack_file(path: &Path, explicit: bool) -> Result<Vec<Section>> {
    let src = read_text(path)?;
    let tok = est_tokens(&src);
    let name = path.display().to_string();

    if tok <= DEFAULT_SECTION_LIMIT || !explicit {
        return Ok(vec![Section::new(format!("file: {name}"), 150, numbered(&src))]);
    }

    let mut sections = Vec::new();
    let outline = outline_for(path, &src).unwrap_or_else(|_| fallback_outline(&src));
    sections.push(Section::new(format!("outline: {name}"), 160, outline));

    let handle = ledger::stash(&src, "pack_file", &name)?;
    sections.push(Section::advisory(
        format!("restore: {name}"),
        90,
        format!("Full file is ~{tok} tokens and was not packed whole. Restore with `obelisk restore {handle}`."),
    ));

    Ok(sections)
}

fn pack_dir(path: &Path) -> Result<Section> {
    let mut files = Vec::new();
    walk_dir(path, &mut files, 0)?;
    files.sort();

    let mut out = String::new();
    out.push_str(&format!("{}\n", path.display()));
    for file in files.iter().take(300) {
        out.push_str(&format!("- {}\n", file.display()));
    }
    if files.len() > 300 {
        out.push_str(&format!("- … {} more files omitted\n", files.len() - 300));
    }

    Ok(Section::advisory(format!("directory map: {}", path.display()), 120, out))
}

fn walk_dir(path: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<()> {
    if depth > 6 || should_skip(path) {
        return Ok(());
    }
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if should_skip(&p) {
            continue;
        }
        if p.is_dir() {
            walk_dir(&p, out, depth + 1)?;
        } else if is_context_file(&p) {
            out.push(p);
        }
    }
    Ok(())
}

fn should_skip(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else { return false; };
    matches!(
        name,
        ".git" | "node_modules" | "target" | "dist" | "build" | ".next" | ".cache"
            | "coverage" | "vendor" | "__pycache__" | ".venv" | "venv"
    )
}

fn is_context_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
        Some(ext) => matches!(
            ext.as_str(),
            "rs" | "py" | "js" | "jsx" | "ts" | "tsx" | "go" | "java" | "c" | "h" | "cpp"
                | "hpp" | "cs" | "kt" | "swift" | "scala" | "php" | "rb" | "ex" | "exs" | "lua"
                | "toml" | "json" | "yaml" | "yml" | "md" | "sql" | "sh"
        ),
        None => false,
    }
}

fn outline_for(path: &Path, src: &str) -> Result<String> {
    let syms = symbols::parse(path, src)?;
    let mut out = String::new();
    out.push_str(&format!("{} ({} lines, {} symbols)\n", path.display(), src.lines().count(), syms.len()));
    for sym in syms {
        out.push_str(&format!("- {:<8} {:<32} L{}-{}\n", sym.kind, sym.name, sym.start, sym.end));
    }
    Ok(out)
}

fn fallback_outline(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = String::new();
    out.push_str(&format!("{} lines. Showing head/tail only.\n\n", lines.len()));
    for line in lines.iter().take(80) {
        out.push_str(line);
        out.push('\n');
    }
    if lines.len() > 120 {
        out.push_str("\n… [middle omitted]\n\n");
    }
    for line in lines.iter().skip(lines.len().saturating_sub(40)) {
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn numbered(src: &str) -> String {
    src.lines()
        .enumerate()
        .map(|(i, line)| format!("{:>5}: {line}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compact_tools(raw: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return squeeze::squeeze(raw, true).map(|r| r.text).unwrap_or_else(|_| raw.to_string());
    };

    if let Some(tools) = value.get("tools").and_then(|v| v.as_array()).or_else(|| value.as_array()) {
        let mut out = String::new();
        out.push_str(&format!("{} tools available. Compact schema view only.\n", tools.len()));
        for tool in tools.iter().take(80) {
            let name = find_str(tool, &["name", "tool.name", "function.name"]).unwrap_or("unnamed");
            let desc = find_str(tool, &["description", "tool.description", "function.description"]).unwrap_or("");
            out.push_str(&format!("- {name}"));
            if !desc.is_empty() {
                out.push_str(&format!(": {}", cap(desc, 180)));
            }
            out.push('\n');
        }
        if tools.len() > 80 {
            out.push_str(&format!("- … {} more tools omitted\n", tools.len() - 80));
        }
        return out;
    }

    let compact = serde_json::to_string(&value).unwrap_or_else(|_| raw.to_string());
    approx_truncate(&compact, 1_000)
}

fn find_str<'a>(value: &'a Value, paths: &[&str]) -> Option<&'a str> {
    for path in paths {
        let mut cur = value;
        let mut found = true;
        for part in path.split('.') {
            match cur.get(part) {
                Some(next) => cur = next,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            if let Some(s) = cur.as_str() {
                return Some(s);
            }
        }
    }
    None
}

fn cap(text: &str, chars: usize) -> String {
    if text.chars().count() <= chars {
        text.to_string()
    } else {
        let mut out = text.chars().take(chars).collect::<String>();
        out.push('…');
        out
    }
}
