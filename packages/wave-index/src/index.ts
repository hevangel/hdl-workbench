import { DecimalTicksSchema, TimescaleSchema } from "@xezim-ide/protocol";
import { z } from "zod";

const DEFAULT_LIMITS = {
  maxInputBytes: 256 * 1024 * 1024,
  maxSignals: 100_000,
  maxChanges: 1_000_000,
  maxIndexBytes: 128 * 1024 * 1024,
  maxSignalWidth: 16_384,
  maxTokenChars: 1_048_576,
} as const;

const TimescaleUnitSchema = z.enum(["s", "ms", "us", "ns", "ps", "fs"]);
const CursorEnvelopeSchema = z.discriminatedUnion("kind", [
  z.strictObject({ v: z.literal(1), indexId: z.string(), kind: z.literal("signals"), offset: z.number().int().nonnegative() }),
  z.strictObject({
    v: z.literal(1),
    indexId: z.string(),
    kind: z.literal("transitions"),
    signalPath: z.string(),
    start: DecimalTicksSchema,
    end: DecimalTicksSchema,
    interpretation: z.enum(["signed", "unsigned", "unknown"]),
    offset: z.number().int().nonnegative(),
  }),
]);

export type WaveformIndexErrorCode =
  | "MALFORMED_VCD"
  | "UNSUPPORTED_VCD"
  | "UNKNOWN_SIGNAL"
  | "TIME_OUT_OF_RANGE"
  | "INVALID_TIME"
  | "INVALID_RANGE"
  | "INVALID_PAGE_SIZE"
  | "INVALID_CURSOR"
  | "RESOURCE_LIMIT";

export class WaveformIndexError extends Error {
  constructor(readonly code: WaveformIndexErrorCode, message: string) {
    super(message);
    this.name = "WaveformIndexError";
  }
}

export interface WaveformIndexLimits {
  maxInputBytes?: number;
  maxSignals?: number;
  maxChanges?: number;
  maxIndexBytes?: number;
  maxSignalWidth?: number;
  maxTokenChars?: number;
}

export type VcdInput = string | AsyncIterable<Uint8Array | string>;
export type SignalSignedness = "signed" | "unsigned" | "unknown";

export interface SignalDescriptor {
  path: string;
  aliases: string[];
  idCode: string;
  width: number;
  vcdType: string;
  signedness: SignalSignedness;
  fourState: boolean;
}

export interface LogicValue {
  /** Most-significant bit first, with 0, 1, x, and z preserved. */
  bits: string;
  signedness: SignalSignedness;
  unsignedDecimal: string | null;
  signedDecimal: string | null;
}

export interface SignalSample extends SignalDescriptor {
  timeTicks: string;
  timescale: { magnitude: string; unit: z.infer<typeof TimescaleUnitSchema> };
  value: LogicValue | null;
}

export interface SignalTransition {
  timeTicks: string;
  value: LogicValue;
}

export interface SignalPage {
  signals: SignalDescriptor[];
  nextCursor?: string;
}

export interface TransitionPage {
  signalPath: string;
  startTicks: string;
  endTicks: string;
  transitions: SignalTransition[];
  nextCursor?: string;
}

export interface SignalInterpretation {
  /** Use source metadata when VCD's reg/wire declaration omitted signedness. */
  signed?: boolean;
}

interface ValueChange {
  tick: bigint;
  bits: string;
}

interface SignalGroup {
  idCode: string;
  width: number;
  vcdType: string;
  signedness: SignalSignedness;
  fourState: boolean;
  paths: string[];
  changes: ValueChange[];
}

interface ParserState {
  groups: Map<string, SignalGroup>;
  paths: Map<string, SignalGroup>;
  timescale: { magnitude: string; unit: z.infer<typeof TimescaleUnitSchema> } | null;
  currentTick: bigint;
  firstTick: bigint | null;
  lastTick: bigint;
  declarations: number;
  changeCount: number;
  estimatedIndexBytes: number;
  limits: Required<WaveformIndexLimits>;
}

function fail(code: WaveformIndexErrorCode, message: string): never {
  throw new WaveformIndexError(code, message);
}

function positiveLimit(name: string, value: number | undefined, fallback: number): number {
  const limit = value ?? fallback;
  if (!Number.isSafeInteger(limit) || limit <= 0) fail("INVALID_PAGE_SIZE", `${name} must be a positive safe integer`);
  return limit;
}

function utf8Length(text: string): number {
  let bytes = 0;
  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index);
    if (code < 0x80) bytes += 1;
    else if (code < 0x800) bytes += 2;
    else if (code >= 0xd800 && code <= 0xdbff && index + 1 < text.length && text.charCodeAt(index + 1) >= 0xdc00 && text.charCodeAt(index + 1) <= 0xdfff) {
      bytes += 4;
      index += 1;
    } else bytes += 3;
  }
  return bytes;
}

function* splitTokens(text: string, previous: string, maxTokenChars: number): Generator<string, string> {
  let start = 0;
  let pending = previous;
  for (let index = 0; index < text.length; index += 1) {
    if (!/\s/.test(text[index] ?? "")) continue;
    const tokenPart = text.slice(start, index);
    const token = pending + tokenPart;
    if (token) yield token;
    pending = "";
    start = index + 1;
  }
  pending += text.slice(start);
  if (pending.length > maxTokenChars) fail("RESOURCE_LIMIT", `A VCD token exceeds ${maxTokenChars} characters`);
  return pending;
}

async function* tokenize(input: VcdInput, limits: Required<WaveformIndexLimits>): AsyncGenerator<string> {
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let pending = "";
  let totalBytes = 0;
  let observedKind: "bytes" | "text" | null = null;
  const chunks: AsyncIterable<Uint8Array | string> | Iterable<Uint8Array | string> = typeof input === "string" ? [input] : input;
  const append = async function* (text: string): AsyncGenerator<string> {
    const scanner = splitTokens(text, pending, limits.maxTokenChars);
    let next = scanner.next();
    while (!next.done) {
      yield next.value;
      next = scanner.next();
    }
    pending = next.value;
  };

  for await (const chunk of chunks) {
    const kind = typeof chunk === "string" ? "text" : "bytes";
    if (observedKind !== null && kind !== observedKind) fail("MALFORMED_VCD", "VCD input cannot mix text and byte chunks");
    observedKind = kind;
    const bytes = kind === "text" ? utf8Length(chunk as string) : (chunk as Uint8Array).byteLength;
    totalBytes += bytes;
    if (totalBytes > limits.maxInputBytes) fail("RESOURCE_LIMIT", `VCD exceeds ${limits.maxInputBytes} input bytes`);
    let text: string;
    try {
      text = kind === "text" ? chunk as string : decoder.decode(chunk as Uint8Array, { stream: true });
    } catch {
      fail("MALFORMED_VCD", "VCD input is not valid UTF-8");
    }
    yield* append(text);
  }
  if (observedKind === "bytes") {
    let tail: string;
    try {
      tail = decoder.decode();
    } catch {
      fail("MALFORMED_VCD", "VCD input ends in an incomplete UTF-8 sequence");
    }
    yield* append(tail);
  }
  if (pending) yield pending;
}

async function readRequired(iterator: AsyncIterator<string>, what: string): Promise<string> {
  const next = await iterator.next();
  if (next.done) fail("MALFORMED_VCD", `Unexpected end of VCD while reading ${what}`);
  return next.value;
}

async function skipToEnd(iterator: AsyncIterator<string>): Promise<void> {
  for (;;) {
    const token = await readRequired(iterator, "$end");
    if (token === "$end") return;
  }
}

function parseTimescale(tokens: string[]): ParserState["timescale"] {
  const combined = tokens.join("").replace(/\s+/g, "");
  const match = /^(1|10|100)(s|ms|us|ns|ps|fs)$/.exec(combined);
  if (!match) fail("MALFORMED_VCD", `Invalid $timescale declaration: ${tokens.join(" ")}`);
  const magnitude = match[1];
  const unit = match[2];
  const parsed = TimescaleSchema.safeParse({ magnitude, unit });
  if (!parsed.success) fail("MALFORMED_VCD", `Invalid $timescale declaration: ${tokens.join(" ")}`);
  return parsed.data;
}

function signednessFor(type: string): SignalSignedness {
  if (type === "integer") return "signed";
  if (type === "time") return "unsigned";
  // Standard VCD does not preserve the signed qualifier on reg/wire vectors.
  return "unknown";
}

function addVariable(state: ParserState, scope: string[], type: string, widthText: string, idCode: string, reference: string): void {
  if (!/^\d+$/.test(widthText)) fail("MALFORMED_VCD", `Invalid VCD signal width: ${widthText}`);
  const width = Number(widthText);
  if (!Number.isSafeInteger(width) || width < 1 || width > state.limits.maxSignalWidth) {
    fail("RESOURCE_LIMIT", `VCD signal width must be between 1 and ${state.limits.maxSignalWidth} bits`);
  }
  if (!idCode || /\s/.test(idCode) || !reference) fail("MALFORMED_VCD", "VCD variable declaration is missing an id code or reference");
  state.declarations += 1;
  if (state.declarations > state.limits.maxSignals) fail("RESOURCE_LIMIT", `VCD contains more than ${state.limits.maxSignals} signal declarations`);
  const signalPath = [...scope, reference].filter(Boolean).join(".");
  if (state.paths.has(signalPath)) fail("MALFORMED_VCD", `Duplicate VCD signal path: ${signalPath}`);
  let group = state.groups.get(idCode);
  if (!group) {
    group = { idCode, width, vcdType: type, signedness: signednessFor(type), fourState: !["real", "realtime", "string"].includes(type), paths: [], changes: [] };
    state.groups.set(idCode, group);
  } else if (group.width !== width) {
    fail("MALFORMED_VCD", `VCD aliases for id ${idCode} have different widths`);
  } else if (group.signedness === "unknown" && signednessFor(type) !== "unknown") {
    group.signedness = signednessFor(type);
  }
  group.fourState = group.fourState || !["real", "realtime", "string"].includes(type);
  group.paths.push(signalPath);
  state.paths.set(signalPath, group);
  state.estimatedIndexBytes += 64 + signalPath.length * 2 + idCode.length * 2;
  if (state.estimatedIndexBytes > state.limits.maxIndexBytes) fail("RESOURCE_LIMIT", `Waveform index exceeds the ${state.limits.maxIndexBytes} byte estimated memory limit`);
}

function normalizeBits(value: string, width: number): string {
  const bits = value.toLowerCase();
  if (!/^[01xz]+$/.test(bits)) fail("MALFORMED_VCD", `Invalid four-state vector: ${value}`);
  const truncated = bits.length > width ? bits.slice(bits.length - width) : bits;
  const extension = truncated[0] === "x" || truncated[0] === "z" ? truncated[0] : "0";
  return extension.repeat(Math.max(0, width - truncated.length)) + truncated;
}

function parseDecimalTicks(input: string): bigint {
  const parsed = DecimalTicksSchema.safeParse(input);
  if (!parsed.success) fail("INVALID_TIME", `Time must be a nonnegative decimal tick string: ${input}`);
  return BigInt(parsed.data);
}

function makeLogicValue(bits: string, signedness: SignalSignedness): LogicValue {
  if (/[xz]/.test(bits)) return { bits, signedness, unsignedDecimal: null, signedDecimal: null };
  const unsigned = BigInt(`0b${bits}`);
  const signed = signedness === "signed" && bits[0] === "1" ? unsigned - (1n << BigInt(bits.length)) : unsigned;
  return {
    bits,
    signedness,
    unsignedDecimal: unsigned.toString(),
    signedDecimal: signedness === "signed" ? signed.toString() : null,
  };
}

function upperBound(changes: ValueChange[], tick: bigint): number {
  let low = 0;
  let high = changes.length;
  while (low < high) {
    const middle = low + Math.floor((high - low) / 2);
    if ((changes[middle]?.tick ?? 0n) <= tick) low = middle + 1;
    else high = middle;
  }
  return low;
}

function lowerBound(changes: ValueChange[], tick: bigint): number {
  let low = 0;
  let high = changes.length;
  while (low < high) {
    const middle = low + Math.floor((high - low) / 2);
    if ((changes[middle]?.tick ?? 0n) < tick) low = middle + 1;
    else high = middle;
  }
  return low;
}

function newIndexId(): string {
  if (typeof globalThis.crypto !== "undefined" && typeof globalThis.crypto.randomUUID === "function") return globalThis.crypto.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}-${Math.random().toString(36).slice(2)}`;
}

function encodeCursor(value: z.infer<typeof CursorEnvelopeSchema>): string {
  const bytes = new TextEncoder().encode(JSON.stringify(value));
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return `v1.${btoa(binary).replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_")}`;
}

function decodeCursor(cursor: string): z.infer<typeof CursorEnvelopeSchema> {
  if (cursor.length > 4096 || !cursor.startsWith("v1.")) fail("INVALID_CURSOR", "Cursor is not a supported waveform index cursor");
  try {
    const base64 = cursor.slice(3).replace(/-/g, "+").replace(/_/g, "/");
    const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
    const binary = atob(padded);
    const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
    const value: unknown = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    const parsed = CursorEnvelopeSchema.safeParse(value);
    if (!parsed.success) fail("INVALID_CURSOR", "Cursor payload is malformed");
    return parsed.data;
  } catch (error) {
    if (error instanceof WaveformIndexError) throw error;
    fail("INVALID_CURSOR", "Cursor payload is malformed");
  }
}

function descriptor(path: string, group: SignalGroup): SignalDescriptor {
  return { path, aliases: [...group.paths], idCode: group.idCode, width: group.width, vcdType: group.vcdType, signedness: group.signedness, fourState: group.fourState };
}

export class WaveformIndex {
  readonly startTicks: string;
  readonly endTicks: string;
  readonly timescale: { magnitude: string; unit: z.infer<typeof TimescaleUnitSchema> };
  readonly signalCount: number;
  readonly transitionCount: number;
  private readonly indexId = newIndexId();
  private readonly signalsByPath: Map<string, SignalGroup>;
  private readonly orderedPaths: string[];
  private readonly start: bigint;
  private readonly end: bigint;

  private constructor(state: ParserState) {
    if (!state.timescale) fail("MALFORMED_VCD", "VCD header has no $timescale declaration");
    this.timescale = state.timescale;
    this.start = state.firstTick ?? 0n;
    this.end = state.lastTick;
    this.startTicks = this.start.toString();
    this.endTicks = this.end.toString();
    this.signalsByPath = state.paths;
    this.orderedPaths = [...state.paths.keys()].sort((left, right) => left < right ? -1 : left > right ? 1 : 0);
    this.signalCount = this.orderedPaths.length;
    this.transitionCount = state.changeCount;
  }

  static async parse(input: VcdInput, options: WaveformIndexLimits = {}): Promise<WaveformIndex> {
    const limits: Required<WaveformIndexLimits> = {
      maxInputBytes: positiveLimit("maxInputBytes", options.maxInputBytes, DEFAULT_LIMITS.maxInputBytes),
      maxSignals: positiveLimit("maxSignals", options.maxSignals, DEFAULT_LIMITS.maxSignals),
      maxChanges: positiveLimit("maxChanges", options.maxChanges, DEFAULT_LIMITS.maxChanges),
      maxIndexBytes: positiveLimit("maxIndexBytes", options.maxIndexBytes, DEFAULT_LIMITS.maxIndexBytes),
      maxSignalWidth: positiveLimit("maxSignalWidth", options.maxSignalWidth, DEFAULT_LIMITS.maxSignalWidth),
      maxTokenChars: positiveLimit("maxTokenChars", options.maxTokenChars, DEFAULT_LIMITS.maxTokenChars),
    };
    const state: ParserState = {
      groups: new Map(),
      paths: new Map(),
      timescale: null,
      currentTick: 0n,
      firstTick: null,
      lastTick: 0n,
      declarations: 0,
      changeCount: 0,
      estimatedIndexBytes: 0,
      limits,
    };
    const iterator = tokenize(input, limits)[Symbol.asyncIterator]();
    const scopes: string[] = [];
    let definitionsEnded = false;

    while (!definitionsEnded) {
      const next = await iterator.next();
      if (next.done) fail("MALFORMED_VCD", "VCD header is missing $enddefinitions");
      const token = next.value;
      if (token === "$date" || token === "$version" || token === "$comment") {
        await skipToEnd(iterator);
      } else if (token === "$timescale") {
        const timescaleTokens: string[] = [];
        for (;;) {
          const item = await readRequired(iterator, "$timescale $end");
          if (item === "$end") break;
          timescaleTokens.push(item);
        }
        state.timescale = parseTimescale(timescaleTokens);
      } else if (token === "$scope") {
        await readRequired(iterator, "scope type");
        scopes.push(await readRequired(iterator, "scope name"));
        if (await readRequired(iterator, "$scope $end") !== "$end") fail("MALFORMED_VCD", "Malformed $scope declaration");
      } else if (token === "$upscope") {
        if (scopes.length === 0) fail("MALFORMED_VCD", "$upscope has no matching $scope");
        scopes.pop();
        if (await readRequired(iterator, "$upscope $end") !== "$end") fail("MALFORMED_VCD", "Malformed $upscope declaration");
      } else if (token === "$var") {
        const type = await readRequired(iterator, "variable type");
        const width = await readRequired(iterator, "variable width");
        const idCode = await readRequired(iterator, "variable id code");
        const reference = await readRequired(iterator, "variable reference");
        for (;;) {
          const item = await readRequired(iterator, "$var $end");
          if (item === "$end") break;
        }
        addVariable(state, scopes, type, width, idCode, reference);
      } else if (token === "$enddefinitions") {
        if (await readRequired(iterator, "$enddefinitions $end") !== "$end") fail("MALFORMED_VCD", "Malformed $enddefinitions directive");
        if (scopes.length !== 0) fail("MALFORMED_VCD", "VCD header ended with unclosed scopes");
        definitionsEnded = true;
      } else if (token.startsWith("$")) {
        // Header extensions are ignored only when they use the standard
        // command/$end framing. Variable and scope commands are parsed above.
        await skipToEnd(iterator);
      } else {
        fail("MALFORMED_VCD", `Unexpected token in VCD header: ${token}`);
      }
    }

    for (;;) {
      const next = await iterator.next();
      if (next.done) break;
      const token = next.value;
      if (!token) continue;
      if (token.startsWith("#")) {
        const tickText = token.slice(1);
        const tick = parseDecimalTicks(tickText);
        if (state.firstTick === null) state.firstTick = tick;
        if (tick < state.currentTick) fail("MALFORMED_VCD", "VCD timestamps must be nondecreasing");
        state.currentTick = tick;
        state.lastTick = tick;
        continue;
      }
      if (token === "$end" || token === "$dumpvars" || token === "$dumpall" || token === "$dumpon" || token === "$dumpoff" || token === "$dumpports" || token === "$dumpportsoff" || token === "$dumpportson") continue;
      if (token === "$comment") {
        await skipToEnd(iterator);
        continue;
      }
      const initial = token[0]?.toLowerCase();
      if (initial === "b") {
        const idCode = await readRequired(iterator, "vector id code");
        recordChange(state, idCode, token.slice(1));
      } else if (initial === "0" || initial === "1" || initial === "x" || initial === "z") {
        recordChange(state, token.slice(1), initial);
      } else if (initial === "r" || initial === "s") {
        // Real and string changes are not four-state logic values. Consume
        // their id code so later records remain aligned, then leave them out
        // of this logic-vector index.
        await readRequired(iterator, "real/string id code");
      } else if (token.startsWith("$")) {
        fail("UNSUPPORTED_VCD", `Unsupported VCD data directive: ${token}`);
      } else {
        fail("MALFORMED_VCD", `Unexpected value-change token: ${token}`);
      }
    }

    if (state.groups.size === 0) fail("MALFORMED_VCD", "VCD file contains no declared signals");
    return new WaveformIndex(state);
  }

  querySignal(signalPath: string, timeTicks: string, interpretation: SignalInterpretation = {}): SignalSample {
    const group = this.requireSignal(signalPath);
    if (!group.fourState) fail("UNSUPPORTED_VCD", `Signal ${signalPath} is not a four-state logic vector`);
    const tick = parseDecimalTicks(timeTicks);
    this.checkTick(tick);
    const index = upperBound(group.changes, tick) - 1;
    const change = group.changes[index];
    const signedness = interpretation.signed === undefined ? group.signedness : interpretation.signed ? "signed" : "unsigned";
    return {
      ...descriptor(signalPath, group),
      signedness,
      timeTicks: tick.toString(),
      timescale: this.timescale,
      value: change ? makeLogicValue(change.bits, signedness) : null,
    };
  }

  listTransitions(signalPath: string, startTicks: string, endTicks: string, pageSize = 100, cursor?: string, interpretation: SignalInterpretation = {}): TransitionPage {
    const group = this.requireSignal(signalPath);
    if (!group.fourState) fail("UNSUPPORTED_VCD", `Signal ${signalPath} is not a four-state logic vector`);
    if (!Number.isInteger(pageSize) || pageSize < 1 || pageSize > 1_000) fail("INVALID_PAGE_SIZE", "pageSize must be an integer from 1 through 1000");
    const start = parseDecimalTicks(startTicks);
    const end = parseDecimalTicks(endTicks);
    if (start > end) fail("INVALID_RANGE", "Transition range start must be less than or equal to end");
    this.checkTick(start);
    this.checkTick(end);
    const first = lowerBound(group.changes, start);
    const last = upperBound(group.changes, end);
    const signedness = interpretation.signed === undefined ? group.signedness : interpretation.signed ? "signed" : "unsigned";
    let offset = first;
    if (cursor !== undefined) {
      const parsed = decodeCursor(cursor);
      if (parsed.kind !== "transitions" || parsed.indexId !== this.indexId || parsed.signalPath !== signalPath || parsed.start !== start.toString() || parsed.end !== end.toString() || parsed.interpretation !== signedness) {
        fail("INVALID_CURSOR", "Cursor does not belong to this signal, range, or waveform index");
      }
      if (parsed.offset < first || parsed.offset > last) fail("INVALID_CURSOR", "Cursor offset is outside the selected transition range");
      offset = parsed.offset;
    }
    const pageEnd = Math.min(last, offset + pageSize);
    const transitions = group.changes.slice(offset, pageEnd).map((change) => ({
      timeTicks: change.tick.toString(),
      value: makeLogicValue(change.bits, signedness),
    }));
    const nextCursor = pageEnd < last
      ? encodeCursor({ v: 1, indexId: this.indexId, kind: "transitions", signalPath, start: start.toString(), end: end.toString(), interpretation: signedness, offset: pageEnd })
      : undefined;
    return {
      signalPath,
      startTicks: start.toString(),
      endTicks: end.toString(),
      transitions,
      ...(nextCursor === undefined ? {} : { nextCursor }),
    };
  }

  listSignals(pageSize = 100, cursor?: string): SignalPage {
    if (!Number.isInteger(pageSize) || pageSize < 1 || pageSize > 1_000) fail("INVALID_PAGE_SIZE", "pageSize must be an integer from 1 through 1000");
    let offset = 0;
    if (cursor !== undefined) {
      const parsed = decodeCursor(cursor);
      if (parsed.kind !== "signals" || parsed.indexId !== this.indexId) fail("INVALID_CURSOR", "Cursor does not belong to this waveform index");
      if (parsed.offset > this.orderedPaths.length) fail("INVALID_CURSOR", "Signal cursor is outside the list");
      offset = parsed.offset;
    }
    const pageEnd = Math.min(this.orderedPaths.length, offset + pageSize);
    const signals = this.orderedPaths.slice(offset, pageEnd).map((path) => descriptor(path, this.signalsByPath.get(path)!));
    const nextCursor = pageEnd < this.orderedPaths.length
      ? encodeCursor({ v: 1, indexId: this.indexId, kind: "signals", offset: pageEnd })
      : undefined;
    return { signals, ...(nextCursor === undefined ? {} : { nextCursor }) };
  }

  private requireSignal(signalPath: string): SignalGroup {
    const group = this.signalsByPath.get(signalPath);
    if (!group) fail("UNKNOWN_SIGNAL", `Unknown VCD signal: ${signalPath}`);
    return group;
  }

  private checkTick(tick: bigint): void {
    if (tick < this.start || tick > this.end) fail("TIME_OUT_OF_RANGE", `Time ${tick} is outside VCD range ${this.start}..${this.end} ticks`);
  }
}

function recordChange(state: ParserState, idCode: string, value: string): void {
  const group = state.groups.get(idCode);
  if (!group) fail("MALFORMED_VCD", `Value change references undeclared id code: ${idCode}`);
  const bits = normalizeBits(value, group.width);
  state.changeCount += 1;
  if (state.changeCount > state.limits.maxChanges) fail("RESOURCE_LIMIT", `VCD exceeds ${state.limits.maxChanges} indexed changes`);
  state.estimatedIndexBytes += 64 + bits.length * 2;
  if (state.estimatedIndexBytes > state.limits.maxIndexBytes) fail("RESOURCE_LIMIT", `Waveform index exceeds the ${state.limits.maxIndexBytes} byte estimated memory limit`);
  group.changes.push({ tick: state.currentTick, bits });
  if (state.firstTick === null) state.firstTick = state.currentTick;
}

export async function parseVcd(input: VcdInput, options?: WaveformIndexLimits): Promise<WaveformIndex> {
  return WaveformIndex.parse(input, options);
}
