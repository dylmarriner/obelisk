#!/usr/bin/env sh
# Obelisk self-improvement loop.
#
# Invoked by `obelisk` itself (src/learn.rs::try_trigger) once enough usage
# gaps have piled up. Never invoked directly by a human in normal operation,
# but safe to run by hand for testing: scripts/self-improve.sh <repo> <lockfile>
#
# What it does, in order:
#   1. pulls main (fast-forward only — refuses to run on a dirty/diverged tree)
#   2. asks Claude Code (headless) to read the pending gaps and implement ONE
#      fix for the most frequent one
#   3. gates the result on `cargo build --release && cargo test`
#   4. only if both pass: commits and pushes straight to main
#   5. on any failure at any step: discards the working tree changes and exits
#
# This is the ONLY place in the whole project that commits or pushes on its
# own. Read it before changing the autonomy model.

set -u
REPO="${1:?repo path required}"
LOCK="${2:?lock file required}"
LOG="$REPO/.self-improve.log"

cleanup() { rm -f "$LOCK"; }
trap cleanup EXIT

log() { printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$1" >> "$LOG"; }

cd "$REPO" || { log "FATAL: cannot cd to $REPO"; exit 1; }

if [ -n "$(git status --porcelain)" ]; then
    log "ABORT: working tree dirty, refusing to touch it"
    exit 1
fi

git fetch origin main >> "$LOG" 2>&1
if ! git merge --ff-only origin/main >> "$LOG" 2>&1; then
    log "ABORT: local main has diverged from origin/main, refusing to guess"
    exit 1
fi

GAPS="$(obelisk learn gaps 2>/dev/null)"
if [ -z "$GAPS" ] || [ "$GAPS" = "[]" ]; then
    log "ABORT: no pending gaps to act on"
    exit 0
fi
log "gaps: $GAPS"

PROMPT="You are improving the Obelisk Rust CLI (a token-compression engine for AI agents) based on real usage data.

Pending coverage/correctness gaps, most frequent first:
$GAPS

Pick exactly ONE gap — the highest count — and implement a minimal, correct fix:
- 'no_filter' gap on program X: add a dedicated filter for X in src/filters.rs (follow the existing pattern: a small function that keeps errors/results/changed-paths and drops noise, wired into the apply() match and added to is_covered()). Add at least one unit test in the existing #[cfg(test)] module proving it compresses without dropping signal.
- 'restore_miss' gap: investigate why ledger::restore returned None for that handle and fix the root cause in src/ledger.rs, with a regression test.

Constraints:
- Touch only what's needed for this one fix. No refactors, no unrelated cleanup, no new dependencies unless unavoidable.
- Do not run git commands yourself — committing and pushing is handled outside this session.
- Do not edit scripts/self-improve.sh or .self-improve.log.
- When done, stop. Do not ask questions; make the best reasonable call and proceed."

if ! command -v claude >/dev/null 2>&1; then
    log "ABORT: claude CLI not found on PATH"
    exit 1
fi

claude -p "$PROMPT" --dangerously-skip-permissions >> "$LOG" 2>&1
CLAUDE_RC=$?
log "claude exit code: $CLAUDE_RC"

if [ -z "$(git status --porcelain)" ]; then
    log "ABORT: no changes produced"
    exit 0
fi

if ! cargo build --release >> "$LOG" 2>&1; then
    log "REVERT: cargo build failed"
    git checkout -- . >> "$LOG" 2>&1
    git clean -fd >> "$LOG" 2>&1
    exit 1
fi

if ! cargo test >> "$LOG" 2>&1; then
    log "REVERT: cargo test failed"
    git checkout -- . >> "$LOG" 2>&1
    git clean -fd >> "$LOG" 2>&1
    exit 1
fi

git add -A
SUMMARY="auto: self-improvement from usage gaps

$(echo "$GAPS" | head -c 500)"
if ! git commit -m "$SUMMARY" >> "$LOG" 2>&1; then
    log "ABORT: nothing to commit after gate"
    exit 0
fi

if git push origin main >> "$LOG" 2>&1; then
    log "OK: pushed $(git rev-parse --short HEAD)"
else
    log "WARN: push failed, commit left local at $(git rev-parse --short HEAD)"
fi
