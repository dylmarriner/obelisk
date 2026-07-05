# Obelisk Pack Context

Use `obelisk_pack` when Hermes needs compact project context for coding, debugging, architecture, or PR review.

## Rules

- Prefer `obelisk_pack` over reading many files.
- Include `diff=true` for active code work.
- Use `dirs` for source maps instead of whole-directory reads.
- Use `files` only for directly relevant files.
- Keep the pack model-agnostic.

## Example tool call

```json
{
  "budget": 12000,
  "diff": true,
  "dirs": ["src"],
  "files": ["README.md", "Cargo.toml"]
}
```

Use restore handles only when the compact pack is insufficient.
