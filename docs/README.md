# Obelisk documentation

This directory contains the practical documentation for Obelisk. The root README is the quick pitch; these docs are the part you actually use when things need to work instead of merely looking impressive on GitHub.

## Start here

- [Setup help](SETUP.md) — full install, build, PATH, agent hook, RTK removal, and troubleshooting flow.
- [Command reference](COMMANDS.md) — every public command and when to use it.
- [Agent integrations](AGENT_INTEGRATIONS.md) — Claude Code, Codex, Hermes, OpenCode, OpenClaw, and Cline setup notes.
- [Self-improvement](SELF_IMPROVEMENT.md) — how the learning loop works, what is risky, and how to run it safely.
- [Troubleshooting](TROUBLESHOOTING.md) — common failures and fixes.

## Recommended setup path

1. Build and install Obelisk from source.
2. Run `obelisk doctor`.
3. Remove old RTK hooks if you were using RTK before.
4. Install Obelisk hooks for the agents you actually use.
5. Test `obelisk rewrite`, `obelisk run`, `obelisk pack`, and `obelisk stats`.
6. Leave self-improvement disabled until you understand its current behavior.

## Design rules

- Obelisk should be model-agnostic by default.
- Provider-specific token counters can wrap Obelisk output, but should not infect the core CLI.
- Compression must preserve recovery through restore handles when context is omitted.
- Agents should prefer outlines and symbols over whole-file reads.
- Autonomous self-improvement should be review-first, not surprise-push-to-main chaos.
