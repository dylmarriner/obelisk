# Obelisk Hermes Plugin

Unified token optimization for Hermes Agent. Merges **Obelisk**'s command-output compression tools with **Token Optimizer**'s per-turn token tracking, context-fill nudges, and session rollup.

## What It Does

### Obelisk (command compression)
- `obelisk_run` — run safe read-heavy commands through compact, reversible output
- `obelisk_pack` — build token-budgeted context packs from files, diffs, history
- `obelisk_outline` — list source file symbols without reading the full file
- `obelisk_symbol` — extract one named symbol from a source file
- `obelisk_restore` — restore a compressed blob/checkpoint by handle
- `obelisk_rewrite` — ask Obelisk whether a command should be wrapped
- `obelisk_stats` — show token savings across Obelisk layers
- `obelisk_doctor` — verify Obelisk installation

### Token Optimizer (usage tracking)
- **Context nudge** — proactively warns when context fill crosses ~70%, once per session
- **Per-turn tally** — accumulates input/output/cache/reasoning tokens per session
- **Session rollup** — writes session data into the shared Token Optimizer `trends.db` at session end for dashboard visibility
- **`/obelisk-token`** — slash command showing token and cost summary for recent sessions
- **`hermes obelisk-token`** — CLI subcommand to open the Token Optimizer dashboard

## Requirements

- **Obelisk binary** on PATH (`~/.local/bin/obelisk`)
- **Token Optimizer repo** cloned at `~/Documents/token-optimizer/` (provides `measure.py` engine)

## Install

Build and install Obelisk:

```bash
cargo build --release
mkdir -p ~/.local/bin
install -m755 target/release/obelisk ~/.local/bin/obelisk
export PATH="$HOME/.local/bin:$PATH"
obelisk doctor
```

Clone Token Optimizer for the dashboard/rollup engine:

```bash
git clone https://github.com/alexgreensh/token-optimizer.git ~/Documents/token-optimizer
```

The plugin is already installed at `~/.hermes/plugins/obelisk/` and enabled in Hermes config.

## Slash Commands

| Command | Description |
|---------|-------------|
| `/obelisk` | Plugin help and available tools |
| `/obelisk-stats` | Obelisk token savings stats |
| `/obelisk-doctor` | Obelisk installation status |
| `/obelisk-token` | Token/cost summary for recent sessions |

## CLI Commands

```bash
hermes obelisk-doctor
hermes obelisk-stats
hermes obelisk-token           # opens dashboard
hermes obelisk-token --port 3000 --session <id>
```

## Tools

| Tool | Description |
|------|-------------|
| `obelisk_run` | Safe read-heavy command through Obelisk |
| `obelisk_pack` | Token-budgeted context pack |
| `obelisk_outline` | Source file symbols |
| `obelisk_symbol` | One symbol from source |
| `obelisk_restore` | Restore compressed blob |
| `obelisk_rewrite` | Command rewrite check |
| `obelisk_stats` | Token savings stats |
| `obelisk_doctor` | Installation check |

## Skills

- `obelisk:pack-context`
- `obelisk:inspect-symbol`
- `obelisk:compact-output`
- `obelisk:restore-context`

## Context Nudge

Before each turn, the `pre_llm_call` hook estimates how full the context window is. If >70%, it appends a one-line warning to the user message:

```
[Obelisk] Context ~73% full (~146,000 input tokens vs assumed 200,000 window) Grade: C. Avoid adding large files; prefers targeted reads.
```

At 85%+, the tip suggests `/compact`.

## How It Works

```
Hermes turn lifecycle
    pre_llm_call        → nudge check (fill > 70%?) → inject warning or stay silent
    post_api_request    → accumulate usage into in-process tally
    on_session_finalize / on_session_end
        → hermes_hook_bridge.run_rollup()
            → measure.py hermes-rollup
                → reads state.db (read-only), writes trends.db
```

### Privacy

- `hermes_state.py` opens `~/.hermes/state.db` read-only with `PRAGMA query_only = ON`
- No data sent to any external service. No telemetry. No network calls.
