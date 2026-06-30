// Obelisk plugin for OpenCode — routes shell output through `obelisk run`.
//
// Rewrite logic lives entirely in Obelisk's Rust `obelisk rewrite` command;
// this plugin is a thin proxy call, not a second copy of the eligibility
// rule. Fails open: any error and the original command runs untouched.
export const obelisk = async ({ $ }) => ({
  "tool.execute.before": async (input, output) => {
    if (input.tool !== "bash") return;
    const cmd = (output.args?.command ?? "").trim();
    if (!cmd || cmd.startsWith("obelisk ")) return;
    try {
      const res = await $`obelisk rewrite ${cmd}`.quiet().nothrow();
      if (res.exitCode !== 0) return;
      const rewritten = res.stdout.toString().trim();
      if (rewritten && rewritten !== cmd) {
        output.args.command = rewritten;
      }
    } catch {
      // obelisk not on PATH or rewrite failed — leave the command alone.
    }
  },
});
