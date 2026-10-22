import { createServer, type IncomingMessage, type Server } from "node:http";
import { randomBytes, randomUUID, timingSafeEqual } from "node:crypto";
import {
  PairingChallengeSchema,
  PairingRequestSchema,
  PROTOCOL_VERSION,
  RpcRequestSchema,
  RpcResponseSchema,
  validateRpcResponseForRequest,
  type RpcRequest,
  type RpcResponse,
} from "@xezim-ide/protocol";
import { WebSocket, WebSocketServer, type RawData } from "ws";
import type { z } from "zod";
import { BridgeError } from "./errors.js";

const BASE32 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const DEFAULT_PAIRING_TTL_MS = 120_000;
const DEFAULT_SESSION_TTL_MS = 3_600_000;
const DEFAULT_REQUEST_TIMEOUT_MS = 30_000;
type PairingChallenge = z.infer<typeof PairingChallengeSchema>;

function base32Code(): string {
  const bytes = randomBytes(5);
  let bits = 0;
  let value = 0;
  let result = "";
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      result += BASE32[(value >>> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  return result.slice(0, 8);
}

function safeEqual(left: string, right: string): boolean {
  const a = Buffer.from(left);
  const b = Buffer.from(right);
  return a.byteLength === b.byteLength && timingSafeEqual(a, b);
}

function textOf(data: RawData): string {
  if (typeof data === "string") return data;
  if (Array.isArray(data)) return Buffer.concat(data).toString("utf8");
  if (data instanceof ArrayBuffer) return Buffer.from(data).toString("utf8");
  return data.toString("utf8");
}

interface SocketSession {
  socket: WebSocket;
  sessionToken: string | null;
  sessionExpiresAt: number | null;
  origin: string;
  pairingTimeout: NodeJS.Timeout | null;
}

interface PendingRequest {
  request: RpcRequest;
  resolve: (response: RpcResponse) => void;
  reject: (error: BridgeError) => void;
  timeout: NodeJS.Timeout;
}

export interface WebSocketBridgeOptions {
  allowedOrigins: string[];
  host?: string;
  port?: number;
  pairingTtlMs?: number;
  sessionTtlMs?: number;
  requestTimeoutMs?: number;
  logger?: (message: string) => void;
}

export interface PairingStatus {
  url: string;
  allowedOrigins: string[];
  connected: boolean;
  pairingChallenge: PairingChallenge | null;
}

export class WebSocketBridge {
  private readonly allowedOrigins: Set<string>;
  private readonly host: string;
  private readonly port: number;
  private readonly pairingTtlMs: number;
  private readonly sessionTtlMs: number;
  private readonly requestTimeoutMs: number;
  private readonly logger: (message: string) => void;
  private httpServer: Server | null = null;
  private wss: WebSocketServer | null = null;
  private socket: SocketSession | null = null;
  private challenge: PairingChallenge | null = null;
  private readonly pending = new Map<string, PendingRequest>();

  constructor(options: WebSocketBridgeOptions) {
    const normalized = options.allowedOrigins.map((origin) => {
      try {
        const parsed = new URL(origin);
        if (parsed.origin !== origin || !["http:", "https:"].includes(parsed.protocol)) throw new Error();
        return parsed.origin;
      } catch {
        throw new BridgeError("INVALID_REQUEST", `Invalid allowed browser origin: ${origin}`);
      }
    });
    if (normalized.length === 0) throw new BridgeError("INVALID_REQUEST", "At least one exact browser Origin must be allowed");
    this.allowedOrigins = new Set(normalized);
    this.host = options.host ?? "127.0.0.1";
    if (this.host !== "127.0.0.1") throw new BridgeError("INVALID_REQUEST", "The WebSocket bridge must bind to 127.0.0.1");
    this.port = options.port ?? 47_531;
    this.pairingTtlMs = options.pairingTtlMs ?? DEFAULT_PAIRING_TTL_MS;
    this.sessionTtlMs = options.sessionTtlMs ?? DEFAULT_SESSION_TTL_MS;
    this.requestTimeoutMs = options.requestTimeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
    this.logger = options.logger ?? (() => undefined);
  }

  async start(): Promise<void> {
    if (this.httpServer) return;
    const httpServer = createServer((_request, response) => {
      response.writeHead(404).end();
    });
    const wss = new WebSocketServer({ noServer: true, maxPayload: 24 * 1024 * 1024, perMessageDeflate: false });
    httpServer.on("upgrade", (request, socket, head) => {
      const origin = request.headers.origin;
      const host = request.headers.host;
      if (request.url !== "/ws" || !this.isLoopbackHost(host) || !origin || !this.allowedOrigins.has(origin)) {
        socket.write("HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n");
        socket.destroy();
        this.logger(`Rejected WebSocket upgrade from origin=${origin ?? "missing"} host=${host ?? "missing"}`);
        return;
      }
      if (this.socket && this.socket.socket.readyState === WebSocket.OPEN) {
        socket.write("HTTP/1.1 409 Conflict\r\nConnection: close\r\n\r\n");
        socket.destroy();
        return;
      }
      wss.handleUpgrade(request, socket, head, (webSocket) => wss.emit("connection", webSocket, request));
    });
    wss.on("connection", (webSocket, request) => this.attach(webSocket, request));
    await new Promise<void>((resolve, reject) => {
      httpServer.once("error", reject);
      httpServer.listen(this.port, this.host, () => {
        httpServer.off("error", reject);
        resolve();
      });
    });
    this.httpServer = httpServer;
    this.wss = wss;
    this.logger(`WebSocket bridge listening on ${this.url}`);
  }

  get url(): string {
    const address = this.httpServer?.address();
    const port = address && typeof address !== "string" ? address.port : this.port;
    return `ws://127.0.0.1:${port}/ws`;
  }

  status(): PairingStatus {
    this.expireSession();
    return {
      url: this.url,
      allowedOrigins: [...this.allowedOrigins],
      connected: Boolean(this.socket && this.socket.sessionToken && this.socket.sessionExpiresAt && this.socket.sessionExpiresAt > Date.now() && this.socket.socket.readyState === WebSocket.OPEN),
      pairingChallenge: this.getPairingChallenge(),
    };
  }

  async close(): Promise<void> {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timeout);
      pending.reject(new BridgeError("BROWSER_DISCONNECTED", "Browser bridge stopped", true));
    }
    this.pending.clear();
    this.socket?.socket.close(1001, "Bridge shutting down");
    this.socket = null;
    const wss = this.wss;
    const httpServer = this.httpServer;
    this.wss = null;
    this.httpServer = null;
    if (wss) await new Promise<void>((resolve) => wss.close(() => resolve()));
    if (httpServer?.listening) await new Promise<void>((resolve, reject) => httpServer.close((error) => error ? reject(error) : resolve()));
  }

  async request(requestInput: RpcRequest): Promise<RpcResponse> {
    const request = RpcRequestSchema.parse(requestInput);
    this.expireSession();
    const session = this.socket;
    if (!session || !session.sessionToken || session.socket.readyState !== WebSocket.OPEN || session.sessionExpiresAt === null || session.sessionExpiresAt <= Date.now()) {
      throw new BridgeError("BROWSER_DISCONNECTED", "No paired browser is connected", true);
    }
    if (this.pending.has(request.id)) throw new BridgeError("INVALID_REQUEST", `Duplicate request id: ${request.id}`);
    return await new Promise<RpcResponse>((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(request.id);
        reject(new BridgeError("TIMEOUT", `Browser request ${request.id} timed out`, true));
      }, this.requestTimeoutMs);
      this.pending.set(request.id, { request, resolve, reject, timeout });
      session.socket.send(JSON.stringify({ type: "rpc.request", token: session.sessionToken, request }), (error) => {
        if (!error) return;
        const pending = this.pending.get(request.id);
        if (!pending) return;
        clearTimeout(pending.timeout);
        this.pending.delete(request.id);
        reject(new BridgeError("BROWSER_DISCONNECTED", "Browser connection closed before the request was sent", true));
      });
    });
  }

  private attach(socket: WebSocket, request: IncomingMessage): void {
    const session: SocketSession = { socket, sessionToken: null, sessionExpiresAt: null, origin: request.headers.origin ?? "", pairingTimeout: null };
    this.socket = session;
    session.pairingTimeout = setTimeout(() => {
      if (!session.sessionToken && socket.readyState === WebSocket.OPEN) socket.close(4408, "Pairing timed out");
    }, this.pairingTtlMs);
    socket.on("message", (raw) => this.onMessage(session, textOf(raw)));
    socket.on("close", () => {
      if (session.pairingTimeout) clearTimeout(session.pairingTimeout);
      if (this.socket === session) this.socket = null;
      for (const [id, pending] of this.pending) {
        clearTimeout(pending.timeout);
        pending.reject(new BridgeError("BROWSER_DISCONNECTED", "Browser disconnected during a request", true));
        this.pending.delete(id);
      }
    });
    socket.on("error", (error) => this.logger(`WebSocket error: ${error.message}`));
  }

  private onMessage(session: SocketSession, raw: string): void {
    let input: unknown;
    try {
      input = JSON.parse(raw) as unknown;
    } catch {
      session.socket.close(4400, "Invalid JSON");
      return;
    }
    if (typeof input !== "object" || input === null || !("type" in input) || typeof input.type !== "string") {
      session.socket.close(4400, "Invalid message");
      return;
    }
    if (!session.sessionToken) {
      if (input.type !== "pair") {
        session.socket.close(4401, "Pairing required");
        return;
      }
      const { type: _type, ...pairingRequest } = input;
      this.acceptPairing(session, pairingRequest);
      return;
    }
    if (!("token" in input) || typeof input.token !== "string" || !safeEqual(input.token, session.sessionToken) || session.sessionExpiresAt === null || session.sessionExpiresAt <= Date.now()) {
      session.socket.close(4401, "Unauthorized");
      return;
    }
    if (input.type !== "rpc.response" || !("response" in input)) {
      session.socket.close(4400, "Expected an authenticated RPC response");
      return;
    }
    const parsed = RpcResponseSchema.safeParse(input.response);
    if (!parsed.success) {
      session.socket.close(4400, "Invalid RPC response");
      return;
    }
    const pending = this.pending.get(parsed.data.id);
    if (!pending) {
      session.socket.close(4400, "Unknown RPC response id");
      return;
    }
    try {
      validateRpcResponseForRequest(pending.request, parsed.data);
    } catch {
      session.socket.close(4400, "RPC response does not match request");
      return;
    }
    clearTimeout(pending.timeout);
    this.pending.delete(parsed.data.id);
    pending.resolve(parsed.data);
  }

  private acceptPairing(session: SocketSession, input: object): void {
    const parsed = PairingRequestSchema.safeParse(input);
    const challenge = this.challenge;
    if (!parsed.success || !challenge || challenge.expiresAt <= new Date().toISOString() || parsed.data.challengeId !== challenge.challengeId || !safeEqual(parsed.data.code, challenge.code)) {
      session.socket.close(4401, "Invalid or expired pairing code");
      return;
    }
    const token = randomBytes(32).toString("hex");
    const expiresAt = new Date(Date.now() + this.sessionTtlMs).toISOString();
    session.sessionToken = token;
    session.sessionExpiresAt = Date.parse(expiresAt);
    if (session.pairingTimeout) clearTimeout(session.pairingTimeout);
    session.pairingTimeout = null;
    this.challenge = null;
    session.socket.send(JSON.stringify({
      type: "paired",
      challengeId: challenge.challengeId,
      sessionToken: token,
      role: parsed.data.requestedRole,
      expiresAt,
    }));
    this.logger(`Paired browser from ${session.origin} as ${parsed.data.clientName}`);
  }

  private getPairingChallenge(): PairingChallenge {
    if (!this.challenge || Date.parse(this.challenge.expiresAt) <= Date.now()) {
      this.challenge = PairingChallengeSchema.parse({
        challengeId: randomUUID(),
        code: base32Code(),
        expiresAt: new Date(Date.now() + this.pairingTtlMs).toISOString(),
      });
    }
    return this.challenge;
  }

  private expireSession(): void {
    const session = this.socket;
    if (session?.sessionExpiresAt !== null && session?.sessionExpiresAt !== undefined && session.sessionExpiresAt <= Date.now()) {
      this.socket = null;
      session.socket.close(4401, "Session expired");
    }
  }

  private isLoopbackHost(host: string | undefined): boolean {
    if (!host) return false;
    const hostname = host.startsWith("[") ? host.slice(1, host.indexOf("]")) : host.split(":")[0];
    return hostname === "127.0.0.1" || hostname === "localhost";
  }
}

export function requestId(): string {
  return randomUUID();
}

export function makeRpcRequest<T extends RpcRequest>(request: Omit<T, "protocol" | "id">): T {
  return RpcRequestSchema.parse({ protocol: PROTOCOL_VERSION, id: requestId(), ...request }) as T;
}
