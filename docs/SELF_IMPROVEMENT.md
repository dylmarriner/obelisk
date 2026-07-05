# Self-improvement

Obelisk has an optional usage-triggered self-improvement loop. It watches real usage gaps, asks an agent to make one minimal fix, gates the result on Rust build/tests, and can commit the result.

Read this before enabling it. Seriously. This is the part where a shell script can ask an agent to edit your repo. Very futuristic, in the same way a chainsaw is technically a productivity tool.

## Status

Self-improvement is disabled by default.

Check status:

```bash
obelisk learn status
```

Enable:

```bash
obelisk learn enable /path/to/obelisk --threshold 15
```

Disable:

```bash
obelisk learn disable
```

Show pending gaps:

```bash
obelisk learn gaps
```

## What is a gap?

A gap is a sign that Obelisk lacks coverage or correctness in some area.

Current examples:

- `no_filter` — a command was routed through Obelisk but has no dedicated filter.
- `restore_miss` — a restore handle could not be found.

A common gap path is:

```text
obelisk run some-tool
↓
no dedicated filter exists
↓
Obelisk records `no_filter` for `some-tool`
↓
if enough gaps pile up, self-improvement may trigger
```

## Current implementation flow

The current script is `scripts/self-improve.sh`.

Current behavior:

1. Refuses to run on a dirty working tree.
2. Fetches `origin/main`.
3. Fast-forwards local `main` only if possible.
4. Reads pending gaps with `obelisk learn gaps`.
5. Calls Claude Code headless with a prompt to fix exactly one highest-count gap.
6. Runs `cargo build --release`.
7. Runs `cargo test`.
8. If build/tests pass, commits and pushes to `main`.
9. If build/tests fail, reverts the working tree.

## Important safety warning

The current checked-in script can push directly to `main` after passing gates.

That is powerful, but risky. Passing tests does not prove the patch is correct, secure, minimal, or architecturally sane. It only proves the tests noticed nothing. Tests, bless them, are not omniscient.

Recommended operating rule:

```text
Do not enable self-improvement on a repo unless you are comfortable with the current script's autonomy model.
```

## Known issue in current flow

There is a sequencing problem in the current design: the Rust side can mark gaps as triggered before the shell script reads them. If that happens, the script may see no pending gaps.

Recommended v2 design:

1. Generate a frozen gap snapshot first.
2. Pass that snapshot file to the script.
3. Mark gaps as triggered only after the snapshot exists.
4. Have the script read the snapshot, not the live pending queue.
5. Push a review branch instead of `main`.
6. Open a PR when `gh` is available.

A v2 patch has been drafted separately. Merge that before treating the self-improvement loop as production-safe.

## Safer target behavior

The safer loop should be:

```text
real usage gaps
↓
frozen gap report with representative samples
↓
agent fixes one highest-count gap
↓
cargo build --release
↓
cargo test
↓
commit to self-improve/<timestamp>-<gap>
↓
push branch
↓
open PR
↓
human review
```

That is still autonomous, but it leaves an audit trail and review step. A tiny concession to sanity.

## Recommended local safeguards

Before enabling:

```bash
cd /path/to/obelisk
git status
cargo test
cargo build --release
```

Make sure your local `main` tracks origin cleanly:

```bash
git checkout main
git fetch origin main
git merge --ff-only origin/main
```

Keep logs visible:

```bash
tail -f .self-improve.log
```

Use a high threshold at first:

```bash
obelisk learn enable /path/to/obelisk --threshold 50
```

Lower it later only after reviewing several runs.

## How to inspect gaps

```bash
obelisk learn status
obelisk learn gaps
```

The current compact gaps view groups by kind/program/count. The safer v2 design should include representative samples so the repair agent can write filters from real output instead of just a count.

## What self-improvement should fix

Good candidates:

- missing command filters
- overly noisy output for known tools
- restore miss bugs
- parser edge cases with tests
- small compression correctness issues

Bad candidates:

- major architecture refactors
- dependency rewrites
- provider-specific model logic
- secret handling
- deployment automation
- anything requiring product judgement

## Disable immediately

```bash
obelisk learn disable
```

If a run is active, remove the lock only after checking that no `self-improve.sh` process is running:

```bash
ps aux | grep self-improve.sh | grep -v grep
rm -f ~/.local/share/obelisk/self-improve.lock
```

The exact data directory can vary by platform.

## Future improvements

Recommended next upgrades:

- frozen gap snapshots
- representative gap samples
- branch/PR workflow instead of direct `main` pushes
- dry-run mode
- allowlist of files the agent may edit
- maximum diff size
- GitHub Actions validation before review
- automatic issue creation for repeated failures
- better gap taxonomy: `bad_filter`, `overcompressed`, `undercompressed`, `parse_miss`, `restore_miss`, `agent_hook_error`
