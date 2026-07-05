# Security notes

The OpenClaw plugin is conservative by default.

## Defaults

- `obelisk_run` is optional.
- `obelisk_run` is disabled unless `allowRunTool` is explicitly set.
- `obelisk_run` requests runtime approval through `before_tool_call`.
- command validation rejects shell metacharacters, redirects, pipes, backgrounding, command chaining, secret-looking strings, mutating Git subcommands, and risky programs.

## Recommended policy

Keep command compression focused on read-heavy diagnostics:

```text
git status
cargo build
cargo test
rg TODO src
```

Do not use Obelisk to run deployment, credential, network, destructive, or interactive commands through OpenClaw tools.
