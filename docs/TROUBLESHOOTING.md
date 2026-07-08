<p align="center">
  <a href="./README.md"><img src="https://img.shields.io/badge/docs-troubleshooting-informational?style=flat-square" alt="Troubleshooting"></a>
  <a href="../README.md"><img src="https://img.shields.io/badge/←%20back-readme-blue?style=flat-square" alt="Back"></a>
</p>

# Troubleshooting

**Common failures and their fixes.** If something is not working, start with the quick diagnostic below, then find the relevant section.

---

## Quick Diagnostic

```bash
which obelisk
obelisk doctor
obelisk rewrite git status
obelisk stats
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

---

## Table of Contents

- [obelisk: command not found](#obelisk-command-not-found)
- [obelisk doctor fails ledger check](#obelisk-doctor-fails-ledger-check)
- [Agent does not rewrite commands](#agent-does-not-rewrite-commands)
- [Compressed output is larger than original](#compressed-output-is-larger-than-original)
- [Restore handle not found](#restore-handle-not-found)
- [Pack output is too small](#pack-output-is-too-small)
- [Pack output is too large](#pack-output-is-too-large)
- [Build fails](#build-fails)
- [Tests fail after local changes](#tests-fail-after-local-changes)
- [Self-improvement did nothing](#self-improvement-did-nothing)
- [Token Optimizer hooks not working](#token-optimizer-hooks-not-working-hermes-plugin)
- [Nuclear reinstall](#nuclear-reinstall)

---

## obelisk: command not found

### Check installation

```bash
ls -l ~/.local/bin/obelisk ~/.cargo/bin/obelisk /usr/local/bin/obelisk 2>/dev/null
```

### Check PATH

```bash
echo "$PATH" | tr ':' '\n' | grep -E 'local/bin|cargo/bin'
```

### Fix for current shell

```bash
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
```

### Persist for Bash

```bash
echo 'export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Verify

```bash
which obelisk
obelisk doctor
```

---

## obelisk doctor fails ledger check

The ledger lives under your platform data directory:

```bash
~/.local/share/obelisk/ledger.db
```

### Check permissions

```bash
ls -ld ~/.local/share ~/.local/share/obelisk 2>/dev/null
ls -l ~/.local/share/obelisk/ledger.db 2>/dev/null
```

### Fix permission issue

```bash
mkdir -p ~/.local/share/obelisk
chmod 700 ~/.local/share/obelisk
```

### If DB is corrupted

Move it aside (you will lose stored restore handles):

```bash
mv ~/.local/share/obelisk/ledger.db ~/.local/share/obelisk/ledger.db.bak.$(date +%s)
obelisk doctor
```

---

---

## Agent does not rewrite commands

### Test rewrite directly

```bash
obelisk rewrite git status
obelisk rewrite cargo build
obelisk rewrite git push
```

**Expected:**
- `git status` and `cargo build` → prints `obelisk run ...` and exits 0
- `git push` → exits 1 with no output

### Check hook installation

```bash
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

**Restart the agent.** Most agents read config only on startup.

---

## Compressed output is larger than original

Obelisk avoids adding restore pointers when they cost more than they save, but some small outputs are already optimal.

### Check stats

```bash
obelisk run git status
obelisk stats
```

For tiny commands, the overhead is negligible. The major wins are noisy builds, logs, package managers, cloud CLI JSON, search output, and whole-file avoidance through `outline`/`symbol`.

---

## Restore handle not found

```bash
obelisk restore <handle>
```

If the handle is missing:

- The ledger may have been garbage-collected (`obelisk gc`)
- The ledger DB may have been moved or deleted
- The handle may come from another machine or user
- The original output may never have been stashed (compression worth less than storage)

### Check store counts

```bash
obelisk doctor
obelisk stats
```

---

## Pack output is too small

### Increase budget

```bash
obelisk pack --budget 24000 --diff --dir src --file README.md
```

### Add explicit files

```bash
obelisk pack --budget 16000 --diff --dir src --file Cargo.toml --file src/main.rs
```

> **Note:** `--dir` creates a directory map — it does not dump every file into context. This is intentional. Whole-project dumps defeat the purpose of token optimization.

---

## Pack output is too large

### Lower budget

```bash
obelisk pack --budget 6000 --diff --dir src
```

### Avoid large explicit files unless needed

```bash
obelisk outline src/large_file.rs
obelisk symbol src/large_file.rs function_name
```

---

## Build fails

### Refresh dependencies and toolchain

```bash
rustup update
cargo clean
cargo fmt
cargo test
cargo build --release
```

### Show current Rust versions

```bash
rustc --version
cargo --version
```

---

## Tests fail after local changes

### If you don't need the changes

```bash
git status
git checkout -- .
git clean -fd
cargo test
```

### If you need the changes

```bash
git stash push -u -m "before obelisk troubleshooting"
cargo test
```

---

## Self-improvement did nothing

### Check status and gaps

```bash
obelisk learn status
obelisk learn gaps
```

### Check the log

```bash
tail -n 200 .self-improve.log
```

### Common reasons

| Reason | Check |
|--------|-------|
| Learning is disabled | `obelisk learn status` |
| Threshold not reached | Raise gap count |
| Working tree is dirty | `git status` |
| Local main diverged from origin | `git merge --ff-only origin/main` |
| `claude` CLI not on PATH | `which claude` |
| No pending gaps by run time | Gaps may have been consumed |
| Build/tests failed | Changes were reverted automatically |

> **See also:** [Self-Improvement docs](SELF_IMPROVEMENT.md) for safety and design caveats.

---

## Token Optimizer hooks not working (Hermes plugin)

The Token Optimizer features (context nudge, per-turn tally, dashboard) are optional and require an external repo.

### Check the bridge

```bash
cd ~/.hermes/plugins/obelisk
python3 hermes_hook_bridge.py
```

If `measure.py not found`:

```bash
git clone https://github.com/alexgreensh/token-optimizer.git ~/Documents/token-optimizer
```

### Check the locator file

```bash
cat ~/.hermes/plugins/obelisk/measure-path
```

If the locator is stale or points to a moved checkout, update it to the correct path or remove it so the bridge falls back to the standard location (`~/Documents/token-optimizer/scripts/measure.py`).

### Nudge not appearing

The nudge fires at most **once per session** at ~70%+ fill. If you see no nudge:
- It may already have fired earlier in the session
- The session may not have crossed the threshold
- Run a longer session or fill context to trigger it

---

## Nuclear Reinstall

```bash
cd /path/to/obelisk
git checkout main
git pull
git clean -fdx
cargo build --release
install -m755 target/release/obelisk ~/.local/bin/obelisk
obelisk doctor
```

Then reinstall only the hooks you need:

```bash
obelisk install claude
obelisk install codex
```

---

<p align="center"><a href="./README.md">← Documentation Index</a> · <a href="../README.md">Back to README</a></p>
