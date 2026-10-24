import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SurferWaveformPanel } from "../src/surfer/SurferWaveformPanel.js";

vi.mock("../src/surfer/SurferIframeAdapter.js", () => {
  class FakeSurferIframeAdapter {
    constructor(_frame: HTMLIFrameElement, _options: unknown) {}
    ready() { return new Promise<void>(() => undefined); }
    dispose() {}
  }
  return {
    PINNED_SURFER_VERSION: "0.7.0",
    SurferAdapterError: class SurferAdapterError extends Error {},
    SurferIframeAdapter: FakeSurferIframeAdapter,
  };
});

describe("SurferWaveformPanel", () => {
  it("shows the pinned license, dated modification notice, and source links", () => {
    render(<SurferWaveformPanel />);

    expect(screen.getByLabelText("Surfer license and source information")).toHaveTextContent(
      "Surfer v0.7.0 · EUPL-1.2 · modified by HDL Workbench on 2026-10-22",
    );
    expect(screen.getByRole("link", { name: "License" })).toHaveAttribute(
      "href",
      "/surfer/LICENSE-EUPL-1.2.txt",
    );
    expect(screen.getByRole("link", { name: "Pinned upstream source" })).toHaveAttribute(
      "href",
      "https://gitlab.com/surfer-project/surfer/-/tree/bd749b1f786c1c62cd67893ca71346cbe6983915",
    );
    expect(screen.getByRole("link", { name: "Modified entrypoint source" })).toHaveAttribute(
      "download",
      "surfer-index.html",
    );
    expect(screen.getByRole("link", { name: "Modified entrypoint source" })).toHaveAttribute(
      "href",
      "/surfer/index.html",
    );
    expect(screen.getByRole("link", { name: "Provenance" })).toHaveAttribute(
      "href",
      "/surfer/UPSTREAM.md",
    );
  });
});
