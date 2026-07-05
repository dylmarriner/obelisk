# Obelisk

**A token-optimizing engine for AI coding agents. Minimize tokens, lose no context.**

Obelisk is a single Rust binary that sits between an AI coding agent and the
things that bloat its context: verbose command output, whole-file reads,
boilerplate, long sessions, tool schemas, diffs, and history files. It makes
all of that smaller while keeping recovery paths through a local reversible
ledger.

## What Obelisk does

| Layer | Command | What it does |
|-------|---------|--------------|
| Command output | `obelisk run <cmd>` | Runs a command and emits a compact, reversible view of stdout/stderr. |
| Boilerplate | `obelisk squeeze` | Strips ANSI, progress bars, duplicate-line runs, opaque blobs, and other noise. |
| Code retrieval | `obelisk outline` / `obelisk symbol` | Reads file structure or one symbol instead of dumping whole files. |
| Input context | `obelisk pack` | Builds a provider-neutral, token-budgeted context bundle. |
| Replies | `obelisk terse` | Drops filler from prose while preserving code blocks. |
| Sessions | `obelisk marker` / `obelisk checkpoint` / `obelisk restore` | Saves and restores compact work state. |
| Transport | `obelisk serve` | Runs a local proxy for model API traffic with token accounting. |
| Visibility | `obelisk stats` | Shows token savings across layers. |
| Agent hooks | `obelisk install <agent>` | Wires Obelisk into supported coding agents. |
| Claude plugin | `plugins/claude-code-obelisk` | Reusable Claude Code plugin with hooks, skills, and a context optimizer agent. |
| Hermes plugin | `plugins/hermes-obelisk` | Hermes Agent plugin with tools, hooks, slash commands, CLI commands, and skills. |
| Paperclip plugin | `plugins/paperclip-obelisk` | Paperclip plugin prototype for heartbeat/task context packing, run-output compression, restore handles, and savings UI. |
| Learning | `obelisk learn` | Optional usage-triggered self-improvement loop. Read the warning docs first. |

## Why it is different

Most token tools do one thing: compress terminal output, trim prose, or checkpoint context.
Obelisk is designed as a full context-optimization layer for coding agents:

- command-output compression
- model-agnostic input packing
- symbol-level code retrieval
- reversible restore handles
- local SQLite ledger
- agent hook installation
- Claude Code plugin packaging
- Hermes plugin packaging
- Paperclip heartbeat/task context optimization
- savings dashboard
- optional gap logging for future improvement

The important design rule: **Obelisk stays model-agnostic by default.** It does not need separate `pack` commands for Claude, GPT, Bedrock, OpenRouter, local models, or the next API-shaped creature the industry releases.

## Quick install

```bash
git clone https://github.com/dylmarriner/obelisk.git
cd obelisk

cargo fmt
cargo test
cargo build --release

mkdir -p ~/.local/bin
install -m755 target/release/obelisk ~/.local/bin/obelisk
export PATH="$HOME/.local/bin:$PATH"

obelisk doctor
```

For the full setup flow, including PATH fixes, RTK removal, agent hooks, smoke tests, and troubleshooting, read:

- [Setup help](docs/SETUP.md)

## Agent setup

Install only the agents you actually use:

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

Then restart the agent.

For details, read:

- [Agent integrations](docs/AGENT_INTEGRATIONS.md)

## Claude Code plugin

Obelisk ships a Claude Code plugin package:

```text
plugins/claude-code-obelisk
```

Test it locally from the repository root:

```bash
claude --plugin-dir ./plugins/claude-code-obelisk
```

Inside Claude Code:

```text
/help
/reload-plugins
/obelisk:pack-context
/obelisk:inspect-symbol
/obelisk:compact-output
/obelisk:restore-context
```

The plugin includes:

- a `PreToolUse` Bash hook that calls `obelisk hook claude`
- `/obelisk:pack-context` for model-agnostic context bundles
- `/obelisk:inspect-symbol` for outline/symbol retrieval
- `/obelisk:compact-output` for noisy command output
- `/obelisk:restore-context` for restore handles
- a `context-optimizer` agent for planning compact context before large coding work

The plugin expects the `obelisk` binary to already be installed on PATH. Bundling release binaries can come later, because cross-platform binary distribution is where simple ideas go to be processed into paste.

Plugin docs:

- [Claude Code plugin README](plugins/claude-code-obelisk/README.md)

## Hermes plugin

Obelisk ships a Hermes Agent plugin package:

```text
plugins/hermes-obelisk
```

Install from the repository root:

```bash
mkdir -p ~/.hermes/plugins
cp -R plugins/hermes-obelisk ~/.hermes/plugins/obelisk
hermes plugins enable obelisk
```

The Hermes plugin adds:

- `obelisk_run`
- `obelisk_pack`
- `obelisk_outline`
- `obelisk_symbol`
- `obelisk_restore`
- `obelisk_rewrite`
- `obelisk_stats`
- `obelisk_doctor`
- `/obelisk`, `/obelisk-stats`, and `/obelisk-doctor`
- bundled skills for pack/context/symbol/restore workflows
- a cautious `pre_tool_call` hook for shell command rewriting where Hermes supports it

Plugin docs:

- [Hermes plugin README](plugins/hermes-obelisk/README.md)

## Paperclip plugin

Obelisk also includes an early Paperclip plugin prototype:

```text
plugins/paperclip-obelisk
```

The Paperclip plugin targets the expensive part of agent orchestration: repeated task-start and heartbeat context. It exposes tools such as:

```text
task-pack
heartbeat-pack
compress-run-output
restore-context
context-diff
savings-report
```

The goal is to make Paperclip send:

```text
compact task capsule
+ changed events since last heartbeat
+ relevant workspace diff
+ restore handles for bulky originals
```

instead of resending the entire company/project/goal/task backstory every heartbeat like a tiny bureaucracy with a token meter.

Build the prototype:

```bash
cd plugins/paperclip-obelisk
npm install
npm run check
npm run build
```

Plugin docs:

- [Paperclip plugin README](plugins/paperclip-obelisk/README.md)

## Basic usage

```bash
obelisk run git status                 # compressed, reversible command output
obelisk run cargo build                # errors/warnings/result only
obelisk outline src/main.rs            # symbols + line ranges
obelisk symbol src/main.rs run         # one function instead of a whole file
obelisk pack --budget 12000 --diff --dir src --file README.md
obelisk pack --budget 8000 --system AGENTS.md --history session.json --tools tools.json --out context.md
echo "$LONG_LOG" | obelisk squeeze     # collapse boilerplate
obelisk restore 7f3a1b2c4d5e           # restore a stashed original
obelisk stats                          # savings across every layer
```

Full command docs:

- [Command reference](docs/COMMANDS.md)

## Model-agnostic input packing

`obelisk pack` accepts a budget and context sources, then emits compact Markdown any agent/provider can consume.

```bash
obelisk pack \
  --budget 12000 \
  --system AGENTS.md \
  --history .agent/session.json \
  --diff \
  --dir src \
  --file Cargo.toml \
  --tools tools.json \
  --out .obelisk/context.md
```

What it packs:

- stable instruction files via `--system`
- compacted chat/session state via `--history`
- current git stat/name-only/patch via `--diff`
- explicit files via `--file`
- directory maps via `--dir`, without reading every file into context
- compact tool schema names/descriptions via `--tools`
- restore handles for omitted or truncated full content

The packer uses Obelisk's provider-neutral token estimate. If you need exact billable token counts, run the provider's counter around the packed output. Do not split the core command into per-model templates unless you enjoy unpaid config gardening.

## Reversible compression

Whenever a layer compresses something worth restoring, Obelisk writes the full original to the local ledger and leaves an inline restore pointer:

```text
[obelisk:restore 7f3a1b2c4d5e — raw via `obelisk restore 7f3a1b2c4d5e`]
```

Restore it with:

```bash
obelisk restore 7f3a1b2c4d5e
```

## Self-improvement warning

Obelisk includes an optional self-improvement loop:

```bash
obelisk learn status
obelisk learn gaps
obelisk learn enable /path/to/obelisk --threshold 15
obelisk learn disable
```

Leave it disabled until you read:

- [Self-improvement](docs/SELF_IMPROVEMENT.md)

Current checked-in behavior can commit and push to `main` after build/test gates. That is powerful, risky, and not something to enable casually because an agent looked lonely.

## Troubleshooting

Common fixes live here:

- [Troubleshooting](docs/TROUBLESHOOTING.md)

Quick checks:

```bash
which obelisk
obelisk doctor
obelisk rewrite git status
obelisk stats
grep -Rni "obelisk\|rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

## Documentation

- [Documentation index](docs/README.md)
- [Setup help](docs/SETUP.md)
- [Command reference](docs/COMMANDS.md)
- [Agent integrations](docs/AGENT_INTEGRATIONS.md)
- [Self-improvement](docs/SELF_IMPROVEMENT.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Claude Code plugin](plugins/claude-code-obelisk/README.md)
- [Hermes plugin](plugins/hermes-obelisk/README.md)
- [Paperclip plugin](plugins/paperclip-obelisk/README.md)

## Development

```bash
cargo fmt
cargo test
cargo build --release
```

## Design notes

- One binary, zero background services unless you explicitly run `obelisk serve`.
- Dependency-light Rust implementation.
- Local-first ledger storage.
- Model-agnostic context packing.
- Recovery through restore handles.
- Agent hooks for command-output reduction.
- Claude Code plugin as a clean reusable integration layer.
- Hermes plugin as a first-class agent-runtime integration layer.
- Paperclip plugin as a control-plane context optimizer for task starts and heartbeats.

MIT licensed.
