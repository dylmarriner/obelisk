# Obelisk Hermes plugin

This plugin exposes Obelisk to Hermes Agent as native tools, slash commands, CLI commands, skills, and a best-effort pre-tool hook.

Hermes plugins live under `~/.hermes/plugins/` and are enabled explicitly in Hermes config or through `hermes plugins enable`.

## Requirements

- Hermes Agent installed.
- Obelisk installed on PATH.

Build and install Obelisk:

```bash
cargo build --release
mkdir -p ~/.local/bin
install -m755 target/release/obelisk ~/.local/bin/obelisk
export PATH="$HOME/.local/bin:$PATH"
obelisk doctor
```

## Install

From the Obelisk repo:

```bash
mkdir -p ~/.hermes/plugins
cp -R plugins/hermes-obelisk ~/.hermes/plugins/obelisk
hermes plugins enable obelisk
```

Or enable manually in `~/.hermes/config.yaml`:

```yaml
plugins:
  enabled:
    - obelisk
```

Restart Hermes after installation.

## Tools

The plugin registers these Hermes tools:

```text
obelisk_run
obelisk_pack
obelisk_outline
obelisk_symbol
obelisk_restore
obelisk_rewrite
obelisk_stats
obelisk_doctor
```

## Slash commands

```text
/obelisk
/obelisk-stats
/obelisk-doctor
```

## CLI commands

If the Hermes version supports plugin CLI commands:

```bash
hermes obelisk-doctor
hermes obelisk-stats
```

## Skills

The plugin registers bundled skills when supported by the host:

```text
obelisk:pack-context
obelisk:inspect-symbol
obelisk:compact-output
obelisk:restore-context
```

## Safety model

The `obelisk_run` tool refuses obvious unsafe command patterns before delegating to the Obelisk binary:

- shell pipes, redirects, backgrounding, or command chaining
- secret-looking commands
- mutating Git commands
- dangerous programs like `rm`, `sudo`, `curl`, `bash`, `python`, `node`, and similar
- already-wrapped Obelisk commands

This plugin is for token/context optimisation, not for turning Hermes into a command-injection festival.

## Hook behavior

The plugin registers a best-effort `pre_tool_call` hook. If Hermes exposes a shell-like tool call with a `command` parameter, the hook asks Obelisk whether it should be rewritten through `obelisk run`.

If Hermes ignores the return shape or the hook signature changes, the plugin still works through explicit tools and commands.

## Test

```bash
python -m py_compile plugins/hermes-obelisk/*.py
obelisk doctor
obelisk rewrite git status
```

Then start Hermes and run:

```text
/obelisk
/obelisk-doctor
/obelisk-stats
```
