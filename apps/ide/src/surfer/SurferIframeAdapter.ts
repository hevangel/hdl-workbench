import {
  WaveformArtifactSchema,
  WaveformTimeSchema,
  type SurferAdapter,
  type WaveformArtifact,
  type WaveformTime,
} from "@xezim-ide/protocol";

export const PINNED_SURFER_VERSION = "0.7.0" as const;
export const PINNED_SURFER_COMMIT = "bd749b1f786c1c62cd67893ca71346cbe6983915" as const;

export type SurferAdapterErrorCode =
  | "DISPOSED"
  | "READY_TIMEOUT"
  | "ARTIFACT_RESOLVER_UNAVAILABLE"
  | "INVALID_ARTIFACT"
  | "INCOMPLETE_ARTIFACT"
  | "INVALID_URL"
  | "CROSS_ORIGIN_URL"
  | "INVALID_SIGNAL_PATH"
  | "INVALID_TIME_RANGE"
  | "UNKNOWN_COMMAND"
  | "VIEWER_UNAVAILABLE";

export class SurferAdapterError extends Error {
  constructor(readonly code: SurferAdapterErrorCode, message: string) {
    super(message);
    this.name = "SurferAdapterError";
  }
}

export type WaveformUrlResolver = (artifact: WaveformArtifact) => Promise<string> | string;

export type SurferAdapterCommand =
  | { op: "loadWaveform"; artifact: WaveformArtifact }
  | { op: "addSignals"; paths: string[] }
  | { op: "setCursor"; time: WaveformTime }
  | { op: "zoom"; start: WaveformTime; end: WaveformTime };

/** Typed command boundary for callers that receive operations from RPC or UI events. */
export async function dispatchSurferCommand(adapter: SurferAdapter, command: unknown): Promise<void> {
  if (typeof command !== "object" || command === null || !("op" in command) || typeof command.op !== "string") {
    throw new SurferAdapterError("UNKNOWN_COMMAND", "Surfer command must be an object with a supported op.");
  }
  const input = command as Record<string, unknown>;
  switch (input.op) {
    case "loadWaveform":
      return adapter.loadWaveform(input.artifact as WaveformArtifact);
    case "addSignals":
      return adapter.addSignals(input.paths as string[]);
    case "setCursor":
      return adapter.setCursor(input.time as WaveformTime);
    case "zoom":
      return adapter.zoom(input.start as WaveformTime, input.end as WaveformTime);
    default:
      throw new SurferAdapterError("UNKNOWN_COMMAND", `Unsupported Surfer command: ${input.op}`);
  }
}

export interface SurferIframeAdapterOptions {
  resolveWaveformUrl?: WaveformUrlResolver;
  readyTimeoutMs?: number;
  onReadyChange?: (ready: boolean, error?: Error) => void;
}

type BigIntSerde = [sign: 0 | 1, words: number[]];

interface DeferredReady {
  nonce: string;
  promise: Promise<void>;
  resolve: () => void;
  reject: (error: Error) => void;
  interval: number;
  timeout: number;
}

interface VariableRefJson {
  path: { strs: string[]; id: "None" };
  name: string;
  id: "None";
  index: number | null;
}

function makeNonce(hostWindow: Window): string {
  const cryptoObject = hostWindow.crypto as Crypto & { randomUUID?: () => string };
  if (typeof cryptoObject.randomUUID === "function") return cryptoObject.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

function toBigIntSerde(decimal: string): BigIntSerde {
  let value = BigInt(decimal);
  if (value === 0n) return [0, []];
  const words: number[] = [];
  while (value > 0n) {
    words.push(Number(value & 0xffff_ffffn));
    value >>= 32n;
  }
  // The pinned Surfer build serialized a positive BigInt cursor as [1, [25]].
  return [1, words];
}

function parseVariableRef(path: string): VariableRefJson {
  if (typeof path !== "string" || path.length === 0 || path.length > 1024 || /[\u0000-\u001f\u007f]/.test(path)) {
    throw new SurferAdapterError("INVALID_SIGNAL_PATH", `Invalid waveform signal path: ${String(path)}`);
  }
  const components = path.split(".");
  if (components.some((component) => component.length === 0 || component.trim() !== component)) {
    throw new SurferAdapterError("INVALID_SIGNAL_PATH", `Signal paths must use non-empty dot-separated VCD hierarchy names: ${path}`);
  }

  let name = components.pop()!;
  let index: number | null = null;
  const indexedName = /^(.*)\[(-?[0-9]+)\]$/.exec(name);
  if (indexedName) {
    name = indexedName[1]!;
    const parsedIndex = BigInt(indexedName[2]!);
    if (name.length === 0 || parsedIndex < -9_223_372_036_854_775_808n || parsedIndex > 9_223_372_036_854_775_807n) {
      throw new SurferAdapterError("INVALID_SIGNAL_PATH", `Array index is outside Surfer's supported i64 range: ${path}`);
    }
    index = Number(parsedIndex);
  }
  if (name.length === 0) {
    throw new SurferAdapterError("INVALID_SIGNAL_PATH", `Signal name cannot be empty: ${path}`);
  }

  return { path: { strs: components, id: "None" }, name, id: "None", index };
}

export class SurferIframeAdapter implements SurferAdapter {
  private readonly frameWindow: Window;
  private readonly hostWindow: Window;
  private readonly origin: string;
  private readonly readyTimeoutMs: number;
  private disposed = false;
  private readyState = false;
  private readinessRequested = false;
  private pendingReady: DeferredReady | null = null;

  constructor(
    private readonly iframe: HTMLIFrameElement,
    private readonly options: SurferIframeAdapterOptions = {},
  ) {
    const hostWindow = iframe.ownerDocument.defaultView;
    const frameWindow = iframe.contentWindow;
    if (!hostWindow || !frameWindow) {
      throw new SurferAdapterError("VIEWER_UNAVAILABLE", "Surfer iframe must be attached to a browser document before creating its adapter.");
    }
    this.hostWindow = hostWindow;
    this.frameWindow = frameWindow;
    this.origin = hostWindow.location.origin;
    this.readyTimeoutMs = options.readyTimeoutMs ?? 30_000;
    hostWindow.addEventListener("message", this.onHostMessage);
    iframe.addEventListener("load", this.onFrameLoad);
  }

  ready(): Promise<void> {
    if (this.disposed) return Promise.reject(new SurferAdapterError("DISPOSED", "Surfer adapter has been disposed."));
    if (this.readyState) return Promise.resolve();
    this.readinessRequested = true;
    if (!this.pendingReady) this.beginHandshake();
    return this.pendingReady!.promise;
  }

  async loadWaveform(artifact: WaveformArtifact): Promise<void> {
    let parsedArtifact: WaveformArtifact;
    try {
      parsedArtifact = WaveformArtifactSchema.parse(artifact);
    } catch (error) {
      throw new SurferAdapterError("INVALID_ARTIFACT", error instanceof Error ? error.message : "Invalid waveform artifact.");
    }
    if (!parsedArtifact.complete) {
      throw new SurferAdapterError("INCOMPLETE_ARTIFACT", `Waveform artifact ${parsedArtifact.id} is incomplete.`);
    }
    if (!this.options.resolveWaveformUrl) {
      throw new SurferAdapterError("ARTIFACT_RESOLVER_UNAVAILABLE", "No artifact URL resolver is configured for this workspace.");
    }
    const url = await this.options.resolveWaveformUrl(parsedArtifact);
    await this.loadUrl(url);
  }

  /** Load a URL when the caller already owns its provenance, such as the bundled demo VCD. */
  async loadUrl(url: string): Promise<void> {
    const validatedUrl = this.validateUrl(url);
    await this.ready();
    this.postToViewer({ command: "LoadUrl", url: validatedUrl.href });
  }

  async addSignals(paths: string[]): Promise<void> {
    if (!Array.isArray(paths) || paths.length === 0 || paths.length > 4096) {
      throw new SurferAdapterError("INVALID_SIGNAL_PATH", "addSignals requires between 1 and 4096 signal paths.");
    }
    const refs = [...new Set(paths)].map(parseVariableRef);
    await this.ready();
    this.injectMessage({ AddVariables: refs });
  }

  async setCursor(time: WaveformTime): Promise<void> {
    let parsedTime: WaveformTime;
    try {
      parsedTime = WaveformTimeSchema.parse(time);
    } catch (error) {
      throw new SurferAdapterError("INVALID_TIME_RANGE", error instanceof Error ? error.message : "Invalid cursor time.");
    }
    await this.ready();
    this.injectMessage({ CursorSet: toBigIntSerde(parsedTime.ticks) });
  }

  async zoom(start: WaveformTime, end: WaveformTime): Promise<void> {
    let parsedStart: WaveformTime;
    let parsedEnd: WaveformTime;
    try {
      parsedStart = WaveformTimeSchema.parse(start);
      parsedEnd = WaveformTimeSchema.parse(end);
    } catch (error) {
      throw new SurferAdapterError("INVALID_TIME_RANGE", error instanceof Error ? error.message : "Invalid zoom range.");
    }
    if (BigInt(parsedStart.ticks) >= BigInt(parsedEnd.ticks)) {
      throw new SurferAdapterError("INVALID_TIME_RANGE", "Zoom start must be earlier than zoom end.");
    }
    await this.ready();
    this.injectMessage({
      ZoomToRange: {
        start: toBigIntSerde(parsedStart.ticks),
        end: toBigIntSerde(parsedEnd.ticks),
        viewport_idx: 0,
      },
    });
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.hostWindow.removeEventListener("message", this.onHostMessage);
    this.iframe.removeEventListener("load", this.onFrameLoad);
    this.clearPendingReady();
    this.pendingReady?.reject(new SurferAdapterError("DISPOSED", "Surfer adapter has been disposed."));
    this.pendingReady = null;
    this.options.onReadyChange?.(false);
  }

  private readonly onFrameLoad = (): void => {
    if (this.disposed) return;
    if (this.readyState) {
      this.readyState = false;
      this.options.onReadyChange?.(false);
      this.clearPendingReady();
      this.pendingReady = null;
      if (this.readinessRequested) this.beginHandshake();
      return;
    }
    this.sendProbe();
  };

  private readonly onHostMessage = (event: MessageEvent<unknown>): void => {
    if (this.disposed || event.source !== this.frameWindow || event.origin !== this.origin) return;
    if (typeof event.data !== "object" || event.data === null || !("type" in event.data) || !("nonce" in event.data)) return;
    const message = event.data as { type?: unknown; nonce?: unknown };
    const pending = this.pendingReady;
    if (!pending || message.type !== "surfer-ready" || message.nonce !== pending.nonce) return;
    this.clearPendingReady();
    this.pendingReady = null;
    this.readyState = true;
    this.options.onReadyChange?.(true);
    pending.resolve();
  };

  private beginHandshake(): void {
    const nonce = makeNonce(this.hostWindow);
    let resolve!: () => void;
    let reject!: (error: Error) => void;
    const promise = new Promise<void>((accept, deny) => { resolve = accept; reject = deny; });
    void promise.catch(() => undefined);
    const interval = this.hostWindow.setInterval(() => this.sendProbe(), 100);
    const timeout = this.hostWindow.setTimeout(() => {
      if (this.pendingReady?.nonce !== nonce) return;
      this.clearPendingReady();
      this.pendingReady = null;
      const error = new SurferAdapterError("READY_TIMEOUT", `Surfer v${PINNED_SURFER_VERSION} did not become ready within ${this.readyTimeoutMs} ms.`);
      this.options.onReadyChange?.(false, error);
      reject(error);
    }, this.readyTimeoutMs);
    this.pendingReady = { nonce, promise, resolve, reject, interval, timeout };
    this.options.onReadyChange?.(false);
    this.sendProbe();
  }

  private sendProbe(): void {
    const pending = this.pendingReady;
    if (!pending || this.disposed) return;
    try {
      this.frameWindow.postMessage({ type: "xezim-surfer-ready-probe", nonce: pending.nonce }, this.origin);
    } catch {
      // A not-yet-navigated iframe may reject a probe; the retry interval will try again.
    }
  }

  private clearPendingReady(): void {
    if (!this.pendingReady) return;
    this.hostWindow.clearInterval(this.pendingReady.interval);
    this.hostWindow.clearTimeout(this.pendingReady.timeout);
  }

  private validateUrl(value: string): URL {
    let url: URL;
    try {
      url = new URL(value, this.hostWindow.location.href);
    } catch {
      throw new SurferAdapterError("INVALID_URL", `Invalid waveform URL: ${value}`);
    }
    if (!(url.protocol === "http:" || url.protocol === "https:" || url.protocol === "blob:")) {
      throw new SurferAdapterError("INVALID_URL", `Unsupported waveform URL scheme: ${url.protocol}`);
    }
    const effectiveOrigin = url.protocol === "blob:" ? new URL(url.href.slice("blob:".length)).origin : url.origin;
    if (effectiveOrigin !== this.origin) {
      throw new SurferAdapterError("CROSS_ORIGIN_URL", "Surfer only loads waveform URLs from the IDE origin or same-origin Blob URLs.");
    }
    return url;
  }

  private postToViewer(message: { command: "LoadUrl"; url: string }): void {
    if (this.disposed) throw new SurferAdapterError("DISPOSED", "Surfer adapter has been disposed.");
    if (!this.frameWindow) throw new SurferAdapterError("VIEWER_UNAVAILABLE", "Surfer iframe is unavailable.");
    this.frameWindow.postMessage(message, this.origin);
  }

  private injectMessage(message: Record<string, unknown>): void {
    if (this.disposed) throw new SurferAdapterError("DISPOSED", "Surfer adapter has been disposed.");
    this.frameWindow.postMessage({ command: "InjectMessage", message: JSON.stringify(message) }, this.origin);
  }
}
