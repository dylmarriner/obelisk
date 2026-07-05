---
description: Restore full original Obelisk context only when a compressed view is insufficient.
---

# Restore Context

Use this skill when Claude sees an Obelisk restore handle and needs the full original content.

## Behavior

1. First inspect the compressed output.
2. Restore only if the compressed output is missing detail required for the task.
3. Use the exact handle printed by Obelisk.
4. Avoid restoring large originals casually; it can undo the token savings.

## Command

```bash
obelisk restore <handle>
```

Example:

```bash
obelisk restore 7f3a1b2c4d5e
```

## Rule

Restore is the emergency hatch, not the front door. Use it when needed, not because the model is feeling nosy.
