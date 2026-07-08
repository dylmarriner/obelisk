<p align="center">
  <a href="./README.md"><img src="https://img.shields.io/badge/docs-commands-informational?style=flat-square" alt="Commands"></a>
  <a href="../README.md"><img src="https://img.shields.io/badge/←%20back-readme-blue?style=flat-square" alt="Back"></a>
</p>

# Command Reference

**Every Obelisk command, flag, and usage pattern.** Obelisk is one binary with several token-reduction layers. All commands are accessed through the `obelisk` entry point.

---

## Table of Contents

- [obelisk doctor](#obelisk-doctor)
- [obelisk run](#obelisk-run)
- [obelisk rewrite](#obelisk-rewrite)
- [obelisk squeeze](#obelisk-squeeze)
- [obelisk terse](#obelisk-terse)
- [obelisk outline](#obelisk-outline)
- [obelisk symbol](#obelisk-symbol)
- [obelisk pack](#obelisk-pack)
- [obelisk marker](#obelisk-marker)
- [obelisk checkpoint](#obelisk-checkpoint)
- [obelisk restore](#obelisk-restore)
- [obelisk serve](#obelisk-serve)
- [obelisk stats](#obelisk-stats)
- [obelisk gc](#obelisk-gc)
- [obelisk install](#obelisk-install)
- [obelisk hook](#obelisk-hook)
- [obelisk learn](#obelisk-learn)

---

## obelisk doctor

Verify that Obelisk can run and that its local ledger works.

```bash
obelisk doctor
```

**Use after:** install, updates, or moving the binary.

---

## obelisk run

Run a command and emit a compact, reversible view of stdout/stderr. Obelisk chooses a command-specific filter when available and falls back to generic squeezing.

```bash
obelisk run git status
obelisk run cargo build
obelisk run pytest
obelisk run rg "TODO" src
```

**Use for:** noisy, read-heavy commands (builds, tests, search output, logs).

**Do not use for:** mutating, interactive, or piped commands.

---

## obelisk rewrite

Show whether Obelisk would rewrite a given command. Exits 0 with the rewritten command if eligible, exits 1 with no output if not.

```bash
obelisk rewrite git status        # prints: obelisk run git status
obelisk rewrite cargo build       # prints: obelisk run cargo build
obelisk rewrite git push          # exits 1 (no output)
```

**Use for:** checking hook eligibility or debugging rewrite logic.

---

## obelisk squeeze

Read text from stdin and remove boilerplate:

- ANSI escape codes
- Progress bars
- Repeated/duplicate lines
- Blank-line runs
- Opaque binary blobs

```bash
journalctl -n 5000 | obelisk squeeze
cat long.log | obelisk squeeze
```

**Use for:** compacting logs, journal output, or any verbose text before feeding it to an agent.

**Flags:** none (reads stdin only).

---

## obelisk terse

Compact prose from stdin while leaving code blocks intact. Supports compression levels:

```bash
cat response.md | obelisk terse        # lite (default)
cat response.md | obelisk terse lite   # conservative
cat response.md | obelisk terse full   # aggressive
cat response.md | obelisk terse ultra  # maximum
```

**Use for:** trimming long assistant replies, planning notes, or handoff documents.

**How levels differ:** `lite` removes only the weakest filler words; `full` condenses sentences; `ultra` shortens aggressively while preserving meaning and all code blocks.

---

## obelisk outline

Print symbols (functions, structs, modules) and their line ranges for a source file. Languages supported: Rust, TypeScript/JavaScript, Python, Go, and others with standard syntax patterns.

```bash
obelisk outline src/main.rs
obelisk outline app/server.ts
obelisk outline src/commands.py
```

**Use for:** understanding file structure before reading the full file. Agents should inspect outlines first, then retrieve specific symbols.

---

## obelisk symbol

Print only one named function, class, or symbol from a source file. This is one of Obelisk's biggest token wins — one function instead of a whole file.

```bash
obelisk symbol src/main.rs run
obelisk symbol src/symbols.rs parse
obelisk symbol src/commands.py handle_install
```

**Use for:** targeted code retrieval when you know the symbol name.

---

## obelisk pack

Build a provider-neutral, token-budgeted context bundle from files, directories, diffs, history, system files, and optional tool schema JSON.

```bash
# Minimal
obelisk pack --budget 12000 --diff --dir src --file README.md

# Full
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

### Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--budget` | integer | 12000 | Approximate provider-neutral token budget |
| `--system` | string (repeatable) | — | Stable instruction/context file |
| `--history` | string (repeatable) | — | Chat/session history file to compact |
| `--diff` | boolean | false | Include current git stat/name-only/patch |
| `--dir` | string (repeatable) | — | Directory map (not full content) |
| `--file` | string (repeatable) | — | Explicit file to include or outline |
| `--tools` | string | — | Tool schema JSON file to compact |
| `--out` | string | — | Write output to file instead of stdout |

> **Note:** `pack` is deliberately model-agnostic. No separate commands for Claude, GPT, Bedrock, or OpenRouter. Provider-specific counters can wrap the output — the core stays provider-neutral.

---

## obelisk marker

Manage compact named resume points — human-curated state for agents to resume from without replaying an entire session.

```bash
obelisk marker save plan           # save from stdin with given name
obelisk marker get plan            # retrieve a named marker
obelisk marker list                # list all markers
obelisk marker delete plan         # delete a marker
```

**Pattern:**
```bash
echo "Current plan and decisions..." | obelisk marker save plan
obelisk marker get plan
```

---

## obelisk checkpoint

Store a full session snapshot from stdin and print a restore handle.

```bash
cat session.md | obelisk checkpoint session
cat session.md | obelisk checkpoint "code-review-2026-07"
```

**Use for:** reversible full-state handoff. The printed handle can be passed to `obelisk restore`.

---

## obelisk restore

Restore a compressed original, checkpoint, or stashed full section by handle.

```bash
obelisk restore 7f3a1b2c4d5e
```

**Use for:** recovering full content when the compressed view is insufficient.

**Common failure modes:**
- Handle was garbage-collected (check `obelisk gc` history)
- Handle from another machine or user
- Content was never stashed (compression was cheaper than storing)

---

## obelisk serve

Run a local proxy for model API traffic with token accounting.

```bash
obelisk serve --port 6767 --upstream https://api.anthropic.com
```

| Flag | Default | Description |
|------|---------|-------------|
| `--port` | 6767 | Local port to bind |
| `--upstream` | `https://api.anthropic.com` | Upstream API endpoint |

**Current role:** proxying and token accounting. Deeper request-body optimization is a future direction.

---

## obelisk stats

Show token savings across all compression layers.

```bash
obelisk stats
```

**Use for:** checking whether Obelisk is actually saving tokens. Shows per-layer breakdown and aggregate savings.

---

## obelisk gc

Evict reversible blobs older than N days from the ledger. Markers and checkpoints are preserved.

```bash
obelisk gc 14     # evict blobs > 14 days
obelisk gc 30     # evict blobs > 30 days
```

**Use for:** reclaiming disk space when the local ledger grows too large.

---

## obelisk install

Wire Obelisk into a supported AI coding agent.

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

**Supported agents:**

| Agent | Hook Type | Notes |
|-------|-----------|-------|
| Claude Code | `PreToolUse` Bash hook | Rewrites eligible commands |
| Codex | Shell tool hook | Accepts multiple tool-name shapes |
| OpenCode | Plugin file | Writes to OpenCode config path |
| Hermes | Awareness + plugin | Also supports the Hermes plugin package |
| OpenClaw | Awareness rules | Guidance-style integration |
| Cline | `.clinerules` guidance | No global shell hook |

**After install:** restart the agent. See [Agent Integrations](AGENT_INTEGRATIONS.md) for details.

---

## obelisk hook

Internal command used by agent hook integrations. Reads hook JSON from stdin and emits the agent-specific hook response.

```bash
obelisk hook claude
obelisk hook codex
```

> **Note:** This is an internal command — normally invoked by the agent's hook system, not run by hand.

---

## obelisk learn

Control the usage-triggered self-improvement loop.

```bash
obelisk learn status                                            # show enabled/disabled + pending gaps
obelisk learn gaps                                              # dump pending gaps as JSON
obelisk learn enable /path/to/obelisk --threshold 15            # enable with gap threshold
obelisk learn disable                                           # disable
```

| Subcommand | Description |
|------------|-------------|
| `status` | Show whether learning is enabled and how many gaps are pending |
| `gaps` | Dump pending gaps as JSON (consumed by `scripts/self-improve.sh`) |
| `enable` | Enable the loop for a repo (must contain `scripts/self-improve.sh`) |
| `disable` | Disable the loop |

> ⚠️ **Warning:** The current self-improvement script can commit and push to `main`. Read [SELF_IMPROVEMENT.md](SELF_IMPROVEMENT.md) before enabling.

---

<p align="center"><a href="./README.md">← Documentation Index</a> · <a href="../README.md">Back to README</a></p>
