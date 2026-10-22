import {
  FileEntrySchema,
  ProjectIdSchema,
  ProjectPathSchema,
  type FileChange,
  type FileEntry,
  type ProjectId,
  type ProjectPath,
  type Revision,
  type SnapshotId,
  type SnapshotFile,
  type WorkspaceAdapter,
} from "@xezim-ide/protocol";
import { exportProjectZip, importProjectZip, type ArchiveFile } from "./projectArchive.js";

const DATABASE_NAME = "xezim-studio-workspace";
const DATABASE_VERSION = 1;
const STORAGE_ROOT = "xezim-studio";
const PROJECTS_DIRECTORY = "projects";
const TEMP_DIRECTORY = "pending";
const ACTIVE_PROJECT_KEY = "activeProjectId";
const DEFAULT_PROJECT_ID = "counter-lab" as ProjectId;
const DEFAULT_PROJECT_NAME = "counter-lab";
const MAX_FILE_BYTES = 16 * 1024 * 1024;
const MAX_PROJECT_BYTES = 64 * 1024 * 1024;

export interface WorkspaceProject {
  id: ProjectId;
  name: string;
  authority: "browser-only";
  createdAt: number;
}

export interface EditorSessionState {
  openTabs: ProjectPath[];
  activePath: ProjectPath | null;
  topModule: string;
}

export interface BrowserStorageStatus {
  persistent: boolean;
  persistentStorageSupported: boolean;
  usageBytes: number | undefined;
  quotaBytes: number | undefined;
}

export interface PersistentWorkspaceManager {
  readonly activeProject: WorkspaceProject;
  readonly storageStatus: BrowserStorageStatus;
  getWorkspace(projectId?: ProjectId): OpfsWorkspace;
  listProjects(): Promise<WorkspaceProject[]>;
  selectProject(projectId: ProjectId): Promise<WorkspaceProject>;
  importProject(archive: Blob, name?: string): Promise<WorkspaceProject>;
  exportProject(projectId?: ProjectId): Promise<Uint8Array>;
  renameFile(projectId: ProjectId, oldPath: ProjectPath, newPath: ProjectPath, expectedRevision: Revision): Promise<Revision>;
  loadEditorState(projectId: ProjectId): Promise<EditorSessionState | null>;
  saveEditorState(projectId: ProjectId, value: EditorSessionState): Promise<void>;
}

interface ProjectRecord extends WorkspaceProject {}

interface PendingWrite {
  operation: "write";
  revision: Revision;
  sizeBytes: number;
  tempName: string;
}

interface PendingDelete {
  operation: "delete";
}

interface FileRecord {
  projectId: ProjectId;
  path: ProjectPath;
  revision: Revision;
  sizeBytes: number;
  pending?: PendingWrite | PendingDelete;
}

interface StoredEditorState extends EditorSessionState {
  projectId: ProjectId;
}

interface SettingsRecord {
  key: string;
  value: unknown;
}

export class WorkspaceStorageError extends Error {
  constructor(readonly code: string, message: string) {
    super(message);
    this.name = "WorkspaceStorageError";
  }
}

function requestValue<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IndexedDB request failed."));
  });
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onabort = () => reject(transaction.error ?? new Error("IndexedDB transaction aborted."));
    transaction.onerror = () => reject(transaction.error ?? new Error("IndexedDB transaction failed."));
  });
}

function openDatabase(): Promise<IDBDatabase> {
  if (typeof indexedDB === "undefined") return Promise.reject(new WorkspaceStorageError("INDEXEDDB_UNAVAILABLE", "IndexedDB is not available in this browser."));
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains("projects")) database.createObjectStore("projects", { keyPath: "id" });
      if (!database.objectStoreNames.contains("files")) database.createObjectStore("files");
      if (!database.objectStoreNames.contains("settings")) database.createObjectStore("settings", { keyPath: "key" });
      if (!database.objectStoreNames.contains("editorState")) database.createObjectStore("editorState", { keyPath: "projectId" });
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new WorkspaceStorageError("INDEXEDDB_OPEN_FAILED", "Could not open workspace metadata storage."));
    request.onblocked = () => reject(new WorkspaceStorageError("INDEXEDDB_BLOCKED", "Close older Xezim Studio tabs to upgrade workspace storage."));
  });
}

async function dbGet<T>(database: IDBDatabase, storeName: string, key: IDBValidKey): Promise<T | undefined> {
  const transaction = database.transaction(storeName, "readonly");
  const value = await requestValue(transaction.objectStore(storeName).get(key)) as T | undefined;
  await transactionDone(transaction);
  return value;
}

async function dbGetAll<T>(database: IDBDatabase, storeName: string): Promise<T[]> {
  const transaction = database.transaction(storeName, "readonly");
  const values = await requestValue(transaction.objectStore(storeName).getAll()) as T[];
  await transactionDone(transaction);
  return values;
}

async function dbPut(database: IDBDatabase, storeName: string, value: unknown, key?: IDBValidKey): Promise<void> {
  const transaction = database.transaction(storeName, "readwrite");
  if (key === undefined) transaction.objectStore(storeName).put(value);
  else transaction.objectStore(storeName).put(value, key);
  await transactionDone(transaction);
}

async function dbDelete(database: IDBDatabase, storeName: string, key: IDBValidKey): Promise<void> {
  const transaction = database.transaction(storeName, "readwrite");
  transaction.objectStore(storeName).delete(key);
  await transactionDone(transaction);
}

async function digest(bytes: Uint8Array): Promise<string> {
  if (!globalThis.crypto?.subtle) throw new WorkspaceStorageError("CRYPTO_UNAVAILABLE", "Secure browser hashing is unavailable; use a secure origin such as localhost or HTTPS.");
  const hash = await crypto.subtle.digest("SHA-256", bytes as BufferSource);
  return `sha256:${[...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("")}`;
}

function validatePath(path: string): ProjectPath {
  const result = ProjectPathSchema.safeParse(path);
  if (!result.success) throw new WorkspaceStorageError("INVALID_PATH", `Invalid project-relative path: ${path}`);
  return result.data;
}

function validateProjectId(projectId: string): ProjectId {
  const result = ProjectIdSchema.safeParse(projectId);
  if (!result.success) throw new WorkspaceStorageError("INVALID_PROJECT_ID", `Invalid project id: ${projectId}`);
  return result.data;
}

function validateProjectName(name: string): string {
  const normalized = name.trim().replace(/[\u0000-\u001f\u007f]/g, "").slice(0, 80);
  if (!normalized) throw new WorkspaceStorageError("INVALID_PROJECT_NAME", "Project name must contain at least one visible character.");
  return normalized;
}

async function directoryFor(root: FileSystemDirectoryHandle, parts: string[], create: boolean): Promise<FileSystemDirectoryHandle> {
  let directory = root;
  for (const part of parts) directory = await directory.getDirectoryHandle(part, { create });
  return directory;
}

async function parentFor(projectRoot: FileSystemDirectoryHandle, path: ProjectPath, create: boolean): Promise<{ parent: FileSystemDirectoryHandle; name: string }> {
  const parts = path.split("/");
  const name = parts.pop();
  if (!name) throw new WorkspaceStorageError("INVALID_PATH", "A file path needs a filename.");
  return { parent: await directoryFor(projectRoot, parts, create), name };
}

async function readPath(projectRoot: FileSystemDirectoryHandle, path: ProjectPath): Promise<Uint8Array> {
  const { parent, name } = await parentFor(projectRoot, path, false);
  const handle = await parent.getFileHandle(name);
  return new Uint8Array(await (await handle.getFile()).arrayBuffer());
}

async function writePath(projectRoot: FileSystemDirectoryHandle, path: ProjectPath, bytes: Uint8Array): Promise<void> {
  const { parent, name } = await parentFor(projectRoot, path, true);
  const handle = await parent.getFileHandle(name, { create: true });
  const writable = await handle.createWritable({ keepExistingData: false });
  try {
    await writable.write(Uint8Array.from(bytes).buffer);
    await writable.close();
  } catch (error) {
    await writable.abort().catch(() => undefined);
    throw error;
  }
}

function isNotFound(error: unknown): boolean {
  return typeof error === "object" && error !== null && "name" in error && error.name === "NotFoundError";
}

function storageError(error: unknown): never {
  if (error instanceof WorkspaceStorageError) throw error;
  const name = typeof error === "object" && error !== null && "name" in error ? String(error.name) : "";
  const message = error instanceof Error
    ? error.message
    : typeof error === "object" && error !== null && "message" in error
      ? String(error.message)
      : "The browser storage operation failed.";
  if (name === "QuotaExceededError") throw new WorkspaceStorageError("STORAGE_QUOTA_EXCEEDED", "Browser storage is full. Export a project or remove files before saving more data.");
  if (name === "NotAllowedError" || name === "SecurityError") throw new WorkspaceStorageError("STORAGE_PERMISSION_DENIED", "Browser storage permission was denied for this origin.");
  throw new WorkspaceStorageError("STORAGE_OPERATION_FAILED", message);
}

async function withProjectLock<T>(projectId: ProjectId, action: () => Promise<T>): Promise<T> {
  if (!navigator.locks) throw new WorkspaceStorageError("CROSS_TAB_LOCK_UNAVAILABLE", "This browser lacks cross-tab workspace locks; writes are disabled to avoid overwriting another tab.");
  return navigator.locks.request(`xezim-studio:${projectId}`, { mode: "exclusive" }, action);
}

function recordsForProject(database: IDBDatabase, projectId: ProjectId): Promise<FileRecord[]> {
  return dbGetAll<FileRecord>(database, "files").then((records) => records.filter((record) => record.projectId === projectId));
}

export class OpfsWorkspace implements WorkspaceAdapter {
  private readonly listeners = new Set<(change: FileChange) => void>();
  private readonly channel: BroadcastChannel | null;

  constructor(
    readonly project: WorkspaceProject,
    private readonly database: IDBDatabase,
    private readonly projectRoot: FileSystemDirectoryHandle,
    private readonly storageRoot: FileSystemDirectoryHandle,
  ) {
    this.channel = typeof BroadcastChannel === "undefined" ? null : new BroadcastChannel("xezim-studio-workspace-v1");
    this.channel?.addEventListener("message", (event: MessageEvent<FileChange & { projectId?: string }>) => {
      if (event.data?.projectId !== this.project.id) return;
      const { projectId: _projectId, ...change } = event.data;
      for (const listener of this.listeners) listener(change);
    });
  }

  private fileKey(path: ProjectPath): IDBValidKey {
    return [this.project.id, path];
  }

  private async getRecord(path: ProjectPath): Promise<FileRecord | undefined> {
    return dbGet<FileRecord>(this.database, "files", this.fileKey(path));
  }

  private async putRecord(record: FileRecord): Promise<void> {
    await dbPut(this.database, "files", record, this.fileKey(record.path));
  }

  private async notify(change: FileChange): Promise<void> {
    this.channel?.postMessage({ ...change, projectId: this.project.id });
  }

  private async tempDirectory(create: boolean): Promise<FileSystemDirectoryHandle> {
    const root = await directoryFor(this.storageRoot, [TEMP_DIRECTORY, this.project.id], create);
    return root;
  }

  private async recoverRecord(record: FileRecord): Promise<FileRecord | undefined> {
    const pending = record.pending;
    if (!pending) return record;
    if (pending.operation === "delete") {
      try {
        await readPath(this.projectRoot, record.path);
        const { pending: _pending, ...clean } = record;
        await this.putRecord(clean);
        return clean;
      } catch (error) {
        if (!isNotFound(error)) throw error;
        await dbDelete(this.database, "files", this.fileKey(record.path));
        return undefined;
      }
    }

    const tempRoot = await this.tempDirectory(false);
    let targetMatches = false;
    try {
      targetMatches = await digest(await readPath(this.projectRoot, record.path)) === pending.revision;
    } catch (error) {
      if (!isNotFound(error)) throw error;
    }
    if (!targetMatches) {
      try {
        const tempHandle = await tempRoot.getFileHandle(pending.tempName);
        const tempBytes = new Uint8Array(await (await tempHandle.getFile()).arrayBuffer());
        if (await digest(tempBytes) === pending.revision) {
          await writePath(this.projectRoot, record.path, tempBytes);
          targetMatches = true;
        }
        await tempRoot.removeEntry(pending.tempName).catch(() => undefined);
      } catch (error) {
        if (!isNotFound(error)) throw error;
      }
    }
    if (targetMatches) {
      const clean: FileRecord = { projectId: this.project.id, path: record.path, revision: pending.revision, sizeBytes: pending.sizeBytes };
      await this.putRecord(clean);
      return clean;
    }
    const { pending: _pending, ...clean } = record;
    await this.putRecord(clean);
    return clean;
  }

  private async recoverPending(): Promise<void> {
    for (const record of await recordsForProject(this.database, this.project.id)) {
      if (record.pending) await this.recoverRecord(record);
    }
  }

  async listFiles(): Promise<FileEntry[]> {
    try {
      return await withProjectLock(this.project.id, async () => {
        await this.recoverPending();
        const records = (await recordsForProject(this.database, this.project.id)).sort((a, b) => a.path.localeCompare(b.path));
        return records.map((record) => FileEntrySchema.parse({ path: record.path, revision: record.revision, sizeBytes: record.sizeBytes }));
      });
    } catch (error) {
      return storageError(error);
    }
  }

  async readFile(path: ProjectPath): Promise<{ bytes: Uint8Array; revision: Revision }> {
    const validPath = validatePath(path);
    try {
      return await withProjectLock(this.project.id, async () => {
        await this.recoverPending();
        const record = await this.getRecord(validPath);
        if (!record) throw new WorkspaceStorageError("NOT_FOUND", `File does not exist: ${validPath}`);
        const bytes = await readPath(this.projectRoot, validPath);
        if (await digest(bytes) !== record.revision) throw new WorkspaceStorageError("STORAGE_CORRUPT", `Stored bytes do not match their revision: ${validPath}`);
        return { bytes, revision: record.revision };
      });
    } catch (error) {
      return storageError(error);
    }
  }

  async writeFile(path: ProjectPath, bytes: Uint8Array, expectedRevision: Revision | null): Promise<{ revision: Revision }> {
    const validPath = validatePath(path);
    if (bytes.byteLength > MAX_FILE_BYTES) throw new WorkspaceStorageError("FILE_TOO_LARGE", "A workspace file cannot exceed 16 MiB.");
    try {
      return await withProjectLock(this.project.id, async () => {
        await this.recoverPending();
        const current = await this.getRecord(validPath);
        if ((current?.revision ?? null) !== expectedRevision) throw new WorkspaceStorageError("REVISION_CONFLICT", `File changed in another tab: ${validPath}`);
        const copied = bytes.slice();
        const revision = await digest(copied);
        if (current?.revision === revision) return { revision };
        const currentTotal = (await recordsForProject(this.database, this.project.id)).reduce((sum, item) => sum + item.sizeBytes, 0) - (current?.sizeBytes ?? 0);
        if (currentTotal + copied.byteLength > MAX_PROJECT_BYTES) throw new WorkspaceStorageError("PROJECT_TOO_LARGE", "Project source files exceed 64 MiB.");
        const tempName = crypto.randomUUID();
        const tempRoot = await this.tempDirectory(true);
        const tempHandle = await tempRoot.getFileHandle(tempName, { create: true });
        const tempWritable = await tempHandle.createWritable({ keepExistingData: false });
        await tempWritable.write(Uint8Array.from(copied).buffer);
        await tempWritable.close();
        const pending: FileRecord = {
          projectId: this.project.id,
          path: validPath,
          revision: current?.revision ?? revision,
          sizeBytes: current?.sizeBytes ?? 0,
          pending: { operation: "write", revision, sizeBytes: copied.byteLength, tempName },
        };
        await this.putRecord(pending);
        try {
          await writePath(this.projectRoot, validPath, copied);
          await tempRoot.removeEntry(tempName).catch(() => undefined);
          await this.putRecord({ projectId: this.project.id, path: validPath, revision, sizeBytes: copied.byteLength });
        } catch (error) {
          await this.recoverRecord(pending).catch(() => undefined);
          throw error;
        }
        await this.notify({ path: validPath, revision, operation: "write" });
        return { revision };
      });
    } catch (error) {
      return storageError(error);
    }
  }

  async deleteFile(path: ProjectPath, expectedRevision: Revision): Promise<void> {
    const validPath = validatePath(path);
    try {
      await withProjectLock(this.project.id, async () => {
        await this.recoverPending();
        const current = await this.getRecord(validPath);
        if (!current) throw new WorkspaceStorageError("NOT_FOUND", `File does not exist: ${validPath}`);
        if (current.revision !== expectedRevision) throw new WorkspaceStorageError("REVISION_CONFLICT", `File changed in another tab: ${validPath}`);
        const pending = { ...current, pending: { operation: "delete" as const } };
        await this.putRecord(pending);
        try {
          const { parent, name } = await parentFor(this.projectRoot, validPath, false);
          await parent.removeEntry(name);
          await dbDelete(this.database, "files", this.fileKey(validPath));
        } catch (error) {
          await this.recoverRecord(pending).catch(() => undefined);
          throw error;
        }
        await this.notify({ path: validPath, revision: null, operation: "delete" });
      });
    } catch (error) {
      storageError(error);
    }
  }

  async snapshot(): Promise<{ snapshotId: SnapshotId; files: SnapshotFile[] }> {
    try {
      return await withProjectLock(this.project.id, async () => {
        await this.recoverPending();
        const records = (await recordsForProject(this.database, this.project.id)).sort((a, b) => a.path.localeCompare(b.path));
        const files: SnapshotFile[] = [];
        let total = 0;
        for (const record of records) {
          const bytes = await readPath(this.projectRoot, record.path);
          if (await digest(bytes) !== record.revision) throw new WorkspaceStorageError("STORAGE_CORRUPT", `Stored bytes do not match their revision: ${record.path}`);
          total += bytes.byteLength;
          if (total > MAX_PROJECT_BYTES) throw new WorkspaceStorageError("PROJECT_TOO_LARGE", "Project source files exceed 64 MiB.");
          files.push({ path: record.path, revision: record.revision, bytes });
        }
        const serialized = new Uint8Array(files.reduce((sum, file) => sum + file.path.length * 4 + file.bytes.byteLength + 1, 0));
        let offset = 0;
        const encoder = new TextEncoder();
        for (const file of files) {
          const pathBytes = encoder.encode(file.path);
          serialized.set(pathBytes, offset);
          offset += pathBytes.byteLength;
          serialized[offset++] = 0;
          serialized.set(file.bytes, offset);
          offset += file.bytes.byteLength;
        }
        return { snapshotId: await digest(serialized), files };
      });
    } catch (error) {
      return storageError(error);
    }
  }

  subscribeChanges(callback: (change: FileChange) => void): () => void {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  dispose(): void {
    this.channel?.close();
    this.listeners.clear();
  }
}

class BrowserWorkspaceManager implements PersistentWorkspaceManager {
  private readonly projectsRoot: FileSystemDirectoryHandle;
  private readonly storageRoot: FileSystemDirectoryHandle;
  private readonly adapters = new Map<ProjectId, OpfsWorkspace>();

  private constructor(
    private readonly database: IDBDatabase,
    storageRoot: FileSystemDirectoryHandle,
    projectsRoot: FileSystemDirectoryHandle,
    private readonly projects: ProjectRecord[],
    private selected: ProjectRecord,
    readonly storageStatus: BrowserStorageStatus,
  ) {
    this.projectsRoot = projectsRoot;
    this.storageRoot = storageRoot;
  }

  get activeProject(): WorkspaceProject {
    return this.selected;
  }

  static async create(seedFiles: Array<{ path: ProjectPath; content: string }>): Promise<BrowserWorkspaceManager> {
    if (!navigator.storage?.getDirectory) throw new WorkspaceStorageError("OPFS_UNAVAILABLE", "Origin Private File System is not available in this browser.");
    const database = await openDatabase();
    const storageRoot = await navigator.storage.getDirectory();
    const root = await directoryFor(storageRoot, [STORAGE_ROOT], true);
    const projectsRoot = await root.getDirectoryHandle(PROJECTS_DIRECTORY, { create: true });
    const existing = (await dbGetAll<ProjectRecord>(database, "projects")).sort((a, b) => a.createdAt - b.createdAt);
    let projects = existing;
    if (projects.length === 0) {
      const initial: ProjectRecord = { id: DEFAULT_PROJECT_ID, name: DEFAULT_PROJECT_NAME, authority: "browser-only", createdAt: Date.now() };
      await dbPut(database, "projects", initial);
      projects = [initial];
      const projectRoot = await projectsRoot.getDirectoryHandle(initial.id, { create: true });
      const workspace = new OpfsWorkspace(initial, database, projectRoot, root);
      for (const file of seedFiles) await workspace.writeFile(file.path, new TextEncoder().encode(file.content), null);
      workspace.dispose();
    }
    const setting = await dbGet<SettingsRecord>(database, "settings", ACTIVE_PROJECT_KEY);
    const activeId = typeof setting?.value === "string" ? setting.value : undefined;
    let selected = projects.find((project) => project.id === activeId) ?? projects[0];
    if (!selected) throw new WorkspaceStorageError("NO_PROJECT", "No browser workspace project is available.");
    if (activeId !== selected.id) await dbPut(database, "settings", { key: ACTIVE_PROJECT_KEY, value: selected.id });

    let persistent = false;
    const persistentStorageSupported = typeof navigator.storage.persist === "function";
    if (persistentStorageSupported) {
      try {
        persistent = await navigator.storage.persist();
      } catch {
        // Ordinary origin storage remains usable when the request is rejected.
      }
    }
    let usageBytes: number | undefined;
    let quotaBytes: number | undefined;
    try {
      const estimate = await navigator.storage.estimate();
      usageBytes = estimate.usage;
      quotaBytes = estimate.quota;
    } catch {
      // Storage estimates are advisory and not available in every browser.
    }
    const manager = new BrowserWorkspaceManager(database, root, projectsRoot, projects, selected, {
      persistent,
      persistentStorageSupported,
      usageBytes,
      quotaBytes,
    });
    return manager;
  }

  getWorkspace(projectId = this.selected.id): OpfsWorkspace {
    const validId = validateProjectId(projectId);
    const record = this.projects.find((project) => project.id === validId);
    if (!record) throw new WorkspaceStorageError("PROJECT_NOT_FOUND", `Unknown project: ${validId}`);
    let adapter = this.adapters.get(validId);
    if (!adapter) {
      // Handle resolution is asynchronous, so callers should use readyWorkspace first.
      throw new WorkspaceStorageError("WORKSPACE_NOT_READY", `Project storage is not open yet: ${validId}`);
    }
    return adapter;
  }

  private async readyWorkspace(projectId: ProjectId): Promise<OpfsWorkspace> {
    const cached = this.adapters.get(projectId);
    if (cached) return cached;
    const record = this.projects.find((project) => project.id === projectId);
    if (!record) throw new WorkspaceStorageError("PROJECT_NOT_FOUND", `Unknown project: ${projectId}`);
    const projectRoot = await this.projectsRoot.getDirectoryHandle(projectId, { create: true });
    const adapter = new OpfsWorkspace(record, this.database, projectRoot, this.storageRoot);
    this.adapters.set(projectId, adapter);
    return adapter;
  }

  async listProjects(): Promise<WorkspaceProject[]> {
    return this.projects.map((project) => ({ ...project }));
  }

  async selectProject(projectId: ProjectId): Promise<WorkspaceProject> {
    const validId = validateProjectId(projectId);
    const project = this.projects.find((item) => item.id === validId);
    if (!project) throw new WorkspaceStorageError("PROJECT_NOT_FOUND", `Unknown project: ${validId}`);
    await this.readyWorkspace(project.id);
    await dbPut(this.database, "settings", { key: ACTIVE_PROJECT_KEY, value: project.id });
    this.selected = project;
    return { ...project };
  }

  async importProject(archive: Blob, name?: string): Promise<WorkspaceProject> {
    if (archive.size > MAX_PROJECT_BYTES) throw new WorkspaceStorageError("ARCHIVE_TOO_LARGE", "Project archive exceeds 64 MiB.");
    const files = await importProjectZip(new Uint8Array(await archive.arrayBuffer()));
    const project: ProjectRecord = {
      id: validateProjectId(`project-${crypto.randomUUID()}`),
      name: validateProjectName(name ?? "Imported project"),
      authority: "browser-only",
      createdAt: Date.now(),
    };
    await dbPut(this.database, "projects", project);
    this.projects.push(project);
    await this.readyWorkspace(project.id);
    try {
      const workspace = await this.readyWorkspace(project.id);
      for (const file of files) await workspace.writeFile(file.path as ProjectPath, file.bytes, null);
    } catch (error) {
      await this.deleteProjectContents(project.id).catch(() => undefined);
      this.projects.splice(this.projects.findIndex((item) => item.id === project.id), 1);
      throw error;
    }
    return { ...project };
  }

  async exportProject(projectId = this.selected.id): Promise<Uint8Array> {
    const workspace = await this.readyWorkspace(validateProjectId(projectId));
    const snapshot = await workspace.snapshot();
    const files: ArchiveFile[] = snapshot.files.map(({ path, bytes }) => ({ path, bytes }));
    return exportProjectZip(files);
  }

  async renameFile(projectId: ProjectId, oldPath: ProjectPath, newPath: ProjectPath, expectedRevision: Revision): Promise<Revision> {
    const from = validatePath(oldPath);
    const to = validatePath(newPath);
    if (from === to) return expectedRevision;
    const workspace = await this.readyWorkspace(validateProjectId(projectId));
    const oldFile = await workspace.readFile(from);
    if (oldFile.revision !== expectedRevision) throw new WorkspaceStorageError("REVISION_CONFLICT", `File changed in another tab: ${from}`);
    const created = await workspace.writeFile(to, oldFile.bytes, null);
    try {
      await workspace.deleteFile(from, expectedRevision);
    } catch (error) {
      await workspace.deleteFile(to, created.revision).catch(() => undefined);
      throw error;
    }
    return created.revision;
  }

  async loadEditorState(projectId: ProjectId): Promise<EditorSessionState | null> {
    const state = await dbGet<StoredEditorState>(this.database, "editorState", validateProjectId(projectId));
    if (!state) return null;
    return { openTabs: state.openTabs.filter((path) => ProjectPathSchema.safeParse(path).success), activePath: state.activePath, topModule: state.topModule };
  }

  async saveEditorState(projectId: ProjectId, value: EditorSessionState): Promise<void> {
    const validId = validateProjectId(projectId);
    await dbPut(this.database, "editorState", {
      projectId: validId,
      openTabs: value.openTabs.map(validatePath),
      activePath: value.activePath ? validatePath(value.activePath) : null,
      topModule: value.topModule.slice(0, 256),
    } satisfies StoredEditorState);
  }

  private async deleteProjectContents(projectId: ProjectId): Promise<void> {
    const records = await recordsForProject(this.database, projectId);
    for (const record of records) await dbDelete(this.database, "files", [projectId, record.path]);
    await dbDelete(this.database, "projects", projectId);
    this.adapters.get(projectId)?.dispose();
    this.adapters.delete(projectId);
    this.projectsRoot.removeEntry(projectId, { recursive: true }).catch(() => undefined);
  }
}

export async function openPersistentWorkspace(seedFiles: Array<{ path: ProjectPath; content: string }>): Promise<PersistentWorkspaceManager> {
  try {
    const manager = await BrowserWorkspaceManager.create(seedFiles);
    await manager.selectProject(manager.activeProject.id);
    return manager;
  } catch (error) {
    storageError(error);
  }
}
