import { describe, expect, it } from "vitest";
import {
  CompileRequestSchema,
  ExecuteRequestSchema,
  PairingRequestSchema,
  ProjectPathSchema,
  RpcRequestSchema,
  RpcResponseSchema,
  RunRequestSchema,
  RunResultSchema,
} from "../src/schemas.js";
import {
  ContractValidationError,
  compareProjectPaths,
  compareTicks,
  isValidTimeRange,
  parseTicks,
  serializeTicks,
  validateCompileResultForRequest,
  validateRpcRequestConsistency,
  validateRpcResponseForRequest,
  validateRunResultForRequest,
} from "../src/validation.js";

const timescale = { magnitude: "1", unit: "ns" } as const;
const request = {
  projectId: "project-a",
  snapshotId: "sha256:abc123",
  topModule: "counter_tb",
  includeDirs: ["rtl/include"],
  defines: { WIDTH: "8" },
  mode: "full",
} as const;

describe("protocol schemas", () => {
  it("accepts a normalized relative POSIX path", () => {
    expect(ProjectPathSchema.parse("rtl/counter.sv")).toBe("rtl/counter.sv");
  });

  it.each(["", "/etc/passwd", "C:/source.sv", "../secret.sv", "rtl/../secret.sv", "rtl//counter.sv", "rtl\\counter.sv", "rtl/./counter.sv", "rtl/\u0000counter.sv"])(
    "rejects unsafe path %j",
    (path) => expect(ProjectPathSchema.safeParse(path).success).toBe(false),
  );

  it("accepts protocol v1 simulation RPC and rejects unversioned operations", () => {
    const valid = {
      protocol: "xezim-ide/1",
      id: "req-1",
      op: "simulation.compile",
      projectId: "project-a",
      params: request,
    };
    expect(RpcRequestSchema.safeParse(valid).success).toBe(true);
    expect(RpcRequestSchema.safeParse({ ...valid, protocol: "xezim-ide/2" }).success).toBe(false);
    expect(RpcRequestSchema.safeParse({ ...valid, op: "simulation.cancel" }).success).toBe(false);
    expect(RpcRequestSchema.safeParse({ ...valid, unexpected: true }).success).toBe(false);
  });

  it("rejects malformed pairing codes and accepts a read-only pairing request", () => {
    const pairing = { challengeId: "challenge-1", code: "ABCD2345", clientName: "Codex", requestedRole: "read" };
    expect(PairingRequestSchema.safeParse(pairing).success).toBe(true);
    expect(PairingRequestSchema.safeParse({ ...pairing, code: "not-a-code" }).success).toBe(false);
  });

  it("validates bounded decimal ticks and requires strings on the wire", () => {
    const valid = {
      projectId: "project-a",
      snapshotId: "snapshot-1",
      compileId: "compile-1",
      until: { ticks: "90071992547409931234567890", timescale },
      workBudget: { maxSteps: 10_000, maxWallTimeMs: 2_000, maxOutputBytes: 1_024, maxVcdBytes: 1_000_000 },
    };
    expect(RunRequestSchema.safeParse(valid).success).toBe(true);
    expect(RunRequestSchema.safeParse({ ...valid, until: { ...valid.until, ticks: 25 } }).success).toBe(false);
    expect(RunRequestSchema.safeParse({ ...valid, until: { ...valid.until, ticks: "01" } }).success).toBe(false);
  });

  it("accepts one-shot execution without inventing a separate compile handle or step budget", () => {
    const valid = {
      projectId: "local-project",
      snapshotId: "snap-0123456789abcdef0123456789abcdef",
      topModule: "counter_tb",
      until: { ticks: "1000", timescale },
      workBudget: { maxWallTimeMs: 5000, maxOutputBytes: 1048576, maxVcdBytes: 64 * 1024 * 1024 },
    };
    expect(ExecuteRequestSchema.safeParse(valid).success).toBe(true);
    expect(ExecuteRequestSchema.safeParse({ ...valid, compileId: "pretend-compile" }).success).toBe(false);
    expect(ExecuteRequestSchema.safeParse({ ...valid, workBudget: { ...valid.workBudget, maxSteps: 1 } }).success).toBe(false);
  });

  it("accepts plusargs and an FST request, and rejects arguments the simulator would ignore", () => {
    const base = {
      projectId: "local-project",
      snapshotId: "snap-0123456789abcdef0123456789abcdef",
      topModule: "counter_tb",
      until: { ticks: "1000", timescale },
      workBudget: { maxWallTimeMs: 5000, maxOutputBytes: 1048576, maxVcdBytes: 64 * 1024 * 1024 },
    };
    expect(ExecuteRequestSchema.safeParse({ ...base, plusargs: ["+verbose", "+N=42"], waveformFormat: "fst" }).success).toBe(true);
    expect(ExecuteRequestSchema.safeParse({ ...base, plusargs: ["verbose"] }).success).toBe(false);
    expect(ExecuteRequestSchema.safeParse({ ...base, plusargs: ["+has space"] }).success).toBe(false);
    expect(ExecuteRequestSchema.safeParse({ ...base, waveformFormat: "fsdb" }).success).toBe(false);
  });
});

describe("snapshot provenance and tick arithmetic", () => {
  it("orders project paths identically across browser and Node locales", () => {
    const paths = ["dpi_top.sv", "dpi/add_cpp.cpp", "dpi/add_cpp.hpp", "dpi/add.c", "dpi/add.h"];
    expect(paths.sort(compareProjectPaths)).toEqual([
      "dpi/add.c",
      "dpi/add.h",
      "dpi/add_cpp.cpp",
      "dpi/add_cpp.hpp",
      "dpi_top.sv",
    ]);
    expect(compareProjectPaths("dpi/add.c", "dpi/add.c")).toBe(0);
  });

  it("rejects compile results produced from another snapshot", () => {
    const compileRequest = CompileRequestSchema.parse(request);
    expect(() => validateCompileResultForRequest(compileRequest, {
      compileId: "compile-1",
      projectId: "project-a",
      snapshotId: "sha256:abc123",
      success: true,
      diagnostics: [],
    })).not.toThrow();
    expect(() => validateCompileResultForRequest(compileRequest, {
      compileId: "compile-1",
      projectId: "project-a",
      snapshotId: "snapshot-other",
      success: true,
      diagnostics: [],
    })).toThrow(ContractValidationError);
  });

  it("rejects waveform provenance that differs from the run", () => {
    const runRequest = RunRequestSchema.parse({
      projectId: "project-a",
      snapshotId: "snapshot-1",
      compileId: "compile-1",
      until: { ticks: "25", timescale },
      workBudget: { maxSteps: 10_000, maxWallTimeMs: 2_000, maxOutputBytes: 1_024, maxVcdBytes: 1_000_000 },
    });
    const runResult = RunResultSchema.parse({
      runId: "run-1",
      projectId: "project-a",
      snapshotId: "snapshot-1",
      status: "completed",
      diagnostics: [],
      stdout: "",
      stderr: "",
      lastTime: { ticks: "25", timescale },
      waveform: { id: "artifact-1", runId: "run-1", snapshotId: "snapshot-other", format: "vcd", sizeBytes: 80, storage: "opfs", timescale, complete: true },
    });
    expect(() => validateRunResultForRequest(runRequest, runResult)).toThrow(ContractValidationError);
    expect(() => validateRunResultForRequest(runRequest, {
      ...runResult,
      waveform: { ...runResult.waveform!, snapshotId: "snapshot-1" },
    })).not.toThrow();
  });

  it("compares ticks above Number.MAX_SAFE_INTEGER exactly", () => {
    const start = "900719925474099312345678901234567890";
    const end = "900719925474099312345678901234567891";
    expect(compareTicks(start, end)).toBe(-1);
    expect(isValidTimeRange(start, end)).toBe(true);
    expect(isValidTimeRange(end, start)).toBe(false);
    expect(compareTicks(start, start)).toBe(0);
  });

  it("round-trips large BigInt ticks through JSON as decimal strings", () => {
    const original = 900719925474099312345678901234567890n;
    const wireValue = serializeTicks(original);
    expect(typeof wireValue).toBe("string");
    const json = JSON.stringify({ ticks: wireValue });
    const decoded: unknown = JSON.parse(json);
    expect(parseTicks((decoded as { ticks: string }).ticks)).toBe(original);
    expect(() => serializeTicks(-1n)).toThrow(ContractValidationError);
    expect(() => parseTicks("01")).toThrow(ContractValidationError);
  });

  it("rejects an RPC envelope whose project differs from its simulation input", () => {
    const mismatch = RpcRequestSchema.parse({
      protocol: "xezim-ide/1",
      id: "req-2",
      op: "simulation.compile",
      projectId: "project-b",
      params: request,
    });
    expect(() => validateRpcRequestConsistency(mismatch)).toThrow(ContractValidationError);
  });

  it("checks RPC response correlation, operation result shape, and provenance", () => {
    const compileRpc = RpcRequestSchema.parse({
      protocol: "xezim-ide/1",
      id: "req-3",
      op: "simulation.compile",
      projectId: "project-a",
      params: request,
    });
    const compileResponse = {
      protocol: "xezim-ide/1",
      id: "req-3",
      ok: true as const,
      result: { compileId: "compile-1", projectId: "project-a", snapshotId: "sha256:abc123", success: true, diagnostics: [] },
    };
    expect(() => validateRpcResponseForRequest(compileRpc, compileResponse)).not.toThrow();
    expect(() => validateRpcResponseForRequest(compileRpc, { ...compileResponse, id: "req-other" })).toThrow(ContractValidationError);
    expect(() => validateRpcResponseForRequest(compileRpc, { ...compileResponse, result: { files: [] } })).toThrow(ContractValidationError);
    expect(RpcResponseSchema.safeParse({ ...compileResponse, result: { arbitrary: true } }).success).toBe(false);
  });
});
