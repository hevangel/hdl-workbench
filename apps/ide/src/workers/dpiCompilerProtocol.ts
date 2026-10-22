export interface DpiCompilerFile {
  path: string;
  contents: string;
}

export interface DpiCompilerRequest {
  id: string;
  files: DpiCompilerFile[];
}

export interface CompiledDpiModule {
  path: string;
  bytes: ArrayBuffer;
}

export type DpiCompilerResponse =
  | { id: string; ok: true; modules: CompiledDpiModule[]; output: string }
  | { id: string; ok: false; error: { code: string; message: string } };
