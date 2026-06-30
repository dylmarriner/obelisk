//! Unified savings dashboard — every layer's token accounting in one view,
//! read from the shared ledger.

use crate::ledger;
use anyhow::Result;
use colored::Colorize;

const USD_PER_MTOK: f64 = 3.0;

fn bar(pct: f64, w: usize) -> String {
    let n = (((pct / 100.0) * w as f64).round() as usize).min(w);
    format!("{}{}", "█".repeat(n), "░".repeat(w - n))
}

fn label(layer: &str) -> String {
    match layer {
        "run" => "command output",
        "symbol" => "symbol retrieval",
        "squeeze" => "boilerplate squeeze",
        "proxy" => "proxy (volume)",
        "terse" => "agent verbosity",
        other => other,
    }
    .to_string()
}

fn rating(pct: f64) -> colored::ColoredString {
    if pct >= 85.0 {
        "★ EXCELLENT (>85%)".green().bold()
    } else if pct >= 70.0 {
        "✓ on target (>70%)".green()
    } else if pct >= 40.0 {
        "• fair (40-70%)".yellow()
    } else {
        "↓ below target (<40%)".red()
    }
}

pub fn run() -> Result<i32> {
    let rows = ledger::event_rollup()?;
    let before: i64 = rows.iter().map(|r| r.1).sum();
    let after: i64 = rows.iter().map(|r| r.2).sum();
    let saved = before - after;
    let pct = if before > 0 { saved as f64 / before as f64 * 100.0 } else { 0.0 };
    let usd = saved as f64 / 1_000_000.0 * USD_PER_MTOK;
    let (blobs, cps, markers) = ledger::store_counts().unwrap_or((0, 0, 0));

    println!("{}", "┌─ OBELISK · token savings ───────────────────────────────".cyan());
    println!("│ {:>12} → {:>12} tokens", format!("{before}"), format!("{after}"));
    println!("│ saved {} tokens  ({})  ≈ {}",
        format!("{saved:>12}").green().bold(),
        format!("{pct:.1}%").green(),
        format!("${usd:.2}").green().bold());
    println!("│");
    if rows.is_empty() {
        println!("│ no activity yet — try `obelisk run git status` or `obelisk symbol <f> <name>`");
    } else {
        println!("│ per layer:");
        for (layer, b, a, n) in &rows {
            let p = if *b > 0 { (b - a) as f64 / *b as f64 * 100.0 } else { 0.0 };
            println!("│  {:<22} {} {:>5.1}%  (-{} tok, n={})", label(layer), bar(p, 20), p, b - a, n);
        }
    }
    println!("│");
    println!("│ savings rate: {}  {}", format!("{pct:.1}%").bold(), rating(pct));
    println!("│ reversible store: {blobs} originals · {cps} checkpoints · {markers} markers");
    println!("│ recover anything with `obelisk restore <handle>`");
    println!("{}", "└─────────────────────────────────────────────────────────".cyan());
    Ok(0)
}
