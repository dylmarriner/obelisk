<p align="center">
  <a href="../../README.md"><img src="https://img.shields.io/badge/plugin-claude--code--obelisk-4A90D9?style=flat-square" alt="Claude Code Plugin"></a>
  <a href="../../docs/README.md"><img src="https://img.shields.io/badge/docs-claude%20code%20plugin-informational?style=flat-square" alt="Plugin Docs"></a>
</p>

# Obelisk Claude Code Plugin

**Reusable Claude Code plugin packaging Obelisk as hooks, skills, and a context optimizer agent.** Expects `obelisk` to already be installed on PATH.

---

## Table of Contents

- [Requirements](#requirements)
- [What's Included](#whats-included)
- [Test Locally](#test-locally)
- [Hook Behavior](#hook-behavior)
- [Skills](#skills)
- [Context Optimizer Agent](#context-optimizer-agent)
- [Validation](#validation)
- [Future Improvements](#future-improvements)

---

## Requirements

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) installed and authenticated
- [Obelisk](../README.md) installed and available on PATH

```bash
cargo build --release
install -m755 target/release/obelisk ~/.local/bin/obelisk
export PATH="$HOME/.local/bin:$PATH"
obelisk doctor
```

---

## What's Included

```
.claude-plugin/
├── plugin.json              Plugin metadata
├── hooks/
│   └── hooks.json           PreToolUse Bash hook
├── skills/
│   ├── pack-context/        Build compact context packs
│   ├── inspect-symbol/      Use outline/symbol retrieval
│   ├── compact-output/      Route noisy commands through obelisk run
│   └── restore-context/     Restore full originals only when needed
└── agents/
    └── context-optimizer.md Planning agent for compact context
```

---

## Test Locally

From the Obelisk repository root:

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

---

## Hook Behavior

The plugin registers a `PreToolUse` hook for the Bash tool:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "obelisk hook claude"
          }
        ]
      }
    ]
  }
}
```

The hook rewrites eligible read-heavy commands. Example:

```bash
git status
# → obelisk run git status
```

**Obelisk does not rewrite** mutating, destructive, interactive, redirected, piped, or already-wrapped commands.

---

## Skills

Plugin skills are namespaced under the plugin name:

| Slash Command | Purpose |
|---------------|---------|
| `/obelisk:pack-context` | Build compact, model-agnostic context bundles |
| `/obelisk:inspect-symbol` | Use outline/symbol retrieval efficiently |
| `/obelisk:compact-output` | Route noisy commands through `obelisk run` |
| `/obelisk:restore-context` | Restore full originals only when needed |

---

## Context Optimizer Agent

The plugin includes a custom agent:

```text
context-optimizer
```

Use it before large coding tasks, debugging sessions, architecture work, or PR review. Its job is to decide what context should be packed, outlined, symbol-read, restored, or ignored.

**Operating rules:**
- Prefer `obelisk pack` for large task handoffs
- Prefer `obelisk outline` before reading large files
- Prefer `obelisk symbol` when one function/class is enough
- Does not edit source files
- Plans context only

---

## Validation

```bash
claude plugin validate ./plugins/claude-code-obelisk
```

Then test with:

```bash
claude --plugin-dir ./plugins/claude-code-obelisk
```

---

## Future Improvements

- Let `obelisk install claude` install this plugin instead of writing ad-hoc hook config
- Add release binaries so the plugin can optionally ship or download the right Obelisk binary
- Add plugin marketplace packaging
- Add a post-tool hook if Claude Code exposes a clean output replacement path

---

<p align="center"><a href="../../README.md">← Back to README</a></p>
