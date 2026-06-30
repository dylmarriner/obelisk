//! Symbol-level code retrieval — the single biggest retrieval win: hand the
//! agent the one function/class it asked for instead of a whole file.
//!
//!   obelisk outline <file>        list symbols with line ranges
//!   obelisk symbol  <file> <name> print just that symbol's source
//!
//! Dependency-free heuristic parsing: regex-detected declarations plus brace
//! or indentation block resolution. Covers Rust, Python, JS/TS, Go, C/C++,
//! Java, C#.

use crate::ledger;
use crate::squeeze::est_tokens;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

pub struct Symbol {
    pub kind: String,
    pub name: String,
    pub start: usize,
    pub end: usize,
}

enum Block {
    Brace,
    Indent,
}

lazy_static! {
    static ref RUST: Vec<(&'static str, Regex)> = vec![
        ("fn", Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)").unwrap()),
        ("struct", Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z0-9_]+)").unwrap()),
        ("enum", Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z0-9_]+)").unwrap()),
        ("trait", Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+([A-Za-z0-9_]+)").unwrap()),
    ];
    static ref PY: Vec<(&'static str, Regex)> = vec![
        ("def", Regex::new(r"^\s*(?:async\s+)?def\s+([A-Za-z0-9_]+)").unwrap()),
        ("class", Regex::new(r"^\s*class\s+([A-Za-z0-9_]+)").unwrap()),
    ];
    static ref JS: Vec<(&'static str, Regex)> = vec![
        ("function", Regex::new(r"^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s*\*?\s*([A-Za-z0-9_$]+)").unwrap()),
        ("class", Regex::new(r"^\s*(?:export\s+)?(?:default\s+)?class\s+([A-Za-z0-9_$]+)").unwrap()),
        ("const", Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+([A-Za-z0-9_$]+)\s*=\s*(?:async\s+)?(?:\([^)]*\)|[A-Za-z0-9_$]+)\s*=>").unwrap()),
    ];
    static ref GO: Vec<(&'static str, Regex)> = vec![
        ("func", Regex::new(r"^\s*func\s+(?:\([^)]*\)\s*)?([A-Za-z0-9_]+)").unwrap()),
        ("type", Regex::new(r"^\s*type\s+([A-Za-z0-9_]+)").unwrap()),
    ];
    static ref CLIKE: Vec<(&'static str, Regex)> = vec![
        ("class", Regex::new(r"^\s*(?:public\s+|private\s+|protected\s+|static\s+|final\s+|abstract\s+)*(?:class|interface|enum)\s+([A-Za-z0-9_]+)").unwrap()),
        ("method", Regex::new(r"^\s*(?:public\s+|private\s+|protected\s+|static\s+|final\s+|virtual\s+|inline\s+)*[A-Za-z_][A-Za-z0-9_:<>,\*&\s]*\s+([A-Za-z0-9_]+)\s*\([^;]*\)\s*\{?\s*$").unwrap()),
    ];
}

fn detect(path: &Path) -> Option<(&'static Vec<(&'static str, Regex)>, Block)> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "rs" => Some((&RUST, Block::Brace)),
        "py" | "pyi" => Some((&PY, Block::Indent)),
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => Some((&JS, Block::Brace)),
        "go" => Some((&GO, Block::Brace)),
        "c" | "h" | "cc" | "cpp" | "hpp" | "cxx" | "java" | "cs" => Some((&CLIKE, Block::Brace)),
        _ => None,
    }
}

fn end_brace(lines: &[&str], start: usize) -> usize {
    let mut depth = 0i64;
    let mut seen = false;
    for (i, raw) in lines.iter().enumerate().skip(start) {
        let line = raw.split("//").next().unwrap_or(raw);
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                seen = true;
            } else if ch == '}' {
                depth -= 1;
                if seen && depth == 0 {
                    return i;
                }
            }
        }
        if !seen && line.trim_end().ends_with(';') {
            return i;
        }
    }
    lines.len() - 1
}

fn indent(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

fn end_indent(lines: &[&str], start: usize) -> usize {
    let base = indent(lines[start]);
    let mut last = start;
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        if line.trim().is_empty() {
            continue;
        }
        if indent(line) <= base {
            return last;
        }
        last = i;
    }
    lines.len() - 1
}

pub fn parse(path: &Path, src: &str) -> Result<Vec<Symbol>> {
    let (defs, style) = detect(path)
        .ok_or_else(|| anyhow!("unsupported file type: {}", path.display()))?;
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for i in 0..lines.len() {
        for (kind, re) in defs.iter() {
            if let Some(c) = re.captures(lines[i]) {
                let name = c.get(1).map(|m| m.as_str()).unwrap_or("");
                if name.is_empty() {
                    continue;
                }
                let end = match style {
                    Block::Brace => end_brace(&lines, i),
                    Block::Indent => end_indent(&lines, i),
                };
                out.push(Symbol { kind: kind.to_string(), name: name.into(), start: i + 1, end: end + 1 });
                break;
            }
        }
    }
    Ok(out)
}

pub fn outline(file: &str) -> Result<i32> {
    let path = Path::new(file);
    let src = std::fs::read_to_string(path)?;
    let syms = parse(path, &src)?;
    if syms.is_empty() {
        println!("no symbols found in {file}");
        return Ok(0);
    }
    println!("{file}  ({} lines, {} symbols)", src.lines().count(), syms.len());
    for s in &syms {
        println!("  {:<8} {:<32} L{}-{}", s.kind, s.name, s.start, s.end);
    }
    Ok(0)
}

pub fn symbol(file: &str, name: &str) -> Result<i32> {
    let path = Path::new(file);
    let src = std::fs::read_to_string(path)?;
    let syms = parse(path, &src)?;
    let Some(sym) = syms.iter().find(|s| s.name == name) else {
        eprintln!("obelisk: symbol '{name}' not found in {file}");
        let names: Vec<_> = syms.iter().map(|s| s.name.as_str()).collect();
        if !names.is_empty() {
            eprintln!("  available: {}", names.join(", "));
        }
        return Ok(1);
    };
    let lines: Vec<&str> = src.lines().collect();
    let slice = lines[sym.start - 1..sym.end].join("\n");
    let file_tok = est_tokens(&src);
    let sym_tok = est_tokens(&slice);
    let _ = ledger::record_event("symbol", &format!("{file}#{name}"), file_tok, sym_tok);
    println!("// {file}:{}-{} ({} {})", sym.start, sym.end, sym.kind, sym.name);
    println!("{slice}");
    eprintln!(
        "[obelisk] symbol retrieval: {file_tok} -> {sym_tok} tok ({:.0}% vs full file)",
        if file_tok > 0 { (file_tok - sym_tok) as f64 / file_tok as f64 * 100.0 } else { 0.0 }
    );
    Ok(0)
}
