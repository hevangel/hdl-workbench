import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Diagnostic, ProjectPath, SimulationService, WorkspaceAdapter } from "@xezim-ide/protocol";
import { EditorSurface } from "./editor/EditorSurface.js";
import { createMockServices } from "./mockServices.js";
import { useLayoutPreferences } from "./hooks/useLayoutPreferences.js";

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
}

const demoServices = createMockServices();

function fileName(path: string): string {
  return path.split("/").at(-1) ?? path;
}

export function App({ workspace = demoServices.workspace, simulation = demoServices.simulation }: AppProps) {
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
  const [layout, setLayout] = useLayoutPreferences();
  const revisionByPath = useRef(new Map<ProjectPath, string>());
  const saveTimers = useRef(new Map<ProjectPath, ReturnType<typeof setTimeout>>());
  const saveQueues = useRef(new Map<ProjectPath, Promise<void>>());

  const activeFile = files.find((file) => file.path === activePath) ?? null;
  const sortedFiles = useMemo(() => [...files].sort((left, right) => left.path.localeCompare(right.path)), [files]);
  const folders = [...new Set(sortedFiles.map((file) => file.path.split("/").slice(0, -1).join("/")).filter(Boolean))].sort();

  useEffect(() => {
    let cancelled = false;
    const loadWorkspace = async () => {
      const entries = await workspace.listFiles();
      const loaded = await Promise.all(entries.map(async (entry) => {
        const result = await workspace.readFile(entry.path);
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
      const firstPath = loaded[0]?.path ?? null;
      setOpenTabs(firstPath ? [firstPath] : []);
      setActivePath(firstPath);
    };
    void loadWorkspace().catch(() => setLog("Workspace load failed. Check storage and retry."));
    return () => {
      cancelled = true;
      for (const timer of saveTimers.current.values()) clearTimeout(timer);
      saveTimers.current.clear();
    };
  }, [workspace]);

  const updateFile = useCallback((path: ProjectPath, update: (file: EditorFile) => EditorFile) => {
    setFiles((current) => current.map((file) => file.path === path ? update(file) : file));
  }, []);

  const saveDraft = useCallback((path: ProjectPath, content: string): Promise<void> => {
    const previous = saveQueues.current.get(path);
    const task = (async () => {
      await previous?.catch(() => undefined);
      updateFile(path, (file) => ({ ...file, saveState: "saving" }));
      try {
        const expectedRevision = revisionByPath.current.get(path) ?? null;
        const saved = await workspace.writeFile(path, new TextEncoder().encode(content), expectedRevision);
        revisionByPath.current.set(path, saved.revision);
        updateFile(path, (file) => ({ ...file, revision: saved.revision, saveState: "saved" }));
      } catch {
        updateFile(path, (file) => ({ ...file, saveState: "save-failed" }));
        throw new Error("Save failed. The workspace may have changed in another tab.");
      }
    })();
    saveQueues.current.set(path, task);
    return task.finally(() => {
      if (saveQueues.current.get(path) === task) saveQueues.current.delete(path);
    });
  }, [updateFile, workspace]);

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
      const snapshot = await workspace.snapshot();
      const compile = await simulation.compile({
        projectId: "demo-project",
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
          <div className="breadcrumb"><span>Projects</span><span className="crumb-separator">/</span><strong>counter-lab</strong><span className="branch-pill">LOCAL</span></div>
        </div>
        <div className="run-controls">
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
          <div className="project-root"><span className="chevron">⌄</span><span className="folder-icon">▰</span> counter-lab</div>
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
          <div className="sidebar-footer"><span className="storage-icon">▱</span><span>In-memory demo workspace</span></div>
        </aside>

        <main className="editor-workspace">
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

          <section className="bottom-panel" aria-label="Diagnostics and output">
            <div className="panel-tabs"><div className="panel-tab active" role="heading" aria-level={2}>PROBLEMS <span className="count-pill">{diagnostics.filter((item) => item.severity === "error").length}</span></div><div className="panel-tab" role="heading" aria-level={2}>OUTPUT</div><span className="panel-spacer" /><span className="panel-caption">Mock adapter response</span></div>
            <div className="panel-content">
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
            </div>
          </section>

          <footer className="status-bar"><span><i className="status-dot green" /> Browser-only shell</span><span>UTF-8</span><span>SystemVerilog</span><span>Ln {activeFile ? activeFile.content.split("\n").length : 1}</span><span>Spaces: 2</span></footer>
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
