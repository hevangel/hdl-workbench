import { McpServer } from "@modelcontextprotocol/server";
import { z } from "zod";
import { ArtifactIdSchema, CompileIdSchema, DecimalTicksSchema, ProjectFileSchema, ProjectPathSchema, RevisionSchema, RunIdSchema, TimescaleSchema } from "@xezim-ide/protocol";
import { asBridgeError } from "./errors.js";
import { IdeMcpService } from "./service.js";

function resultContent(value: unknown): { content: [{ type: "text"; text: string }] } {
  return { content: [{ type: "text", text: JSON.stringify(value) }] };
}

function errorContent(error: unknown): { content: [{ type: "text"; text: string }]; isError: true } {
  const bridgeError = asBridgeError(error);
  return { content: [{ type: "text", text: JSON.stringify(bridgeError.toJSON()) }], isError: true };
}

function tool<T extends z.ZodRawShape>(
  server: McpServer,
  name: string,
  description: string,
  inputSchema: z.ZodObject<T>,
  handler: (input: z.infer<z.ZodObject<T>>) => Promise<unknown>,
): void {
  server.registerTool(name, { description, inputSchema }, async (input) => {
    try {
      return resultContent(await handler(input as z.infer<z.ZodObject<T>>));
    } catch (error) {
      return errorContent(error);
    }
  });
}

export function createMcpServer(service: IdeMcpService): McpServer {
  const server = new McpServer({ name: "xezim-studio", version: "0.1.0" });
  tool(server, "get_connection_status", "Check browser bridge status and receive a short-lived pairing code.", z.strictObject({}), () => service.getConnectionStatus());
  tool(server, "list_project_files", "List files inside the explicitly selected project root.", z.strictObject({}), () => service.listProjectFiles());
  tool(server, "read_project_file", "Read a project-relative file and its revision.", z.strictObject({ path: ProjectPathSchema }), ({ path }) => service.readProjectFile(path));
  tool(server, "write_project_file", "Write a project-relative file only when expected_revision matches its current revision (null creates a new file).", z.strictObject({
    path: ProjectPathSchema,
    expected_revision: RevisionSchema.nullable(),
    content_base64: ProjectFileSchema.shape.contentBase64,
  }), ({ path, expected_revision, content_base64 }) => service.writeProjectFile(path, content_base64, expected_revision));
  tool(server, "compile", "Synchronize the selected project snapshot to the paired browser and compile it there.", z.strictObject({
    top_module: z.string().min(1).max(512),
    include_dirs: z.array(ProjectPathSchema).max(256).optional(),
    defines: z.record(z.string(), z.string().max(4096)).optional(),
    mode: z.enum(["syntax_only", "full"]).optional(),
  }), (input) => service.compile({
    topModule: input.top_module,
    ...(input.include_dirs === undefined ? {} : { includeDirs: input.include_dirs }),
    ...(input.defines === undefined ? {} : { defines: input.defines }),
    ...(input.mode === undefined ? {} : { mode: input.mode }),
  }));
  tool(server, "run_simulation", "Compile and run the current project snapshot in the paired browser's Xezim WASM Worker. This is one bounded operation because the browser binding does not expose separate compile and run handles; no native simulator fallback is used.", z.strictObject({
    top_module: z.string().min(1).max(512),
    until_ticks: DecimalTicksSchema,
    timescale: TimescaleSchema,
    work_budget: z.strictObject({
      max_wall_time_ms: z.number().int().positive().max(300_000),
      max_output_bytes: z.number().int().nonnegative().max(16 * 1024 * 1024),
      max_vcd_bytes: z.number().int().nonnegative().max(64 * 1024 * 1024),
    }).optional(),
  }), (input) => service.runSimulation({
    topModule: input.top_module,
    untilTicks: input.until_ticks,
    timescale: input.timescale,
    ...(input.work_budget === undefined ? {} : { workBudget: {
      maxWallTimeMs: input.work_budget.max_wall_time_ms,
      maxOutputBytes: input.work_budget.max_output_bytes,
      maxVcdBytes: input.work_budget.max_vcd_bytes,
    } }),
  }));
  tool(server, "get_diagnostics", "Get diagnostics for a compile or simulation, or the diagnostics retained by this bridge.", z.strictObject({
    compile_id: CompileIdSchema.optional(),
    run_id: RunIdSchema.optional(),
  }), ({ compile_id, run_id }) => service.getDiagnostics({
    ...(compile_id === undefined ? {} : { compileId: compile_id }),
    ...(run_id === undefined ? {} : { runId: run_id }),
  }));
  tool(server, "list_runs", "List recent browser simulation runs.", z.strictObject({ limit: z.number().int().positive().max(100).default(20) }), ({ limit }) => service.listRuns(limit));
  tool(server, "show_waveforms", "Open a VCD waveform artifact in the paired browser's Surfer panel.", z.strictObject({
    artifact_id: ArtifactIdSchema,
    signal_paths: z.array(z.string().min(1).max(1024)).max(4096).optional(),
    cursor: z.strictObject({ ticks: DecimalTicksSchema }).optional(),
  }), ({ artifact_id, signal_paths, cursor }) => service.showWaveforms({
    artifactId: artifact_id,
    ...(signal_paths === undefined ? {} : { signalPaths: signal_paths }),
    ...(cursor === undefined ? {} : { cursor }),
  }));
  tool(server, "get_simulation_log", "Get the captured stdout and stderr for a simulation run.", z.strictObject({
    run_id: RunIdSchema,
    tail_bytes: z.number().int().positive().max(1_048_576).default(65_536),
  }), ({ run_id, tail_bytes }) => service.getSimulationLog(run_id, tail_bytes));
  return server;
}
