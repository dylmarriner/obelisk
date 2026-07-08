<p align="center">
  <a href="./README.md"><img src="https://img.shields.io/badge/docs-agent%20integrations-informational?style=flat-square" alt="Agent Integrations"></a>
  <a href="../README.md"><img src="https://img.shields.io/badge/←%20back-readme-blue?style=flat-square" alt="Back"></a>
</p>

# Agent Integrations

**Wire Obelisk into your AI coding agent.** Configure shell hooks so noisy command output and repeated context are routed through Obelisk's compression before entering the model's context window.

---

## Table of Contents

- [Quick Setup](#quick-setup)
- [Verification](#verification)
- [Claude Code](#claude-code)
- [Codex](#codex)
- [OpenCode](#opencode)
- [Hermes](#hermes)
- [OpenClaw](#openclaw)
- [Cline](#cline)
- [Paperclip](#paperclip)
- [Prompt Guidance for Agents](#prompt-guidance-for-agents)
- [Troubleshooting Hooks](#troubleshooting-hooks)
- [Removing RTK](#removing-rtk)

---

## Quick Setup

Install only the integrations you use:

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

**Always restart the agent after installing hooks** — most agents read config only on startup.

---

## Verification

### Check installation

```bash
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

### Test rewrite logic

```bash
obelisk rewrite git status
obelisk rewrite cargo build
obelisk rewrite git push
```

**Expected:**
- Read-heavy commands (`git status`, `cargo build`) → prints `obelisk run ...` and exits 0
- Mutating commands (`git push`) → exits 1 with no output

---

## Claude Code

### Basic hook installation

```bash
obelisk install claude
```

**Expected behavior:**
- Adds a `PreToolUse` hook for Bash
- Rewrites eligible read-heavy shell commands to `obelisk run <cmd>`
- Backs up existing settings before writing

**Verify:**

```bash
grep -Rni "obelisk hook claude" ~/.claude 2>/dev/null || true
```

**Test:** Restart Claude Code, then ask it to run `git status`. If wired correctly, the command routes through Obelisk.

### Plugin package

Obelisk also ships a reusable Claude Code plugin with additional features:

```bash
claude --plugin-dir ./plugins/claude-code-obelisk
```

**Plugin capabilities:**

| Feature | Description |
|---------|-------------|
| `PreToolUse` Bash hook | Calls `obelisk hook claude` |
| `/obelisk:pack-context` | Token-budgeted context bundles |
| `/obelisk:inspect-symbol` | Outline/symbol retrieval |
| `/obelisk:compact-output` | Noisy command compression |
| `/obelisk:restore-context` | Restore handle retrieval |
| `context-optimizer` agent | Plans compact context before large work |

**Plugin docs:** [../plugins/claude-code-obelisk/README.md](../plugins/claude-code-obelisk/README.md)

---

## Codex

```bash
obelisk install codex
```

Codex shell tool names can vary, so Obelisk's Codex hook accepts several tool-name/input shapes.

**Verify:**

```bash
grep -Rni "obelisk hook codex" ~/.codex 2>/dev/null || true
```

**Restart Codex** after installing.

---

## OpenCode

```bash
obelisk install opencode
```

**Expected behavior:**
- Writes an OpenCode plugin file under the OpenCode config path
- Plugin calls Obelisk rewrite/run logic for eligible commands

**Verify:**

```bash
grep -Rni "obelisk" ~/.config/opencode 2>/dev/null || true
```

**Restart OpenCode** after installing.

---

## Hermes

### Basic hook installation

```bash
obelisk install hermes
```

**Expected behavior:**
- Writes an awareness file under `~/.hermes`
- Installs an `obelisk-rewrite` plugin under `~/.hermes/plugins`

**Verify:**

```bash
grep -Rni "obelisk" ~/.hermes 2>/dev/null || true
```

**Restart Hermes** after installing.

### Token Optimizer hooks

The Hermes plugin also registers optional Token Optimizer hooks that go beyond command compression:

| Hook | Trigger | What it does |
|------|---------|-------------|
| `post_api_request` | After each API call | Accumulates per-turn token usage (input, output, cache, reasoning) |
| `pre_llm_call` | Before each LLM call | Checks context fill ratio; inserts a one-line nudge when >70% |
| `on_session_finalize` | Session finalization | Fires rollup to Token Optimizer `trends.db` |
| `on_session_end` | Session end | Same rollup (deduped) |

**Additional slash commands:**
- `/obelisk-token` — show token/cost summary for recent sessions
- `hermes obelisk-token` — open the Token Optimizer dashboard

**Requirements:** Token Optimizer repo cloned at `~/Documents/token-optimizer/`

### Hermes plugin package

Obelisk also ships a full Hermes-native plugin package:

```bash
mkdir -p ~/.hermes/plugins
cp -R plugins/hermes-obelisk ~/.hermes/plugins/obelisk
hermes plugins enable obelisk
```

Or manually in `~/.hermes/config.yaml`:

```yaml
plugins:
  enabled:
    - obelisk
```

**Plugin capabilities:**

| Category | Components |
|----------|-----------|
| Tools | `obelisk_run`, `obelisk_pack`, `obelisk_outline`, `obelisk_symbol`, `obelisk_restore`, `obelisk_rewrite`, `obelisk_stats`, `obelisk_doctor` |
| Slash commands | `/obelisk`, `/obelisk-stats`, `/obelisk-doctor`, `/obelisk-token` |
| CLI commands | `hermes obelisk-doctor`, `hermes obelisk-stats`, `hermes obelisk-token` |
| Skills | `obelisk:pack-context`, `obelisk:inspect-symbol`, `obelisk:compact-output`, `obelisk:restore-context` |
| Hooks | `pre_tool_call` (command rewrites), Token Optimizer hooks (above) |

**Plugin docs:** [../plugins/hermes-obelisk/README.md](../plugins/hermes-obelisk/README.md)

---

## OpenClaw

```bash
obelisk install openclaw
```

**Expected behavior:** Writes awareness guidance telling OpenClaw to use `obelisk run`, `obelisk outline`, and `obelisk symbol`.

**Verify:**

```bash
grep -Rni "obelisk" ~/.openclaw 2>/dev/null || true
```

---

## Cline

```bash
obelisk install cline
```

Run from the project directory where Cline should see the rule.

**Expected behavior:** Appends Obelisk guidance to project-local `.clinerules`. Does not globally intercept shell commands.

**Verify:**

```bash
grep -n "obelisk" .clinerules
```

> **Note:** Cline has no universal shell hook API in this setup, so this is guidance-style rather than command interception.

---

## Paperclip

Paperclip is a control plane for teams of agents — tasks, goals, budgets, and heartbeats. Obelisk integrates with it differently: through a plugin prototype that targets repeated heartbeat/task-start context.

```bash
cd plugins/paperclip-obelisk
npm install
npm run check
npm run build
```

**Plugin tools:**

| Tool | Purpose |
|------|---------|
| `task-pack` | Compact task-start context pack |
| `heartbeat-pack` | Delta-based heartbeat context (key token saver) |
| `compress-run-output` | Squeeze Paperclip run logs |
| `restore-context` | Restore via Obelisk handle |
| `context-diff` | Stable hashes and changed-key list |
| `savings-report` | Obelisk stats wrapper |

**Target flow:**

```
Paperclip task or heartbeat
↓
Obelisk task-pack / heartbeat-pack
↓
compact task capsule + delta events + workspace diff + restore handles
↓
agent run
```

**Plugin docs:** [../plugins/paperclip-obelisk/README.md](../plugins/paperclip-obelisk/README.md)

---

## Prompt Guidance for Agents

Use this in agent project rules when only guidance-style integration is available:

```text
Use Obelisk to minimize token usage:
- Route noisy read-only shell commands through `obelisk run`.
- Use `obelisk outline <file>` before reading large source files.
- Use `obelisk symbol <file> <name>` when only one function/class is needed.
- Use `obelisk pack --budget <n>` to build compact project context.
- Use `obelisk restore <handle>` only when compressed context is insufficient.
Do not route mutating, interactive, destructive, or credential-handling commands through Obelisk.
```

---

## Troubleshooting Hooks

If commands are not being rewritten:

1. Confirm `obelisk` is on PATH inside the agent environment:
   ```bash
   which obelisk
   ```
2. Restart the agent.
3. Check config files for the hook command:
   ```bash
   grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
   ```
4. Test rewrite directly:
   ```bash
   obelisk rewrite git status
   ```
5. Check Obelisk health:
   ```bash
   obelisk doctor
   ```
6. Restore backed-up config if needed (from `~/obelisk-agent-config-backups`).

---

## Removing RTK

If you previously used RTK, remove it before relying on Obelisk:

```bash
# Find all RTK references
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true

# Remove plugin files
rm -rf ~/.hermes/plugins/rtk-rewrite
rm -f ~/.config/opencode/plugins/rtk.ts

# Remove binary
cargo uninstall rtk 2>/dev/null || true
cargo uninstall rtk-cli 2>/dev/null || true
rm -f ~/.cargo/bin/rtk ~/.local/bin/rtk
sudo rm -f /usr/local/bin/rtk 2>/dev/null || true

# Confirm removal
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
which rtk || true
```

---

<p align="center"><a href="./README.md">← Documentation Index</a> · <a href="../README.md">Back to README</a></p>
