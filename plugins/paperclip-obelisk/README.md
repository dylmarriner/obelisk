# Obelisk Paperclip plugin

This package is a Paperclip plugin prototype that exposes Obelisk as a context firewall for Paperclip-managed agents.

Paperclip is a control plane for agent teams. Its expensive context surfaces are task starts and recurring heartbeats: company goals, project state, issue history, comments, prior run logs, skills, and workspace context can be re-sent over and over. This plugin focuses on reducing that repeated context.

## Status

Prototype / early integration.

Paperclip's full plugin system is documented as a post-V1 target architecture with current implementation caveats. This package follows that target shape where possible:

```text
package.json -> paperclipPlugin manifest/worker/ui keys
src/manifest.ts -> plugin manifest, tools, capabilities, UI slots
src/worker.ts -> JSON-RPC-ish worker entrypoint
src/tools.ts -> agent tool handlers
src/obelisk.ts -> Obelisk process helpers and context hashing
src/ui/index.tsx -> dashboard/detail/settings placeholders
```

## What it does

Agent tools:

```text
task-pack
heartbeat-pack
compress-run-output
restore-context
context-diff
savings-report
```

UI slots:

```text
ObeliskSavingsWidget
ObeliskRunDetailTab
ObeliskSettingsPage
```

## Why this exists

Paperclip heartbeats can become expensive when each run reloads the same task/project/company context. Obelisk should help Paperclip send:

```text
compact task capsule
+ changed events since last heartbeat
+ relevant workspace diff
+ restore handles for bulky originals
```

instead of:

```text
everything again, forever, until the token bill becomes a small weather event
```

## Build

```bash
cd plugins/paperclip-obelisk
npm install
npm run check
npm run build
```

## Requirements

- Node.js with TypeScript build tooling.
- Paperclip plugin runtime support.
- Obelisk installed on PATH, or configured through plugin instance config:

```json
{
  "obeliskBinary": "/home/me/.local/bin/obelisk",
  "defaultTaskBudget": 12000,
  "defaultHeartbeatBudget": 8000,
  "allowCommandCompression": true
}
```

## Tool behavior

### `task-pack`

Builds a compact task-start context pack. With a `workspacePath`, it writes the Paperclip task context to a temporary Markdown file and calls:

```bash
obelisk pack --budget <budget> --system <temp-context> --diff --dir <dir> --file <file>
```

Without a workspace path, it falls back to `obelisk squeeze` over the generated context Markdown.

### `heartbeat-pack`

Builds a compact heartbeat pack with:

```text
current context hash
previous context hash
changed event count
changed events
candidate Paperclip context
optional workspace diff/files/dirs
```

This is the key token saver. Heartbeats should be deltas, not the full bureaucratic ancestry of the robot employee.

### `compress-run-output`

Runs large Paperclip run logs/tool outputs through `obelisk squeeze`.

### `restore-context`

Runs:

```bash
obelisk restore <handle>
```

### `context-diff`

Computes stable hashes and a shallow changed-key list for Paperclip context objects.

### `savings-report`

Runs:

```bash
obelisk stats
```

## Capabilities requested

```text
agent.tools.register
project.workspaces.read
plugin.state.read
plugin.state.write
activity.log.write
ui.dashboardWidget.register
ui.detailTab.register
ui.page.register
```

## Integration direction

The production version should eventually:

1. Store last heartbeat hashes in Paperclip plugin state.
2. Save per-agent/per-project savings metrics.
3. Add a real dashboard widget through Paperclip's plugin UI bridge.
4. Attach Obelisk summaries to run logs.
5. Add settings for redaction and command-compression policy.
6. Add tests against Paperclip's official plugin SDK once stable.

## Safety

This plugin does not directly run arbitrary user shell commands. It focuses on context packing, output squeezing, restore handles, and savings stats.

For command execution, use explicit Paperclip governance and Obelisk's own safe command policy. Do not accidentally turn a cost-saving plugin into an unattended terminal with delusions of competence.
