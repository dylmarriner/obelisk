//! Obelisk — a token-optimizing engine for AI coding agents.
//!
//! One binary, several reversible layers, one shared ledger:
//!   run      compress a command's output before it enters context
//!   squeeze  collapse boilerplate from any text stream
//!   terse    terse-ify prose (code left intact)
//!   outline/symbol   structural code retrieval — fetch a symbol, not a file
//!   pack     build a model-agnostic, token-budgeted context bundle
//!   marker/checkpoint/restore   save & reload context compactly
//!   serve    a local optimization proxy for the model API
//!   stats    one savings dashboard across every layer
//!
//! Everything compressible is stashed first, so any output is recoverable
//! with `obelisk restore <handle>` — minimize tokens, lose no context.

mod dashboard;
mod engine;
mod filters;
mod hook;
mod install;
mod learn;
mod ledger;
mod marker;
mod pack;
mod proxy;
mod squeeze;
mod symbols;
mod terse;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "obelisk",
    version,
    about = "Token-optimizing engine for AI coding agents — minimize tokens, lose no context"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a command and emit a compressed, reversible view of its output.
    #[command(trailing_var_arg = true)]
    Run {
        #[arg(allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    /// Reversibly squeeze boilerplate (ANSI/dup-lines/blobs) from stdin.
    Squeeze,
    /// Terse-ify prose from stdin (off|lite|full|ultra). Code blocks untouched.
    Terse {
        #[arg(default_value = "lite")]
        level: String,
    },
    /// List a source file's symbols with line ranges — structure without content.
    Outline { file: String },
    /// Extract just one symbol's source (fetch the function, not the whole file).
    Symbol { file: String, name: String },
    /// Build a provider-neutral, token-budgeted context bundle.
    Pack {
        /// Approximate token budget for the whole packed context.
        #[arg(long, default_value_t = 12_000)]
        budget: usize,
        /// Stable system/instruction file to include. Repeatable.
        #[arg(long)]
        system: Vec<String>,
        /// Chat/session history file to squeeze into compact state. Repeatable.
        #[arg(long)]
        history: Vec<String>,
        /// Explicit file to include or outline. Repeatable.
        #[arg(long = "file")]
        file: Vec<String>,
        /// Directory to map without reading every file. Repeatable.
        #[arg(long = "dir")]
        dir: Vec<String>,
        /// Include current git diff/stat/name-only if available.
        #[arg(long)]
        diff: bool,
        /// Tool schema JSON to compact into a names/descriptions view.
        #[arg(long)]
        tools: Option<String>,
        /// Write packed context to a file instead of stdout.
        #[arg(long)]
        out: Option<String>,
    },
    /// Context markers: compact named summaries to resume work without reloading.
    #[command(trailing_var_arg = true)]
    Marker {
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Snapshot session state from stdin; survives compaction. Prints a handle.
    Checkpoint {
        #[arg(default_value = "session")]
        label: String,
    },
    /// Restore a stashed original or checkpoint by handle.
    Restore { handle: String },
    /// Run the local optimization proxy: plain HTTP in, HTTPS out, token-accounted.
    Serve {
        #[arg(long, default_value_t = 6767)]
        port: u16,
        #[arg(long, default_value = "https://api.anthropic.com")]
        upstream: String,
    },
    /// Unified savings dashboard across every layer.
    Stats,
    /// Evict reversible blobs older than N days (checkpoints/markers kept).
    Gc {
        #[arg(default_value_t = 14)]
        days: i64,
    },
    /// Wire Obelisk into an AI coding agent (claude|hermes|opencode|openclaw|codex|cline).
    Install { agent: String },
    /// Hook processor invoked by an agent on tool use (reads JSON from stdin).
    Hook { agent: String },
    /// Rewrite a raw command to its obelisk-wrapped form, if eligible. Prints
    /// the rewritten command and exits 0, or prints nothing and exits 1 if
    /// the command should be left alone. Single source of truth for plugins
    /// that don't speak Claude/Codex's PreToolUse JSON (e.g. Hermes).
    #[command(trailing_var_arg = true)]
    Rewrite {
        #[arg(allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    /// Verify the install is wired correctly.
    Doctor,
    /// Usage-triggered self-improvement: log gaps, enable/disable the loop.
    Learn {
        #[command(subcommand)]
        action: LearnAction,
    },
}

#[derive(Subcommand)]
enum LearnAction {
    /// Enable the self-improve loop for a repo (must contain scripts/self-improve.sh).
    Enable {
        repo_path: String,
        #[arg(long)]
        threshold: Option<i64>,
    },
    /// Disable the self-improve loop.
    Disable,
    /// Show whether learning is enabled and how many gaps are pending.
    Status,
    /// Dump pending gaps as JSON (consumed by scripts/self-improve.sh).
    Gaps,
}

fn main() {
    let cli = Cli::parse();
    let code = match run(cli) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("obelisk: {e:#}");
            1
        }
    };
    std::process::exit(code);
}

fn read_stdin() -> String {
    use std::io::Read;
    let mut s = String::new();
    let _ = std::io::stdin().read_to_string(&mut s);
    s
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    match cli.command {
        Commands::Run { cmd } => engine::run(&cmd),
        Commands::Squeeze => {
            let input = read_stdin();
            let r = squeeze::squeeze(&input, true)?;
            print!("{}", r.text);
            let pct = if r.before > 0 {
                (r.before - r.after) as f64 / r.before as f64 * 100.0
            } else {
                0.0
            };
            eprintln!("\n[obelisk] {} -> {} tok ({pct:.1}% saved)", r.before, r.after);
            Ok(0)
        }
        Commands::Terse { level } => {
            print!("{}", terse::terse(&read_stdin(), &level));
            Ok(0)
        }
        Commands::Outline { file } => symbols::outline(&file),
        Commands::Symbol { file, name } => symbols::symbol(&file, &name),
        Commands::Pack { budget, system, history, file, dir, diff, tools, out } => {
            pack::run(budget, &system, &history, &file, &dir, diff, tools.as_ref(), out.as_ref())
        }
        Commands::Marker { args } => marker::run(&args),
        Commands::Checkpoint { label } => {
            let h = ledger::checkpoint(&read_stdin(), &label)?;
            println!("{h}");
            Ok(0)
        }
        Commands::Restore { handle } => match ledger::restore(&handle)? {
            Some(o) => {
                print!("{o}");
                Ok(0)
            }
            None => {
                eprintln!("obelisk: no blob/checkpoint for handle {handle}");
                let _ = ledger::record_gap("restore_miss", &handle, "");
                learn::maybe_trigger();
                Ok(1)
            }
        },
        Commands::Serve { port, upstream } => proxy::serve(port, &upstream),
        Commands::Stats => dashboard::run(),
        Commands::Gc { days } => {
            let n = ledger::gc(days)?;
            println!("evicted {n} blobs");
            Ok(0)
        }
        Commands::Install { agent } => install::run(&agent),
        Commands::Rewrite { cmd } => match hook::rewrite(&cmd.join(" ")) {
            Some(rewritten) => {
                println!("{rewritten}");
                Ok(0)
            }
            None => Ok(1),
        },
        Commands::Hook { agent } => match agent.as_str() {
            "claude" => hook::claude(),
            "codex" => hook::codex(),
            other => {
                eprintln!("obelisk: no hook processor for '{other}'");
                Ok(1)
            }
        },
        Commands::Doctor => install::doctor(),
        Commands::Learn { action } => match action {
            LearnAction::Enable { repo_path, threshold } => learn::enable(&repo_path, threshold),
            LearnAction::Disable => learn::disable(),
            LearnAction::Status => learn::status(),
            LearnAction::Gaps => learn::gaps_json(),
        },
    }
}
