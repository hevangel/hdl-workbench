import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ProjectPathSchema, type Diagnostic, type ProjectId, type ProjectPath, type SimulationService, type SurferAdapter, type WorkspaceAdapter } from "@xezim-ide/protocol";
import { EditorSurface } from "./editor/EditorSurface.js";
import { createMockServices } from "./mockServices.js";
import { useLayoutPreferences } from "./hooks/useLayoutPreferences.js";
import type { PersistentWorkspaceManager, WorkspaceProject } from "./workspace/opfsWorkspace.js";
import { exportProjectZip } from "./workspace/projectArchive.js";
import { SurferWaveformPanel } from "./surfer/SurferWaveformPanel.js";

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
  projectManager?: PersistentWorkspaceManager;
  onSurferAdapterChange?: (adapter: SurferAdapter | null) => void;
}

const demoServices = createMockServices();

function fileName(path: string): string {
  return path.split("/").at(-1) ?? path;
}

export function App({ workspace, simulation = demoServices.simulation, projectManager, onSurferAdapterChange }: AppProps) {
  const [workspaceService, setWorkspaceService] = useState<WorkspaceAdapter>(() => workspace ?? projectManager?.getWorkspace() ?? demoServices.workspace);
  const [activeProject, setActiveProject] = useState<WorkspaceProject>(() => projectManager?.activeProject ?? {
    id: "demo-project" as ProjectId,
    name: "counter-lab",
    authority: "browser-only",
    createdAt: 0,
  });
  const [projects, setProjects] = useState<WorkspaceProject[]>([]);
  const [files, setFiles] = useState<EditorFile[]>([]);
  const filesRef = useRef(files);
  filesRef.current = files;
  const [openTabs, setOpenTabs] = useState<ProjectPath[]>([]);
  const [activePath, setActivePath] = useState<ProjectPath | null>(null);
  const [topModule, setTopModule] = useState("counter_tb");
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([]);
  const [log, setLog] = useState("Mock services are active. No HDL simulation is performed.");
  const [busy, setBusy] = useState(false);
  const [pendingClose, setPendingClose] = useState<ProjectPath | null>(null);
  const [conflictPath, setConflictPath] = useState<ProjectPath | null>(null);
  const [workspaceMessage, setWorkspaceMessage] = useState("");
  const [fileActionsOpen, setFileActionsOpen] = useState(false);
  const [activePanel, setActivePanel] = useState<"problems" | "output" | "waveforms">("problems");
  const projectArchiveInput = useRef<HTMLInputElement>(null);
  const [layout, setLayout] = useLayoutPreferences();
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

  useEffect(() => {
    if (workspace) setWorkspaceService(workspace);
  }, [workspace]);

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
      setWorkspaceService(projectManager.getWorkspace(selected.id));
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

  const renameActiveFile = async () => {
    if (!projectManager || !activeFile) return;
    const proposed = window.prompt("New project-relative file path", activeFile.path);
    if (!proposed || proposed === activeFile.path) return;
    const parsed = ProjectPathSchema.safeParse(proposed);
    if (!parsed.success) {
      setWorkspaceMessage("File paths must be normalized project-relative paths without traversal segments.");
      return;
    }
    if (!await saveProject()) return;
    setBusy(true);
    try {
      const revision = revisionByPath.current.get(activeFile.path);
      if (!revision) throw new Error("The current file revision is unavailable; reload the project before renaming it.");
      const nextRevision = await projectManager.renameFile(activeProject.id, activeFile.path, parsed.data, revision);
      const oldPath = activeFile.path;
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

  const deleteActiveFile = async () => {
    if (!activeFile) return;
    const file = activeFile;
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
    if (!activeFile) return;
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
    setLog("Saving current draft and freezing a project snapshot…");
    try {
      if (activeFile) await flushSave(activeFile.path, activeFile.content);
      const snapshot = await workspaceService.snapshot();
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

  return (
    <div className="ide-shell">
      <header className="app-header">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">X</div>
          <div>
            <div className="brand-name">xezim <span>studio</span></div>
            <div className="brand-caption">SystemVerilog workspace</div>
          </div>
        </div>
        <div className="mock-badge" role="status"><span className="status-dot" /> MOCK SERVICES · NO HDL SIMULATION</div>
        <div className="header-actions">
          <button className="icon-button" type="button" aria-label="Settings" title="Settings are not available in this preview">⚙</button>
          <div className="avatar" aria-label="Local workspace">LX</div>
        </div>
      </header>

      <div className="workspace-toolbar">
        <div className="toolbar-project">
          <button
            className="explorer-toggle"
            type="button"
            aria-label={layout.explorerCollapsed ? "Show file explorer" : "Hide file explorer"}
            title="Toggle file explorer"
            onClick={() => setLayout({ explorerCollapsed: !layout.explorerCollapsed })}
          >▤</button>
          {projectManager ? (
            <div className="project-controls">
              <label className="sr-only" htmlFor="project-select">Project</label>
              <select id="project-select" value={activeProject.id} onChange={(event) => void switchProject(event.target.value)} disabled={busy}>
                {projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}
              </select>
              <span className="branch-pill">BROWSER ONLY</span>
            </div>
          ) : (
            <div className="breadcrumb"><span>Projects</span><span className="crumb-separator">/</span><strong>counter-lab</strong><span className="branch-pill">LOCAL</span></div>
          )}
        </div>
        <div className="run-controls">
          {projectManager && <>
            <button className="secondary-button toolbar-button" type="button" onClick={() => void saveProject()} disabled={busy}>Save</button>
            <button className="secondary-button toolbar-button" type="button" onClick={() => projectArchiveInput.current?.click()} disabled={busy}>Import ZIP</button>
            <button className="secondary-button toolbar-button" type="button" onClick={exportProject} disabled={busy || files.length === 0}>Export ZIP</button>
            <input
              ref={projectArchiveInput}
              className="sr-only"
              type="file"
              accept=".zip,application/zip"
              aria-label="Import project ZIP"
              onChange={(event) => void importProject(event.currentTarget.files?.[0])}
            />
          </>}
          <label className="top-module-label" htmlFor="top-module">Top module</label>
          <input id="top-module" className="top-module-input" value={topModule} onChange={(event) => setTopModule(event.target.value)} disabled={busy} />
          <button className="run-button" type="button" onClick={() => void run()} disabled={busy || !activeFile || !topModule.trim()}>
            <span aria-hidden="true">{busy ? "◌" : "▶"}</span> {busy ? "Running…" : "Run"}
          </button>
        </div>
      </div>

      <div className="workspace-main">
        <aside className={`file-sidebar ${layout.explorerCollapsed ? "collapsed" : ""}`} aria-label="Project files">
          <div className="sidebar-heading"><span>EXPLORER</span><button type="button" className="subtle-icon" aria-label="More file actions">···</button></div>
          <div className="project-root"><span className="chevron">⌄</span><span className="folder-icon">▰</span> {activeProject.name}</div>
          {projectManager && <div className="authority-row">
            <label htmlFor="authority-mode">Authority</label>
            <select id="authority-mode" value="browser-only" onChange={() => undefined}>
              <option value="browser-only">Browser only</option>
              <option value="connected" disabled>Connected (bridge not configured)</option>
            </select>
          </div>}
          {projectManager && <div className="file-actions">
            <button type="button" className="secondary-button toolbar-button" onClick={() => setFileActionsOpen((open) => !open)} aria-expanded={fileActionsOpen} aria-label="File actions" disabled={!activeFile || busy}>File actions</button>
            {fileActionsOpen && <div className="file-actions-menu">
              <button type="button" onClick={() => { setFileActionsOpen(false); void renameActiveFile(); }} disabled={!activeFile || busy}>Rename active file…</button>
              <button type="button" onClick={() => { setFileActionsOpen(false); void deleteActiveFile(); }} disabled={!activeFile || busy}>Delete active file…</button>
            </div>}
          </div>}
          <div className="file-tree">
            {folders.map((folder) => <div className="folder-row" key={folder}><span className="tree-indent" /><span className="chevron">⌄</span><span className="folder-icon">▰</span>{folder}</div>)}
            {sortedFiles.map((file) => (
              <button
                className={`file-row ${activePath === file.path ? "selected" : ""}`}
                key={file.path}
                type="button"
                aria-label={`Open ${file.path}`}
                onClick={() => openFile(file.path)}
                disabled={busy}
              >
                <span className="tree-indent tree-indent-two" />
                <span className="sv-icon">SV</span>
                <span>{fileName(file.path)}</span>
                {file.saveState !== "saved" && <span className="dirty-dot" aria-label="Unsaved changes" />}
              </button>
            ))}
          </div>
          <div className="sidebar-footer"><span className="storage-icon">▱</span><span>{projectManager
            ? projectManager.storageStatus.persistent ? "Persistent storage requested and granted" : "Browser may clear stored data"
            : "In-memory demo workspace"}</span></div>
        </aside>

        <main className="editor-workspace">
          {projectManager && (workspaceMessage || conflictPath) && <div className={`workspace-notice ${conflictPath ? "conflict" : ""}`} role={conflictPath ? "alert" : "status"}>
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
                  <span className="sv-icon">SV</span>{fileName(path)}
                  {files.find((file) => file.path === path)?.saveState !== "saved" && <span className="tab-dirty" />}
                </button>
                <button className="tab-close" type="button" aria-label={`Close ${fileName(path)}`} onClick={() => closeFile(path)} disabled={busy}>×</button>
              </div>
            ))}
            <div className="save-state" aria-live="polite"><span className={`save-state-dot ${activeFile?.saveState ?? "saved"}`} />{activeSaveLabel}</div>
          </div>

          <div className="editor-breadcrumb"><span>⌂</span><span className="crumb-separator">/</span>{activePath?.split("/").map((segment, index, all) => <span key={`${segment}-${index}`} className={index === all.length - 1 ? "current-segment" : ""}>{index > 0 ? <span className="crumb-separator">/</span> : null}{segment}</span>)}</div>
          <div className="editor-region">
            <EditorSurface path={activePath} value={activeFile?.content ?? ""} diagnostics={diagnostics} onChange={handleEdit} />
          </div>

          <section className="bottom-panel" aria-label="Diagnostics, output and waveform viewer">
            <div className="panel-tabs" role="tablist" aria-label="Bottom panel">
              <button type="button" role="tab" aria-selected={activePanel === "problems"} className={`panel-tab ${activePanel === "problems" ? "active" : ""}`} onClick={() => setActivePanel("problems")}>PROBLEMS <span className="count-pill">{diagnostics.filter((item) => item.severity === "error").length}</span></button>
              <button type="button" role="tab" aria-selected={activePanel === "output"} className={`panel-tab ${activePanel === "output" ? "active" : ""}`} onClick={() => setActivePanel("output")}>OUTPUT</button>
              <button type="button" role="tab" aria-selected={activePanel === "waveforms"} className={`panel-tab ${activePanel === "waveforms" ? "active" : ""}`} onClick={() => setActivePanel("waveforms")}>WAVEFORMS</button>
              <span className="panel-spacer" />
              {activePanel !== "waveforms" && <span className="panel-caption">Mock adapter response</span>}
            </div>
            {activePanel === "problems" && <div className="panel-content">
              <section className="diagnostics-list" aria-label="Diagnostics">
                {diagnostics.length === 0 ? (
                  <div className="empty-panel"><span className="checkmark">✓</span> No diagnostics yet <span>Run the mock checks to see editor markers.</span></div>
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
            {activePanel === "waveforms" && <div className="waveform-panel-content"><SurferWaveformPanel {...(onSurferAdapterChange ? { onAdapterChange: onSurferAdapterChange } : {})} /></div>}
          </section>

          <footer className="status-bar"><span><i className="status-dot green" /> {activeProject.authority === "browser-only" ? "Browser-only project" : "Connected project"}</span><span>UTF-8</span><span>SystemVerilog</span><span>Ln {activeFile ? activeFile.content.split("\n").length : 1}</span><span>Spaces: 2</span></footer>
        </main>
      </div>

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
