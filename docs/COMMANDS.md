# Obelisk command reference

Obelisk is one binary with several token-reduction layers. This page lists the public commands and how to use them without turning your terminal into a shrine of half-remembered flags.

## `obelisk doctor`

Checks that Obelisk can run and that its local ledger works.

```bash
obelisk doctor
```

Use this after install, after updates, and after moving the binary.

## `obelisk run <cmd>`

Runs a command and prints a compact, reversible version of stdout/stderr.

```bash
obelisk run git status
obelisk run cargo build
obelisk run pytest
obelisk run rg "TODO" src
```

Use for noisy, read-heavy commands. Obelisk chooses a command-specific filter when available and falls back to generic squeezing.

Avoid wrapping mutating or interactive commands manually. Obelisk hooks try to avoid rewriting unsafe commands, but do not outsource judgement to a regex and call it civilisation.

## `obelisk rewrite <cmd>`

Shows whether Obelisk would rewrite a raw command.

```bash
obelisk rewrite git status
obelisk rewrite cargo build
obelisk rewrite git push
```

If eligible, it prints something like:

```bash
obelisk run git status
```

If not eligible, it exits with a non-zero status and prints nothing.

## `obelisk squeeze`

Reads text from stdin and removes boilerplate such as ANSI codes, progress bars, repeated lines, blank-line runs, and opaque blobs.

```bash
journalctl -n 5000 | obelisk squeeze
cat long.log | obelisk squeeze
```

Use this when you already have text and want to compact it before pasting into an agent.

## `obelisk terse [level]`

Compacts prose from stdin while leaving code blocks alone.

```bash
cat response.md | obelisk terse
cat response.md | obelisk terse lite
cat response.md | obelisk terse full
cat response.md | obelisk terse ultra
```

Useful for trimming long assistant replies, planning notes, or handoff text.

## `obelisk outline <file>`

Prints symbols and line ranges for a source file.

```bash
obelisk outline src/main.rs
obelisk outline app/server.ts
```

Use this before reading large files. Agents should inspect outlines first, then ask for specific symbols.

## `obelisk symbol <file> <name>`

Prints only one function/class/symbol from a file.

```bash
obelisk symbol src/main.rs run
obelisk symbol src/symbols.rs parse
```

This is one of Obelisk's biggest token wins: one function instead of a whole file.

## `obelisk pack`

Builds a model-agnostic, token-budgeted context bundle.

```bash
obelisk pack --budget 12000 --diff --dir src --file README.md
```

Common full example:

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

Flags:

| Flag | Meaning |
|---|---|
| `--budget <tokens>` | Approximate provider-neutral token budget. |
| `--system <file>` | Stable instruction/context file. Repeatable. |
| `--history <file>` | Chat/session history file to compact. Repeatable. |
| `--diff` | Include current git stat/name-only/patch if available. |
| `--dir <path>` | Include a directory map without reading every file. Repeatable. |
| `--file <path>` | Include or outline an explicit file. Repeatable. |
| `--tools <json>` | Compact a tool-schema JSON file into names/descriptions. |
| `--out <file>` | Write pack output to disk instead of stdout. |

`pack` is deliberately not tied to Claude, GPT, Bedrock, OpenRouter, or any other provider. Exact billing counters can wrap the packed output later. The core stays model-agnostic because per-model pack templates are how maintenance gets fleas.

## `obelisk marker`

Manages compact named resume points.

Common pattern:

```bash
echo "Current plan and decisions..." | obelisk marker save plan
obelisk marker get plan
obelisk marker list
obelisk marker delete plan
```

Use markers for human-curated state you want an agent to resume from without replaying an entire session.

## `obelisk checkpoint [label]`

Stores a full session snapshot from stdin and prints a restore handle.

```bash
cat session.md | obelisk checkpoint session
```

Use checkpoints when you want a reversible full-state handoff.

## `obelisk restore <handle>`

Restores a compressed original, checkpoint, or stashed full section.

```bash
obelisk restore 7f3a1b2c4d5e
```

Use restore only when the compressed view is not enough.

## `obelisk stats`

Shows token accounting across layers.

```bash
obelisk stats
```

Use it to check whether Obelisk is actually saving tokens or just giving you a warm dashboard hug.

## `obelisk gc [days]`

Deletes old reversible blobs from the ledger. Markers and checkpoints are kept.

```bash
obelisk gc 14
obelisk gc 30
```

Use this when the local ledger grows too large.

## `obelisk install <agent>`

Wires Obelisk into an agent.

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

See [Agent integrations](AGENT_INTEGRATIONS.md) for details.

## `obelisk hook <agent>`

Internal command used by agent hook integrations. You normally do not run it by hand.

```bash
obelisk hook claude
obelisk hook codex
```

It reads hook JSON from stdin and emits the agent-specific hook response.

## `obelisk serve`

Runs a local proxy for model API traffic.

```bash
obelisk serve --port 6767 --upstream https://api.anthropic.com
```

Current role: proxying and token accounting. Treat deeper request-body optimisation as a future direction unless implemented in code.

## `obelisk learn`

Controls the usage-triggered self-improvement loop.

```bash
obelisk learn status
obelisk learn gaps
obelisk learn enable /path/to/obelisk --threshold 15
obelisk learn disable
```

Read [Self-improvement](SELF_IMPROVEMENT.md) before enabling it. Current behavior can commit and push to `main` after passing gates, which is not something you casually enable like dark mode.
