# Agent integrations

Obelisk can be wired into coding agents so noisy shell output is routed through `obelisk run` before it lands in model context.

## General rule

Install only the integrations you use:

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

Restart the agent after installing. Agents love caching stale config, because apparently one source of confusion was not enough.

## Check what was installed

Search common config locations:

```bash
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

Test rewrite logic directly:

```bash
obelisk rewrite git status
obelisk rewrite cargo build
obelisk rewrite git push
```

Expected:

- read-heavy commands such as `git status` or `cargo build` should rewrite to `obelisk run ...`
- mutating commands such as `git push` should not rewrite

## Claude Code

Install:

```bash
obelisk install claude
```

Expected behavior:

- adds a `PreToolUse` hook for Bash
- rewrites eligible read-heavy shell commands to `obelisk run <cmd>`
- backs up existing settings before writing where supported

Check:

```bash
grep -Rni "obelisk hook claude" ~/.claude 2>/dev/null || true
```

Restart Claude Code, then ask it to run:

```bash
git status
```

If wired correctly, the command should be routed through Obelisk.

## Codex

Install:

```bash
obelisk install codex
```

Codex shell tool names can vary, so Obelisk's Codex hook accepts a few tool-name/input shapes internally.

Check:

```bash
grep -Rni "obelisk hook codex" ~/.codex 2>/dev/null || true
```

Restart Codex after installing.

## OpenCode

Install:

```bash
obelisk install opencode
```

Expected behavior:

- writes an OpenCode plugin file under the OpenCode config path
- plugin calls Obelisk rewrite/run logic for eligible commands

Check:

```bash
grep -Rni "obelisk" ~/.config/opencode 2>/dev/null || true
```

Restart OpenCode after installing.

## Hermes

Install:

```bash
obelisk install hermes
```

Expected behavior:

- writes an awareness file under `~/.hermes`
- installs an `obelisk-rewrite` plugin under `~/.hermes/plugins`

Check:

```bash
grep -Rni "obelisk" ~/.hermes 2>/dev/null || true
```

Restart Hermes after installing.

## OpenClaw

Install:

```bash
obelisk install openclaw
```

Expected behavior:

- writes awareness guidance telling OpenClaw to use `obelisk run`, `obelisk outline`, and `obelisk symbol`

Check:

```bash
grep -Rni "obelisk" ~/.openclaw 2>/dev/null || true
```

## Cline

Install from the project directory where you want Cline to see the rule:

```bash
obelisk install cline
```

Expected behavior:

- appends Obelisk guidance to project-local `.clinerules`
- does not globally intercept shell commands

Check:

```bash
grep -n "obelisk" .clinerules
```

Cline has no universal shell hook API in this setup, so this is guidance rather than command interception.

## Removing RTK integration

If you previously used RTK, remove it before relying on Obelisk:

```bash
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
rm -rf ~/.hermes/plugins/rtk-rewrite
rm -f ~/.config/opencode/plugins/rtk.ts
cargo uninstall rtk 2>/dev/null || true
cargo uninstall rtk-cli 2>/dev/null || true
rm -f ~/.cargo/bin/rtk ~/.local/bin/rtk
sudo rm -f /usr/local/bin/rtk 2>/dev/null || true
```

Then re-check:

```bash
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
which rtk || true
which obelisk
obelisk doctor
```

## Agent prompt guidance

Use this in agent project rules if an integration can only provide guidance:

```text
Use Obelisk to minimize token usage:
- Route noisy read-only shell commands through `obelisk run`.
- Use `obelisk outline <file>` before reading large source files.
- Use `obelisk symbol <file> <name>` when only one function/class is needed.
- Use `obelisk pack --budget <n>` to build compact project context.
- Use `obelisk restore <handle>` only when compressed context is insufficient.
Do not route mutating, interactive, destructive, or credential-handling commands through Obelisk.
```

## Troubleshooting hooks

If commands are not being rewritten:

1. Confirm `obelisk` is on PATH inside the agent environment.
2. Restart the agent.
3. Check config files for the hook command.
4. Run `obelisk rewrite <cmd>` manually.
5. Run `obelisk doctor`.
6. Restore backed-up config if needed.

Useful checks:

```bash
which obelisk
obelisk doctor
obelisk rewrite git status
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```
