---
description: Inspect source code efficiently with Obelisk outline and symbol retrieval instead of reading entire large files.
---

# Inspect Symbol

Use this skill before reading large source files or when only one function, class, module, or symbol is needed.

## Behavior

1. Run `obelisk outline <file>` before reading a large file.
2. Identify the relevant symbol name and line range.
3. Run `obelisk symbol <file> <name>` to retrieve only that symbol.
4. Read the full file only if the outline/symbol view is not enough.

## Commands

```bash
obelisk outline src/main.rs
obelisk symbol src/main.rs run
```

For another language:

```bash
obelisk outline app/server.ts
obelisk symbol app/server.ts createServer
```

## Rule

Do not spend thousands of tokens reading a whole file when a symbol lookup answers the question. That is not diligence. That is context arson.
