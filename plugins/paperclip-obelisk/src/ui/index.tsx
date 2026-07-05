import * as React from "react";

interface PluginContextProps {
  context?: Record<string, unknown>;
}

type BridgeLike = {
  usePluginData?: (key: string, params?: Record<string, unknown>) => { data?: unknown; loading?: boolean; error?: unknown };
  usePluginAction?: (key: string) => (params?: Record<string, unknown>) => Promise<unknown>;
};

function readBridge(): BridgeLike {
  return {};
}

export function ObeliskSavingsWidget(_props: PluginContextProps) {
  const bridge = readBridge();
  const dataState = bridge.usePluginData?.("savings") ?? { data: null, loading: false, error: null };

  return (
    <section style={{ padding: 12 }}>
      <h3>Obelisk Savings</h3>
      {dataState.loading ? <p>Loading Obelisk stats...</p> : null}
      {dataState.error ? <p>Could not load Obelisk stats.</p> : null}
      <pre style={{ whiteSpace: "pre-wrap", fontSize: 12 }}>
        {JSON.stringify(dataState.data ?? { message: "Connect Paperclip UI bridge to show live savings." }, null, 2)}
      </pre>
    </section>
  );
}

export function ObeliskRunDetailTab(props: PluginContextProps) {
  return (
    <section style={{ padding: 12 }}>
      <h3>Obelisk Context</h3>
      <p>
        This tab is reserved for Obelisk run diagnostics: compressed output, restore handles,
        context hashes, heartbeat pack summaries, and per-run token savings.
      </p>
      <pre style={{ whiteSpace: "pre-wrap", fontSize: 12 }}>{JSON.stringify(props.context ?? {}, null, 2)}</pre>
    </section>
  );
}

export function ObeliskSettingsPage() {
  return (
    <section style={{ padding: 12 }}>
      <h2>Obelisk Settings</h2>
      <p>
        Configure the Obelisk binary path, task context budget, heartbeat budget, and command
        compression policy from the plugin instance settings.
      </p>
      <p>
        The first implementation keeps settings in the Paperclip plugin manifest config. A richer
        UI can be wired once Paperclip's plugin UI bridge stabilizes.
      </p>
    </section>
  );
}
