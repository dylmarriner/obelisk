export const manifest = {
  id: "@obelisk/paperclip-plugin",
  apiVersion: 1,
  version: "0.1.0",
  displayName: "Obelisk",
  description:
    "Heartbeat/task context optimizer for Paperclip agents. Compresses repeated context, command output, run logs, and exposes restore handles through Obelisk.",
  author: "Dylan Marriner",
  categories: ["workspace", "automation", "ui"],
  capabilities: [
    "agent.tools.register",
    "project.workspaces.read",
    "plugin.state.read",
    "plugin.state.write",
    "activity.log.write",
    "ui.dashboardWidget.register",
    "ui.detailTab.register",
    "ui.page.register"
  ],
  entrypoints: {
    worker: "./worker.js",
    ui: "./ui"
  },
  instanceConfigSchema: {
    type: "object",
    properties: {
      obeliskBinary: {
        type: "string",
        description: "Path to the Obelisk binary. Defaults to `obelisk` on PATH.",
        default: "obelisk"
      },
      defaultTaskBudget: {
        type: "integer",
        description: "Default token budget for task-start packs.",
        default: 12000,
        minimum: 1000
      },
      defaultHeartbeatBudget: {
        type: "integer",
        description: "Default token budget for heartbeat packs.",
        default: 8000,
        minimum: 1000
      },
      allowCommandCompression: {
        type: "boolean",
        description: "Allow Paperclip agents to call the Obelisk run-output compression tool.",
        default: true
      }
    }
  },
  tools: [
    {
      name: "task-pack",
      displayName: "Build task context pack",
      description: "Create a compact Obelisk context pack for a Paperclip task start.",
      parametersSchema: {
        type: "object",
        properties: {
          taskContext: { type: "object", description: "Paperclip task, goal, project, agent, and prior state context." },
          budget: { type: "integer", default: 12000 },
          workspacePath: { type: "string" },
          files: { type: "array", items: { type: "string" }, default: [] },
          dirs: { type: "array", items: { type: "string" }, default: [] },
          diff: { type: "boolean", default: true }
        },
        required: ["taskContext"]
      }
    },
    {
      name: "heartbeat-pack",
      displayName: "Build heartbeat context pack",
      description: "Create a compact delta-oriented pack for a recurring Paperclip heartbeat.",
      parametersSchema: {
        type: "object",
        properties: {
          currentContext: { type: "object" },
          previousContextHash: { type: "string" },
          changedEvents: { type: "array", items: { type: "object" }, default: [] },
          budget: { type: "integer", default: 8000 },
          workspacePath: { type: "string" },
          files: { type: "array", items: { type: "string" }, default: [] },
          dirs: { type: "array", items: { type: "string" }, default: [] },
          diff: { type: "boolean", default: true }
        },
        required: ["currentContext"]
      }
    },
    {
      name: "compress-run-output",
      displayName: "Compress run output",
      description: "Compress a large Paperclip run log or tool result through Obelisk squeeze.",
      parametersSchema: {
        type: "object",
        properties: {
          output: { type: "string" },
          label: { type: "string" }
        },
        required: ["output"]
      }
    },
    {
      name: "restore-context",
      displayName: "Restore Obelisk context",
      description: "Restore an Obelisk blob/checkpoint by handle.",
      parametersSchema: {
        type: "object",
        properties: {
          handle: { type: "string" }
        },
        required: ["handle"]
      }
    },
    {
      name: "context-diff",
      displayName: "Compare context hashes",
      description: "Compute a stable hash and a shallow changed-key summary for Paperclip context objects.",
      parametersSchema: {
        type: "object",
        properties: {
          previousContext: { type: "object" },
          currentContext: { type: "object" }
        },
        required: ["currentContext"]
      }
    },
    {
      name: "savings-report",
      displayName: "Obelisk savings report",
      description: "Return Obelisk savings stats for the local ledger.",
      parametersSchema: {
        type: "object",
        properties: {}
      }
    }
  ],
  ui: {
    slots: [
      {
        type: "dashboardWidget",
        id: "obelisk-savings-widget",
        displayName: "Obelisk Savings",
        exportName: "ObeliskSavingsWidget"
      },
      {
        type: "detailTab",
        id: "obelisk-run-context-tab",
        displayName: "Obelisk Context",
        exportName: "ObeliskRunDetailTab",
        entityTypes: ["run", "agent", "project"]
      },
      {
        type: "settingsPage",
        id: "obelisk-settings",
        displayName: "Obelisk Settings",
        exportName: "ObeliskSettingsPage",
        routePath: "obelisk"
      }
    ]
  }
} as const;

export default manifest;
