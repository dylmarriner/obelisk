//! Context markers — compact, named, resumable summaries. Instead of reloading
//! a large slice of prior context to resume a task, save a small marker of the
//! decisions/state and reload just that.
//!
//!   obelisk marker save <name>   reads the summary from stdin
//!   obelisk marker list
//!   obelisk marker show <name>
//!   obelisk marker rm <name>

use crate::ledger;
use crate::squeeze::est_tokens;
use anyhow::{anyhow, Result};
use std::io::Read;

fn age(secs: i64) -> String {
    if secs < 90 {
        format!("{secs}s")
    } else if secs < 5400 {
        format!("{}m", secs / 60)
    } else if secs < 172800 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

pub fn run(args: &[String]) -> Result<i32> {
    match args.first().map(|s| s.as_str()).unwrap_or("list") {
        "save" | "set" => {
            let name = args.get(1).ok_or_else(|| anyhow!("marker save <name>"))?;
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            let content = content.trim_end().to_string();
            if content.is_empty() {
                return Err(anyhow!("marker save: empty stdin"));
            }
            ledger::marker_save(name, &content)?;
            eprintln!("[obelisk] marker '{name}' saved ({} tok)", est_tokens(&content));
            Ok(0)
        }
        "show" | "get" => {
            let name = args.get(1).ok_or_else(|| anyhow!("marker show <name>"))?;
            match ledger::marker_get(name)? {
                Some(c) => {
                    print!("{c}");
                    if !c.ends_with('\n') {
                        println!();
                    }
                    Ok(0)
                }
                None => {
                    eprintln!("obelisk: no marker named '{name}'");
                    Ok(1)
                }
            }
        }
        "list" | "ls" => {
            let markers = ledger::marker_list()?;
            if markers.is_empty() {
                println!("no markers — save one with: <summary> | obelisk marker save <name>");
                return Ok(0);
            }
            println!("{:<24} {:>8} {:>6}", "MARKER", "TOKENS", "AGE");
            for (name, bytes, a) in markers {
                println!("{:<24} {:>8} {:>6}", name, (bytes as f64 / 4.0).max(1.0) as i64, age(a));
            }
            Ok(0)
        }
        "rm" | "delete" | "del" => {
            let name = args.get(1).ok_or_else(|| anyhow!("marker rm <name>"))?;
            if ledger::marker_delete(name)? {
                eprintln!("[obelisk] marker '{name}' removed");
                Ok(0)
            } else {
                eprintln!("obelisk: no marker named '{name}'");
                Ok(1)
            }
        }
        other => Err(anyhow!("unknown marker subcommand '{other}' (save|show|list|rm)")),
    }
}
