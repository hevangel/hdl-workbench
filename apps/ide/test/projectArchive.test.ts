import { describe, expect, it } from "vitest";
import { exportProjectZip, importProjectZip, ProjectArchiveError } from "../src/workspace/projectArchive.js";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function crc32(bytes: Uint8Array): number {
  let value = 0xffffffff;
  for (const byte of bytes) {
    value ^= byte;
    for (let bit = 0; bit < 8; bit += 1) value = (value & 1) === 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  }
  return (value ^ 0xffffffff) >>> 0;
}

function deflatedZipWithDataDescriptor(path: string, contents: string): Uint8Array {
  const name = encoder.encode(path);
  const data = encoder.encode(contents);
  const compressed = data;
  const checksum = crc32(data);
  const local = new Uint8Array(30 + name.byteLength);
  const localView = new DataView(local.buffer);
  localView.setUint32(0, 0x04034b50, true);
  localView.setUint16(4, 20, true);
  localView.setUint16(6, 0x0808, true);
  localView.setUint16(8, 0, true);
  localView.setUint16(26, name.byteLength, true);
  local.set(name, 30);
  const descriptor = new Uint8Array(16);
  const descriptorView = new DataView(descriptor.buffer);
  descriptorView.setUint32(0, 0x08074b50, true);
  descriptorView.setUint32(4, checksum, true);
  descriptorView.setUint32(8, compressed.byteLength, true);
  descriptorView.setUint32(12, data.byteLength, true);
  const central = new Uint8Array(46 + name.byteLength);
  const centralView = new DataView(central.buffer);
  centralView.setUint32(0, 0x02014b50, true);
  centralView.setUint16(4, 0x031e, true);
  centralView.setUint16(6, 20, true);
  centralView.setUint16(8, 0x0808, true);
  centralView.setUint16(10, 0, true);
  centralView.setUint32(16, checksum, true);
  centralView.setUint32(20, compressed.byteLength, true);
  centralView.setUint32(24, data.byteLength, true);
  centralView.setUint16(28, name.byteLength, true);
  central.set(name, 46);
  const end = new Uint8Array(22);
  const endView = new DataView(end.buffer);
  endView.setUint32(0, 0x06054b50, true);
  endView.setUint16(8, 1, true);
  endView.setUint16(10, 1, true);
  endView.setUint32(12, central.byteLength, true);
  endView.setUint32(16, local.byteLength + compressed.byteLength + descriptor.byteLength, true);
  const chunks = [local, compressed, descriptor, central, end];
  const result = new Uint8Array(chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0));
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return result;
}

describe("project ZIP archives", () => {
  it("round trips nested UTF-8 project files", async () => {
    const files = [
      { path: "rtl/top.sv", bytes: encoder.encode("module café; endmodule\n") },
      { path: "rom.hex", bytes: encoder.encode("f0\n0f\n") },
    ];
    const archive = exportProjectZip(files);
    const decoded = await importProjectZip(archive);
    expect(decoded.map((file) => file.path)).toEqual(["rom.hex", "rtl/top.sv"]);
    expect(decoded.map((file) => decoder.decode(file.bytes))).toEqual(["f0\n0f\n", "module café; endmodule\n"]);
  });

  it("rejects unsafe and duplicate paths before export", () => {
    expect(() => exportProjectZip([{ path: "../escape.sv", bytes: encoder.encode("x") }])).toThrowError(ProjectArchiveError);
    expect(() => exportProjectZip([
      { path: "rtl/top.sv", bytes: encoder.encode("one") },
      { path: "rtl/top.sv", bytes: encoder.encode("two") },
    ])).toThrow(/Duplicate archive path/);
  });

  it("detects payload corruption on import", async () => {
    const archive = exportProjectZip([{ path: "rtl/top.sv", bytes: encoder.encode("module top; endmodule") }]);
    archive[40] = (archive[40] ?? 0) ^ 0xff;
    await expect(importProjectZip(archive)).rejects.toThrow(/checksum validation/);
  });

  it("imports stored entries that use a ZIP data descriptor", async () => {
    const decoded = await importProjectZip(deflatedZipWithDataDescriptor("rtl/top.sv", "module top; endmodule"));
    expect(decoded.map((file) => file.path)).toEqual(["rtl/top.sv"]);
    expect(decoder.decode(decoded[0]?.bytes)).toBe("module top; endmodule");
  });
});
