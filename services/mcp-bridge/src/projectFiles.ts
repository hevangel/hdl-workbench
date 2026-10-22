import { createHash, randomBytes } from "node:crypto";
import { promises as fs } from "node:fs";
import path from "node:path";
import {
  FileEntrySchema,
  ProjectFileSchema,
  ProjectPathSchema,
  RevisionSchema,
  type FileEntry,
  type ProjectPath,
  type Revision,
  type SnapshotFile,
} from "@xezim-ide/protocol";
import { BridgeError } from "./errors.js";

const MAX_FILE_BYTES = 16 * 1024 * 1024;

function revisionOf(bytes: Uint8Array): Revision {
  return RevisionSchema.parse(`sha256:${createHash("sha256").update(bytes).digest("hex")}`);
}

function isWithin(root: string, target: string): boolean {
  const relative = path.relative(root, target);
  return relative === "" || (!path.isAbsolute(relative) && relative !== ".." && !relative.startsWith(`..${path.sep}`));
}

function validateProjectPath(input: string): ProjectPath {
  const parsed = ProjectPathSchema.safeParse(input);
  if (!parsed.success) throw new BridgeError("INVALID_PATH", "Path must be a relative project path without traversal");
  if (process.platform === "win32") {
    const invalidWindowsName = parsed.data.split("/").some((segment) =>
      /[<>:"|?*]/.test(segment)
      || /[. ]$/.test(segment)
      || /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\..*)?$/i.test(segment),
    );
    if (invalidWindowsName) throw new BridgeError("INVALID_PATH", "Path contains a Windows-reserved file name or stream syntax");
  }
  return parsed.data;
}

export class ProjectFiles {
  readonly root: string;

  private constructor(root: string) {
    this.root = root;
  }

  static async open(selectedRoot: string): Promise<ProjectFiles> {
    if (!selectedRoot) throw new BridgeError("INVALID_PATH", "Select a project root with --project-root or XEZIM_PROJECT_ROOT");
    let resolved: string;
    try {
      resolved = await fs.realpath(path.resolve(selectedRoot));
    } catch {
      throw new BridgeError("NOT_FOUND", "Selected project root does not exist");
    }
    const stat = await fs.stat(resolved);
    if (!stat.isDirectory()) throw new BridgeError("INVALID_PATH", "Selected project root must be a directory");
    return new ProjectFiles(resolved);
  }

  async list(): Promise<FileEntry[]> {
    const files: FileEntry[] = [];
    const visit = async (absoluteDir: string, relativeDir: string): Promise<void> => {
      const entries = await fs.readdir(absoluteDir, { withFileTypes: true });
      entries.sort((a, b) => a.name.localeCompare(b.name));
      for (const entry of entries) {
        const rel = relativeDir ? `${relativeDir}/${entry.name}` : entry.name;
        if (entry.isDirectory()) {
          await visit(path.join(absoluteDir, entry.name), rel);
          continue;
        }
        // Symlinks are deliberately excluded from discovery. Explicit reads and
        // writes still validate their resolved target before touching it.
        if (!entry.isFile()) continue;
        const projectPath = validateProjectPath(rel);
        const absolute = await this.resolveExisting(projectPath);
        const stat = await fs.stat(absolute);
        if (!stat.isFile()) continue;
        if (stat.size > MAX_FILE_BYTES) throw new BridgeError("RESOURCE_LIMIT", `File exceeds ${MAX_FILE_BYTES} bytes: ${rel}`);
        const bytes = await fs.readFile(absolute);
        files.push(FileEntrySchema.parse({ path: projectPath, revision: revisionOf(bytes), sizeBytes: bytes.byteLength }));
      }
    };
    await visit(this.root, "");
    return files;
  }

  async read(projectPathInput: string): Promise<{ path: ProjectPath; revision: Revision; bytes: Uint8Array }> {
    const projectPath = validateProjectPath(projectPathInput);
    const absolute = await this.resolveExisting(projectPath);
    const stat = await fs.stat(absolute);
    if (!stat.isFile()) throw new BridgeError("INVALID_PATH", "Requested project path is not a regular file");
    if (stat.size > MAX_FILE_BYTES) throw new BridgeError("RESOURCE_LIMIT", `File exceeds ${MAX_FILE_BYTES} bytes`);
    const bytes = await fs.readFile(absolute);
    return { path: projectPath, revision: revisionOf(bytes), bytes };
  }

  async write(projectPathInput: string, contentBase64: string, expectedRevision: Revision | null): Promise<{ revision: Revision }> {
    const projectPath = validateProjectPath(projectPathInput);
    const parsedContent = ProjectFileSchema.shape.contentBase64.safeParse(contentBase64);
    if (!parsedContent.success) throw new BridgeError("INVALID_REQUEST", "content_base64 is not valid base64 or exceeds the file size limit");
    const bytes = Buffer.from(parsedContent.data, "base64");
    if (bytes.byteLength > MAX_FILE_BYTES) throw new BridgeError("RESOURCE_LIMIT", `File exceeds ${MAX_FILE_BYTES} bytes`);
    const absolute = this.lexicalTarget(projectPath);
    const parent = path.dirname(absolute);
    const safeParent = await this.resolveExistingAbsolute(parent);
    if (!isWithin(this.root, safeParent)) throw new BridgeError("INVALID_PATH", "Parent directory resolves outside the selected project root");

    let currentRevision: Revision | null = null;
    try {
      await fs.lstat(absolute);
      const target = await this.resolveExisting(projectPath);
      const stat = await fs.stat(target);
      if (!stat.isFile()) throw new BridgeError("INVALID_PATH", "Requested project path is not a regular file");
      if (stat.size > MAX_FILE_BYTES) throw new BridgeError("RESOURCE_LIMIT", `File exceeds ${MAX_FILE_BYTES} bytes`);
      currentRevision = revisionOf(await fs.readFile(target));
    } catch (error) {
      if (error instanceof BridgeError) throw error;
      if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
    }
    if (currentRevision !== expectedRevision) {
      throw new BridgeError("REVISION_CONFLICT", "Project file revision changed", true);
    }

    const tempName = `.xezim-write-${randomBytes(12).toString("hex")}.tmp`;
    const temporary = path.join(safeParent, tempName);
    try {
      await fs.writeFile(temporary, bytes, { flag: "wx" });
      // Revalidate immediately before replacement so a changed parent symlink
      // cannot redirect the final operation outside the selected root.
      const latestParent = await this.resolveExistingAbsolute(parent);
      if (latestParent !== safeParent || !isWithin(this.root, latestParent)) {
        throw new BridgeError("INVALID_PATH", "Project directory changed while writing");
      }
      const latestTarget = await this.resolveExistingAbsolute(absolute).catch((error: unknown) => {
        if (error instanceof BridgeError && error.code === "NOT_FOUND") return null;
        throw error;
      });
      if (latestTarget && !isWithin(this.root, latestTarget)) {
        throw new BridgeError("INVALID_PATH", "Target resolves outside the selected project root");
      }
      const latestRevision = latestTarget ? revisionOf(await fs.readFile(latestTarget)) : null;
      if (latestRevision !== expectedRevision) throw new BridgeError("REVISION_CONFLICT", "Project file revision changed while writing", true);
      await fs.rename(temporary, absolute);
    } finally {
      await fs.rm(temporary, { force: true }).catch(() => undefined);
    }
    return { revision: revisionOf(bytes) };
  }

  async snapshot(): Promise<{ snapshotId: string; files: SnapshotFile[] }> {
    const listed = await this.list();
    if (listed.length > 10_000) throw new BridgeError("RESOURCE_LIMIT", "Project contains more than 10000 files");
    const files: SnapshotFile[] = [];
    for (const file of listed) {
      const read = await this.read(file.path);
      files.push({ path: read.path, revision: read.revision, bytes: read.bytes });
    }
    const provenance = createHash("sha256");
    for (const file of files) provenance.update(`${file.path}\0${file.revision}\0`);
    return { snapshotId: `snap-${provenance.digest("hex").slice(0, 32)}`, files };
  }

  private lexicalTarget(projectPath: ProjectPath): string {
    const target = path.resolve(this.root, ...projectPath.split("/"));
    if (!isWithin(this.root, target) || target === this.root) throw new BridgeError("INVALID_PATH", "Path resolves outside the selected project root");
    return target;
  }

  private async resolveExisting(projectPath: ProjectPath): Promise<string> {
    return this.resolveExistingAbsolute(this.lexicalTarget(projectPath));
  }

  private async resolveExistingAbsolute(absolute: string): Promise<string> {
    let real: string;
    try {
      real = await fs.realpath(absolute);
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT") throw new BridgeError("NOT_FOUND", "Project file was not found");
      throw error;
    }
    if (!isWithin(this.root, real)) throw new BridgeError("INVALID_PATH", "Path resolves outside the selected project root");
    return real;
  }
}
