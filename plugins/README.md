# Obelisk plugin packages

Obelisk keeps the Rust binary as the core engine and uses thin plugin packages for agent/runtime-specific integration.

## Packages

| Package | Target | Status | Purpose |
|---|---|---|---|
| [`claude-code-obelisk`](claude-code-obelisk/README.md) | Claude Code | Local plugin package | Hooks, skills, and context optimizer agent. |
| [`hermes-obelisk`](hermes-obelisk/README.md) | Hermes Agent | Native plugin package | Tools, hooks, slash commands, CLI commands, and skills. |
| [`paperclip-obelisk`](paperclip-obelisk/README.md) | Paperclip | Prototype | Task-start and heartbeat context packing, run-output compression, restore handles, and savings UI. |
| [`openclaw-obelisk`](openclaw-obelisk/README.md) | OpenClaw | Native tool plugin package | Agent-callable Obelisk tools, optional command compression, and approval hook. |

## Design rule

Plugins are adapters. Obelisk remains the engine.

```text
agent/runtime/control plane
↓
thin plugin adapter
↓
obelisk binary
↓
local ledger + compression + packing + restore handles
```

Do not reimplement Obelisk logic inside a plugin unless the host API requires a tiny compatibility layer. Reimplementing the same engine four times would be a monument to human overconfidence.

## Safety rule

All plugins must default to conservative behavior:

- read-heavy commands only
- no destructive shell rewriting
- no credential scraping
- no blind restore of large blobs
- no provider-specific context pack formats in the core path
- no direct access to secrets unless the host grants and scopes it explicitly
