import { spawn } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

export interface ObeliskConfig {
  obeliskBinary?: string;
  defaultPackBudget?: number;
  allowRunTool?: boolean;
}

export interface ObeliskResult {
  ok: boolean;
  code: number | null;
  stdout: string;
  stderr: string;
  command: string[];
}

const SHELL_META = ["|", ">", "<", "&&", "||", ";", "`", "$(", "&"];
const SECRET_RE = /(api[_-]?key|token|secret|password|passwd|bearer|private[_-]?key|aws_access_key)/i;
const DANGEROUS_PROGRAMS = new Set([
  "rm",
  "rmdir",
  "mv",
  "cp",
  "chmod",
  "chown",
  "dd",
  "mkfs",
  "mount",
  "umount",
  "sudo",
  "su",
  "ssh",
  "scp",
  "rsync",
  "curl",
  "wget",
  "nc",
  "ncat",
  "telnet",
  "ftp",
  "sftp",
  "python",
  "python3",
  "node",
  "bash",
  "sh",
  "zsh",
  "fish"
]);
const DANGEROUS_GIT = new Set([
  "push",
  "pull",
  "commit",
  "merge",
  "rebase",
  "reset",
  "checkout",
  "switch",
  "clean",
  "stash",
  "tag",
  "branch",
  "remote"
]);

export function asText(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

export function asBool(value: unknown, fallback = false): boolean {
  return typeof value === "boolean" ? value : fallback;
}

export function asNumber(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

export function asStringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}

function splitCommand(command: string): string[] {
  return command.trim().split(/\s+/).filter(Boolean);
}

export function validateReadOnlyCommand(command: string): string[] {
  if (!command.trim()) {
    throw new Error("empty command");
  }
  if (SECRET_RE.test(command)) {
    throw new Error("refusing command that appears to contain or request secrets");
  }
  if (SHELL_META.some((token) => command.includes(token))) {
    throw new Error("refusing shell metacharacters, pipes, redirects, backgrounding, or command chaining");
  }

  const parts = splitCommand(command);
  const program = parts[0];
  if (!program) {
    throw new Error("empty command");
  }
  if (program === "obelisk") {
    throw new Error("command is already an Obelisk command");
  }
  if (DANGEROUS_PROGRAMS.has(program)) {
    throw new Error(`refusing potentially unsafe program: ${program}`);
  }
  if (program === "git" && parts[1] && DANGEROUS_GIT.has(parts[1])) {
    throw new Error(`refusing mutating git subcommand: git ${parts[1]}`);
  }

  return parts;
}

export async function runObelisk(
  args: string[],
  options: { config?: ObeliskConfig; cwd?: string; stdin?: string; timeoutMs?: number } = {}
): Promise<ObeliskResult> {
  const binary = options.config?.obeliskBinary || "obelisk";
  const timeoutMs = options.timeoutMs ?? 120_000;

  return await new Promise((resolve) => {
    const child = spawn(binary, args, {
      cwd: options.cwd,
      stdio: ["pipe", "pipe", "pipe"]
    });

    let stdout = "";
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      setTimeout(() => child.kill("SIGKILL"), 5_000).unref();
    }, timeoutMs);

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => (stdout += chunk));
    child.stderr.on("data", (chunk) => (stderr += chunk));
    child.on("error", (error) => {
      clearTimeout(timer);
      resolve({ ok: false, code: null, stdout, stderr: error.message, command: [binary, ...args] });
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      resolve({ ok: code === 0, code, stdout, stderr, command: [binary, ...args] });
    });

    if (options.stdin) {
      child.stdin.write(options.stdin);
    }
    child.stdin.end();
  });
}

export async function packWithTemporarySystem(input: {
  config: ObeliskConfig;
  contextMarkdown?: string;
  cwd?: string;
  budget: number;
  files?: string[];
  dirs?: string[];
  diff?: boolean;
}): Promise<ObeliskResult> {
  const tempRoot = input.contextMarkdown ? await mkdtemp(path.join(tmpdir(), "obelisk-openclaw-")) : null;
  const args = ["pack", "--budget", String(input.budget)];

  try {
    if (tempRoot && input.contextMarkdown) {
      const systemPath = path.join(tempRoot, "openclaw-context.md");
      await writeFile(systemPath, input.contextMarkdown, "utf8");
      args.push("--system", systemPath);
    }

    for (const dir of input.dirs ?? []) {
      args.push("--dir", dir);
    }
    for (const file of input.files ?? []) {
      args.push("--file", file);
    }
    if (input.diff ?? true) {
      args.push("--diff");
    }

    return await runObelisk(args, { config: input.config, cwd: input.cwd, timeoutMs: 120_000 });
  } finally {
    if (tempRoot) {
      await rm(tempRoot, { recursive: true, force: true }).catch(() => undefined);
    }
  }
}

export function toToolContent(result: unknown) {
  return {
    content: [
      {
        type: "text" as const,
        text: typeof result === "string" ? result : JSON.stringify(result, null, 2)
      }
    ]
  };
}
