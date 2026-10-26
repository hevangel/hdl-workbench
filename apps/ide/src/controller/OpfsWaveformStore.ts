import { WaveformArtifactSchema, type WaveformArtifact } from "@xezim-ide/protocol";

const STORE_DIRECTORY = "hdl-workbench-waveforms";
const ARTIFACTS_DIRECTORY = "artifacts";
const MAX_RETAINED_ARTIFACTS = 8;
const MAX_ARTIFACT_BYTES = 64 * 1024 * 1024;
const MAX_RETAINED_BYTES = 128 * 1024 * 1024;

interface StoredArtifact {
  artifact: WaveformArtifact;
  createdAt: number;
}

export class WaveformStorageError extends Error {
  readonly code = "QUOTA_EXCEEDED";

  constructor(message: string) {
    super(message);
    this.name = "WaveformStorageError";
  }
}

function safeArtifactId(value: string): string {
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(value)) throw new Error("Invalid waveform artifact ID.");
  return value;
}

async function writeFile(directory: FileSystemDirectoryHandle, name: string, contents: Blob | string): Promise<void> {
  const handle = await directory.getFileHandle(name, { create: true });
  const writable = await handle.createWritable({ keepExistingData: false });
  try {
    await writable.write(contents);
    await writable.close();
  } catch (error) {
    await writable.abort().catch(() => undefined);
    throw error;
  }
}

/** OPFS store for bounded VCD and FST results. Metadata is published after the waveform file. */
export class OpfsWaveformStore {
  private directoryPromise: Promise<FileSystemDirectoryHandle> | null = null;
  private readonly objectUrls = new Map<string, string>();

  async put(artifact: WaveformArtifact, contents: string | Uint8Array): Promise<void> {
    const parsed = WaveformArtifactSchema.parse(artifact);
    safeArtifactId(parsed.id);
    if (!parsed.complete) throw new Error("Only complete waveform artifacts can be stored.");
    const bytes = typeof contents === "string" ? new TextEncoder().encode(contents) : contents;
    const label = parsed.format.toUpperCase();
    if (bytes.byteLength !== parsed.sizeBytes) throw new Error(`${label} byte count does not match its artifact metadata.`);
    if (bytes.byteLength > MAX_ARTIFACT_BYTES) throw new Error(`${label} artifact exceeds the ${MAX_ARTIFACT_BYTES} byte storage limit.`);
    const directory = await this.directory();
    await this.prune(directory, parsed.id, bytes.byteLength);
    try {
      await writeFile(directory, `${parsed.id}.${parsed.format}`, new Blob([bytes as BlobPart], { type: parsed.format === "vcd" ? "text/vnd.vcd" : "application/octet-stream" }));
      const stored: StoredArtifact = { artifact: parsed, createdAt: Date.now() };
      await writeFile(directory, `${parsed.id}.json`, JSON.stringify(stored));
      await this.prune(directory, parsed.id);
    } catch (error) {
      await directory.removeEntry(`${parsed.id}.json`).catch(() => undefined);
      await directory.removeEntry(`${parsed.id}.${parsed.format}`).catch(() => undefined);
      if (typeof error === "object" && error !== null && "name" in error && error.name === "QuotaExceededError") {
        throw new WaveformStorageError("Browser storage is full. Export source files or remove old projects before saving this waveform.");
      }
      throw error;
    }
  }

  async get(id: string): Promise<WaveformArtifact> {
    const safeId = safeArtifactId(id);
    const directory = await this.directory();
    const metadataFile = await directory.getFileHandle(`${safeId}.json`).then((handle) => handle.getFile());
    const stored = JSON.parse(await metadataFile.text()) as StoredArtifact;
    const artifact = WaveformArtifactSchema.parse(stored.artifact);
    if (artifact.id !== safeId) throw new Error("Waveform metadata does not match the requested artifact.");
    const waveform = await directory.getFileHandle(`${safeId}.${artifact.format}`).then((handle) => handle.getFile());
    if (waveform.size !== artifact.sizeBytes) throw new Error(`Stored ${artifact.format.toUpperCase()} size does not match its metadata.`);
    return artifact;
  }

  async objectUrl(artifact: WaveformArtifact): Promise<string> {
    const parsed = WaveformArtifactSchema.parse(artifact);
    const stored = await this.get(parsed.id);
    if (JSON.stringify(stored) !== JSON.stringify(parsed)) throw new Error("Requested waveform metadata differs from the stored artifact.");
    const existing = this.objectUrls.get(parsed.id);
    if (existing) return existing;
    const directory = await this.directory();
    const file = await directory.getFileHandle(`${parsed.id}.${parsed.format}`).then((handle) => handle.getFile());
    const url = URL.createObjectURL(file);
    this.objectUrls.set(parsed.id, url);
    return url;
  }

  dispose(): void {
    for (const url of this.objectUrls.values()) URL.revokeObjectURL(url);
    this.objectUrls.clear();
  }

  private async directory(): Promise<FileSystemDirectoryHandle> {
    if (!navigator.storage?.getDirectory) throw new Error("Origin Private File System is unavailable; waveform artifacts cannot be persisted.");
    this.directoryPromise ??= navigator.storage.getDirectory()
      .then((root) => root.getDirectoryHandle(STORE_DIRECTORY, { create: true }))
      .then((root) => root.getDirectoryHandle(ARTIFACTS_DIRECTORY, { create: true }));
    return this.directoryPromise;
  }

  private async prune(directory: FileSystemDirectoryHandle, newestId: string, reservedBytes = 0): Promise<void> {
    const records: Array<{ id: string; createdAt: number; sizeBytes: number }> = [];
    const validIds = new Set<string>();
    for await (const [name, handle] of directory.entries()) {
      if (!name.endsWith(".json") || handle.kind !== "file") continue;
      try {
        const raw = await (handle as FileSystemFileHandle).getFile();
        const parsed = JSON.parse(await raw.text()) as StoredArtifact;
        const artifact = WaveformArtifactSchema.safeParse(parsed.artifact);
        if (artifact.success && Number.isFinite(parsed.createdAt)) {
          records.push({ id: artifact.data.id, createdAt: parsed.createdAt, sizeBytes: artifact.data.sizeBytes });
          validIds.add(artifact.data.id);
        }
      } catch {
        // Ignore unreadable metadata; it cannot resolve to a waveform.
      }
    }
    records.sort((a, b) => b.createdAt - a.createdAt);
    const keep = new Set([newestId]);
    let retainedCount = 1;
    let retainedBytes = reservedBytes;
    for (const record of records) {
      const { id } = record;
      if (id === newestId) {
        if (reservedBytes === 0) retainedBytes += record.sizeBytes;
        continue;
      }
      if (retainedCount < MAX_RETAINED_ARTIFACTS && retainedBytes + record.sizeBytes <= MAX_RETAINED_BYTES) {
        keep.add(id);
        retainedCount += 1;
        retainedBytes += record.sizeBytes;
      }
    }
    for (const { id } of records) {
      if (keep.has(id)) continue;
      const url = this.objectUrls.get(id);
      if (url) URL.revokeObjectURL(url);
      this.objectUrls.delete(id);
      await directory.removeEntry(`${id}.json`).catch(() => undefined);
      await directory.removeEntry(`${id}.vcd`).catch(() => undefined);
      await directory.removeEntry(`${id}.fst`).catch(() => undefined);
    }
    for await (const [name] of directory.entries()) {
      if (name.endsWith(".vcd") || name.endsWith(".fst")) {
        const id = name.slice(0, -4);
        if (!validIds.has(id) && id !== newestId) await directory.removeEntry(name).catch(() => undefined);
      }
    }
  }
}
