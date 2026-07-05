# OpenClaw plugin implementation notes

This package follows the OpenClaw tool-plugin direction rather than channel, provider, or CLI backend plugin shapes.

## Why tool plugin

Obelisk is a context firewall and tool layer. It does not own messaging transport, model inference, or a local AI CLI backend.

Therefore the plugin exposes Obelisk capabilities as OpenClaw agent tools:

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

## Important OpenClaw contract points

- package root ships `dist/`, `openclaw.plugin.json`, and `package.json`
- `package.json` points `openclaw.extensions` at `./dist/index.js`
- `openclaw.plugin.json` declares `contracts.tools`
- optional tools are marked in `toolMetadata`
- `typebox` is a runtime dependency
- OpenClaw `>=2026.5.17` is targeted for tool-plugin support
- plugin build/validate should be run after changing tool names or metadata

## Security choices

- `obelisk_run` is optional
- `obelisk_run` is disabled by default in config
- `obelisk_run` is guarded by a `before_tool_call` approval hook
- the command validator refuses risky programs, shell metacharacters, redirects, pipes, secret-looking strings, and mutating Git subcommands

## Future work

- Convert to `defineToolPlugin` if Obelisk stays tool-only and the approval hook is removed.
- Keep `definePluginEntry` if hooks, setup flows, or mixed OpenClaw capabilities are added.
- Add tests once the exact OpenClaw plugin SDK test helpers are available in the consuming workspace.
