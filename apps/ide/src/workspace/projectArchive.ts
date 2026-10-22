import { ProjectPathSchema } from "@xezim-ide/protocol";

const MAX_FILES = 4096;
const MAX_FILE_BYTES = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES = 64 * 1024 * 1024;
const UTF8_FLAG = 0x0800;
const EOCD_SIGNATURE = 0x06054b50;
const CENTRAL_SIGNATURE = 0x02014b50;
const LOCAL_SIGNATURE = 0x04034b50;

export interface ArchiveFile {
  path: string;
  bytes: Uint8Array;
}

export class ProjectArchiveError extends Error {
  constructor(readonly code: string, message: string) {
    super(message);
    this.name = "ProjectArchiveError";
  }
}

const crcTable = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < table.length; n += 1) {
    let value = n;
    for (let bit = 0; bit < 8; bit += 1) value = (value & 1) === 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    table[n] = value >>> 0;
  }
  return table;
})();

function crc32(bytes: Uint8Array): number {
  let value = 0xffffffff;
  for (const byte of bytes) value = (value >>> 8) ^ (crcTable[(value ^ byte) & 0xff] ?? 0);
  return (value ^ 0xffffffff) >>> 0;
}

function assertPath(path: string): void {
  if (!ProjectPathSchema.safeParse(path).success) {
    throw new ProjectArchiveError("INVALID_ARCHIVE_PATH", `Archive contains an invalid project path: ${path}`);
  }
}

function writeU16(view: DataView, offset: number, value: number): void {
  view.setUint16(offset, value, true);
}

function writeU32(view: DataView, offset: number, value: number): void {
  view.setUint32(offset, value, true);
}

function concat(chunks: Uint8Array[]): Uint8Array {
  const total = chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0);
  const result = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return result;
}

/** Create a standards-compliant, store-only ZIP. Its size is bounded by the workspace limit. */
export function exportProjectZip(files: ArchiveFile[]): Uint8Array {
  if (files.length === 0 || files.length > MAX_FILES) {
    throw new ProjectArchiveError("INVALID_ARCHIVE_SIZE", "A project archive must contain 1 to 4096 files.");
  }
  const encoder = new TextEncoder();
  const names = new Set<string>();
  const localChunks: Uint8Array[] = [];
  const centralChunks: Uint8Array[] = [];
  let localOffset = 0;
  let totalBytes = 0;

  for (const file of [...files].sort((a, b) => a.path.localeCompare(b.path))) {
    assertPath(file.path);
    if (names.has(file.path)) throw new ProjectArchiveError("DUPLICATE_ARCHIVE_PATH", `Duplicate archive path: ${file.path}`);
    names.add(file.path);
    if (file.bytes.byteLength > MAX_FILE_BYTES) throw new ProjectArchiveError("FILE_TOO_LARGE", `Archive file exceeds 16 MiB: ${file.path}`);
    totalBytes += file.bytes.byteLength;
    if (totalBytes > MAX_TOTAL_BYTES) throw new ProjectArchiveError("ARCHIVE_TOO_LARGE", "Project archive exceeds 64 MiB.");

    const name = encoder.encode(file.path);
    if (name.byteLength > 0xffff || file.bytes.byteLength > 0xffffffff || localOffset > 0xffffffff) {
      throw new ProjectArchiveError("ARCHIVE_TOO_LARGE", "Project archive exceeds ZIP32 limits.");
    }
    const checksum = crc32(file.bytes);
    const local = new Uint8Array(30 + name.byteLength);
    const localView = new DataView(local.buffer);
    writeU32(localView, 0, LOCAL_SIGNATURE);
    writeU16(localView, 4, 20);
    writeU16(localView, 6, UTF8_FLAG);
    writeU16(localView, 8, 0);
    writeU16(localView, 10, 0);
    writeU16(localView, 12, 0);
    writeU32(localView, 14, checksum);
    writeU32(localView, 18, file.bytes.byteLength);
    writeU32(localView, 22, file.bytes.byteLength);
    writeU16(localView, 26, name.byteLength);
    writeU16(localView, 28, 0);
    local.set(name, 30);
    localChunks.push(local, file.bytes);

    const central = new Uint8Array(46 + name.byteLength);
    const centralView = new DataView(central.buffer);
    writeU32(centralView, 0, CENTRAL_SIGNATURE);
    writeU16(centralView, 4, 0x031e);
    writeU16(centralView, 6, 20);
    writeU16(centralView, 8, UTF8_FLAG);
    writeU16(centralView, 10, 0);
    writeU16(centralView, 12, 0);
    writeU16(centralView, 14, 0);
    writeU32(centralView, 16, checksum);
    writeU32(centralView, 20, file.bytes.byteLength);
    writeU32(centralView, 24, file.bytes.byteLength);
    writeU16(centralView, 28, name.byteLength);
    writeU16(centralView, 30, 0);
    writeU16(centralView, 32, 0);
    writeU16(centralView, 34, 0);
    writeU16(centralView, 36, 0);
    writeU32(centralView, 38, 0);
    writeU32(centralView, 42, localOffset);
    central.set(name, 46);
    centralChunks.push(central);
    localOffset += local.byteLength + file.bytes.byteLength;
  }

  const centralDirectory = concat(centralChunks);
  const end = new Uint8Array(22);
  const endView = new DataView(end.buffer);
  writeU32(endView, 0, EOCD_SIGNATURE);
  writeU16(endView, 4, 0);
  writeU16(endView, 6, 0);
  writeU16(endView, 8, files.length);
  writeU16(endView, 10, files.length);
  writeU32(endView, 12, centralDirectory.byteLength);
  writeU32(endView, 16, localOffset);
  writeU16(endView, 20, 0);
  return concat([...localChunks, centralDirectory, end]);
}

function readU16(view: DataView, offset: number): number {
  if (offset < 0 || offset + 2 > view.byteLength) throw new ProjectArchiveError("CORRUPT_ARCHIVE", "ZIP field is out of bounds.");
  return view.getUint16(offset, true);
}

function readU32(view: DataView, offset: number): number {
  if (offset < 0 || offset + 4 > view.byteLength) throw new ProjectArchiveError("CORRUPT_ARCHIVE", "ZIP field is out of bounds.");
  return view.getUint32(offset, true);
}

async function inflateRaw(data: Uint8Array): Promise<Uint8Array> {
  if (typeof DecompressionStream === "undefined") {
    throw new ProjectArchiveError("ZIP_DEFLATE_UNSUPPORTED", "This browser cannot import compressed ZIP entries.");
  }
  try {
    const format = "deflate-raw" as CompressionFormat;
    const stream = new Blob([Uint8Array.from(data).buffer]).stream().pipeThrough(new DecompressionStream(format));
    return new Uint8Array(await new Response(stream).arrayBuffer());
  } catch {
    throw new ProjectArchiveError("CORRUPT_ARCHIVE", "A compressed ZIP entry could not be decoded.");
  }
}

/** Read stored and DEFLATE ZIP entries with path, checksum, count, and expansion limits. */
export async function importProjectZip(bytes: Uint8Array): Promise<ArchiveFile[]> {
  if (bytes.byteLength < 22 || bytes.byteLength > MAX_TOTAL_BYTES) {
    throw new ProjectArchiveError("INVALID_ARCHIVE_SIZE", "ZIP archive must be between 22 bytes and 64 MiB.");
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const minEocd = Math.max(0, bytes.byteLength - 22 - 0xffff);
  let endOffset = -1;
  for (let offset = bytes.byteLength - 22; offset >= minEocd; offset -= 1) {
    if (readU32(view, offset) === EOCD_SIGNATURE) {
      const commentLength = readU16(view, offset + 20);
      if (offset + 22 + commentLength === bytes.byteLength) {
        endOffset = offset;
        break;
      }
    }
  }
  if (endOffset < 0) throw new ProjectArchiveError("CORRUPT_ARCHIVE", "ZIP end record is missing.");
  if (readU16(view, endOffset + 4) !== 0 || readU16(view, endOffset + 6) !== 0) {
    throw new ProjectArchiveError("UNSUPPORTED_ARCHIVE", "Multi-disk ZIP archives are not supported.");
  }
  const entryCount = readU16(view, endOffset + 10);
  const centralSize = readU32(view, endOffset + 12);
  const centralOffset = readU32(view, endOffset + 16);
  if (entryCount === 0 || entryCount > MAX_FILES || centralOffset + centralSize > endOffset) {
    throw new ProjectArchiveError("INVALID_ARCHIVE_SIZE", "ZIP central directory is outside the accepted limits.");
  }

  const decoder = new TextDecoder("utf-8", { fatal: true });
  const entries: Array<{ path: string; method: number; crc: number; compressedSize: number; size: number; localOffset: number }> = [];
  const seen = new Set<string>();
  let cursor = centralOffset;
  let totalBytes = 0;
  for (let index = 0; index < entryCount; index += 1) {
    if (readU32(view, cursor) !== CENTRAL_SIGNATURE) throw new ProjectArchiveError("CORRUPT_ARCHIVE", "ZIP central directory entry is invalid.");
    const flags = readU16(view, cursor + 8);
    const method = readU16(view, cursor + 10);
    const crc = readU32(view, cursor + 16);
    const compressedSize = readU32(view, cursor + 20);
    const size = readU32(view, cursor + 24);
    const nameLength = readU16(view, cursor + 28);
    const extraLength = readU16(view, cursor + 30);
    const commentLength = readU16(view, cursor + 32);
    const localOffset = readU32(view, cursor + 42);
    const nameStart = cursor + 46;
    if (nameStart + nameLength + extraLength + commentLength > centralOffset + centralSize) {
      throw new ProjectArchiveError("CORRUPT_ARCHIVE", "ZIP entry name or metadata is truncated.");
    }
    if ((flags & 1) !== 0) throw new ProjectArchiveError("UNSUPPORTED_ARCHIVE", "Encrypted ZIPs are not supported.");
    if (method !== 0 && method !== 8) throw new ProjectArchiveError("UNSUPPORTED_ARCHIVE", `ZIP compression method ${method} is not supported.`);
    let path: string;
    try {
      path = decoder.decode(bytes.subarray(nameStart, nameStart + nameLength));
    } catch {
      throw new ProjectArchiveError("INVALID_ARCHIVE_PATH", "ZIP entry path is not valid UTF-8.");
    }
    cursor = nameStart + nameLength + extraLength + commentLength;
    if (path.endsWith("/")) continue;
    assertPath(path);
    if (seen.has(path)) throw new ProjectArchiveError("DUPLICATE_ARCHIVE_PATH", `Duplicate archive path: ${path}`);
    seen.add(path);
    if (size > MAX_FILE_BYTES || compressedSize > MAX_TOTAL_BYTES) throw new ProjectArchiveError("FILE_TOO_LARGE", `ZIP entry exceeds the file limit: ${path}`);
    totalBytes += size;
    if (totalBytes > MAX_TOTAL_BYTES) throw new ProjectArchiveError("ARCHIVE_TOO_LARGE", "Expanded project exceeds 64 MiB.");
    entries.push({ path, method, crc, compressedSize, size, localOffset });
  }
  if (entries.length === 0) throw new ProjectArchiveError("EMPTY_ARCHIVE", "ZIP archive contains no project files.");

  const files: ArchiveFile[] = [];
  for (const entry of entries) {
    if (readU32(view, entry.localOffset) !== LOCAL_SIGNATURE) throw new ProjectArchiveError("CORRUPT_ARCHIVE", `ZIP local header is invalid: ${entry.path}`);
    const localFlags = readU16(view, entry.localOffset + 6);
    const localMethod = readU16(view, entry.localOffset + 8);
    const nameLength = readU16(view, entry.localOffset + 26);
    const extraLength = readU16(view, entry.localOffset + 28);
    const nameStart = entry.localOffset + 30;
    const dataStart = nameStart + nameLength + extraLength;
    const dataEnd = dataStart + entry.compressedSize;
    // Data-descriptor archives leave CRC and sizes out of the local header;
    // use the already bounded central-directory values in that case.
    if (dataEnd > centralOffset || (localFlags & 1) !== 0 || localMethod !== entry.method) {
      throw new ProjectArchiveError("CORRUPT_ARCHIVE", `ZIP local metadata does not match: ${entry.path}`);
    }
    const localName = decoder.decode(bytes.subarray(nameStart, nameStart + nameLength));
    if (localName !== entry.path) throw new ProjectArchiveError("CORRUPT_ARCHIVE", `ZIP paths do not match: ${entry.path}`);
    const compressed = bytes.subarray(dataStart, dataEnd);
    const data = entry.method === 0 ? compressed.slice() : await inflateRaw(compressed);
    if (data.byteLength !== entry.size || crc32(data) !== entry.crc) {
      throw new ProjectArchiveError("CORRUPT_ARCHIVE", `ZIP entry failed size or checksum validation: ${entry.path}`);
    }
    files.push({ path: entry.path, bytes: data });
  }
  return files.sort((a, b) => a.path.localeCompare(b.path));
}
