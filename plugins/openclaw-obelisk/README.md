# Obelisk OpenClaw plugin

This package exposes Obelisk as a native OpenClaw plugin for agent-callable context firewall tools.

It targets the OpenClaw tool-plugin contract: package metadata in `package.json`, discovery metadata in `openclaw.plugin.json`, and runtime registration through `definePluginEntry`.

## Status

Early native plugin package.

It is intentionally conservative:

- `obelisk_run` is optional and disabled by config by default.
- `obelisk_rewrite` is optional.
- `obelisk_run` also has a `before_tool_call` approval hook.
- the command validator refuses shell metacharacters, pipes, redirects, command chaining, secret-looking commands, mutating Git commands, and dangerous programs.

## Requirements

OpenClaw tool plugin docs currently require:

- Node 22.19+, Node 23.11+, or Node 24+
- TypeScript ESM output
- `typebox` as a runtime dependency
- OpenClaw `>=2026.5.17`
- a package root shipping `dist/`, `openclaw.plugin.json`, and `package.json`

## Tools

The manifest declares these tools in `contracts.tools`:

```text
obelisk_pack
obelisk_outline
obelisk_symbol
obelisk_restore
obelisk_stats
obelisk_doctor
obelisk_rewrite
obelisk_run
```

Optional tools:

```text
obelisk_rewrite
obelisk_run
```

## Configuration

```json
{
  "obeliskBinary": "obelisk",
  "defaultPackBudget": 12000,
  "allowRunTool": false
}
```

Keep `allowRunTool` false unless the workspace is trusted and operator-approved. Even when enabled, `obelisk_run` still applies a read-only command policy and asks for approval through `before_tool_call`.

## Build and validate

```bash
cd plugins/openclaw-obelisk
npm install
npm run check
npm run build
npm run plugin:validate
```

If OpenClaw reports stale metadata, regenerate it:

```bash
npm run plugin:build
```

Then commit both `openclaw.plugin.json` and `package.json` changes.

## Install locally

```bash
openclaw plugins install ./plugins/openclaw-obelisk
openclaw plugins inspect obelisk --runtime
```

Restart or reload the Gateway after install.

## Why this is not a channel/provider/CLI backend

Obelisk is not a messaging channel, not a model provider, and not a local AI CLI backend. It is a context firewall and tool layer.

So this package uses OpenClaw tools:

```text
OpenClaw agent
↓
obelisk_* tool
↓
Obelisk binary
↓
compact output / pack / symbol lookup / restore / stats
```

Trying to shove this into a provider or channel plugin would be impressively wrong, like installing a turbocharger on a kettle.
