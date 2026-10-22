import { WaveformArtifactSchema, type WaveformArtifact } from "@xezim-ide/protocol";

const STORE_DIRECTORY = "xezim-studio-waveforms";
const ARTIFACTS_DIRECTORY = "artifacts";
const MAX_RETAINED_ARTIFACTS = 8;

interface StoredArtifact {
  artifact: WaveformArtifact;
  createdAt: number;
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

/** OPFS store for bounded VCD results. Metadata is published after the VCD file. */
export class OpfsWaveformStore {
  private directoryPromise: Promise<FileSystemDirectoryHandle> | null = null;
  private readonly objectUrls = new Map<string, string>();

  async put(artifact: WaveformArtifact, vcd: string): Promise<void> {
    const parsed = WaveformArtifactSchema.parse(artifact);
    safeArtifactId(parsed.id);
    if (!parsed.complete || parsed.format !== "vcd") throw new Error("Only complete VCD artifacts can be stored.");
    const bytes = new TextEncoder().encode(vcd);
    if (bytes.byteLength !== parsed.sizeBytes) throw new Error("VCD byte count does not match its artifact metadata.");
    const directory = await this.directory();
    await writeFile(directory, `${parsed.id}.vcd`, new Blob([bytes], { type: "text/vnd.vcd" }));
    const stored: StoredArtifact = { artifact: parsed, createdAt: Date.now() };
    await writeFile(directory, `${parsed.id}.json`, JSON.stringify(stored));
    await this.prune(directory, parsed.id);
  }

  async get(id: string): Promise<WaveformArtifact> {
    const safeId = safeArtifactId(id);
    const directory = await this.directory();
    const metadataFile = await directory.getFileHandle(`${safeId}.json`).then((handle) => handle.getFile());
    const stored = JSON.parse(await metadataFile.text()) as StoredArtifact;
    const artifact = WaveformArtifactSchema.parse(stored.artifact);
    if (artifact.id !== safeId) throw new Error("Waveform metadata does not match the requested artifact.");
    const vcd = await directory.getFileHandle(`${safeId}.vcd`).then((handle) => handle.getFile());
    if (vcd.size !== artifact.sizeBytes) throw new Error("Stored VCD size does not match its metadata.");
    return artifact;
  }

  async objectUrl(artifact: WaveformArtifact): Promise<string> {
    const parsed = WaveformArtifactSchema.parse(artifact);
    const stored = await this.get(parsed.id);
    if (JSON.stringify(stored) !== JSON.stringify(parsed)) throw new Error("Requested waveform metadata differs from the stored artifact.");
    const existing = this.objectUrls.get(parsed.id);
    if (existing) return existing;
    const directory = await this.directory();
    const file = await directory.getFileHandle(`${parsed.id}.vcd`).then((handle) => handle.getFile());
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

  private async prune(directory: FileSystemDirectoryHandle, newestId: string): Promise<void> {
    const records: Array<{ id: string; createdAt: number }> = [];
    for await (const [name, handle] of directory.entries()) {
      if (!name.endsWith(".json") || handle.kind !== "file") continue;
      try {
        const raw = await (handle as FileSystemFileHandle).getFile();
        const parsed = JSON.parse(await raw.text()) as StoredArtifact;
        const artifact = WaveformArtifactSchema.safeParse(parsed.artifact);
        if (artifact.success) records.push({ id: artifact.data.id, createdAt: parsed.createdAt });
      } catch {
        // Ignore unreadable metadata; it cannot resolve to a waveform.
      }
    }
    records.sort((a, b) => b.createdAt - a.createdAt);
    const keep = new Set(records.slice(0, MAX_RETAINED_ARTIFACTS).map(({ id }) => id));
    keep.add(newestId);
    for (const { id } of records) {
      if (keep.has(id)) continue;
      this.objectUrls.get(id) && URL.revokeObjectURL(this.objectUrls.get(id)!);
      this.objectUrls.delete(id);
      await directory.removeEntry(`${id}.json`).catch(() => undefined);
      await directory.removeEntry(`${id}.vcd`).catch(() => undefined);
    }
  }
}
