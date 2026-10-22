import { afterEach, describe, expect, it } from "vitest";
import { DpiWasmModuleError, installDpiWasmModules } from "../src/workers/dpiWasmModules.js";

// wasm-ld output for: int add_ints(int a, int b) { return a + b; }
const addIntsModule = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f,
  0x03, 0x02, 0x01, 0x00,
  0x07, 0x0c, 0x01, 0x08, 0x61, 0x64, 0x64, 0x5f, 0x69, 0x6e, 0x74, 0x73, 0x00, 0x00,
  0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
]);

const returnsI64Module = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7e,
  0x03, 0x02, 0x01, 0x00,
  0x07, 0x0f, 0x01, 0x0b, 0x72, 0x65, 0x74, 0x75, 0x72, 0x6e, 0x73, 0x5f, 0x69, 0x36, 0x34, 0x00, 0x00,
  0x0a, 0x06, 0x01, 0x04, 0x00, 0x42, 0x00, 0x0b,
]);

const oversizedMemoryModule = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  0x05, 0x06, 0x01, 0x01, 0x80, 0x08, 0x80, 0x10,
]);

function moduleInput(path = "dpi/add.wasm", bytes: Uint8Array = addIntsModule) {
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  return { path, bytes: buffer };
}

let uninstall: (() => void) | undefined;
afterEach(() => {
  uninstall?.();
  uninstall = undefined;
});

describe("browser DPI Wasm modules", () => {
  it("loads an import-free scalar function and restores the previous dispatcher", async () => {
    const previous = () => 7;
    Reflect.set(globalThis, "__xezimDpiCall", previous);
    uninstall = await installDpiWasmModules([moduleInput()]);
    const dispatcher = Reflect.get(globalThis, "__xezimDpiCall") as (name: string, args: number[]) => number;

    expect(dispatcher("add_ints", [10, 20])).toBe(30);
    uninstall();
    uninstall = undefined;
    expect(Reflect.get(globalThis, "__xezimDpiCall")).toBe(previous);
    Reflect.deleteProperty(globalThis, "__xezimDpiCall");
  });

  it("rejects host imports instead of instantiating an ambient capability", async () => {
    // Valid module with one imported function (`env.foo`).
    const imported = new Uint8Array([
      0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
      0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
      0x02, 0x0b, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x03, 0x66, 0x6f, 0x6f, 0x00, 0x00,
    ]);
    await expect(installDpiWasmModules([moduleInput("dpi/imported.wasm", imported)]))
      .rejects.toMatchObject<Partial<DpiWasmModuleError>>({ code: "UNSUPPORTED_CAPABILITY" });
  });

  it("rejects invalid binaries and reports ambiguous DPI symbols when called", async () => {
    await expect(installDpiWasmModules([moduleInput("dpi/broken.wasm", new Uint8Array([0, 1]))]))
      .rejects.toMatchObject<Partial<DpiWasmModuleError>>({ code: "INVALID_REQUEST" });
    uninstall = await installDpiWasmModules([moduleInput("dpi/one.wasm"), moduleInput("dpi/two.wasm")]);
    const dispatcher = Reflect.get(globalThis, "__xezimDpiCall") as (name: string, args: number[]) => number;
    expect(() => dispatcher("add_ints", [1, 2])).toThrow("exported by more than one Wasm module");
  });

  it("rejects non-int Wasm signatures and linear memory above the cap", async () => {
    uninstall = await installDpiWasmModules([moduleInput("dpi/i64.wasm", returnsI64Module)]);
    const dispatcher = Reflect.get(globalThis, "__xezimDpiCall") as (name: string, args: number[]) => number;
    expect(() => dispatcher("returns_i64", [])).toThrow("must have a WebAssembly signature of (i32, ...) -> i32");
    uninstall();
    uninstall = undefined;
    await expect(installDpiWasmModules([moduleInput("dpi/large-memory.wasm", oversizedMemoryModule)]))
      .rejects.toMatchObject<Partial<DpiWasmModuleError>>({ code: "RESOURCE_LIMIT" });
  });

  it("fails explicitly for missing C symbols and invalid return values", async () => {
    uninstall = await installDpiWasmModules([moduleInput()]);
    const dispatcher = Reflect.get(globalThis, "__xezimDpiCall") as (name: string, args: number[]) => number;
    expect(() => dispatcher("missing", [1, 2])).toThrow("no loaded Wasm module exports");
    expect(() => dispatcher("add_ints", [1])).toThrow("expects 2 arguments");
  });
});
