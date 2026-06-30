# Obelisk

**A token-optimizing engine for AI coding agents. Minimize tokens, lose no context.**

Obelisk is a single Rust binary that sits between an AI coding agent and the
things that bloat its context — verbose command output, whole-file reads,
boilerplate, long sessions — and makes all of it small *and reversible*. Every
compression stashes its original in a local ledger, so anything Obelisk shrinks
is one `obelisk restore <handle>` away from being whole again.

## Why it's different

Most token tools do one thing — compress command output, *or* terse replies,
*or* checkpoint context. Obelisk does the whole pipeline in one binary, with a
single shared ledger and one dashboard, and adds the piece the others lack:
**symbol-level code retrieval** — hand the agent the one function it asked for
instead of a 2,000-line file (typically a 90%+ cut on a single read).

| Layer | Command | What it does |
|-------|---------|--------------|
| Command output | `obelisk run <cmd>` | run a command, emit a compact, reversible view of its output |
| Boilerplate | `obelisk squeeze` | strip ANSI, progress bars, duplicate-line runs, opaque blobs |
| Code retrieval | `obelisk outline` / `obelisk symbol` | file structure, or one symbol's source — not the whole file |
| Replies | `obelisk terse` | drop filler/greetings; code blocks left intact |
| Sessions | `obelisk marker` / `obelisk checkpoint` / `obelisk restore` | save & reload context compactly; survive compaction |
| Transport | `obelisk serve` | a local proxy for the model API, token-accounted |
| Visibility | `obelisk stats` | one savings dashboard across every layer |

## Install

```bash
cargo build --release          # -> target/release/obelisk
install -m755 target/release/obelisk ~/.local/bin/obelisk
obelisk doctor
```

Wire it into an agent (all reversible, configs backed up):

```bash
obelisk install claude         # also: hermes | opencode | openclaw
```

For Claude Code this adds a `PreToolUse` hook that routes output-heavy,
read-only shell commands through `obelisk run` automatically.

## Use

```bash
obelisk run git status                 # compressed, reversible
obelisk run cargo build                # errors/warnings/result only
obelisk outline src/main.rs            # symbols + line ranges
obelisk symbol src/main.rs run         # just that function
echo "$LONG_LOG" | obelisk squeeze     # collapse boilerplate
<summary> | obelisk marker save plan   # compact resume point
obelisk restore 7f3a1b2c4d5e           # pull any original back, in full
obelisk stats                          # savings across every layer
```

## Measured savings

Token reduction on representative outputs (estimated tokens, lower is better):

| Command | Raw | Obelisk | Saved |
|---|--:|--:|--:|
| `aws ec2 describe-instances` (JSON) | 13,974 | 151 | **99%** |
| `cargo build` (verbose) | 31,786 | 51 | **99.8%** |
| `npm install` | 3,175 | 37 | **99%** |
| `grep -rn` (large tree) | 70,750 | 2,171 | **97%** |
| `journalctl` (5k log lines) | 87,259 | 4,404 | **95%** |
| `jest` (81 suites) | 452 | 23 | **95%** |
| `gcc` compile errors | 990 | 62 | **94%** |
| `terraform plan` | 3,268 | 310 | **91%** |
| `git diff` | 46 | 1 | **98%** |
| `ping` | 343 | 54 | **85%** |
| `git status` | 26 | 6 | **77%** |
| symbol retrieval (one fn vs file) | 14,801 | 164 | **99%** |

## Coverage

**Tuned filters** for ~80 tools across these families, with a generic squeeze
fallback so *every* command is compressed:

- **VCS** git (status/log/diff/show/branch)
- **Search** grep, rg, ag, ack — grouped by file, relevance-sorted, capped
- **Build** cargo, go, make, gradle, mvn, ninja, cmake, bazel, meson, msbuild, xcodebuild, dotnet
- **Compilers** gcc, g++, clang, rustc, javac, swiftc, tsc, valgrind
- **Linters** eslint, biome, ruff, mypy, pylint, clippy, golangci-lint, rubocop, shellcheck, …
- **Tests** pytest, jest, vitest, rspec, mocha, playwright, cypress, …
- **Logs** journalctl, dmesg, docker/kubectl logs, strace, ltrace — dedup with counts
- **Infra** terraform, tofu, pulumi, ansible, cdk
- **Cloud** aws, gcloud, az, doctl, eksctl, fly, heroku, vercel, wrangler — JSON-aware
- **Containers/cluster** docker, podman, kubectl, oc, helm
- **Databases** psql, mysql, sqlite3, mongosh, redis-cli, cqlsh, duckdb, …
- **System/net** systemctl, ps, df, du, netstat, ss, ip, ping, traceroute, dig, mtr, nmap
- **Packages** pip, poetry, npm, gem, bundle, apt, dnf, pacman, brew, snap, …
- **Transfer/archive** rsync, scp, sftp, tar, unzip, zip, 7z
- **Misc** ls, find, tree, cat, head, tail, curl, wget, diff, env, jq, yq

**Symbol retrieval** parses 14 languages: Rust, Python, JavaScript, TypeScript,
Go, C, C++, Java, C#, Kotlin, Swift, Scala, PHP, Ruby, Elixir, Lua.

Behavior is locked in by a unit-test suite (`cargo test`).

## How "lose no context" works

Whenever a layer compresses something it writes the full original to a local
SQLite ledger under a short handle, and leaves an inline pointer in the output:

```
…compressed…
[obelisk:restore 7f3a1b2c4d5e — full original via `obelisk restore 7f3a1b2c4d5e`]
```

Compress aggressively; the original is always recoverable. The same ledger
records per-layer token accounting that `obelisk stats` rolls up.

## Design

- One binary, zero background services (the proxy runs only when you start it).
- Dependency-light: stdlib + a handful of well-known crates; no async runtime.
- Local-only: the ledger lives under your data dir; nothing is sent anywhere
  except, when you run the proxy, the upstream API you point it at.

MIT licensed.
