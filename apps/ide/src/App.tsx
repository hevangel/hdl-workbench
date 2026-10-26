import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties, type KeyboardEvent as ReactKeyboardEvent, type PointerEvent as ReactPointerEvent } from "react";
import { ProjectPathSchema, type Diagnostic, type ProjectId, type ProjectPath, type SimulationService, type SurferAdapter, type WaveformArtifact, type WorkspaceAdapter } from "@xezim-ide/protocol";
import { EditorSurface } from "./editor/EditorSurface.js";
import { createMockServices } from "./mockServices.js";
import { clampBottomPanelHeight, DEFAULT_BOTTOM_PANEL_HEIGHT, MIN_BOTTOM_PANEL_HEIGHT, useLayoutPreferences } from "./hooks/useLayoutPreferences.js";
import type { BrowserStorageStatus, PersistentWorkspaceManager, WorkspaceProject } from "./workspace/opfsWorkspace.js";
import { exportProjectZip } from "./workspace/projectArchive.js";
import { SurferWaveformPanel } from "./surfer/SurferWaveformPanel.js";
import type { BrowserIdeController } from "./controller/BrowserIdeController.js";
import { McpBrowserClient, type BrowserBridgeState } from "./controller/McpBrowserClient.js";
import { McpConnectionPanel } from "./controller/McpConnectionPanel.js";
import { AgentIcon, BrandMark, ChevronIcon, CollapseLeftIcon, ExportIcon, FolderIcon, HelpIcon, ImportIcon, NewFileIcon, PlayIcon, SaveIcon, SidebarIcon, SpinnerIcon, WarningIcon } from "./icons.js";

type SaveState = "dirty" | "saving" | "saved" | "save-failed";
interface EditorFile {
  path: ProjectPath;
  content: string;
  revision: string;
  saveState: SaveState;
}
interface AppProps {
  workspace?: WorkspaceAdapter;
  simulation?: SimulationService;
  controller?: BrowserIdeController;
  projectManager?: PersistentWorkspaceManager;
  onSurferAdapterChange?: (adapter: SurferAdapter | null) => void;
}

const demoServices = createMockServices();

function fileName(path: string): string {
  return path.split("/").at(-1) ?? path;
}

function fileKindLabel(path: string): string {
  const extension = fileName(path).split(".").at(-1)?.toLowerCase();
  if (extension === "c") return "C";
  if (["cc", "cpp", "cxx", "hh", "hpp", "hxx"].includes(extension ?? "")) return "C++";
  if (["h", "inc"].includes(extension ?? "")) return "H";
  return "SV";
}

// Rows of .editor-workspace other than the bottom panel: tabs, breadcrumb and the
// editor's minimum height. Keep in step with grid-template-rows in styles.css.
const EDITOR_WORKSPACE_RESERVED_HEIGHT = 36 + 26 + 180;
const BOTTOM_PANEL_KEYBOARD_STEP = 20;

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${Math.max(0, Math.round(bytes))} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MiB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GiB`;
}

export function App({ workspace, simulation = demoServices.simulation, controller, projectManager, onSurferAdapterChange }: AppProps) {
  const [workspaceService, setWorkspaceService] = useState<WorkspaceAdapter>(() => workspace ?? controller?.workspace ?? projectManager?.getWorkspace() ?? demoServices.workspace);
  const [activeProject, setActiveProject] = useState<WorkspaceProject>(() => projectManager?.activeProject ?? {
    id: "demo-project" as ProjectId,
    name: "counter-lab",
    authority: "browser-only",
    createdAt: 0,
  });
  const [projects, setProjects] = useState<WorkspaceProject[]>([]);
  const [storageStatus, setStorageStatus] = useState<BrowserStorageStatus | null>(() => projectManager?.storageStatus ?? null);
  const [files, setFiles] = useState<EditorFile[]>([]);
  const filesRef = useRef(files);
  filesRef.current = files;
  const [openTabs, setOpenTabs] = useState<ProjectPath[]>([]);
  const [activePath, setActivePath] = useState<ProjectPath | null>(null);
  const [topModule, setTopModule] = useState("counter_tb");
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([]);
  const [log, setLog] = useState(() => !controller
    ? "Mock services are active. No HDL simulation is performed."
    : controller.canExecute
      ? "Ready. Run compiles and simulates the top module with Xezim WASM in a browser Worker."
      : "The Xezim WASM assets were not found; no native simulation fallback is configured.");
  const [busy, setBusy] = useState(false);
  const [pendingClose, setPendingClose] = useState<ProjectPath | null>(null);
  const [conflictPath, setConflictPath] = useState<ProjectPath | null>(null);
  const [workspaceMessage, setWorkspaceMessage] = useState("");
  const [fileMenu, setFileMenu] = useState<{ path: ProjectPath; x: number; y: number } | null>(null);
  // The browser Xezim build only emits VCD; "off" runs the design without opening the waveform viewer.
  const [dumpFormat, setDumpFormat] = useState<"vcd" | "off">("vcd");
  const [bridgePanelOpen, setBridgePanelOpen] = useState(false);
  const [bridgeState, setBridgeState] = useState<BrowserBridgeState>("disconnected");
  const [bridgeMessage, setBridgeMessage] = useState("");
  const bridgeClientRef = useRef<McpBrowserClient | null>(null);
  const [activePanel, setActivePanel] = useState<"problems" | "output" | "waveforms">("problems");
  const projectArchiveInput = useRef<HTMLInputElement>(null);
  const [layout, setLayout] = useLayoutPreferences();
  const [resizingPanel, setResizingPanel] = useState(false);
  const editorWorkspaceRef = useRef<HTMLElement>(null);
  const revisionByPath = useRef(new Map<ProjectPath, string>());
  const loadedSessionProject = useRef<ProjectId | null>(null);
  const saveTimers = useRef(new Map<ProjectPath, ReturnType<typeof setTimeout>>());
  const saveQueues = useRef(new Map<ProjectPath, Promise<void>>());

  const activeFile = files.find((file) => file.path === activePath) ?? null;
  const sortedFiles = useMemo(() => [...files].sort((left, right) => left.path.localeCompare(right.path)), [files]);
  const folders = [...new Set(sortedFiles.map((file) => file.path.split("/").slice(0, -1).join("/")).filter(Boolean))].sort();

  const updateFile = useCallback((path: ProjectPath, update: (file: EditorFile) => EditorFile) => {
    setFiles((current) => current.map((file) => file.path === path ? update(file) : file));
  }, []);

  const handleSurferAdapterChange = useCallback((adapter: SurferAdapter | null) => {
    controller?.setSurferAdapter(adapter);
    onSurferAdapterChange?.(adapter);
  }, [controller, onSurferAdapterChange]);

  const resolveWaveformUrl = useCallback((artifact: WaveformArtifact) => {
    if (!controller) return Promise.reject(new Error("No browser waveform artifact store is configured."));
    return controller.artifacts.objectUrl(artifact);
  }, [controller]);

  const toggleExplorer = useCallback(() => {
    setLayout((current) => ({ ...current, explorerCollapsed: !current.explorerCollapsed }));
  }, [setLayout]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "b") {
        event.preventDefault();
        toggleExplorer();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [toggleExplorer]);

  const maxBottomPanelHeight = useCallback(() => {
    const height = editorWorkspaceRef.current?.getBoundingClientRect().height ?? 0;
    // Layout is unmeasured in non-rendering environments; fall back to the stored limit.
    return height > 0 ? Math.max(MIN_BOTTOM_PANEL_HEIGHT, height - EDITOR_WORKSPACE_RESERVED_HEIGHT) : Number.POSITIVE_INFINITY;
  }, []);

  const setBottomPanelHeight = useCallback((height: number) => {
    const next = clampBottomPanelHeight(Math.min(height, maxBottomPanelHeight()));
    setLayout((current) => current.bottomPanelHeight === next ? current : { ...current, bottomPanelHeight: next });
  }, [maxBottomPanelHeight, setLayout]);

  const startPanelResize = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    setResizingPanel(true);
  };

  const movePanelResize = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
    const bounds = editorWorkspaceRef.current?.getBoundingClientRect();
    if (bounds) setBottomPanelHeight(bounds.bottom - event.clientY);
  };

  const endPanelResize = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    setResizingPanel(false);
  };

  const resizePanelWithKeyboard = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    const steps: Record<string, number> = {
      ArrowUp: layout.bottomPanelHeight + BOTTOM_PANEL_KEYBOARD_STEP,
      ArrowDown: layout.bottomPanelHeight - BOTTOM_PANEL_KEYBOARD_STEP,
      Home: MIN_BOTTOM_PANEL_HEIGHT,
      End: maxBottomPanelHeight(),
    };
    const target = steps[event.key];
    if (target === undefined) return;
    event.preventDefault();
    setBottomPanelHeight(target);
  };

  useEffect(() => {
    const nextWorkspace = workspace ?? controller?.workspace;
    if (nextWorkspace) setWorkspaceService(nextWorkspace);
  }, [controller, workspace]);

  useEffect(() => {
    if (!controller) return;
    const client = new McpBrowserClient(controller, {
      onStateChange: (state, message) => {
        setBridgeState(state);
        setBridgeMessage(message ?? "");
      },
    });
    bridgeClientRef.current = client;
    controller.setOnWaveformShown(() => setActivePanel("waveforms"));
    return () => {
      client.disconnect();
      bridgeClientRef.current = null;
      controller.setOnWaveformShown(undefined);
      controller.setSurferAdapter(null);
    };
  }, [controller]);

  useEffect(() => {
    if (!controller) return;
    let refreshTimer: ReturnType<typeof setTimeout> | undefined;
    let generation = 0;
    const refreshFromWorkspace = () => {
      generation += 1;
      const requestedGeneration = generation;
      if (refreshTimer) clearTimeout(refreshTimer);
      refreshTimer = setTimeout(() => {
        void (async () => {
          const entries = await controller.workspace.listFiles();
          const refreshed = await Promise.all(entries.map(async (entry) => {
            const file = await controller.workspace.readFile(entry.path);
            return {
              path: entry.path,
              revision: file.revision,
              content: new TextDecoder().decode(file.bytes),
              saveState: "saved" as const,
            };
          }));
          if (requestedGeneration !== generation) return;
          const available = new Set(refreshed.map((file) => file.path));
          revisionByPath.current = new Map(refreshed.map((file) => [file.path, file.revision]));
          setFiles(refreshed);
          setOpenTabs((current) => {
            const retained = current.filter((path) => available.has(path));
            return retained.length > 0 ? retained : refreshed[0] ? [refreshed[0].path] : [];
          });
          setActivePath((current) => current && available.has(current) ? current : refreshed[0]?.path ?? null);
        })().catch((error: unknown) => {
          setWorkspaceMessage(error instanceof Error ? error.message : "Could not refresh files from the connected project.");
        });
      }, 50);
    };
    controller.setOnWorkspaceChanged(refreshFromWorkspace);
    return () => {
      generation += 1;
      if (refreshTimer) clearTimeout(refreshTimer);
      controller.setOnWorkspaceChanged(undefined);
    };
  }, [controller]);

  useEffect(() => {
    if (!projectManager) return;
    let cancelled = false;
    void projectManager.listProjects().then((items) => {
      if (!cancelled) setProjects(items);
    }).catch((error: unknown) => {
      if (!cancelled) setWorkspaceMessage(error instanceof Error ? error.message : "Could not list browser projects.");
    });
    return () => { cancelled = true; };
  }, [projectManager]);

  useEffect(() => {
    let cancelled = false;
    loadedSessionProject.current = null;
    setFiles([]);
    setDiagnostics([]);
    setConflictPath(null);
    setWorkspaceMessage("");
    const loadWorkspace = async () => {
      const entries = await workspaceService.listFiles();
      const loaded = await Promise.all(entries.map(async (entry) => {
        const result = await workspaceService.readFile(entry.path);
        return {
          path: entry.path,
          revision: result.revision,
          content: new TextDecoder().decode(result.bytes),
          saveState: "saved" as const,
        };
      }));
      if (cancelled) return;
      revisionByPath.current = new Map(loaded.map((file) => [file.path, file.revision]));
      setFiles(loaded);
      const available = new Set(loaded.map((file) => file.path));
      const session = await projectManager?.loadEditorState(activeProject.id).catch(() => null);
      if (cancelled) return;
      const restoredTabs = session?.openTabs.filter((path) => available.has(path)) ?? [];
      const nextTabs = restoredTabs.length > 0 ? restoredTabs : loaded[0] ? [loaded[0].path] : [];
      const nextActive = session?.activePath && available.has(session.activePath)
        ? session.activePath
        : nextTabs[0] ?? null;
      setOpenTabs(nextTabs);
      setActivePath(nextActive);
      setTopModule(session?.topModule || "counter_tb");
      loadedSessionProject.current = activeProject.id;
    };
    void loadWorkspace().catch((error: unknown) => {
      const message = error instanceof Error ? error.message : "Workspace load failed. Check browser storage and retry.";
      setWorkspaceMessage(message);
      setLog(message);
    });
    return () => {
      cancelled = true;
      for (const timer of saveTimers.current.values()) clearTimeout(timer);
      saveTimers.current.clear();
    };
  }, [activeProject.id, projectManager, workspaceService]);

  useEffect(() => {
    const unsubscribe = workspaceService.subscribeChanges((change) => {
      const current = filesRef.current.find((file) => file.path === change.path);
      if (current && current.saveState !== "saved") {
        updateFile(change.path, (file) => ({ ...file, saveState: "save-failed" }));
        setConflictPath(change.path);
        setWorkspaceMessage(`Another tab changed ${change.path}. Your editor draft is still here; choose reload or replace the stored copy.`);
        return;
      }
      if (change.operation === "delete") {
        revisionByPath.current.delete(change.path);
        setFiles((currentFiles) => currentFiles.filter((file) => file.path !== change.path));
        setOpenTabs((currentTabs) => currentTabs.filter((path) => path !== change.path));
        setActivePath((currentPath) => currentPath === change.path ? null : currentPath);
        return;
      }
      void workspaceService.readFile(change.path).then((latest) => {
        revisionByPath.current.set(change.path, latest.revision);
        const latestContent = new TextDecoder().decode(latest.bytes);
        setFiles((currentFiles) => {
          const found = currentFiles.some((file) => file.path === change.path);
          return found
            ? currentFiles.map((file) => file.path === change.path ? { ...file, content: latestContent, revision: latest.revision, saveState: "saved" } : file)
            : [...currentFiles, { path: change.path, content: latestContent, revision: latest.revision, saveState: "saved" }];
        });
      }).catch((error: unknown) => setWorkspaceMessage(error instanceof Error ? error.message : "A file changed in another tab and could not be reloaded."));
    });
    return unsubscribe;
  }, [updateFile, workspaceService]);

  useEffect(() => {
    if (!projectManager) {
      setStorageStatus(null);
      return;
    }
    setStorageStatus(projectManager.storageStatus);
    let cancelled = false;
    let refreshTimer: ReturnType<typeof setTimeout> | undefined;
    const refreshEstimate = async () => {
      try {
        const estimate = await navigator.storage?.estimate();
        if (cancelled || !estimate) return;
        setStorageStatus((current) => ({
          ...(current ?? projectManager.storageStatus),
          usageBytes: estimate.usage,
          quotaBytes: estimate.quota,
        }));
      } catch {
        // Storage estimates are advisory and may be unavailable.
      }
    };
    const scheduleEstimate = () => {
      if (refreshTimer) clearTimeout(refreshTimer);
      refreshTimer = setTimeout(() => { void refreshEstimate(); }, 250);
    };
    void refreshEstimate();
    const unsubscribe = workspaceService.subscribeChanges(scheduleEstimate);
    return () => {
      cancelled = true;
      if (refreshTimer) clearTimeout(refreshTimer);
      unsubscribe();
    };
  }, [projectManager, workspaceService]);

  useEffect(() => {
    if (!projectManager || loadedSessionProject.current !== activeProject.id) return;
    const timer = setTimeout(() => {
      void projectManager.saveEditorState(activeProject.id, { openTabs, activePath, topModule }).catch((error: unknown) => {
        setWorkspaceMessage(error instanceof Error ? error.message : "Could not save editor settings.");
      });
    }, 250);
    return () => clearTimeout(timer);
  }, [activePath, activeProject.id, openTabs, projectManager, topModule]);

  const saveDraft = useCallback((path: ProjectPath, content: string): Promise<void> => {
    const previous = saveQueues.current.get(path);
    const task = (async () => {
      await previous?.catch(() => undefined);
      updateFile(path, (file) => ({ ...file, saveState: "saving" }));
      try {
        const expectedRevision = revisionByPath.current.get(path) ?? null;
        const saved = await workspaceService.writeFile(path, new TextEncoder().encode(content), expectedRevision);
        revisionByPath.current.set(path, saved.revision);
        updateFile(path, (file) => ({
          ...file,
          revision: saved.revision,
          saveState: file.content === content ? "saved" : "dirty",
        }));
        setWorkspaceMessage("");
      } catch (error) {
        updateFile(path, (file) => ({ ...file, saveState: "save-failed" }));
        const code = typeof error === "object" && error !== null && "code" in error ? String(error.code) : "";
        const message = error instanceof Error ? error.message : "The browser could not save this file.";
        if (code === "REVISION_CONFLICT") setConflictPath(path);
        setWorkspaceMessage(message);
        throw error;
      }
    })();
    saveQueues.current.set(path, task);
    return task.finally(() => {
      if (saveQueues.current.get(path) === task) saveQueues.current.delete(path);
    });
  }, [updateFile, workspaceService]);

  const scheduleSave = useCallback((path: ProjectPath, content: string) => {
    const previous = saveTimers.current.get(path);
    if (previous) clearTimeout(previous);
    const timer = setTimeout(() => {
      saveTimers.current.delete(path);
      void saveDraft(path, content).catch(() => undefined);
    }, 450);
    saveTimers.current.set(path, timer);
  }, [saveDraft]);

  const flushSave = useCallback(async (path: ProjectPath, content: string) => {
    const timer = saveTimers.current.get(path);
    if (timer) {
      clearTimeout(timer);
      saveTimers.current.delete(path);
    }
    const current = filesRef.current.find((file) => file.path === path);
    if (current?.saveState === "saved" && !saveQueues.current.has(path)) return;
    await saveDraft(path, content);
  }, [saveDraft]);

  const saveProject = async (): Promise<boolean> => {
    setWorkspaceMessage("");
    try {
      await Promise.all(filesRef.current.filter((file) => file.saveState !== "saved").map((file) => flushSave(file.path, file.content)));
      setWorkspaceMessage("All project files are saved in this browser.");
      return true;
    } catch {
      return false;
    }
  };

  const switchProject = async (projectId: string) => {
    if (!projectManager || projectId === activeProject.id) return;
    if (!await saveProject()) return;
    setBusy(true);
    try {
      const selected = await projectManager.selectProject(projectId as ProjectId);
      setActiveProject(selected);
      const selectedWorkspace = projectManager.getWorkspace(selected.id);
      controller?.setWorkspace(selectedWorkspace);
      setWorkspaceService(selectedWorkspace);
      setDiagnostics([]);
      const latestProjects = await projectManager.listProjects();
      setProjects(latestProjects);
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not switch browser projects.");
    } finally {
      setBusy(false);
    }
  };

  const exportProject = () => {
    try {
      const archive = exportProjectZip(filesRef.current.map((file) => ({
        path: file.path,
        bytes: new TextEncoder().encode(file.content),
      })));
      const url = URL.createObjectURL(new Blob([Uint8Array.from(archive).buffer], { type: "application/zip" }));
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `${activeProject.name}.zip`;
      anchor.click();
      URL.revokeObjectURL(url);
      setWorkspaceMessage("Project archive exported, including the current editor contents.");
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not export this project.");
    }
  };

  const importProject = async (file?: File) => {
    if (!file || !projectManager) return;
    if (!await saveProject()) return;
    setBusy(true);
    try {
      const imported = await projectManager.importProject(file, file.name.replace(/\.zip$/i, ""));
      const latestProjects = await projectManager.listProjects();
      setProjects(latestProjects);
      const selected = await projectManager.selectProject(imported.id);
      setActiveProject(selected);
      setWorkspaceService(projectManager.getWorkspace(selected.id));
      setDiagnostics([]);
      setWorkspaceMessage(`Imported ${file.name} as ${imported.name}.`);
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not import this project archive.");
    } finally {
      setBusy(false);
      if (projectArchiveInput.current) projectArchiveInput.current.value = "";
    }
  };

  useEffect(() => {
    if (!fileMenu) return;
    const close = () => setFileMenu(null);
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === "Escape") close(); };
    window.addEventListener("pointerdown", close);
    window.addEventListener("scroll", close, true);
    window.addEventListener("resize", close);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("scroll", close, true);
      window.removeEventListener("resize", close);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [fileMenu]);

  const createFile = async () => {
    const proposed = window.prompt("New file path, relative to the project root", "rtl/new_module.sv")?.trim();
    if (!proposed) return;
    const parsed = ProjectPathSchema.safeParse(proposed);
    if (!parsed.success) {
      setWorkspaceMessage("File paths must be normalized project-relative paths without traversal segments.");
      return;
    }
    const path = parsed.data;
    if (filesRef.current.some((file) => file.path === path)) {
      setWorkspaceMessage(`${path} already exists.`);
      openFile(path);
      return;
    }
    setBusy(true);
    try {
      const { revision } = await workspaceService.writeFile(path, new Uint8Array(), null);
      revisionByPath.current.set(path, revision);
      // The workspace change subscription may have added the file already.
      setFiles((current) => current.some((file) => file.path === path) ? current : [...current, { path, content: "", revision, saveState: "saved" }]);
      openFile(path);
      setWorkspaceMessage(`Created ${path}.`);
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not create this file.");
    } finally {
      setBusy(false);
    }
  };

  const renameFile = async (target: EditorFile) => {
    if (!projectManager) return;
    const proposed = window.prompt("New project-relative file path", target.path);
    if (!proposed || proposed === target.path) return;
    const parsed = ProjectPathSchema.safeParse(proposed);
    if (!parsed.success) {
      setWorkspaceMessage("File paths must be normalized project-relative paths without traversal segments.");
      return;
    }
    if (!await saveProject()) return;
    setBusy(true);
    try {
      const revision = revisionByPath.current.get(target.path);
      if (!revision) throw new Error("The current file revision is unavailable; reload the project before renaming it.");
      const nextRevision = await projectManager.renameFile(activeProject.id, target.path, parsed.data, revision);
      const oldPath = target.path;
      revisionByPath.current.delete(oldPath);
      revisionByPath.current.set(parsed.data, nextRevision);
      setFiles((current) => current.map((file) => file.path === oldPath ? { ...file, path: parsed.data, revision: nextRevision, saveState: "saved" } : file));
      setOpenTabs((current) => current.map((path) => path === oldPath ? parsed.data : path));
      setActivePath((current) => current === oldPath ? parsed.data : current);
      setWorkspaceMessage(`Renamed ${oldPath} to ${parsed.data}.`);
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not rename this file.");
    } finally {
      setBusy(false);
    }
  };

  const deleteFile = async (file: EditorFile) => {
    if (!window.confirm(`Delete ${file.path} from ${activeProject.name}? Any editor draft for this file will be discarded.`)) return;
    setBusy(true);
    try {
      const revision = revisionByPath.current.get(file.path);
      if (!revision) throw new Error("The current file revision is unavailable; reload the project before deleting it.");
      await workspaceService.deleteFile(file.path, revision);
      revisionByPath.current.delete(file.path);
      setFiles((current) => current.filter((candidate) => candidate.path !== file.path));
      setOpenTabs((current) => current.filter((path) => path !== file.path));
      setActivePath((current) => current === file.path ? openTabs.find((path) => path !== file.path) ?? null : current);
      setWorkspaceMessage(`Deleted ${file.path}.`);
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not delete this file.");
    } finally {
      setBusy(false);
    }
  };

  const resolveConflict = async (action: "reload" | "replace") => {
    if (!conflictPath) return;
    const path = conflictPath;
    const pendingSave = saveTimers.current.get(path);
    if (pendingSave) {
      clearTimeout(pendingSave);
      saveTimers.current.delete(path);
    }
    try {
      if (action === "reload") {
        try {
          const latest = await workspaceService.readFile(path);
          revisionByPath.current.set(path, latest.revision);
          setFiles((current) => current.map((file) => file.path === path
            ? { ...file, content: new TextDecoder().decode(latest.bytes), revision: latest.revision, saveState: "saved" }
            : file));
        } catch (error) {
          const code = typeof error === "object" && error !== null && "code" in error ? String(error.code) : "";
          if (code !== "NOT_FOUND") throw error;
          revisionByPath.current.delete(path);
          setFiles((current) => current.filter((file) => file.path !== path));
          setOpenTabs((current) => current.filter((tab) => tab !== path));
          setActivePath((current) => current === path ? null : current);
        }
        setWorkspaceMessage(`Reloaded ${path} from browser storage.`);
      } else {
        if (!window.confirm(`Replace the other tab's stored version of ${path} with your editor draft?`)) return;
        let expectedRevision: string | null = null;
        try {
          expectedRevision = (await workspaceService.readFile(path)).revision;
        } catch (error) {
          const code = typeof error === "object" && error !== null && "code" in error ? String(error.code) : "";
          if (code !== "NOT_FOUND") throw error;
        }
        if (expectedRevision === null) revisionByPath.current.delete(path);
        else revisionByPath.current.set(path, expectedRevision);
        const draft = filesRef.current.find((file) => file.path === path)?.content;
        if (draft === undefined) throw new Error("The editor draft no longer exists.");
        await saveDraft(path, draft);
        setWorkspaceMessage(`Stored your draft for ${path}.`);
      }
      setConflictPath(null);
    } catch (error) {
      setWorkspaceMessage(error instanceof Error ? error.message : "Could not resolve the workspace conflict.");
    }
  };

  const handleEdit = (value: string) => {
    if (!activeFile || bridgeState === "connected") return;
    const path = activeFile.path;
    updateFile(path, (file) => ({ ...file, content: value, saveState: "dirty" }));
    scheduleSave(path, value);
  };

  const openFile = (path: ProjectPath) => {
    setOpenTabs((current) => current.includes(path) ? current : [...current, path]);
    setActivePath(path);
  };

  const closeFile = (path: ProjectPath) => {
    const file = filesRef.current.find((candidate) => candidate.path === path);
    if (file && file.saveState !== "saved") {
      setPendingClose(path);
      return;
    }
    removeTab(path);
  };

  const closeAfterSave = async (path: ProjectPath) => {
    const file = filesRef.current.find((candidate) => candidate.path === path);
    if (!file) return removeTab(path);
    setPendingClose(null);
    try {
      await saveDraft(path, file.content);
      removeTab(path);
    } catch {
      setPendingClose(path);
    }
  };

  const removeTab = (path: ProjectPath) => {
    const timer = saveTimers.current.get(path);
    if (timer) clearTimeout(timer);
    saveTimers.current.delete(path);
    const remaining = openTabs.filter((tab) => tab !== path);
    setOpenTabs(remaining);
    if (activePath === path) setActivePath(remaining.at(-1) ?? null);
    setPendingClose(null);
  };

  const run = async () => {
    if (busy || !topModule.trim()) return;
    setBusy(true);
    setLog(controller ? "Saving current draft and freezing a project snapshot for Xezim WASM…" : "Saving current draft and freezing a project snapshot…");
    try {
      if (activeFile) await flushSave(activeFile.path, activeFile.content);
      const snapshot = await workspaceService.snapshot();
      if (controller) {
        const result = await controller.execute({
          projectId: activeProject.id,
          snapshotId: snapshot.snapshotId,
          topModule: topModule.trim(),
          until: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } },
          workBudget: { maxWallTimeMs: 5_000, maxOutputBytes: 1_048_576, maxVcdBytes: 64 * 1024 * 1024 },
        });
        setDiagnostics(result.diagnostics);
        setLog([result.stdout, result.stderr].filter(Boolean).join("\n") || `Xezim WASM status: ${result.status}`);
        if (result.waveform && dumpFormat === "vcd") {
          setActivePanel("waveforms");
          await controller.showWaveform({ artifactId: result.waveform.id });
        }
        return;
      }
      const compile = await simulation.compile({
        projectId: activeProject.id,
        snapshotId: snapshot.snapshotId,
        topModule: topModule.trim(),
        includeDirs: ["rtl", "tb"],
        defines: {},
        mode: "full",
      });
      setDiagnostics(compile.diagnostics);
      if (!compile.success) {
        setLog("Mock compile rejected the fixture and returned editor diagnostics. No run was started.");
        return;
      }
      const result = await simulation.run({
        projectId: compile.projectId,
        snapshotId: compile.snapshotId,
        compileId: compile.compileId,
        until: { ticks: "1000", timescale: { magnitude: "1", unit: "ns" } },
        workBudget: { maxSteps: 250_000, maxWallTimeMs: 5_000, maxOutputBytes: 1_048_576, maxVcdBytes: 64 * 1024 * 1024 },
      });
      setDiagnostics(result.diagnostics);
      setLog([result.stdout, result.stderr].filter(Boolean).join("\n") || `Mock run status: ${result.status}`);
    } catch (error) {
      setLog(error instanceof Error ? error.message : "The requested operation failed.");
    } finally {
      setBusy(false);
    }
  };

  const activeSaveLabel = activeFile?.saveState === "dirty"
    ? "Unsaved"
    : activeFile?.saveState === "saving"
      ? "Saving…"
      : activeFile?.saveState === "save-failed"
        ? "Save failed"
        : "Saved";

  // Only abnormal runtime states earn space in the app bar; the status bar always reports the runtime.
  const runtime: { tone: "ok" | "warn"; label: string; detail: string } = controller
    ? controller.canExecute
      ? { tone: "ok", label: "Xezim WASM", detail: "Simulation runs Xezim WebAssembly in a browser Worker." }
      : { tone: "warn", label: "WASM unavailable", detail: "The Xezim WASM assets were not found; no native simulation fallback is configured." }
    : { tone: "warn", label: "Mock services", detail: "Mock services are active. No HDL simulation is performed." };

  return (
    <div className="ide-shell">
      <header className="app-bar">
        <div className="app-bar-start">
          <div className="brand" title="HDL Workbench">
            <span className="brand-mark"><BrandMark /></span>
            <span className="brand-name">HDL Workbench</span>
          </div>
          <span className="bar-divider" aria-hidden="true" />
          <button
            className="icon-button explorer-toggle"
            type="button"
            aria-label={layout.explorerCollapsed ? "Show file explorer" : "Hide file explorer"}
            title="Toggle file explorer (Ctrl+B)"
            aria-expanded={!layout.explorerCollapsed}
            onClick={toggleExplorer}
          ><SidebarIcon /></button>
          {projectManager ? (
            <div className="project-controls">
              <label className="sr-only" htmlFor="project-select">Project</label>
              <select id="project-select" value={activeProject.id} onChange={(event) => void switchProject(event.target.value)} disabled={busy}>
                {projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}
              </select>
            </div>
          ) : (
            <div className="breadcrumb"><span>Projects</span><span className="crumb-separator">/</span><strong>counter-lab</strong><span className="mode-pill">Local</span></div>
          )}
          {runtime.tone === "warn" && <span className="runtime-warning" title={runtime.detail}><WarningIcon />{runtime.label}</span>}
        </div>

        <div className="app-bar-end">
          {projectManager && <div className="button-group" role="group" aria-label="Project files">
            <button className="icon-button" type="button" aria-label="Save" title="Save all files" onClick={() => void saveProject()} disabled={busy || bridgeState === "connected"}><SaveIcon /></button>
            <button className="icon-button" type="button" aria-label="Import ZIP" title="Import project from ZIP" onClick={() => projectArchiveInput.current?.click()} disabled={busy || bridgeState === "connected"}><ImportIcon /></button>
            <button className="icon-button" type="button" aria-label="Export ZIP" title="Export project as ZIP" onClick={exportProject} disabled={busy || files.length === 0}><ExportIcon /></button>
            <input
              ref={projectArchiveInput}
              className="sr-only"
              type="file"
              accept=".zip,application/zip"
              aria-label="Import project ZIP"
              onChange={(event) => void importProject(event.currentTarget.files?.[0])}
            />
          </div>}
          <label className="dump-field" htmlFor="dump-format" title="Waveform dump format. The browser build of Xezim writes VCD only.">
            <span className="top-module-prefix" aria-hidden="true">dump</span>
            <select id="dump-format" aria-label="Waveform dump" value={dumpFormat} onChange={(event) => setDumpFormat(event.target.value === "off" ? "off" : "vcd")} disabled={busy}>
              <option value="vcd">VCD</option>
              <option value="fst" disabled>FST (not in browser build)</option>
              <option value="off">Off</option>
            </select>
          </label>
          <div className="run-group">
            <label className="top-module-field" htmlFor="top-module" title="Top module">
              <span className="top-module-prefix" aria-hidden="true">top</span>
              <input id="top-module" aria-label="Top module" className="top-module-input" value={topModule} onChange={(event) => setTopModule(event.target.value)} disabled={busy} spellCheck={false} />
            </label>
            <button className="run-button" type="button" onClick={() => void run()} disabled={busy || !activeFile || !topModule.trim() || Boolean(controller && !controller.canExecute)} title={controller && !controller.canExecute ? runtime.detail : "Compile and simulate the top module"}>
              {busy ? <SpinnerIcon /> : <PlayIcon />} {busy ? "Running…" : "Run"}
            </button>
          </div>
          {controller && <div className="agent-anchor">
            <button className={`agent-button ${bridgeState === "connected" ? "connected" : ""}`} type="button" onClick={() => setBridgePanelOpen((open) => !open)} aria-expanded={bridgePanelOpen}>
              <AgentIcon />{bridgeState === "connected" ? "Agent connected" : "Connect agent"}
            </button>
            {bridgePanelOpen && bridgeClientRef.current && <McpConnectionPanel
              client={bridgeClientRef.current}
              state={bridgeState}
              message={bridgeMessage}
              onClose={() => setBridgePanelOpen(false)}
            />}
          </div>}
          <a className="icon-button help-link" href="/docs/index.html" target="_blank" rel="noopener" aria-label="How HDL Workbench works" title="How HDL Workbench works"><HelpIcon /></a>
        </div>
      </header>

      <div className="workspace-main">
        <aside className={`file-sidebar ${layout.explorerCollapsed ? "collapsed" : ""}`} aria-label="Project files">
          <div className="sidebar-heading">
            <span>EXPLORER</span>
            <span className="sidebar-heading-actions">
              <button type="button" className="subtle-icon" aria-label="New file" title="New file" onClick={() => void createFile()} disabled={busy || bridgeState === "connected"}><NewFileIcon /></button>
              <button type="button" className="subtle-icon" aria-label="Collapse file explorer" title="Collapse file explorer (Ctrl+B)" onClick={toggleExplorer}><CollapseLeftIcon /></button>
            </span>
          </div>
          <div className="project-root"><ChevronIcon className="chevron" /><span className="project-root-name">{activeProject.name}</span></div>
          {projectManager && <div className="authority-row">
            <label htmlFor="authority-mode">Authority</label>
            <select id="authority-mode" value={bridgeState === "connected" ? "connected" : "browser-only"} onChange={() => undefined}>
              <option value="browser-only">Browser only</option>
              <option value="connected">Local MCP bridge project</option>
            </select>
          </div>}
          <div className="file-tree">
            {folders.map((folder) => <div className="folder-row" key={folder}><span className="tree-indent" /><ChevronIcon className="chevron" /><FolderIcon className="folder-icon" />{folder}</div>)}
            {sortedFiles.map((file) => (
              <button
                className={`file-row ${activePath === file.path ? "selected" : ""}`}
                key={file.path}
                type="button"
                aria-label={`Open ${file.path}`}
                onClick={() => openFile(file.path)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  if (bridgeState === "connected") return;
                  setFileMenu({ path: file.path, x: event.clientX, y: event.clientY });
                }}
                disabled={busy}
              >
                <span className="tree-indent tree-indent-two" />
                <span className="sv-icon" aria-label={fileKindLabel(file.path)}>{fileKindLabel(file.path)}</span>
                <span>{fileName(file.path)}</span>
                {file.saveState !== "saved" && <span className="dirty-dot" aria-label="Unsaved changes" />}
              </button>
            ))}
          </div>
          {fileMenu && <div className="file-context-menu" role="menu" aria-label={`Actions for ${fileMenu.path}`} style={{ left: fileMenu.x, top: fileMenu.y }} onPointerDown={(event) => event.stopPropagation()}>
            <button type="button" role="menuitem" disabled={!projectManager} title={projectManager ? undefined : "Renaming needs browser project storage"} onClick={() => {
              const target = filesRef.current.find((file) => file.path === fileMenu.path);
              setFileMenu(null);
              if (target) void renameFile(target);
            }}>Rename…</button>
            <button type="button" role="menuitem" onClick={() => {
              const target = filesRef.current.find((file) => file.path === fileMenu.path);
              setFileMenu(null);
              if (target) void deleteFile(target);
            }}>Delete…</button>
          </div>}
          <div className="sidebar-footer">
            <span className="storage-icon">▱</span>
            <span>{projectManager
              ? storageStatus?.persistent ? "Persistent storage requested and granted" : "Browser may clear stored data"
              : "In-memory demo workspace"}</span>
            {storageStatus && <span className="storage-estimate" title="Origin-wide estimate from navigator.storage.estimate()">
              {storageStatus.usageBytes === undefined
                ? "Origin storage estimate unavailable"
                : `Origin storage estimate: ${formatBytes(storageStatus.usageBytes)} used${storageStatus.quotaBytes === undefined ? "" : ` of ${formatBytes(storageStatus.quotaBytes)} quota`}`}
            </span>}
          </div>
        </aside>

        <main
          ref={editorWorkspaceRef}
          className={`editor-workspace ${resizingPanel ? "resizing" : ""}`}
          style={{ "--bottom-panel-height": `${layout.bottomPanelHeight}px` } as CSSProperties}
        >
          {(workspaceMessage || conflictPath) && <div className={`workspace-notice ${conflictPath ? "conflict" : ""}`} role={conflictPath ? "alert" : "status"}>
            <span>{conflictPath ? `Storage conflict for ${conflictPath}. Your editor draft remains available.` : workspaceMessage}</span>
            {conflictPath && <div className="workspace-notice-actions">
              <button type="button" onClick={() => void resolveConflict("reload")}>Reload stored version</button>
              <button type="button" onClick={() => void resolveConflict("replace")}>Replace stored version…</button>
            </div>}
          </div>}
          <div className="editor-tabs" role="tablist" aria-label="Open files">
            {openTabs.map((path) => (
              <div className={`editor-tab ${activePath === path ? "active" : ""}`} role="presentation" key={path}>
                <button type="button" role="tab" aria-selected={activePath === path} onClick={() => setActivePath(path)} disabled={busy}>
                  <span className="sv-icon" aria-label={fileKindLabel(path)}>{fileKindLabel(path)}</span>{fileName(path)}
                  {files.find((file) => file.path === path)?.saveState !== "saved" && <span className="tab-dirty" />}
                </button>
                <button className="tab-close" type="button" aria-label={`Close ${fileName(path)}`} onClick={() => closeFile(path)} disabled={busy}>×</button>
              </div>
            ))}
            <div className="save-state" aria-live="polite"><span className={`save-state-dot ${activeFile?.saveState ?? "saved"}`} />{activeSaveLabel}</div>
          </div>

          <div className="editor-breadcrumb"><span>⌂</span><span className="crumb-separator">/</span>{activePath?.split("/").map((segment, index, all) => <span key={`${segment}-${index}`} className={index === all.length - 1 ? "current-segment" : ""}>{index > 0 ? <span className="crumb-separator">/</span> : null}{segment}</span>)}</div>
          <div className="editor-region">
            <EditorSurface path={activePath} value={activeFile?.content ?? ""} diagnostics={diagnostics} onChange={handleEdit} readOnly={bridgeState === "connected"} />
          </div>

          <section className="bottom-panel" aria-label="Diagnostics, output and waveform viewer">
            <div
              className="panel-resizer"
              role="separator"
              aria-orientation="horizontal"
              aria-label="Resize bottom panel"
              aria-valuemin={MIN_BOTTOM_PANEL_HEIGHT}
              aria-valuenow={layout.bottomPanelHeight}
              tabIndex={0}
              title="Drag to resize · double-click to reset"
              onPointerDown={startPanelResize}
              onPointerMove={movePanelResize}
              onPointerUp={endPanelResize}
              onPointerCancel={endPanelResize}
              onKeyDown={resizePanelWithKeyboard}
              onDoubleClick={() => setBottomPanelHeight(DEFAULT_BOTTOM_PANEL_HEIGHT)}
            />
            <div className="panel-tabs" role="tablist" aria-label="Bottom panel">
              <button type="button" role="tab" aria-selected={activePanel === "problems"} className={`panel-tab ${activePanel === "problems" ? "active" : ""}`} onClick={() => setActivePanel("problems")}>PROBLEMS <span className="count-pill">{diagnostics.filter((item) => item.severity === "error").length}</span></button>
              <button type="button" role="tab" aria-selected={activePanel === "output"} className={`panel-tab ${activePanel === "output" ? "active" : ""}`} onClick={() => setActivePanel("output")}>OUTPUT</button>
              <button type="button" role="tab" aria-selected={activePanel === "waveforms"} className={`panel-tab ${activePanel === "waveforms" ? "active" : ""}`} onClick={() => setActivePanel("waveforms")}>WAVEFORMS</button>
              <span className="panel-spacer" />
              {activePanel !== "waveforms" && <span className="panel-caption">{controller ? "Xezim WASM Worker" : "Mock adapter response"}</span>}
            </div>
            {activePanel === "problems" && <div className="panel-content">
              <section className="diagnostics-list" aria-label="Diagnostics">
                {diagnostics.length === 0 ? (
                  <div className="empty-panel"><span className="checkmark">✓</span> No diagnostics yet <span>{controller ? "Run Xezim in the browser Worker to see diagnostics." : "Run the mock checks to see editor markers."}</span></div>
                ) : diagnostics.map((diagnostic, index) => (
                  <div className={`diagnostic-row ${diagnostic.severity}`} key={`${diagnostic.code}-${index}`}>
                    <span className="diagnostic-symbol">{diagnostic.severity === "error" ? "●" : "▲"}</span>
                    <span className="diagnostic-code">{diagnostic.code}</span>
                    <span className="diagnostic-message">{diagnostic.message}</span>
                    <span className="diagnostic-location">{diagnostic.path ? fileName(diagnostic.path) : ""}{diagnostic.line ? `:${diagnostic.line}:${diagnostic.column ?? 1}` : ""}</span>
                  </div>
                ))}
              </section>
              <pre className="output-preview" aria-label="Run output">{log}</pre>
            </div>}
            {activePanel === "output" && <pre className="output-full" aria-label="Run output">{log}</pre>}
            <div className="waveform-panel-content" style={{ display: activePanel === "waveforms" ? "flex" : "none" }}>
              <SurferWaveformPanel
                onAdapterChange={handleSurferAdapterChange}
                resolveWaveformUrl={resolveWaveformUrl}
              />
            </div>
          </section>
        </main>
      </div>

      <footer className="status-bar">
        <span className={`status-runtime ${runtime.tone}`} role="status" title={runtime.detail}><i className="status-dot" />{runtime.tone === "warn" && !controller ? `${runtime.label} · no HDL simulation` : runtime.label}</span>
        <span className="status-project">{bridgeState === "connected" ? "Local bridge project · editor read-only" : activeProject.authority === "browser-only" ? "Browser-only project" : "Connected project"}</span>
        <span className="status-spacer" />
        <span>{activeFile ? activeFile.content.split("\n").length : 0} lines</span>
        <span>Spaces: 2</span>
        <span>UTF-8</span>
        <span>{activePath ? fileKindLabel(activePath) === "SV" ? "SystemVerilog" : fileKindLabel(activePath) : "SystemVerilog"}</span>
        <a href="/licenses/THIRD_PARTY.md" target="_blank" rel="noreferrer">Third-party notices</a>
      </footer>

      {pendingClose && (
        <div className="dialog-backdrop">
          <section className="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="close-dialog-title">
            <div className="dialog-icon">!</div>
            <h2 id="close-dialog-title">Unsaved changes</h2>
            <p><strong>{fileName(pendingClose)}</strong> {files.find((file) => file.path === pendingClose)?.saveState === "saving" ? "is being saved. Wait for the save to finish before closing." : "has changes that have not been saved to the workspace."}</p>
            <div className="dialog-actions">
              <button type="button" className="secondary-button" onClick={() => setPendingClose(null)}>Keep editing</button>
              {files.find((file) => file.path === pendingClose)?.saveState === "saving"
                ? <button type="button" className="secondary-button" onClick={() => void closeAfterSave(pendingClose)}>Close after save</button>
                : <button type="button" className="danger-button" onClick={() => removeTab(pendingClose)}>Close without saving</button>}
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
