import { createHash } from "node:crypto";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

export interface ObeliskConfig {
  obeliskBinary?: string;
  defaultTaskBudget?: number;
  defaultHeartbeatBudget?: number;
  allowCommandCompression?: boolean;
}

export interface ObeliskResult {
  ok: boolean;
  code: number | null;
  stdout: string;
  stderr: string;
  command: string[];
}

export function stableJson(value: unknown): string {
  if (value === null || typeof value !== "object") {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map(stableJson).join(",")}]`;
  }
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record)
    .sort()
    .map((key) => `${JSON.stringify(key)}:${stableJson(record[key])}`)
    .join(",")}}`;
}

export function contextHash(value: unknown): string {
  return createHash("sha256").update(stableJson(value)).digest("hex");
}

export function shallowChangedKeys(previousContext: unknown, currentContext: unknown): string[] {
  if (!previousContext || typeof previousContext !== "object" || !currentContext || typeof currentContext !== "object") {
    return [];
  }
  const previous = previousContext as Record<string, unknown>;
  const current = currentContext as Record<string, unknown>;
  const keys = new Set([...Object.keys(previous), ...Object.keys(current)]);
  const changed: string[] = [];
  for (const key of Array.from(keys).sort()) {
    if (stableJson(previous[key]) !== stableJson(current[key])) {
      changed.push(key);
    }
  }
  return changed;
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

export function renderContextMarkdown(input: {
  title: string;
  mode: "task" | "heartbeat";
  context: unknown;
  previousContextHash?: string;
  changedEvents?: unknown[];
}): string {
  const currentHash = contextHash(input.context);
  const changedEvents = input.changedEvents ?? [];

  return [
    `# ${input.title}`,
    "",
    `Mode: ${input.mode}`,
    `Current context hash: ${currentHash}`,
    input.previousContextHash ? `Previous context hash: ${input.previousContextHash}` : undefined,
    input.previousContextHash === currentHash ? "Context status: unchanged" : "Context status: changed or first run",
    "",
    "## Changed events since previous heartbeat",
    changedEvents.length ? JSON.stringify(changedEvents, null, 2) : "No changed events provided.",
    "",
    "## Candidate Paperclip context",
    JSON.stringify(input.context, null, 2),
    ""
  ]
    .filter(Boolean)
    .join("\n");
}

export async function squeezeMarkdown(markdown: string, config: ObeliskConfig): Promise<ObeliskResult> {
  return runObelisk(["squeeze"], { config, stdin: markdown, timeoutMs: 120_000 });
}

export async function packWorkspace(input: {
  contextMarkdown: string;
  config: ObeliskConfig;
  cwd?: string;
  budget: number;
  files?: string[];
  dirs?: string[];
  diff?: boolean;
}): Promise<ObeliskResult> {
  const tempRoot = await mkdtemp(path.join(tmpdir(), "obelisk-paperclip-"));
  const contextPath = path.join(tempRoot, "paperclip-context.md");

  try {
    await writeFile(contextPath, input.contextMarkdown, "utf8");
    const args = ["pack", "--budget", String(input.budget), "--system", contextPath];

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
    await rm(tempRoot, { recursive: true, force: true }).catch(() => undefined);
  }
}
