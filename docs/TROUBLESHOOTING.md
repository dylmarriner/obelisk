# Troubleshooting

This page covers the boring failures that waste the most time. Computers are deterministic until they meet user configs, shell PATHs, and agent hook systems, at which point everyone starts pretending YAML is a personality test.

## `obelisk: command not found`

Check where the binary is installed:

```bash
ls -l ~/.local/bin/obelisk ~/.cargo/bin/obelisk /usr/local/bin/obelisk 2>/dev/null
```

Check PATH:

```bash
echo "$PATH" | tr ':' '\n' | grep -E 'local/bin|cargo/bin'
```

Fix for current shell:

```bash
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
```

Persist for Bash:

```bash
echo 'export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Verify:

```bash
which obelisk
obelisk doctor
```

## `obelisk doctor` fails ledger check

The ledger lives under your platform data directory, usually something like:

```bash
~/.local/share/obelisk/ledger.db
```

Check permissions:

```bash
ls -ld ~/.local/share ~/.local/share/obelisk 2>/dev/null
ls -l ~/.local/share/obelisk/ledger.db 2>/dev/null
```

Fix common permission issue:

```bash
mkdir -p ~/.local/share/obelisk
chmod 700 ~/.local/share/obelisk
```

If the DB is corrupted and you accept losing stored restore handles:

```bash
mv ~/.local/share/obelisk/ledger.db ~/.local/share/obelisk/ledger.db.bak.$(date +%s)
obelisk doctor
```

## Agent still calls RTK

Find stale RTK references:

```bash
grep -Rni "rtk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

Remove common plugin files:

```bash
rm -rf ~/.hermes/plugins/rtk-rewrite
rm -f ~/.config/opencode/plugins/rtk.ts
```

Remove binary:

```bash
cargo uninstall rtk 2>/dev/null || true
cargo uninstall rtk-cli 2>/dev/null || true
rm -f ~/.cargo/bin/rtk ~/.local/bin/rtk
sudo rm -f /usr/local/bin/rtk 2>/dev/null || true
```

Then install Obelisk hooks:

```bash
obelisk install claude
obelisk install codex
obelisk install opencode
obelisk install hermes
```

## Agent does not rewrite commands

Check Obelisk rewrite directly:

```bash
obelisk rewrite git status
obelisk rewrite cargo build
obelisk rewrite git push
```

Expected:

- `git status` and `cargo build` should print `obelisk run ...`
- `git push` should print nothing and exit non-zero

Check hook installation:

```bash
grep -Rni "obelisk" ~/.claude ~/.config/opencode ~/.codex ~/.hermes .clinerules 2>/dev/null || true
```

Restart the agent after installing hooks. Some agents read config only on startup, because apparently live reload was too humane.

## `obelisk run` makes small output bigger

Obelisk tries to avoid adding restore pointers when they cost more than they save, but some small outputs are already optimal.

Check stats:

```bash
obelisk run git status
obelisk stats
```

For tiny commands, do not worry about it. The major wins are noisy builds, logs, package managers, cloud CLI JSON, search output, and whole-file avoidance through `outline`/`symbol`.

## Restore handle not found

Try:

```bash
obelisk restore <handle>
```

If missing:

- the ledger may have been garbage-collected
- the ledger DB may have been moved or deleted
- the handle may come from another machine/user
- the original output may never have been stashed because compression was not worth it

Check store counts:

```bash
obelisk doctor
obelisk stats
```

## `obelisk pack` output is too small

Increase budget:

```bash
obelisk pack --budget 24000 --diff --dir src --file README.md
```

Add explicit files:

```bash
obelisk pack --budget 16000 --diff --dir src --file Cargo.toml --file src/main.rs
```

Remember: `--dir` creates a directory map. It does not dump every file. This is on purpose. Whole-project dumps are how token budgets go to die.

## `obelisk pack` output is too large

Lower budget:

```bash
obelisk pack --budget 6000 --diff --dir src
```

Avoid large explicit files unless needed:

```bash
obelisk outline src/large_file.rs
obelisk symbol src/large_file.rs function_name
```

## Build fails

Refresh dependencies and toolchain:

```bash
rustup update
cargo clean
cargo fmt
cargo test
cargo build --release
```

Show Rust versions:

```bash
rustc --version
cargo --version
```

## `cargo test` fails after local changes

Reset local changes if you do not need them:

```bash
git status
git checkout -- .
git clean -fd
cargo test
```

If you do need them, stash first:

```bash
git stash push -u -m "before obelisk troubleshooting"
cargo test
```

## Self-improvement did nothing

Check status and gaps:

```bash
obelisk learn status
obelisk learn gaps
```

Check log:

```bash
tail -n 200 .self-improve.log
```

Common reasons:

- learning is disabled
- threshold not reached
- working tree is dirty
- local main diverged from origin/main
- `claude` CLI is not on PATH
- no pending gaps existed by the time script ran
- build or tests failed and changes were reverted

Current self-improvement behavior has known safety/design caveats. Read `docs/SELF_IMPROVEMENT.md` before relying on it.

## Nuclear reinstall

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
