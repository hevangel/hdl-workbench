import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { parseVcd, WaveformIndexError, type VcdInput } from "../src/index.js";

const fixture = path.join(path.dirname(fileURLToPath(import.meta.url)), "fixtures", "four-state.vcd");
const existingSurferFixture = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "apps", "ide", "public", "fixtures", "counter.vcd");
const vcd = await readFile(fixture, "utf8");

async function* byteChunks(input: string, chunkSize: number): AsyncGenerator<Uint8Array> {
  const bytes = new TextEncoder().encode(input);
  for (let offset = 0; offset < bytes.length; offset += chunkSize) yield bytes.slice(offset, offset + chunkSize);
}

function expectWaveError(action: () => unknown, code: WaveformIndexError["code"]): void {
  try {
    action();
  } catch (error) {
    expect(error).toBeInstanceOf(WaveformIndexError);
    expect((error as WaveformIndexError).code).toBe(code);
    return;
  }
  throw new Error(`Expected WaveformIndexError ${code}`);
}

describe("WaveformIndex", () => {
  it("parses hierarchical scopes, identifier aliases, four-state values, and timescale ticks", async () => {
    const index = await parseVcd(vcd);
    expect(index.timescale).toEqual({ magnitude: "10", unit: "ps" });
    expect(index.startTicks).toBe("0");
    expect(index.endTicks).toBe("40");
    expect(index.signalCount).toBe(7);

    expect(index.querySignal("tb.state", "0").value?.bits).toBe("xxxx");
    expect(index.querySignal("tb.state", "9").value?.bits).toBe("xxxx");
    expect(index.querySignal("tb.state", "10").value?.bits).toBe("0011");
    expect(index.querySignal("tb.state", "19").value?.bits).toBe("0011");
    expect(index.querySignal("tb.state", "20").value?.bits).toBe("zzzz");
    expect(index.querySignal("tb.state", "40").value?.bits).toBe("1111");
    expect(index.querySignal("tb.state_alias", "20").value?.bits).toBe("zzzz");
    expect(index.querySignal("tb.child.child_alias", "10").value?.bits).toBe("0011");
    expect(index.querySignal("tb.state", "40").aliases).toEqual(["tb.state", "tb.state_alias", "tb.child.child_alias"]);
  });

  it("interprets integer declarations as signed and leaves reg signedness unknown", async () => {
    const index = await parseVcd(vcd);
    const negativeOne = index.querySignal("tb.signed_value", "0");
    expect(negativeOne.signedness).toBe("signed");
    expect(negativeOne.value).toMatchObject({
      bits: "11111111111111111111111111111111",
      unsignedDecimal: "4294967295",
      signedDecimal: "-1",
    });
    expect(index.querySignal("tb.signed_value", "10").value?.signedDecimal).toBe("-2");
    const signedReg = index.querySignal("tb.signed_bits", "10");
    expect(signedReg.value?.bits).toBe("00000001");
    expect(signedReg.signedness).toBe("unknown");
    expect(signedReg.value?.signedDecimal).toBeNull();
    expect(index.querySignal("tb.signed_bits", "10", { signed: true }).value?.signedDecimal).toBe("1");
    expect(index.querySignal("tb.signed_bits", "0", { signed: true }).value?.signedDecimal).toBe("-2");
    expectWaveError(() => index.querySignal("tb.temperature", "10"), "UNSUPPORTED_VCD");
  });

  it("paginates transitions without losing same-tick values and rejects unrelated cursors", async () => {
    const index = await parseVcd(vcd);
    const first = index.listTransitions("tb.state", "0", "40", 2);
    expect(first.transitions.map((entry) => [entry.timeTicks, entry.value.bits])).toEqual([["0", "xxxx"], ["10", "0010"]]);
    expect(first.nextCursor).toBeDefined();
    const second = index.listTransitions("tb.state", "0", "40", 2, first.nextCursor);
    expect(second.transitions.map((entry) => [entry.timeTicks, entry.value.bits])).toEqual([["10", "0011"], ["20", "zzzz"]]);
    expect(second.nextCursor).toBeDefined();
    const third = index.listTransitions("tb.state", "0", "40", 2, second.nextCursor);
    expect(third.transitions.map((entry) => [entry.timeTicks, entry.value.bits])).toEqual([["40", "1111"]]);
    expect(third.nextCursor).toBeUndefined();
    expectWaveError(() => index.listTransitions("tb.state_alias", "0", "40", 2, first.nextCursor), "INVALID_CURSOR");
    expectWaveError(() => index.listTransitions("tb.state", "0", "40", 0), "INVALID_PAGE_SIZE");
  });

  it("paginates large signal lists and streams byte chunks across token boundaries", async () => {
    const declarations = Array.from({ length: 1_205 }, (_, index) => `$var wire 1 id${index} signal_${String(index).padStart(4, "0")} $end`).join("\n");
    const largeListVcd = `$timescale 1 ns $end\n$scope module top $end\n${declarations}\n$upscope $end\n$enddefinitions $end\n#0\n`;
    const stream: VcdInput = byteChunks(largeListVcd, 11);
    const index = await parseVcd(stream);
    const first = index.listSignals(300);
    expect(first.signals).toHaveLength(300);
    expect(first.signals[0]?.path).toBe("top.signal_0000");
    let count = first.signals.length;
    let cursor = first.nextCursor;
    while (cursor) {
      const page = index.listSignals(300, cursor);
      count += page.signals.length;
      cursor = page.nextCursor;
    }
    expect(count).toBe(1_205);
  });

  it("returns explicit unknown-signal, range, malformed-input, and resource-limit errors", async () => {
    const index = await parseVcd(vcd);
    expectWaveError(() => index.querySignal("tb.missing", "0"), "UNKNOWN_SIGNAL");
    expectWaveError(() => index.querySignal("tb.state", "41"), "TIME_OUT_OF_RANGE");
    expectWaveError(() => index.listTransitions("tb.state", "20", "10", 10), "INVALID_RANGE");
    await expect(parseVcd(vcd.replace("$enddefinitions $end", "$enddefinitions"))).rejects.toMatchObject({ code: "MALFORMED_VCD" });
    await expect(parseVcd(vcd, { maxInputBytes: 100 })).rejects.toMatchObject({ code: "RESOURCE_LIMIT" });
    await expect(parseVcd(vcd, { maxChanges: 2 })).rejects.toMatchObject({ code: "RESOURCE_LIMIT" });
    await expect(parseVcd(vcd.replace('$var wire 4 " state_alias', '$var wire 8 " state_alias'))).rejects.toMatchObject({ code: "MALFORMED_VCD" });
  });

  it("reads the existing Surfer counter VCD fixture at exact tick values", async () => {
    const index = await parseVcd(await readFile(existingSurferFixture, "utf8"));
    expect(index.timescale).toEqual({ magnitude: "1", unit: "ns" });
    expect(index.querySignal("top.count", "0").value?.bits).toBe("0000");
    expect(index.querySignal("top.count", "5").value?.bits).toBe("0001");
    expect(index.listTransitions("top.count", "0", "75", 100).transitions).toHaveLength(16);
  });
});
