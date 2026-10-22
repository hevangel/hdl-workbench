import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RunResult, SimulationService } from "@xezim-ide/protocol";
import { App } from "../src/App.js";
import { createMockServices } from "../src/mockServices.js";

const { setModelMarkers } = vi.hoisted(() => ({ setModelMarkers: vi.fn() }));

vi.mock("@monaco-editor/react", async () => {
  const React = await import("react");
  const markerSeverity = { Error: 8, Warning: 4, Info: 2 };
  return {
    default: (props: {
      value?: string;
      onChange?: (value: string) => void;
      onMount?: (editor: unknown, monaco: unknown) => void;
      options?: { ariaLabel?: string };
    }) => {
      const model = {};
      React.useEffect(() => {
        props.onMount?.(
          { getModel: () => model },
          {
            MarkerSeverity: markerSeverity,
            languages: { register: vi.fn(), setMonarchTokensProvider: vi.fn() },
            editor: { defineTheme: vi.fn(), setModelMarkers },
          },
        );
      }, []);
      return React.createElement("textarea", {
        "aria-label": props.options?.ariaLabel ?? "SystemVerilog editor",
        value: props.value ?? "",
        onChange: (event: { target: { value: string } }) => props.onChange?.(event.target.value),
      });
    },
  };
});

function renderDemo(simulation?: SimulationService) {
  const services = createMockServices();
  return render(<App workspace={services.workspace} simulation={simulation ?? services.simulation} />);
}

describe("IDE shell", () => {
  beforeEach(() => {
    setModelMarkers.mockClear();
    window.localStorage.clear();
  });

  it("maps mocked compiler diagnostics to Monaco markers with source location", async () => {
    const user = userEvent.setup();
    renderDemo();
    const editor = await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    fireEvent.change(editor, { target: { value: `${(editor as HTMLTextAreaElement).value}\n// syntax_error` } });
    await user.click(screen.getByRole("button", { name: "Run" }));

    expect(await screen.findByText("MOCK_SYNTAX_ERROR")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Diagnostics" })).toHaveTextContent("Fixture marker");
    await waitFor(() => {
      expect(setModelMarkers).toHaveBeenCalledWith(
        expect.any(Object),
        "xezim-studio",
        expect.arrayContaining([
          expect.objectContaining({
            severity: 8,
            code: "MOCK_SYNTAX_ERROR",
            startLineNumber: expect.any(Number),
          }),
        ]),
      );
    });
    expect(screen.getByRole("status")).toHaveTextContent("MOCK SERVICES");
  });

  it("asks before closing a dirty tab and can discard the draft", async () => {
    const user = userEvent.setup();
    renderDemo();
    const editor = await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    await user.click(screen.getByRole("button", { name: "Open tb/counter_tb.sv" }));
    await user.click(screen.getByRole("tab", { name: /counter\.sv/ }));
    fireEvent.change(editor, { target: { value: `${(editor as HTMLTextAreaElement).value}\n// unsaved edit` } });

    await user.click(screen.getByRole("button", { name: "Close counter.sv" }));
    expect(screen.getByRole("dialog", { name: "Unsaved changes" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Keep editing" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Close counter.sv" }));
    await user.click(screen.getByRole("button", { name: "Close without saving" }));
    expect(screen.queryByRole("tab", { name: /counter\.sv/ })).not.toBeInTheDocument();
    expect((screen.getByRole("textbox", { name: "SystemVerilog editor" }) as HTMLTextAreaElement).value).toContain("counter_tb");
  });

  it("shows draft and saved states while debounced autosave completes", async () => {
    renderDemo();
    const editor = await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    fireEvent.change(editor, { target: { value: `${(editor as HTMLTextAreaElement).value}\n// autosave` } });
    expect(screen.getByText("Unsaved")).toBeInTheDocument();
    expect(await screen.findByText("Saved", {}, { timeout: 2_000 })).toBeInTheDocument();
  });

  it("disables run controls while a mock run is in flight", async () => {
    const user = userEvent.setup();
    let releaseRun: ((result: RunResult) => void) | undefined;
    const simulation: SimulationService = {
      compile: async (input) => ({
        compileId: "mock-compile-busy",
        projectId: input.projectId,
        snapshotId: input.snapshotId,
        success: true,
        diagnostics: [],
      }),
      run: async (input) => new Promise<RunResult>((resolve) => {
        releaseRun = resolve;
        void input;
      }),
    };
    renderDemo(simulation);
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    const runButton = screen.getByRole("button", { name: "Run" });
    const topModuleInput = screen.getByRole("textbox", { name: "Top module" });
    await user.click(runButton);

    await waitFor(() => expect(releaseRun).toBeDefined());
    expect(runButton).toBeDisabled();
    expect(topModuleInput).toBeDisabled();
    releaseRun?.({
      runId: "mock-run-busy",
      projectId: "demo-project",
      snapshotId: "mock-snapshot-1",
      status: "failed",
      diagnostics: [],
      stdout: "",
      stderr: "Mock service only.",
      lastTime: { ticks: "0", timescale: { magnitude: "1", unit: "ns" } },
    });
    await waitFor(() => expect(runButton).toBeEnabled());
  });

  it("persists the explorer layout preference across reloads", async () => {
    const user = userEvent.setup();
    const firstRender = renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    await user.click(screen.getByRole("button", { name: "Hide file explorer" }));
    await waitFor(() => expect(window.localStorage.getItem("xezim-ide/layout-v1")).toContain('"explorerCollapsed":true'));
    firstRender.unmount();

    renderDemo();
    expect(await screen.findByRole("button", { name: "Show file explorer" })).toBeInTheDocument();
  });
});
