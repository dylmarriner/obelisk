---
description: Route noisy read-only shell commands through Obelisk so Claude sees compact, reversible output.
---

# Compact Output

Use this skill when a command is likely to produce verbose output, logs, build noise, package-manager chatter, search results, or large JSON.

## Behavior

1. Prefer `obelisk run <command>` for noisy, read-heavy commands.
2. Do not wrap mutating, destructive, interactive, credential-handling, or long-running background commands.
3. Use restore handles only when the compact output is not enough.
4. Prefer exact commands over shell pipelines where possible.

## Good candidates

```bash
obelisk run git status
obelisk run cargo build
obelisk run pytest
obelisk run rg "TODO" src
obelisk run npm test
obelisk run docker logs app
```

## Bad candidates

```bash
git push
rm -rf target
sudo apt install something
vim src/main.rs
export API_KEY=...
```

## Restore rule

If Obelisk prints a handle like:

```text
[obelisk:restore abc123 — raw via `obelisk restore abc123`]
```

only restore when the compressed view loses important detail.
