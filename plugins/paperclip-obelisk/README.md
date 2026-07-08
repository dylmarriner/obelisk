<p align="center">
  <a href="../../README.md"><img src="https://img.shields.io/badge/plugin-paperclip--obelisk-orange?style=flat-square" alt="Paperclip Plugin"></a>
  <a href="../../docs/README.md"><img src="https://img.shields.io/badge/docs-paperclip%20plugin-informational?style=flat-square" alt="Plugin Docs"></a>
</p>

# Obelisk Paperclip Plugin

**Prototype: Obelisk as a context firewall for Paperclip-managed agent teams.** Targets the expensive context surfaces in agent orchestration — repeated task starts and recurring heartbeats.

---

## Table of Contents

- [Status](#status)
- [What It Does](#what-it-does)
- [Build](#build)
- [Requirements](#requirements)
- [Tools](#tools)
- [Capabilities Requested](#capabilities-requested)
- [Integration Direction](#integration-direction)
- [Safety](#safety)

---

## Status

**Prototype / early integration.** Paperclip's full plugin system is documented as a post-V1 target architecture. This package follows that target shape where possible:

```
package.json             → paperclipPlugin manifest/worker/ui keys
src/manifest.ts          → plugin manifest, tools, capabilities, UI slots
src/worker.ts            → JSON-RPC-ish worker entrypoint
src/tools.ts             → agent tool handlers
src/obelisk.ts           → Obelisk process helpers and context hashing
src/ui/index.tsx         → dashboard/detail/settings placeholders
```

---

## What It Does

Paperclip heartbeats can become expensive when each run reloads the same task, project, and company context. Obelisk helps Paperclip send:

```
compact task capsule
+ changed events since last heartbeat
+ relevant workspace diff
+ restore handles for bulky originals
```

instead of:

```
everything again, forever
```

---

## Build

```bash
cd plugins/paperclip-obelisk
npm install
npm run check
npm run build
```

---

## Requirements

| Dependency | Purpose |
|-----------|---------|
| Node.js | TypeScript build tooling |
| Paperclip plugin runtime | Plugin host |
| Obelisk on PATH | Core engine |

**Optional configuration:**

```json
{
  "obeliskBinary": "/home/me/.local/bin/obelisk",
  "defaultTaskBudget": 12000,
  "defaultHeartbeatBudget": 8000,
  "allowCommandCompression": true
}
```

---

## Tools

| Tool | Description |
|------|-------------|
| `task-pack` | Build a compact task-start context pack using `obelisk pack` |
| `heartbeat-pack` | Delta-based heartbeat pack: context hashes, changed events, workspace diff |
| `compress-run-output` | Squeeze large Paperclip run logs/tool output |
| `restore-context` | Restore via `obelisk restore <handle>` |
| `context-diff` | Stable hashes and changed-key list for Paperclip context objects |
| `savings-report` | Token savings via `obelisk stats` |

### Tool Details

**`task-pack`** — With a `workspacePath`, writes task context to a temp file and calls:

```bash
obelisk pack --budget <budget> --system <temp-context> --diff --dir <dir> --file <file>
```

Without workspace path, falls back to `obelisk squeeze` over the context Markdown.

**`heartbeat-pack`** — The key token saver. Sends context hash, previous hash, changed event count, changed events, and optional workspace diff/files/dirs. Heartbeats should be deltas, not full bureaucracy.

---

## Capabilities Requested

```
agent.tools.register
project.workspaces.read
plugin.state.read
plugin.state.write
activity.log.write
ui.dashboardWidget.register
ui.detailTab.register
ui.page.register
```

---

## Integration Direction

The production version should eventually:

1. Store last heartbeat hashes in Paperclip plugin state
2. Save per-agent/per-project savings metrics
3. Add a real dashboard widget through Paperclip's plugin UI bridge
4. Attach Obelisk summaries to run logs
5. Add settings for redaction and command-compression policy
6. Add tests against Paperclip's official plugin SDK once stable

---

## Safety

This plugin does not directly run arbitrary user shell commands. It focuses on context packing, output squeezing, restore handles, and savings stats. For command execution, use explicit Paperclip governance and Obelisk's safe command policy.

---

<p align="center"><a href="../../README.md">← Back to README</a></p>
