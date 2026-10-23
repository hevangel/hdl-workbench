import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProjectPath, RunResult, SimulationService } from "@xezim-ide/protocol";
import { App } from "../src/App.js";
import { createMockServices } from "../src/mockServices.js";
import { WorkspaceStorageError, type PersistentWorkspaceManager } from "../src/workspace/opfsWorkspace.js";

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

  it("shows quota errors without losing the draft and can save after storage recovers", async () => {
    const services = createMockServices();
    const activeProject = { id: "demo-project", name: "counter-lab", authority: "browser-only" as const, createdAt: 0 };
    vi.spyOn(services.workspace, "subscribeChanges").mockReturnValue(() => undefined);
    const projectManager = {
      activeProject,
      storageStatus: { persistent: false, persistentStorageSupported: true, usageBytes: 0, quotaBytes: 1024 },
      getWorkspace: () => services.workspace,
      listProjects: async () => [activeProject],
      loadEditorState: async () => null,
      saveEditorState: async () => undefined,
    } as unknown as PersistentWorkspaceManager;
    vi.spyOn(services.workspace, "writeFile").mockRejectedValueOnce(new WorkspaceStorageError(
      "STORAGE_QUOTA_EXCEEDED",
      "Browser storage is full. Export a project or remove files before saving more data.",
    ));
    render(<App workspace={services.workspace} simulation={services.simulation} projectManager={projectManager} />);

    expect(screen.getByText("Origin storage estimate: 0 B used of 1.0 KiB quota")).toBeInTheDocument();
    const editor = await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    const marker = "// quota failure keeps this draft";
    fireEvent.change(editor, { target: { value: `${(editor as HTMLTextAreaElement).value}\n${marker}` } });
    expect(await screen.findByText(/Browser storage is full/, {}, { timeout: 2_000 })).toBeInTheDocument();
    expect((editor as HTMLTextAreaElement).value).toContain(marker);

    await userEvent.setup().click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("All project files are saved in this browser.")).toBeInTheDocument();
    const saved = await services.workspace.readFile("rtl/counter.sv" as ProjectPath);
    expect(new TextDecoder().decode(saved.bytes)).toContain(marker);
  });

  it("opens the pinned Surfer panel with a clearly labeled sample VCD", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("tab", { name: "WAVEFORMS" }));

    expect(await screen.findByTitle("Surfer waveform viewer")).toHaveAttribute("src", "/surfer/index.html#dev");
    expect(screen.getByRole("button", { name: "Load sample VCD" })).toBeInTheDocument();
    expect(screen.getByText("Loading Surfer v0.7.0…")).toBeInTheDocument();
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
