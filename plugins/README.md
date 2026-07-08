<p align="center">
  <a href="../../README.md"><img src="https://img.shields.io/badge/obelisk-v1.0.0-4A90D9?style=flat-square" alt="Obelisk"></a>
  <a href="../../docs/README.md"><img src="https://img.shields.io/badge/docs-plugins-informational?style=flat-square" alt="Plugins"></a>
</p>

# Plugin Packages

**Thin adapters that bridge the Obelisk engine with agent-specific runtimes.** Each plugin targets a specific agent or control plane while preserving Obelisk as the core engine.

---

## Available Packages

| Package | Target | Status | Purpose |
|---------|--------|--------|---------|
| [claude-code-obelisk](./claude-code-obelisk/README.md) | Claude Code | Local plugin package | Hooks, skills, and context optimizer agent |
| [hermes-obelisk](./hermes-obelisk/README.md) | Hermes Agent | Native plugin package | Tools, hooks, slash commands, CLI commands, skills, and Token Optimizer integration |
| [paperclip-obelisk](./paperclip-obelisk/README.md) | Paperclip | Prototype | Task-start and heartbeat context packing, output compression, restore handles, savings UI |

---

## Architecture

```
agent / runtime / control plane
        ↓
  thin plugin adapter
        ↓
   obelisk binary
        ↓
local ledger + compression + packing + restore handles
```

---

## Design Rules

| Rule | Rationale |
|------|-----------|
| **Plugins are adapters** | Obelisk remains the engine. Do not reimplement its logic inside a plugin. |
| **Conservative defaults** | Read-heavy commands only. No destructive shell rewriting. No credential scraping. |
| **No blind restoration** | Avoid restoring large blobs into context unless explicitly requested. |
| **No provider-specific formats** | Context pack format stays model-agnostic in the core path. |
| **Secrets isolation** | No direct access to secrets unless the host grants and scopes it explicitly. |

---

<p align="center"><a href="../../README.md">← Back to README</a></p>
