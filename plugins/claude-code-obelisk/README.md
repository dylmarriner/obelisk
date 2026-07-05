# Obelisk Claude Code plugin

This plugin packages Obelisk for Claude Code as a reusable plugin with hooks, skills, and a context-optimizer agent.

Claude Code plugins are self-contained directories that can include skills, agents, hooks, MCP servers, LSP servers, monitors, binaries, and settings. This plugin keeps the compiled Obelisk binary separate for now and expects `obelisk` to already be installed on PATH.

## What it includes

```text
.claude-plugin/plugin.json       plugin metadata
hooks/hooks.json                 PreToolUse Bash hook
skills/pack-context/SKILL.md     build compact context packs
skills/inspect-symbol/SKILL.md   use outline/symbol retrieval
skills/compact-output/SKILL.md   route noisy commands through obelisk run
skills/restore-context/SKILL.md  restore full originals only when needed
agents/context-optimizer.md      planning agent for compact context selection
```

## Requirements

- Claude Code installed and authenticated.
- Obelisk installed and available on PATH.

Build and install Obelisk:

```bash
cargo build --release
install -m755 target/release/obelisk ~/.local/bin/obelisk
export PATH="$HOME/.local/bin:$PATH"
obelisk doctor
```

## Test locally

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

## Hook behavior

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

The hook lets Obelisk rewrite eligible read-heavy commands before Bash runs them.

Example:

```bash
git status
```

may become:

```bash
obelisk run git status
```

Obelisk deliberately avoids mutating, destructive, interactive, redirected, piped, or already-wrapped commands. A compression tool that rewrites `git push` would deserve to be unplugged.

## Skills

Plugin skills are namespaced under the plugin name:

```text
/obelisk:pack-context
/obelisk:inspect-symbol
/obelisk:compact-output
/obelisk:restore-context
```

Use them when Claude needs guidance on context size, noisy shell output, symbol-level inspection, or restore handles.

## Context optimizer agent

The plugin includes a custom agent:

```text
context-optimizer
```

Use it before large coding tasks, debugging sessions, architecture work, or PR review. Its job is to decide what context should be packed, outlined, symbol-read, restored, or ignored.

## Validation

Run:

```bash
claude plugin validate ./plugins/claude-code-obelisk
```

Then test with:

```bash
claude --plugin-dir ./plugins/claude-code-obelisk
```

## Future improvements

- Let `obelisk install claude` install this plugin instead of writing ad-hoc hook config.
- Add release binaries so the plugin can optionally ship or download the right Obelisk binary.
- Add plugin marketplace packaging.
- Add a post-tool hook later if Claude Code exposes a clean output replacement path suitable for Obelisk compression.
