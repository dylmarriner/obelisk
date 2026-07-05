---
description: Build a compact, model-agnostic Obelisk context pack for large coding, debugging, architecture, or PR review tasks.
---

# Pack Context

Use this skill when the current task needs compact project context, a handoff file, or a smaller prompt for a coding model.

## Behavior

1. Prefer `obelisk pack` over manually reading many files.
2. Keep packing model-agnostic. Do not create provider-specific prompt formats.
3. Include current changes with `--diff` when the task involves local edits or PR review.
4. Use `--dir` for source maps instead of dumping whole directories.
5. Use `--file` only for files that are directly relevant.
6. Write output to `.obelisk/context.md` when a persistent handoff is useful.

## Common commands

```bash
mkdir -p .obelisk
obelisk pack --budget 12000 --diff --dir src --file README.md --out .obelisk/context.md
```

For architecture or planning work:

```bash
obelisk pack --budget 16000 --system AGENTS.md --diff --dir src --file README.md --file Cargo.toml --out .obelisk/context.md
```

For small debugging work:

```bash
obelisk pack --budget 8000 --diff --dir src --out .obelisk/context.md
```

## Restore rule

If the pack output contains an Obelisk restore handle, use `obelisk restore <handle>` only when the compact output is insufficient.
