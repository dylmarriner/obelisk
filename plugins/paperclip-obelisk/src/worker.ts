import { executeTool } from "./tools.js";
import { manifest } from "./manifest.js";
import type { ObeliskConfig } from "./obelisk.js";

interface RpcRequest {
  id?: string | number | null;
  method: string;
  params?: Record<string, unknown>;
}

let config: ObeliskConfig = {
  obeliskBinary: "obelisk",
  defaultTaskBudget: 12000,
  defaultHeartbeatBudget: 8000,
  allowCommandCompression: true
};

function mergeConfig(value: unknown): ObeliskConfig {
  const next = value && typeof value === "object" ? (value as Record<string, unknown>) : {};
  return {
    obeliskBinary: typeof next.obeliskBinary === "string" ? next.obeliskBinary : "obelisk",
    defaultTaskBudget: typeof next.defaultTaskBudget === "number" ? next.defaultTaskBudget : 12000,
    defaultHeartbeatBudget: typeof next.defaultHeartbeatBudget === "number" ? next.defaultHeartbeatBudget : 8000,
    allowCommandCompression:
      typeof next.allowCommandCompression === "boolean" ? next.allowCommandCompression : true
  };
}

async function handleRpc(request: RpcRequest): Promise<unknown> {
  switch (request.method) {
    case "initialize": {
      config = mergeConfig(request.params?.config);
      return { ok: true, manifest, config };
    }

    case "health": {
      return { ok: true, status: "ready", manifestId: manifest.id, config };
    }

    case "shutdown": {
      queueMicrotask(() => process.exit(0));
      return { ok: true };
    }

    case "validateConfig": {
      const candidate = mergeConfig(request.params?.config);
      return {
        ok: true,
        warnings: candidate.obeliskBinary === "obelisk" ? ["Using `obelisk` from PATH"] : [],
        errors: []
      };
    }

    case "configChanged": {
      config = mergeConfig(request.params?.config);
      return { ok: true, config };
    }

    case "executeTool": {
      const name = String(request.params?.name ?? request.params?.tool ?? "");
      const parameters =
        request.params?.parameters && typeof request.params.parameters === "object"
          ? (request.params.parameters as Record<string, unknown>)
          : {};
      const context =
        request.params?.context && typeof request.params.context === "object"
          ? (request.params.context as Record<string, unknown>)
          : undefined;
      return await executeTool({ name, parameters, context }, config);
    }

    case "getData": {
      const key = String(request.params?.key ?? "");
      if (key === "savings") {
        return await executeTool({ name: "savings-report", parameters: {} }, config);
      }
      return { ok: false, error: `Unknown Obelisk data key: ${key}` };
    }

    case "performAction": {
      const key = String(request.params?.key ?? "");
      if (key === "refresh-savings") {
        return await executeTool({ name: "savings-report", parameters: {} }, config);
      }
      return { ok: false, error: `Unknown Obelisk action key: ${key}` };
    }

    default:
      return { ok: false, error: `Unknown Obelisk worker method: ${request.method}` };
  }
}

function respond(id: RpcRequest["id"], result?: unknown, error?: unknown): void {
  process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id, result, error }) + "\n");
}

let buffer = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  buffer += chunk;
  let newlineIndex = buffer.indexOf("\n");
  while (newlineIndex !== -1) {
    const line = buffer.slice(0, newlineIndex).trim();
    buffer = buffer.slice(newlineIndex + 1);
    newlineIndex = buffer.indexOf("\n");

    if (!line) continue;

    void (async () => {
      try {
        const request = JSON.parse(line) as RpcRequest;
        const result = await handleRpc(request);
        respond(request.id, result);
      } catch (error) {
        respond(null, undefined, error instanceof Error ? error.message : String(error));
      }
    })();
  }
});

process.on("SIGTERM", () => process.exit(0));
process.on("SIGINT", () => process.exit(0));
