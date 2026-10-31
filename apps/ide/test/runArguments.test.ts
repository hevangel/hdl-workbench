import { describe, expect, it } from "vitest";
import { parseRunArguments, RunArgumentsError } from "../src/runArguments.js";

describe("parseRunArguments", () => {
  it("returns nothing for an empty line", () => {
    expect(parseRunArguments("")).toEqual([]);
    expect(parseRunArguments("   ")).toEqual([]);
  });

  it("keeps +plusargs and maps -sv_seed like the native CLI", () => {
    expect(parseRunArguments("+verbose +N=42 -sv_seed 7")).toEqual(["+verbose", "+N=42", "+seed=7"]);
    expect(parseRunArguments("-sv_seed random")).toEqual(["+seed=random"]);
  });

  it("passes a value through exactly as typed, including quote characters", () => {
    expect(parseRunArguments('+MSG="hi" +b=1')).toEqual(['+MSG="hi"', "+b=1"]);
  });

  it("rejects arguments the browser simulator would silently ignore", () => {
    expect(() => parseRunArguments("-gui")).toThrow(RunArgumentsError);
    expect(() => parseRunArguments("verbose")).toThrow(/Unsupported argument 'verbose'/);
    expect(() => parseRunArguments("-sv_seed")).toThrow(/-sv_seed/);
    expect(() => parseRunArguments("-sv_seed abc")).toThrow(/-sv_seed/);
  });

  it("limits the number and length of arguments", () => {
    expect(() => parseRunArguments(Array.from({ length: 65 }, (_, index) => `+a${index}`).join(" "))).toThrow(/At most 64/);
    expect(() => parseRunArguments(`+a=${"x".repeat(5000)}`)).toThrow(/limited to/);
  });
});
