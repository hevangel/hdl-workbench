import {
  ErrorCodeSchema,
  PairingChallengeSchema,
  PairingResultSchema,
  PROTOCOL_VERSION,
  RpcRequestSchema,
  RpcResponseSchema,
  RpcResultSchemas,
  validateRpcResponseForRequest,
  type RpcRequest,
  type RpcResponse,
} from "@xezim-ide/protocol";
import type { BrowserIdeController } from "./BrowserIdeController.js";

type PairingChallenge = { challengeId: string; code: string; expiresAt: string };

export type BrowserBridgeState = "disconnected" | "connecting" | "pairing" | "connected" | "error";
export interface PairingDetails {
  websocketUrl: string;
  challenge: PairingChallenge;
}

export interface McpBrowserClientOptions {
  WebSocketClass?: typeof WebSocket;
  onStateChange?: (state: BrowserBridgeState, message?: string) => void;
}

function checkedBridgeUrl(value: string, pageLocation: Location): URL {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    throw new Error("Enter the WebSocket URL returned by get_connection_status.");
  }
  if (url.protocol !== "ws:" || !["127.0.0.1", "localhost"].includes(url.hostname) || url.pathname !== "/ws" || url.search || url.hash) {
    throw new Error("The MCP bridge must use ws://127.0.0.1 or ws://localhost and the /ws path.");
  }
  if (pageLocation.protocol === "https:") throw new Error("A secure IDE page cannot connect to the bridge's plain WebSocket. Run the IDE from its allowlisted local HTTP origin.");
  return url;
}

function errorCode(error: unknown): string {
  if (typeof error === "object" && error !== null && "code" in error && typeof error.code === "string") return error.code;
  return "INVALID_REQUEST";
}

function rpcError(error: unknown): { code: string; message: string; retryable?: boolean } {
  const code = ErrorCodeSchema.safeParse(errorCode(error));
  const message = error instanceof Error ? error.message : "The browser operation failed.";
  return {
    code: code.success ? code.data : "INTERNAL_ERROR",
    message: message.slice(0, 4096),
    ...(typeof error === "object" && error !== null && "retryable" in error && error.retryable === true ? { retryable: true } : {}),
  };
}

/** Authenticated local WebSocket client for the LUNA-07 bridge. */
export class McpBrowserClient {
  private socket: WebSocket | null = null;
  private token: string | null = null;
  private challengeId: string | null = null;
  private state: BrowserBridgeState = "disconnected";
  private connectedResolve: (() => void) | null = null;
  private connectedReject: ((error: Error) => void) | null = null;

  constructor(private readonly controller: BrowserIdeController, private readonly options: McpBrowserClientOptions = {}) {}

  get currentState(): BrowserBridgeState { return this.state; }

  async connect(input: PairingDetails): Promise<void> {
    this.disconnect();
    const bridgeUrl = checkedBridgeUrl(input.websocketUrl, window.location);
    const challenge = PairingChallengeSchema.parse(input.challenge);
    if (Date.parse(challenge.expiresAt) <= Date.now()) throw new Error("This pairing code has expired. Call get_connection_status again for a new code.");
    this.challengeId = challenge.challengeId;
    const SocketClass = this.options.WebSocketClass ?? WebSocket;
    this.setState("connecting");
    const socket = new SocketClass(bridgeUrl.href);
    this.socket = socket;
    const paired = new Promise<void>((resolve, reject) => {
      this.connectedResolve = resolve;
      this.connectedReject = reject;
    });
    const timeout = window.setTimeout(() => this.failConnection(new Error("The browser bridge did not pair within 15 seconds.")), 15_000);
    socket.addEventListener("open", () => {
      if (this.socket !== socket) return;
      this.setState("pairing");
      socket.send(JSON.stringify({
        type: "pair",
        challengeId: challenge.challengeId,
        code: challenge.code,
        clientName: "HDL Workbench browser",
        requestedRole: "write",
      }));
    });
    socket.addEventListener("message", (event) => { void this.onMessage(socket, event.data); });
    socket.addEventListener("error", () => this.failConnection(new Error("Could not connect to the local MCP bridge.")));
    socket.addEventListener("close", (event) => {
      if (this.socket !== socket) return;
      this.token = null;
      this.socket = null;
      if (this.state !== "error") this.setState("disconnected", event.reason || `Bridge disconnected (${event.code}).`);
      this.connectedReject?.(new Error(event.reason || "The local MCP bridge disconnected before pairing."));
      this.clearPairWaiters();
    });
    try {
      await paired;
    } finally {
      window.clearTimeout(timeout);
    }
  }

  disconnect(): void {
    const socket = this.socket;
    this.socket = null;
    this.token = null;
    this.challengeId = null;
    this.clearPairWaiters();
    if (socket && socket.readyState < 2) socket.close(1000, "Browser disconnected by user");
    this.setState("disconnected");
  }

  private async onMessage(socket: WebSocket, raw: unknown): Promise<void> {
    if (this.socket !== socket) return;
    let text: string;
    if (typeof raw === "string") text = raw;
    else if (raw instanceof Blob) text = await raw.text();
    else if (raw instanceof ArrayBuffer) text = new TextDecoder().decode(raw);
    else {
      socket.close(4400, "Unsupported message encoding");
      return;
    }
    let input: unknown;
    try { input = JSON.parse(text) as unknown; } catch { socket.close(4400, "Invalid JSON"); return; }
    if (typeof input !== "object" || input === null || !("type" in input) || typeof input.type !== "string") {
      socket.close(4400, "Invalid message");
      return;
    }
    if (input.type === "paired") {
      const paired = input as { challengeId?: unknown; sessionToken?: unknown; role?: unknown; expiresAt?: unknown };
      const result = PairingResultSchema.safeParse({ sessionToken: paired.sessionToken, role: paired.role, expiresAt: paired.expiresAt });
      if (!result.success || paired.challengeId !== this.challengeId || result.data.role !== "write") {
        socket.close(4401, "Invalid pairing response");
        return;
      }
      this.token = result.data.sessionToken;
      this.setState("connected", `Connected until ${result.data.expiresAt}.`);
      this.connectedResolve?.();
      this.clearPairWaiters();
      return;
    }
    if (input.type === "rpc.request") {
      await this.onRpcRequest(socket, input as { token?: unknown; request?: unknown });
      return;
    }
    socket.close(4400, "Unknown message type");
  }

  private async onRpcRequest(socket: WebSocket, input: { token?: unknown; request?: unknown }): Promise<void> {
    if (!this.token || input.token !== this.token) {
      socket.close(4401, "Unauthorized");
      return;
    }
    const parsed = RpcRequestSchema.safeParse(input.request);
    if (!parsed.success) {
      socket.close(4400, "Invalid RPC request");
      return;
    }
    const request = parsed.data;
    let response: RpcResponse;
    try {
      const result = await this.dispatch(request);
      const checkedResult = RpcResultSchemas[request.op].parse(result);
      response = RpcResponseSchema.parse({ protocol: PROTOCOL_VERSION, id: request.id, ok: true, result: checkedResult });
      validateRpcResponseForRequest(request, response);
    } catch (error) {
      response = RpcResponseSchema.parse({ protocol: PROTOCOL_VERSION, id: request.id, ok: false, error: rpcError(error) });
    }
    if (this.socket !== socket || socket.readyState !== WebSocket.OPEN || !this.token) return;
    socket.send(JSON.stringify({ type: "rpc.response", token: this.token, response }));
  }

  private async dispatch(request: RpcRequest): Promise<unknown> {
    switch (request.op) {
      case "connection.get_status": return this.controller.connectionStatus(true);
      case "workspace.list_files": return { files: await this.controller.listFiles() };
      case "workspace.read_file": {
        const file = await this.controller.readFile(request.params.path);
        return { path: request.params.path, revision: file.revision, contentBase64: bytesToBase64(file.bytes) };
      }
      case "workspace.write_file": {
        const revision = await this.controller.writeFile(request.params.path, base64ToBytes(request.params.contentBase64), request.params.expectedRevision);
        return revision;
      }
      case "workspace.delete_file":
        await this.controller.deleteFile(request.params.path, request.params.expectedRevision);
        return { deleted: true };
      case "simulation.compile": return this.controller.compile(request.params);
      case "simulation.run": return this.controller.run(request.params);
      case "simulation.execute": return this.controller.execute(request.params);
      case "simulation.get_diagnostics": return { diagnostics: await this.controller.diagnostics({
        ...(request.params.compileId === undefined ? {} : { compileId: request.params.compileId }),
        ...(request.params.runId === undefined ? {} : { runId: request.params.runId }),
      }) };
      case "simulation.list_runs": return { runs: await this.controller.listRuns(request.projectId, request.params.limit) };
      case "simulation.get_log": return { runId: request.params.runId, ...await this.controller.getLog(request.params) };
      case "waveform.show": {
        await this.controller.showWaveform({
          artifactId: request.params.artifactId,
          ...(request.params.signalPaths === undefined ? {} : { signalPaths: request.params.signalPaths }),
          ...(request.params.cursor === undefined ? {} : { cursor: request.params.cursor }),
        });
        return { artifactId: request.params.artifactId, opened: true };
      }
    }
  }

  private failConnection(error: Error): void {
    if (this.state === "connected") return;
    this.setState("error", error.message);
    this.connectedReject?.(error);
    this.clearPairWaiters();
    this.socket?.close(4401, "Pairing failed");
  }

  private setState(state: BrowserBridgeState, message?: string): void {
    this.state = state;
    this.options.onStateChange?.(state, message);
  }

  private clearPairWaiters(): void {
    this.connectedResolve = null;
    this.connectedReject = null;
  }
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(offset, Math.min(bytes.length, offset + 0x8000)));
  }
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  const binary = atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}
