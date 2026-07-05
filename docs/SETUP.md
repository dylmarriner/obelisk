# Obelisk setup help

This guide takes you from clone to working agent hooks. It also covers removing RTK so your coding setup stops calling two different token layers like a confused plumbing system.

## Requirements

- Linux, macOS, or WSL. Linux is the most tested path.
- Rust toolchain: `cargo`, `rustc`, and `rustfmt`.
- Git.
- The agent CLIs you want to wire in, such as Claude Code, Codex, Hermes, OpenCode, OpenClaw, or Cline.

Install Rust if needed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
```

On Ubuntu/Kubuntu, install build basics:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config git curl
```

## Clone and build

```bash
git clone https://github.com/dylmarriner/obelisk.git
cd obelisk

cargo fmt
cargo test
cargo build --release
```

The compiled binary will be at:

```bash
target/release/obelisk
```

## Install the binary

```bash
mkdir -p ~/.local/bin
install -m755 target/release/obelisk ~/.local/bin/obelisk
```

Make sure `~/.local/bin` is in your shell path:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Persist it for Bash:

```bash
grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' ~/.bashrc || \
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

Persist it for Zsh:

```bash
grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' ~/.zshrc || \
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

Verify:

```bash
which obelisk
obelisk doctor
```

Expected result: `obelisk doctor` should show the binary path, ledger status, store counts, and supported agent install targets.

## Remove RTK first if you used it

Do this before installing Obelisk hooks. If your agent config still calls `rtk` after you delete the binary, your hooks will fail. Yes, order matters. Machines are petty.

Find RTK:

```bash
which rtk || true
rtk --version 2>/dev/null || true
```

Find RTK references in common agent configs:

```bash
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

Back up configs:

```bash
mkdir -p ~/obelisk-agent-config-backups
cp -a ~/.claude ~/obelisk-agent-config-backups/claude 2>/dev/null || true
cp -a ~/.config/opencode ~/obelisk-agent-config-backups/opencode 2>/dev/null || true
cp -a ~/.codex ~/obelisk-agent-config-backups/codex 2>/dev/null || true
cp -a ~/.hermes ~/obelisk-agent-config-backups/hermes 2>/dev/null || true
cp -a .clinerules ~/obelisk-agent-config-backups/clinerules 2>/dev/null || true
```

Remove RTK plugin files if present:

```bash
rm -rf ~/.hermes/plugins/rtk-rewrite
rm -f ~/.config/opencode/plugins/rtk.ts
```

Remove the RTK binary:

```bash
cargo uninstall rtk 2>/dev/null || true
cargo uninstall rtk-cli 2>/dev/null || true
rm -f ~/.cargo/bin/rtk ~/.local/bin/rtk
sudo rm -f /usr/local/bin/rtk 2>/dev/null || true
rm -rf ~/.config/rtk ~/.local/share/rtk
```

Check that nothing still references it:

```bash
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
which rtk || true
```

## Install Obelisk agent hooks

Install only the agents you actually use:

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

What this does:

- Claude Code and Codex get hook wiring where supported.
- Hermes gets an awareness file and plugin wiring.
- OpenCode gets plugin wiring.
- OpenClaw gets awareness rules.
- Cline gets `.clinerules` guidance because Cline does not expose the same kind of global shell hook.

After installing hooks, restart the agent.

## Smoke tests

Run these from inside the Obelisk repo or any test project:

```bash
obelisk rewrite git status
```

Expected output:

```bash
obelisk run git status
```

Run a compressed command:

```bash
obelisk run git status
obelisk run cargo build
obelisk stats
```

Test symbol retrieval:

```bash
obelisk outline src/main.rs
obelisk symbol src/main.rs run
```

Test model-agnostic input packing:

```bash
obelisk pack --budget 12000 --diff --dir src --file README.md
```

Write a context pack to disk:

```bash
mkdir -p .obelisk
obelisk pack --budget 8000 --diff --dir src --file Cargo.toml --out .obelisk/context.md
```

## Recommended daily workflow

Use agents normally, but nudge them toward Obelisk patterns:

```text
Use `obelisk run` for noisy commands.
Use `obelisk outline` before reading big source files.
Use `obelisk symbol <file> <name>` for targeted code retrieval.
Use `obelisk pack` before sending a large project context to a model.
Use `obelisk restore <handle>` only when the compressed view is not enough.
```

## Self-improvement setup

Self-improvement is disabled by default. Leave it that way until you read [SELF_IMPROVEMENT.md](SELF_IMPROVEMENT.md).

Current command:

```bash
obelisk learn enable /path/to/obelisk --threshold 15
```

Check status:

```bash
obelisk learn status
obelisk learn gaps
```

Disable:

```bash
obelisk learn disable
```

Important: the current script can commit and push to `main` after passing build and tests. That is powerful and risky. Review the script before enabling it. Software autonomy is not magic; it is just shell scripts with confidence.

## Updating Obelisk

```bash
cd /path/to/obelisk
git checkout main
git pull
cargo fmt
cargo test
cargo build --release
install -m755 target/release/obelisk ~/.local/bin/obelisk
obelisk doctor
```

Restart any agent that has Obelisk hooks loaded.

## Quick recovery

If a hook breaks an agent session:

```bash
which obelisk
obelisk doctor
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

If needed, restore your config backups from `~/obelisk-agent-config-backups`.
