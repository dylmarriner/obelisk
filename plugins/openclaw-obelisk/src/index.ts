import { Type } from "typebox";
import { definePluginEntry } from "openclaw/plugin-sdk/plugin-entry";
import {
  asBool,
  asNumber,
  asStringArray,
  asText,
  packWithTemporarySystem,
  runObelisk,
  toToolContent,
  validateReadOnlyCommand,
  type ObeliskConfig
} from "./obelisk.js";

type PluginApi = Parameters<Parameters<typeof definePluginEntry>[0]["register"]>[0];

type RuntimeConfig = ObeliskConfig;

function readConfig(api: PluginApi): RuntimeConfig {
  const maybeConfig = (api as unknown as { config?: { get?: () => unknown } }).config?.get?.();
  if (!maybeConfig || typeof maybeConfig !== "object") {
    return { obeliskBinary: "obelisk", defaultPackBudget: 12000, allowRunTool: false };
  }
  const config = maybeConfig as Record<string, unknown>;
  return {
    obeliskBinary: asText(config.obeliskBinary, "obelisk"),
    defaultPackBudget: asNumber(config.defaultPackBudget, 12000),
    allowRunTool: asBool(config.allowRunTool, false)
  };
}

function registerObeliskTools(api: PluginApi) {
  api.registerTool({
    name: "obelisk_pack",
    description:
      "Build a provider-neutral Obelisk context pack using selected files, directories, optional diff, and optional OpenClaw context Markdown.",
    parameters: Type.Object({
      budget: Type.Optional(Type.Number()),
      contextMarkdown: Type.Optional(Type.String()),
      cwd: Type.Optional(Type.String()),
      files: Type.Optional(Type.Array(Type.String())),
      dirs: Type.Optional(Type.Array(Type.String())),
      diff: Type.Optional(Type.Boolean())
    }),
    async execute(_id, params) {
      const config = readConfig(api);
      const result = await packWithTemporarySystem({
        config,
        budget: asNumber(params.budget, config.defaultPackBudget ?? 12000),
        contextMarkdown: asText(params.contextMarkdown, ""),
        cwd: asText(params.cwd, undefined as unknown as string),
        files: asStringArray(params.files),
        dirs: asStringArray(params.dirs),
        diff: asBool(params.diff, true)
      });
      return toToolContent(result);
    }
  });

  api.registerTool({
    name: "obelisk_outline",
    description: "List symbols and line ranges from a source file without reading the whole file into model context.",
    parameters: Type.Object({
      file: Type.String(),
      cwd: Type.Optional(Type.String())
    }),
    async execute(_id, params) {
      const result = await runObelisk(["outline", params.file], {
        config: readConfig(api),
        cwd: asText(params.cwd, undefined as unknown as string),
        timeoutMs: 30_000
      });
      return toToolContent(result);
    }
  });

  api.registerTool({
    name: "obelisk_symbol",
    description: "Extract one named symbol from a source file instead of loading the full file.",
    parameters: Type.Object({
      file: Type.String(),
      name: Type.String(),
      cwd: Type.Optional(Type.String())
    }),
    async execute(_id, params) {
      const result = await runObelisk(["symbol", params.file, params.name], {
        config: readConfig(api),
        cwd: asText(params.cwd, undefined as unknown as string),
        timeoutMs: 30_000
      });
      return toToolContent(result);
    }
  });

  api.registerTool({
    name: "obelisk_restore",
    description: "Restore a compressed Obelisk blob/checkpoint by handle when compact output is insufficient.",
    parameters: Type.Object({
      handle: Type.String(),
      cwd: Type.Optional(Type.String())
    }),
    async execute(_id, params) {
      const result = await runObelisk(["restore", params.handle], {
        config: readConfig(api),
        cwd: asText(params.cwd, undefined as unknown as string),
        timeoutMs: 60_000
      });
      return toToolContent(result);
    }
  });

  api.registerTool({
    name: "obelisk_stats",
    description: "Show token savings across Obelisk layers from the local ledger.",
    parameters: Type.Object({ cwd: Type.Optional(Type.String()) }),
    async execute(_id, params) {
      const result = await runObelisk(["stats"], {
        config: readConfig(api),
        cwd: asText(params.cwd, undefined as unknown as string),
        timeoutMs: 30_000
      });
      return toToolContent(result);
    }
  });

  api.registerTool({
    name: "obelisk_doctor",
    description: "Check Obelisk binary and integration health.",
    parameters: Type.Object({ cwd: Type.Optional(Type.String()) }),
    async execute(_id, params) {
      const result = await runObelisk(["doctor"], {
        config: readConfig(api),
        cwd: asText(params.cwd, undefined as unknown as string),
        timeoutMs: 30_000
      });
      return toToolContent(result);
    }
  });

  api.registerTool(
    {
      name: "obelisk_rewrite",
      description: "Ask Obelisk whether a read-heavy shell command should be wrapped with `obelisk run`.",
      parameters: Type.Object({
        command: Type.String(),
        cwd: Type.Optional(Type.String())
      }),
      async execute(_id, params) {
        validateReadOnlyCommand(params.command);
        const parts = params.command.trim().split(/\s+/).filter(Boolean);
        const result = await runObelisk(["rewrite", ...parts], {
          config: readConfig(api),
          cwd: asText(params.cwd, undefined as unknown as string),
          timeoutMs: 10_000
        });
        return toToolContent(result);
      }
    },
    { optional: true }
  );

  api.registerTool(
    {
      name: "obelisk_run",
      description:
        "Run a safe, read-heavy shell command through `obelisk run`. Disabled by default and still guarded by a conservative read-only command policy.",
      parameters: Type.Object({
        command: Type.String(),
        cwd: Type.Optional(Type.String()),
        timeoutMs: Type.Optional(Type.Number())
      }),
      async execute(_id, params) {
        const config = readConfig(api);
        if (!config.allowRunTool) {
          return toToolContent({
            ok: false,
            error: "obelisk_run is disabled by plugin config. Set plugins.entries.obelisk.config.allowRunTool=true to enable it."
          });
        }
        const parts = validateReadOnlyCommand(params.command);
        const result = await runObelisk(["run", ...parts], {
          config,
          cwd: asText(params.cwd, undefined as unknown as string),
          timeoutMs: asNumber(params.timeoutMs, 120_000)
        });
        return toToolContent(result);
      }
    },
    { optional: true }
  );
}

export default definePluginEntry({
  id: "obelisk",
  name: "Obelisk",
  description: "OpenClaw context firewall tools backed by the Obelisk binary.",
  register(api) {
    registerObeliskTools(api);
  }
});
