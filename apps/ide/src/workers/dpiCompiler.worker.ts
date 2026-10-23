import * as browserccModule from "browsercc";
import type { FileList as BrowserccFileList } from "browsercc";
import type { DpiCompilerFile, DpiCompilerRequest, DpiCompilerResponse } from "./dpiCompilerProtocol.js";
import browserccSysrootUrl from "browsercc/dist/sysroot.tar?url";

interface CompilerWorkerScope {
  onmessage: ((event: MessageEvent<DpiCompilerRequest>) => void) | null;
  postMessage(message: DpiCompilerResponse, transfer?: Transferable[]): void;
}

const workerScope = globalThis as unknown as CompilerWorkerScope;
const MAX_FILES = 64;
const MAX_SOURCE_UNITS = 16;
const MAX_SOURCE_BYTES = 4 * 1024 * 1024;
const MAX_PROJECT_BYTES = 16 * 1024 * 1024;
const MAX_COMPILER_OUTPUT_BYTES = 64 * 1024;
const MAX_MODULE_BYTES = 16 * 1024 * 1024;

interface CompilerRuntime {
  FS: {
    writeFile(path: string, contents: string | Uint8Array): void;
    readFile(path: string, options: { encoding: "binary" }): Uint8Array;
  };
  callMain(args: string[]): number;
}

interface CompilerInvocation {
  compilerArgs: string[];
  compilerArtifact: string;
  linkerArgs: string[];
  linerArtifact: string;
}

interface BrowserccApi {
  Clang(options: { thisProgram: string; printErr: (message: string) => void }): Promise<CompilerRuntime>;
  LLD(options: { thisProgram: string; printErr: (message: string) => void }): Promise<CompilerRuntime>;
  getCompilerInvocation(fileName: string, source: string, flags: string[]): Promise<CompilerInvocation>;
  setUpSysroot(runtime: CompilerRuntime, tar: ArrayBuffer, extraFiles: BrowserccFileList): void;
}

const browsercc = browserccModule as unknown as BrowserccApi;
let sysrootPromise: Promise<ArrayBuffer> | undefined;

function fail(id: string, code: string, message: string): DpiCompilerResponse {
  return { id, ok: false, error: { code, message: message.slice(0, 4096) } };
}

function extension(path: string): string {
  return path.split("/").at(-1)?.split(".").at(-1)?.toLowerCase() ?? "";
}

function sourceKind(path: string): "c" | "cpp" | null {
  const ext = extension(path);
  if (ext === "c") return "c";
  if (["cc", "cpp", "cxx"].includes(ext)) return "cpp";
  return null;
}

function isHeader(path: string): boolean {
  return ["h", "hh", "hpp", "hxx", "inc"].includes(extension(path));
}

function validProjectPath(path: string): boolean {
  const normalized = path.replaceAll("\\", "/");
  return normalized.length > 0
    && normalized === path
    && !normalized.startsWith("/")
    && !/^[A-Za-z]:/.test(normalized)
    && !normalized.split("/").includes("..")
    && !normalized.split("/").includes("");
}

function includeDirectories(sourcePath: string, headers: DpiCompilerFile[]): string[] {
  const dirs = new Set(["."]);
  for (const path of [sourcePath, ...headers.map(({ path }) => path)]) {
    const index = path.lastIndexOf("/");
    if (index > 0) dirs.add(path.slice(0, index));
  }
  return [...dirs].flatMap((dir) => ["-I", dir]);
}

function compilerFlags(kind: "c" | "cpp", sourcePath: string, headers: DpiCompilerFile[]): string[] {
  const flags = [
    "--target=wasm32-wasi",
    "-O2",
    "-nostdlib",
    "-fno-exceptions",
    "-fno-rtti",
    "-Wl,--no-entry",
    "-Wl,--export-all",
    "-Wl,--initial-memory=16777216",
    "-Wl,--max-memory=67108864",
    ...includeDirectories(sourcePath, headers),
  ];
  // browsercc asks Clang's driver for a `clang++` invocation. `-x c` switches
  // the translation unit language, while its driver rejects C-only -std flags.
  // LLVM 20's default C mode is gnu17; don't pass a C++-driver-incompatible flag.
  if (kind === "c") flags.push("-x", "c");
  else flags.push("-std=c++20");
  return flags;
}

function errorOutput(output: string): string {
  const trimmed = output.trim();
  return trimmed || "Clang/LLD did not produce a WebAssembly module.";
}

async function compileTranslationUnit(input: {
  source: string;
  fileName: string;
  flags: string[];
  extraFiles: BrowserccFileList;
}, kind: "c" | "cpp"): Promise<{ compileOutput: string; bytes: ArrayBuffer | null }> {
  let compileOutput = "";
  const collect = (message: string) => { compileOutput = `${compileOutput}${message}\n`; };
  const compilerPromise = browsercc.Clang({ thisProgram: "clang++", printErr: collect });
  const linkerPromise = browsercc.LLD({ thisProgram: "wasm-ld", printErr: collect });
  sysrootPromise ??= fetch(browserccSysrootUrl).then(async (response) => {
    if (!response.ok) throw new Error(`Could not load browsercc WASI sysroot (${response.status}).`);
    return response.arrayBuffer();
  });
  const [compilerInvocation, sysroot] = await Promise.all([
    browsercc.getCompilerInvocation(input.fileName, input.source, input.flags),
    sysrootPromise,
  ]);
  // browsercc's getCompilerInvocation uses clang++ as argv[0]. Its driver
  // reports a C++ frontend even for .c input, so explicitly override the cc1
  // language after those derived args to compile the source as actual C.
  if (kind === "c") compilerInvocation.compilerArgs.push("-x", "c");
  const compiler = await compilerPromise;
  compiler.FS.writeFile(input.fileName, input.source);
  browsercc.setUpSysroot(compiler, sysroot, input.extraFiles);
  if (compiler.callMain(compilerInvocation.compilerArgs) !== 0) {
    return { compileOutput, bytes: null };
  }
  const object = compiler.FS.readFile(compilerInvocation.compilerArtifact, { encoding: "binary" });
  const linker = await linkerPromise;
  linker.FS.writeFile(compilerInvocation.compilerArtifact, object);
  browsercc.setUpSysroot(linker, sysroot, input.extraFiles);
  if (linker.callMain(compilerInvocation.linkerArgs) !== 0) {
    return { compileOutput, bytes: null };
  }
  const wasm = linker.FS.readFile(compilerInvocation.linerArtifact, { encoding: "binary" });
  if (wasm.byteLength === 0 || wasm.byteLength > MAX_MODULE_BYTES) {
    return { compileOutput, bytes: null };
  }
  const wasmBytes = new Uint8Array(wasm.byteLength);
  wasmBytes.set(wasm);
  return { compileOutput, bytes: wasmBytes.buffer };
}

async function compileProject(request: DpiCompilerRequest): Promise<DpiCompilerResponse> {
  const { id, files } = request;
  if (!id || id.length > 256 || !Array.isArray(files) || files.length === 0 || files.length > MAX_FILES) {
    return fail(id ?? "", "INVALID_REQUEST", "DPI compilation requires 1 to 64 project files and a request ID.");
  }

  let projectBytes = 0;
  for (const file of files) {
    if (!file || typeof file.path !== "string" || typeof file.contents !== "string" || !validProjectPath(file.path)) {
      return fail(id, "INVALID_PATH", "DPI source and header files must use safe project-relative paths.");
    }
    const bytes = new TextEncoder().encode(file.contents).byteLength;
    projectBytes += new TextEncoder().encode(file.path).byteLength + bytes;
    if (bytes > MAX_SOURCE_BYTES || projectBytes > MAX_PROJECT_BYTES) {
      return fail(id, "RESOURCE_LIMIT", `DPI source files exceed the ${MAX_PROJECT_BYTES} byte compiler limit.`);
    }
  }

  const sources = files.filter((file) => sourceKind(file.path) !== null);
  if (sources.length === 0 || sources.length > MAX_SOURCE_UNITS) {
    return fail(id, "INVALID_REQUEST", `Add between 1 and ${MAX_SOURCE_UNITS} C or C++ source files to compile.`);
  }
  const headers = files.filter((file) => isHeader(file.path));
  const extraFiles: BrowserccFileList = Object.fromEntries(headers.map(({ path, contents }) => [path, contents]));
  const modules: Array<{ path: string; bytes: ArrayBuffer }> = [];
  let allOutput = "";

  for (const [index, source] of sources.entries()) {
    const kind = sourceKind(source.path)!;
    const fileName = `xezim_dpi_${index}.${kind === "c" ? "c" : "cpp"}`;
    let result;
    try {
      const job = {
        source: source.contents,
        fileName,
        flags: compilerFlags(kind, source.path, headers),
        extraFiles,
      };
      result = await compileTranslationUnit(job, kind);
    } catch (error) {
      return fail(id, "DPI_COMPILATION_FAILED", error instanceof Error ? error.message : "The browser C/C++ compiler failed.");
    }
    allOutput = `${allOutput}${result.compileOutput}`.slice(-MAX_COMPILER_OUTPUT_BYTES);
    if (!result.bytes) {
      const message = allOutput.trim() || `Compiled module ${source.path} exceeds the ${MAX_MODULE_BYTES} byte WebAssembly module limit.`;
      return fail(id, "DPI_COMPILATION_FAILED", errorOutput(message));
    }
    modules.push({ path: source.path, bytes: result.bytes });
  }

  return { id, ok: true, modules, output: allOutput.slice(-MAX_COMPILER_OUTPUT_BYTES) };
}

workerScope.onmessage = async (event: MessageEvent<DpiCompilerRequest>) => {
  const request = event.data;
  if (!request || typeof request.id !== "string") return;
  try {
    const response = await compileProject(request);
    workerScope.postMessage(response, response.ok ? response.modules.map(({ bytes }) => bytes) : []);
  } catch (error) {
    workerScope.postMessage(fail(request.id, "INTERNAL_ERROR", error instanceof Error ? error.message : "The browser DPI compiler worker failed."));
  }
};
