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
        "hdl-workbench",
        expect.arrayContaining([
          expect.objectContaining({
            severity: 8,
            code: "MOCK_SYNTAX_ERROR",
            startLineNumber: expect.any(Number),
          }),
        ]),
      );
    });
    expect(screen.getByRole("status")).toHaveTextContent("Mock services · no HDL simulation");
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

  it("collapses the explorer from its header and toggles it with Ctrl+B", async () => {
    const user = userEvent.setup();
    renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    const sidebar = screen.getByRole("complementary", { name: "Project files" });

    await user.click(screen.getByRole("button", { name: "Collapse file explorer" }));
    expect(sidebar).toHaveClass("collapsed");
    expect(screen.getByRole("button", { name: "Show file explorer" })).toHaveAttribute("aria-expanded", "false");

    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(sidebar).not.toHaveClass("collapsed");
    // Other modifier combinations are left to the editor.
    fireEvent.keyDown(window, { key: "b", ctrlKey: true, shiftKey: true });
    expect(sidebar).not.toHaveClass("collapsed");
  });

  it("resizes the bottom panel from the keyboard and keeps the height across reloads", async () => {
    const firstRender = renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    const resizer = screen.getByRole("separator", { name: "Resize bottom panel" });
    expect(resizer).toHaveAttribute("aria-valuenow", "220");

    fireEvent.keyDown(resizer, { key: "ArrowUp" });
    fireEvent.keyDown(resizer, { key: "ArrowUp" });
    expect(resizer).toHaveAttribute("aria-valuenow", "260");
    expect(screen.getByRole("main")).toHaveStyle({ "--bottom-panel-height": "260px" });
    await waitFor(() => expect(window.localStorage.getItem("xezim-ide/layout-v1")).toContain('"bottomPanelHeight":260'));
    firstRender.unmount();

    renderDemo();
    expect(await screen.findByRole("separator", { name: "Resize bottom panel" })).toHaveAttribute("aria-valuenow", "260");
  });

  it("clamps the bottom panel to its minimum and resets it on double-click", async () => {
    renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    const resizer = screen.getByRole("separator", { name: "Resize bottom panel" });

    fireEvent.keyDown(resizer, { key: "Home" });
    expect(resizer).toHaveAttribute("aria-valuenow", "80");
    fireEvent.keyDown(resizer, { key: "ArrowDown" });
    expect(resizer).toHaveAttribute("aria-valuenow", "80");

    fireEvent.doubleClick(resizer);
    expect(resizer).toHaveAttribute("aria-valuenow", "220");
  });

  it("links the help button to the user guide in a new tab", async () => {
    renderDemo();
    const help = await screen.findByRole("link", { name: "How HDL Workbench works" });
    expect(help).toHaveAttribute("href", "/docs/index.html");
    expect(help).toHaveAttribute("target", "_blank");
  });

  it("creates a new file from the explorer header and opens it", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "prompt").mockReturnValue("rtl/fresh.sv");
    renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });

    await user.click(screen.getByRole("button", { name: "New file" }));

    expect(await screen.findByRole("button", { name: "Open rtl/fresh.sv" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /fresh\.sv/, selected: true })).toBeInTheDocument();
    expect((screen.getByRole("textbox", { name: "SystemVerilog editor" }) as HTMLTextAreaElement).value).toBe("");
  });

  it("rejects a new file path that escapes the project", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "prompt").mockReturnValue("../outside.sv");
    renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    await user.click(screen.getByRole("button", { name: "New file" }));
    expect(await screen.findByText(/normalized project-relative paths/)).toBeInTheDocument();
  });

  it("offers file actions in a right-click menu and deletes the clicked file", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "confirm").mockReturnValue(true);
    renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    expect(screen.queryByRole("button", { name: "File actions" })).not.toBeInTheDocument();

    fireEvent.contextMenu(screen.getByRole("button", { name: "Open tb/counter_tb.sv" }));
    expect(screen.getByRole("menu", { name: "Actions for tb/counter_tb.sv" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Rename…" })).toBeDisabled();
    await user.click(screen.getByRole("menuitem", { name: "Delete…" }));

    await waitFor(() => expect(screen.queryByRole("button", { name: "Open tb/counter_tb.sv" })).not.toBeInTheDocument());
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("closes the file menu on Escape", async () => {
    renderDemo();
    await screen.findByRole("textbox", { name: "SystemVerilog editor" });
    fireEvent.contextMenu(screen.getByRole("button", { name: "Open tb/counter_tb.sv" }));
    expect(screen.getByRole("menu")).toBeInTheDocument();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("offers VCD and Off dump choices and shows FST as unavailable", async () => {
    renderDemo();
    const dump = await screen.findByRole("combobox", { name: "Waveform dump" });
    expect(dump).toHaveValue("vcd");
    expect(screen.getByRole("option", { name: /FST/ })).toBeDisabled();
  });

  it("ignores a malformed stored panel height but keeps the stored explorer state", async () => {
    window.localStorage.setItem("xezim-ide/layout-v1", JSON.stringify({ explorerCollapsed: true, bottomPanelHeight: "tall" }));
    renderDemo();
    expect(await screen.findByRole("button", { name: "Show file explorer" })).toBeInTheDocument();
    expect(screen.getByRole("separator", { name: "Resize bottom panel" })).toHaveAttribute("aria-valuenow", "220");
  });
});
