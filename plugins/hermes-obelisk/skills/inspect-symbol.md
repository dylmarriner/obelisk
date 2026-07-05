# Obelisk Inspect Symbol

Use `obelisk_outline` and `obelisk_symbol` before reading large files.

## Workflow

1. Call `obelisk_outline` for the file.
2. Identify the relevant symbol.
3. Call `obelisk_symbol` for that symbol.
4. Read the full file only if symbol-level retrieval is insufficient.

## Example tool calls

```json
{"file": "src/main.rs"}
```

then:

```json
{"file": "src/main.rs", "name": "run"}
```

A whole-file read is not a personality trait. Avoid it unless needed.
