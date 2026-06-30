# Obelisk

**A token-optimizing engine for AI coding agents. Minimize tokens, lose no context.**

Obelisk is a single Rust binary that sits between an AI coding agent and the
things that bloat its context — verbose command output, whole-file reads,
boilerplate, long sessions — and makes all of it small *and reversible*. Every
compression stashes its original in a local ledger, so anything Obelisk shrinks
is one `obelisk restore <handle>` away from being whole again.

## Why it's different

Most token tools do one thing — compress command output, *or* terse replies,
*or* checkpoint context. Obelisk does the whole pipeline in one binary, with a
single shared ledger and one dashboard, and adds the piece the others lack:
**symbol-level code retrieval** — hand the agent the one function it asked for
instead of a 2,000-line file (typically a 90%+ cut on a single read).

| Layer | Command | What it does |
|-------|---------|--------------|
| Command output | `obelisk run <cmd>` | run a command, emit a compact, reversible view of its output |
| Boilerplate | `obelisk squeeze` | strip ANSI, progress bars, duplicate-line runs, opaque blobs |
| Code retrieval | `obelisk outline` / `obelisk symbol` | file structure, or one symbol's source — not the whole file |
| Replies | `obelisk terse` | drop filler/greetings; code blocks left intact |
| Sessions | `obelisk marker` / `obelisk checkpoint` / `obelisk restore` | save & reload context compactly; survive compaction |
| Transport | `obelisk serve` | a local proxy for the model API, token-accounted |
| Visibility | `obelisk stats` | one savings dashboard across every layer |

## Install

```bash
cargo build --release          # -> target/release/obelisk
install -m755 target/release/obelisk ~/.local/bin/obelisk
obelisk doctor
```

Wire it into an agent (all reversible, configs backed up):

```bash
obelisk install claude         # also: hermes | opencode | openclaw
```

For Claude Code this adds a `PreToolUse` hook that routes output-heavy,
read-only shell commands through `obelisk run` automatically.

## Use

```bash
obelisk run git status                 # compressed, reversible
obelisk run cargo build                # errors/warnings/result only
obelisk outline src/main.rs            # symbols + line ranges
obelisk symbol src/main.rs run         # just that function
echo "$LONG_LOG" | obelisk squeeze     # collapse boilerplate
<summary> | obelisk marker save plan   # compact resume point
obelisk restore 7f3a1b2c4d5e           # pull any original back, in full
obelisk stats                          # savings across every layer
```

## How "lose no context" works

Whenever a layer compresses something it writes the full original to a local
SQLite ledger under a short handle, and leaves an inline pointer in the output:

```
…compressed…
[obelisk:restore 7f3a1b2c4d5e — full original via `obelisk restore 7f3a1b2c4d5e`]
```

Compress aggressively; the original is always recoverable. The same ledger
records per-layer token accounting that `obelisk stats` rolls up.

## Design

- One binary, zero background services (the proxy runs only when you start it).
- Dependency-light: stdlib + a handful of well-known crates; no async runtime.
- Local-only: the ledger lives under your data dir; nothing is sent anywhere
  except, when you run the proxy, the upstream API you point it at.

MIT licensed.
