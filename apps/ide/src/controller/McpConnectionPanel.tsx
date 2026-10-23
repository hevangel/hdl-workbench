import { useState } from "react";
import type { BrowserBridgeState, McpBrowserClient } from "./McpBrowserClient.js";

interface McpConnectionPanelProps {
  client: McpBrowserClient;
  state: BrowserBridgeState;
  message: string;
  onClose: () => void;
}

export function McpConnectionPanel({ client, state, message, onClose }: McpConnectionPanelProps) {
  const [pairingJson, setPairingJson] = useState("");
  const [error, setError] = useState("");
  const busy = state === "connecting" || state === "pairing";

  const connect = async () => {
    setError("");
    try {
      const parsed = JSON.parse(pairingJson) as { websocketUrl?: unknown; pairingChallenge?: unknown };
      if (typeof parsed.websocketUrl !== "string" || typeof parsed.pairingChallenge !== "object" || parsed.pairingChallenge === null) {
        throw new Error("Paste the complete JSON result from get_connection_status.");
      }
      await client.connect({ websocketUrl: parsed.websocketUrl, challenge: parsed.pairingChallenge as { challengeId: string; code: string; expiresAt: string } });
      setPairingJson("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not connect to the MCP bridge.");
    }
  };

  return (
    <section className="mcp-connection-panel" aria-label="MCP bridge connection">
      <div className="mcp-connection-heading">
        <strong>Local Codex MCP bridge</strong>
        <button type="button" aria-label="Close bridge connection" onClick={onClose}>×</button>
      </div>
      <p>Run <code>get_connection_status</code> in the MCP client, then paste its JSON response. Pairing grants this local bridge project read and write access in this browser.</p>
      <label htmlFor="mcp-pairing-json">Connection status response</label>
      <textarea id="mcp-pairing-json" value={pairingJson} onChange={(event) => setPairingJson(event.target.value)} rows={5} spellCheck={false} disabled={busy || state === "connected"} />
      <div className="mcp-connection-actions">
        {state === "connected"
          ? <button type="button" onClick={() => client.disconnect()}>Disconnect</button>
          : <button type="button" onClick={() => void connect()} disabled={busy || !pairingJson.trim()}>{busy ? "Connecting…" : "Pair browser"}</button>}
        <span className={`mcp-connection-state ${state}`} role="status">{state === "connected" ? "Connected" : state === "pairing" ? "Pairing" : state === "connecting" ? "Connecting" : state === "error" ? "Connection failed" : "Not connected"}</span>
      </div>
      {(error || message) && <p className="mcp-connection-error" role="alert">{error || message}</p>}
    </section>
  );
}
