<p align="center">
  <a href="./README.md"><img src="https://img.shields.io/badge/docs-setup-informational?style=flat-square" alt="Setup"></a>
  <a href="../README.md"><img src="https://img.shields.io/badge/←%20back-readme-blue?style=flat-square" alt="Back"></a>
</p>

# Setup Guide

**From clone to working agent hooks in one guide.** Covers building from source, PATH persistence, agent hook installation, smoke tests, and Token Optimizer setup.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Clone and Build](#clone-and-build)
- [Install the Binary](#install-the-binary)
- [Install Agent Hooks](#install-agent-hooks)
- [Smoke Tests](#smoke-tests)
- [Recommended Daily Workflow](#recommended-daily-workflow)
- [Token Optimizer Setup (Hermes Plugin)](#token-optimizer-setup-hermes-plugin)
- [Self-Improvement Setup](#self-improvement-setup)
- [Updating Obelisk](#updating-obelisk)
- [Quick Recovery](#quick-recovery)

---

## Prerequisites

| Dependency | Version | Notes |
|-----------|---------|-------|
| Rust toolchain | stable | `cargo`, `rustc`, `rustfmt` |
| Git | any | For cloning |
| Agent CLI(s) | latest | Claude Code, Codex, Hermes, OpenCode, OpenClaw, or Cline |

Install Rust if needed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
```

On Ubuntu/Kubuntu, install build essentials:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config git curl
```

---

## Clone and Build

```bash
git clone https://github.com/dylmarriner/obelisk.git
cd obelisk

cargo fmt
cargo test
cargo build --release
```

The compiled binary will be at:

```
target/release/obelisk
```

---

## Install the Binary

```bash
mkdir -p ~/.local/bin
install -m755 target/release/obelisk ~/.local/bin/obelisk
```

### Add to PATH

For the current shell:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Make it permanent:

**Bash:**
```bash
grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' ~/.bashrc || \
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

**Zsh:**
```bash
grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' ~/.zshrc || \
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

### Verify

```bash
which obelisk
obelisk doctor
```

Expected output: `obelisk doctor` shows the binary path, ledger status, store counts, and supported agent install targets.

---

## Install Agent Hooks

Install only the agents you actually use:

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
obelisk install openclaw
obelisk install cline
```

### What each agent gets

| Agent | Integration Type |
|-------|-----------------|
| Claude Code | Hook wiring where supported |
| Codex | Hook wiring where supported |
| Hermes | Awareness file + plugin wiring |
| OpenCode | Plugin wiring |
| OpenClaw | Awareness rules |
| Cline | `.clinerules` guidance (no global shell hook) |

After installing hooks, **restart the agent**.

---

## Smoke Tests

Run these from inside the Obelisk repo or any test project:

### 1. Test rewrite logic

```bash
obelisk rewrite git status
```

Expected output:

```
obelisk run git status
```

### 2. Run compressed commands

```bash
obelisk run git status
obelisk run cargo build
obelisk stats
```

### 3. Test symbol retrieval

```bash
obelisk outline src/main.rs
obelisk symbol src/main.rs run
```

### 4. Test context packing

```bash
obelisk pack --budget 12000 --diff --dir src --file README.md
```

### 5. Write a pack to disk

```bash
mkdir -p .obelisk
obelisk pack --budget 8000 --diff --dir src --file Cargo.toml --out .obelisk/context.md
```

---

## Recommended Daily Workflow

Use agents normally, but nudge them toward Obelisk patterns:

| Pattern | Command | Benefit |
|---------|---------|---------|
| Noisy commands | `obelisk run <cmd>` | Compact, reversible output |
| Large files | `obelisk outline <file>` | Symbols without content overhead |
| One function/class | `obelisk symbol <file> <name>` | Minimal token cost |
| Project context | `obelisk pack --budget <n>` | Provider-neutral context bundle |
| Restore only when needed | `obelisk restore <handle>` | Avoids undoing compression |

---

## Token Optimizer Setup (Hermes Plugin)

The Hermes plugin's Token Optimizer features (context nudge, per-turn tally, session rollup, dashboard) are optional and require an external repo.

### 1. Clone the Token Optimizer repo

```bash
git clone https://github.com/alexgreensh/token-optimizer.git ~/Documents/token-optimizer
```

### 2. Verify the bridge works

```bash
cd ~/.hermes/plugins/obelisk
python3 hermes_hook_bridge.py
```

Expected output:

```
[OK] measure.py found at /home/me/Documents/token-optimizer/scripts/measure.py
```

If the bridge reports `[WARN] measure.py not found`, the dashboard, nudge, and rollup features are paused. Either clone the repo or check the `measure-path` locator file.

---

## Self-Improvement Setup

Self-improvement is **disabled by default**. Leave it that way until you read [SELF_IMPROVEMENT.md](SELF_IMPROVEMENT.md).

```bash
# Check status
obelisk learn status

# View pending gaps
obelisk learn gaps

# Enable (read SELF_IMPROVEMENT.md first!)
obelisk learn enable /path/to/obelisk --threshold 15

# Disable
obelisk learn disable
```

> ⚠️ **Risk:** The current script can commit and push to `main` after passing build and tests. Review the script before enabling.

---

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

---

## Quick Recovery

If a hook breaks an agent session:

```bash
which obelisk
obelisk doctor
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

---

<p align="center"><a href="./README.md">← Documentation Index</a> · <a href="../README.md">Back to README</a></p>
