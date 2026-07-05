import {
  contextHash,
  packWorkspace,
  renderContextMarkdown,
  runObelisk,
  shallowChangedKeys,
  squeezeMarkdown,
  type ObeliskConfig
} from "./obelisk.js";

export interface ToolInput {
  name: string;
  parameters?: Record<string, unknown>;
  context?: {
    agentId?: string;
    runId?: string;
    companyId?: string;
    projectId?: string;
  };
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
}

function stringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}

function numberValue(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function boolValue(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

export async function executeTool(input: ToolInput, config: ObeliskConfig): Promise<unknown> {
  const params = asRecord(input.parameters);

  switch (input.name) {
    case "task-pack": {
      const taskContext = params.taskContext ?? {};
      const budget = numberValue(params.budget, config.defaultTaskBudget ?? 12000);
      const workspacePath = typeof params.workspacePath === "string" ? params.workspacePath : undefined;
      const contextMarkdown = renderContextMarkdown({
        title: "Paperclip Task Context Pack",
        mode: "task",
        context: taskContext
      });

      const result = workspacePath
        ? await packWorkspace({
            contextMarkdown,
            config,
            cwd: workspacePath,
            budget,
            files: stringArray(params.files),
            dirs: stringArray(params.dirs),
            diff: boolValue(params.diff, true)
          })
        : await squeezeMarkdown(contextMarkdown, config);

      return {
        type: "task-pack",
        contextHash: contextHash(taskContext),
        budget,
        result
      };
    }

    case "heartbeat-pack": {
      const currentContext = params.currentContext ?? {};
      const changedEvents = Array.isArray(params.changedEvents) ? params.changedEvents : [];
      const previousContextHash = typeof params.previousContextHash === "string" ? params.previousContextHash : undefined;
      const budget = numberValue(params.budget, config.defaultHeartbeatBudget ?? 8000);
      const workspacePath = typeof params.workspacePath === "string" ? params.workspacePath : undefined;
      const currentHash = contextHash(currentContext);
      const unchanged = previousContextHash === currentHash;

      const contextMarkdown = renderContextMarkdown({
        title: "Paperclip Heartbeat Context Pack",
        mode: "heartbeat",
        context: currentContext,
        previousContextHash,
        changedEvents
      });

      const result = workspacePath
        ? await packWorkspace({
            contextMarkdown,
            config,
            cwd: workspacePath,
            budget,
            files: stringArray(params.files),
            dirs: stringArray(params.dirs),
            diff: boolValue(params.diff, true)
          })
        : await squeezeMarkdown(contextMarkdown, config);

      return {
        type: "heartbeat-pack",
        previousContextHash,
        currentContextHash: currentHash,
        unchanged,
        changedEventCount: changedEvents.length,
        budget,
        result
      };
    }

    case "compress-run-output": {
      if (config.allowCommandCompression === false) {
        return { ok: false, error: "compress-run-output is disabled by plugin configuration" };
      }
      const output = typeof params.output === "string" ? params.output : "";
      const label = typeof params.label === "string" ? params.label : "run-output";
      const result = await squeezeMarkdown(`# ${label}\n\n\`\`\`text\n${output}\n\`\`\`\n`, config);
      return { type: "compress-run-output", originalChars: output.length, result };
    }

    case "restore-context": {
      const handle = typeof params.handle === "string" ? params.handle : "";
      if (!handle) {
        return { ok: false, error: "handle is required" };
      }
      const result = await runObelisk(["restore", handle], { config, timeoutMs: 60_000 });
      return { type: "restore-context", handle, result };
    }

    case "context-diff": {
      const previousContext = params.previousContext;
      const currentContext = params.currentContext ?? {};
      return {
        type: "context-diff",
        previousContextHash: previousContext ? contextHash(previousContext) : null,
        currentContextHash: contextHash(currentContext),
        changedKeys: shallowChangedKeys(previousContext, currentContext)
      };
    }

    case "savings-report": {
      const result = await runObelisk(["stats"], { config, timeoutMs: 60_000 });
      return { type: "savings-report", result };
    }

    default:
      return { ok: false, error: `Unknown Obelisk Paperclip tool: ${input.name}` };
  }
}
