import type { DpiWasmModuleInput } from "./xezimProtocol.js";

const MAX_DPI_MODULES = 32;
const MAX_DPI_MODULE_BYTES = 16 * 1024 * 1024;
const MAX_DPI_TOTAL_BYTES = 32 * 1024 * 1024;
const MAX_DPI_LINEAR_MEMORY_BYTES = 64 * 1024 * 1024;
const DPI_DISPATCHER_KEY = "__xezimDpiCall";

type DpiDispatcher = (cName: string, args: number[]) => number;
type FunctionSignature = { params: number[]; results: number[] };

function readU32(bytes: Uint8Array, offset: number, end: number): { value: number; next: number } {
  let value = 0;
  let shift = 0;
  for (let index = 0; index < 5; index += 1) {
    if (offset >= end) throw new Error("truncated WebAssembly section");
    const byte = bytes[offset++]!;
    value |= (byte & 0x7f) << shift;
    if ((byte & 0x80) === 0) return { value: value >>> 0, next: offset };
    shift += 7;
  }
  throw new Error("oversized WebAssembly integer");
}

function parseFunctionSignatures(bytes: Uint8Array): Map<string, FunctionSignature> {
  if (bytes.length < 8 || bytes[0] !== 0 || bytes[1] !== 0x61 || bytes[2] !== 0x73 || bytes[3] !== 0x6d) {
    throw new Error("invalid WebAssembly header");
  }
  const types: FunctionSignature[] = [];
  const functionTypeIndices: number[] = [];
  const exports = new Map<string, number>();
  let offset = 8;
  const decoder = new TextDecoder("utf-8", { fatal: true });
  while (offset < bytes.length) {
    const sectionId = bytes[offset++]!;
    const sectionSize = readU32(bytes, offset, bytes.length);
    offset = sectionSize.next;
    const end = offset + sectionSize.value;
    if (end > bytes.length) throw new Error("truncated WebAssembly section");

    if (sectionId === 1) {
      const count = readU32(bytes, offset, end);
      offset = count.next;
      for (let i = 0; i < count.value; i += 1) {
        if (offset >= end || bytes[offset++] !== 0x60) throw new Error("unsupported WebAssembly type");
        const paramCount = readU32(bytes, offset, end);
        offset = paramCount.next;
        if (paramCount.value > 1024 || offset + paramCount.value > end) throw new Error("invalid WebAssembly function type");
        const params = Array.from(bytes.slice(offset, offset + paramCount.value));
        offset += paramCount.value;
        const resultCount = readU32(bytes, offset, end);
        offset = resultCount.next;
        if (resultCount.value > 16 || offset + resultCount.value > end) throw new Error("invalid WebAssembly function type");
        const results = Array.from(bytes.slice(offset, offset + resultCount.value));
        offset += resultCount.value;
        types.push({ params, results });
      }
      if (offset !== end) throw new Error("invalid WebAssembly type section");
    } else if (sectionId === 3) {
      const count = readU32(bytes, offset, end);
      offset = count.next;
      for (let i = 0; i < count.value; i += 1) {
        const type = readU32(bytes, offset, end);
        functionTypeIndices.push(type.value);
        offset = type.next;
      }
      if (offset !== end) throw new Error("invalid WebAssembly function section");
    } else if (sectionId === 5) {
      const count = readU32(bytes, offset, end);
      offset = count.next;
      if (count.value > 1) throw new Error("multiple linear memories are unsupported");
      for (let i = 0; i < count.value; i += 1) {
        const flags = readU32(bytes, offset, end);
        offset = flags.next;
        if (flags.value !== 1) {
          throw new Error("DPI Wasm memory must be unshared, 32-bit, and declare a maximum");
        }
        const minimum = readU32(bytes, offset, end);
        const maximum = readU32(bytes, minimum.next, end);
        offset = maximum.next;
        const maxPages = MAX_DPI_LINEAR_MEMORY_BYTES / 65536;
        if (minimum.value > maxPages || maximum.value > maxPages || minimum.value > maximum.value) {
          throw new RangeError(`DPI Wasm memory must stay within ${MAX_DPI_LINEAR_MEMORY_BYTES} bytes`);
        }
      }
      if (offset !== end) throw new Error("invalid WebAssembly memory section");
    } else if (sectionId === 7) {
      const count = readU32(bytes, offset, end);
      offset = count.next;
      for (let i = 0; i < count.value; i += 1) {
        const nameLength = readU32(bytes, offset, end);
        offset = nameLength.next;
        if (offset + nameLength.value + 1 > end) throw new Error("truncated WebAssembly export");
        const name = decoder.decode(bytes.slice(offset, offset + nameLength.value));
        offset += nameLength.value;
        const kind = bytes[offset++]!;
        const itemIndex = readU32(bytes, offset, end);
        offset = itemIndex.next;
        if (kind === 0) {
          if (exports.has(name)) throw new Error(`duplicate function export ${name}`);
          exports.set(name, itemIndex.value);
        }
      }
      if (offset !== end) throw new Error("invalid WebAssembly export section");
    }
    offset = end;
  }

  const signatures = new Map<string, FunctionSignature>();
  for (const [name, functionIndex] of exports) {
    const typeIndex = functionTypeIndices[functionIndex];
    const signature = typeIndex === undefined ? undefined : types[typeIndex];
    if (!signature) throw new Error(`missing type for WebAssembly export ${name}`);
    signatures.set(name, signature);
  }
  return signatures;
}

export class DpiWasmModuleError extends Error {
  constructor(readonly code: "INVALID_REQUEST" | "UNSUPPORTED_CAPABILITY" | "RESOURCE_LIMIT" | "INTERNAL_ERROR", message: string) {
    super(message);
    this.name = "DpiWasmModuleError";
  }
}

/** Installs a synchronous registry for standalone, import-free DPI Wasm modules. */
export async function installDpiWasmModules(
  modules: readonly DpiWasmModuleInput[] = [],
): Promise<() => void> {
  if (modules.length > MAX_DPI_MODULES) {
    throw new DpiWasmModuleError("RESOURCE_LIMIT", `A project may load at most ${MAX_DPI_MODULES} DPI Wasm modules.`);
  }

  let totalBytes = 0;
  const functions = new Map<string, Array<{ value: (...args: number[]) => number; signature: FunctionSignature }>>();
  for (const input of modules) {
    if (!input || typeof input.path !== "string" || !(input.bytes instanceof ArrayBuffer)) {
      throw new DpiWasmModuleError("INVALID_REQUEST", "Each DPI module needs a project path and WebAssembly bytes.");
    }
    const path = input.path.replaceAll("\\", "/");
    if (!path || path.startsWith("/") || /^[A-Za-z]:/.test(path) || path.split("/").includes("..")) {
      throw new DpiWasmModuleError("INVALID_REQUEST", `DPI module path is not project-relative: ${input.path}`);
    }
    if (input.bytes.byteLength === 0 || input.bytes.byteLength > MAX_DPI_MODULE_BYTES) {
      throw new DpiWasmModuleError("RESOURCE_LIMIT", `DPI module ${path} must be between 1 byte and ${MAX_DPI_MODULE_BYTES} bytes.`);
    }
    totalBytes += input.bytes.byteLength;
    if (totalBytes > MAX_DPI_TOTAL_BYTES) {
      throw new DpiWasmModuleError("RESOURCE_LIMIT", `DPI modules exceed the ${MAX_DPI_TOTAL_BYTES} byte project limit.`);
    }

    let module: WebAssembly.Module;
    try {
      module = await WebAssembly.compile(input.bytes);
    } catch (error) {
      throw new DpiWasmModuleError("INVALID_REQUEST", `DPI module ${path} is not valid WebAssembly: ${error instanceof Error ? error.message : "compile failed"}`);
    }
    const imports = WebAssembly.Module.imports(module);
    if (imports.length > 0) {
      const names = imports.map(({ module: namespace, name }) => `${namespace}.${name}`).join(", ");
      throw new DpiWasmModuleError("UNSUPPORTED_CAPABILITY", `DPI module ${path} imports host or shared-module functions (${names}); only standalone import-free modules are supported.`);
    }

    let signatures: Map<string, FunctionSignature>;
    try {
      signatures = parseFunctionSignatures(new Uint8Array(input.bytes));
    } catch (error) {
      const code = error instanceof RangeError ? "RESOURCE_LIMIT" : "UNSUPPORTED_CAPABILITY";
      throw new DpiWasmModuleError(code, `DPI module ${path} has an unsupported WebAssembly layout: ${error instanceof Error ? error.message : "invalid module"}`);
    }

    let instance: WebAssembly.Instance;
    try {
      instance = await WebAssembly.instantiate(module, {});
    } catch (error) {
      throw new DpiWasmModuleError("INVALID_REQUEST", `DPI module ${path} could not be instantiated: ${error instanceof Error ? error.message : "instantiation failed"}`);
    }
    for (const [name, exported] of Object.entries(instance.exports)) {
      if (typeof exported !== "function") continue;
      const signature = signatures.get(name);
      if (!signature) throw new DpiWasmModuleError("INVALID_REQUEST", `DPI module ${path} is missing an export type for ${name}.`);
      const exports = functions.get(name) ?? [];
      exports.push({ value: exported as (...args: number[]) => number, signature });
      functions.set(name, exports);
    }
  }

  const previous = Reflect.get(globalThis, DPI_DISPATCHER_KEY);
  const dispatcher: DpiDispatcher = (cName, args) => {
    const targets = functions.get(cName);
    if (!targets?.length) {
      throw new Error(`no loaded Wasm module exports C symbol '${cName}'`);
    }
    if (targets.length > 1) {
      throw new Error(`C symbol '${cName}' is exported by more than one Wasm module`);
    }
    const target = targets[0]!;
    if (!target.signature.params.every((type) => type === 0x7f)
      || target.signature.results.length !== 1 || target.signature.results[0] !== 0x7f) {
      throw new Error(`C symbol '${cName}' must have a WebAssembly signature of (i32, ...) -> i32`);
    }
    if (target.signature.params.length !== args.length) {
      throw new Error(`C symbol '${cName}' expects ${target.signature.params.length} arguments, received ${args.length}`);
    }
    const value = target.value(...args);
    if (typeof value !== "number" || !Number.isInteger(value)
      || value < -0x8000_0000 || value > 0x7fff_ffff) {
      throw new Error(`C symbol '${cName}' did not return a signed 32-bit integer`);
    }
    return value;
  };
  if (!Reflect.set(globalThis, DPI_DISPATCHER_KEY, dispatcher)) {
    throw new DpiWasmModuleError("INTERNAL_ERROR", "The browser DPI dispatcher could not be installed.");
  }

  return () => {
    if (previous === undefined) Reflect.deleteProperty(globalThis, DPI_DISPATCHER_KEY);
    else Reflect.set(globalThis, DPI_DISPATCHER_KEY, previous);
  };
}
