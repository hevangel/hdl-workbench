//! `$fopen` handles with buffered writes and an optional project VFS backend.
//!
//! Native callers keep regular `File` handles. Browser callers install a
//! project-relative `VirtualFileSystem`; in that mode no host filesystem path
//! is opened and read/write operations use an in-memory cursor.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

/// Bytes collected before a handle writes its buffer out.
const SV_FILE_BUF_CAP: usize = 64 * 1024;

#[derive(Clone)]
pub(super) struct VirtualFileSystem(Arc<Mutex<VirtualFileState>>);

struct VirtualFileState {
    files: HashMap<String, Vec<u8>>,
    dirty: HashSet<String>,
    max_bytes: usize,
    total_bytes: usize,
    limit_exceeded: bool,
}

impl VirtualFileSystem {
    pub(super) fn new(files: Vec<(String, Vec<u8>)>, max_bytes: usize) -> Result<Self, String> {
        let mut map = HashMap::new();
        let mut total_bytes = 0usize;
        for (path, bytes) in files {
            let path = normalize_project_path(&path)
                .ok_or_else(|| format!("invalid project-relative file path: {path}"))?;
            if map.contains_key(&path) {
                return Err(format!("duplicate project file path: {path}"));
            }
            total_bytes = total_bytes
                .checked_add(bytes.len())
                .ok_or_else(|| "project file byte count overflow".to_string())?;
            if total_bytes > max_bytes {
                return Err(format!("project files exceed the {max_bytes} byte limit"));
            }
            map.insert(path, bytes);
        }
        Ok(Self(Arc::new(Mutex::new(VirtualFileState {
            files: map,
            dirty: HashSet::new(),
            max_bytes,
            total_bytes,
            limit_exceeded: false,
        }))))
    }

    pub(super) fn read(&self, path: &str) -> Option<Vec<u8>> {
        let path = normalize_project_path(path)?;
        self.0.lock().ok()?.files.get(&path).cloned()
    }

    pub(super) fn read_to_string(&self, path: &str) -> io::Result<String> {
        let bytes = self
            .read(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "virtual file not found"))?;
        String::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub(super) fn write(&self, path: &str, bytes: Vec<u8>) -> io::Result<()> {
        let path = normalize_project_path(path).ok_or_else(|| {
            io::Error::new(io::ErrorKind::PermissionDenied, "invalid virtual path")
        })?;
        let mut state = self
            .0
            .lock()
            .map_err(|_| io::Error::other("virtual filesystem lock poisoned"))?;
        let old_size = state.files.get(&path).map_or(0, Vec::len);
        let next_total = state
            .total_bytes
            .saturating_sub(old_size)
            .checked_add(bytes.len())
            .unwrap_or(usize::MAX);
        if next_total > state.max_bytes {
            state.limit_exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "virtual filesystem byte limit exceeded",
            ));
        }
        state.total_bytes = next_total;
        state.files.insert(path.clone(), bytes);
        state.dirty.insert(path);
        Ok(())
    }

    pub(super) fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        let Ok(state) = self.0.lock() else {
            return Vec::new();
        };
        let mut files: Vec<_> = state
            .dirty
            .iter()
            .filter_map(|path| {
                state
                    .files
                    .get(path)
                    .map(|bytes| (path.clone(), bytes.clone()))
            })
            .collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    }

    pub(super) fn limit_exceeded(&self) -> bool {
        self.0.lock().map(|s| s.limit_exceeded).unwrap_or(true)
    }
}

/// Normalize an SV filename into the browser's logical POSIX project path.
/// Rooted paths, drive prefixes, and any `..` component are rejected.
pub(super) fn normalize_project_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.as_bytes().get(1) == Some(&b':')
    {
        return None;
    }
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            p => parts.push(p),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

enum Backend {
    Native(File),
    Virtual {
        fs: VirtualFileSystem,
        path: String,
        cursor: Cursor<Vec<u8>>,
    },
}

pub(super) struct SvFile {
    backend: Backend,
    buf: Vec<u8>,
    buffered: bool,
    /// (device, inode) of a regular file, to find two handles on one path.
    ident: Option<(u64, u64)>,
}

impl SvFile {
    pub(super) fn new(file: File, path: &str) -> Self {
        let ident = regular_file_ident(&file);
        let buffered = ident.is_some() && !path.starts_with("/dev/") && !path.starts_with("/proc/");
        SvFile {
            backend: Backend::Native(file),
            buf: Vec::new(),
            buffered,
            ident,
        }
    }

    pub(super) fn new_virtual(fs: VirtualFileSystem, path: &str, mode: &str) -> io::Result<Self> {
        let path = normalize_project_path(path).ok_or_else(|| {
            io::Error::new(io::ErrorKind::PermissionDenied, "invalid virtual path")
        })?;
        let read = mode.contains('r') || mode.contains('+');
        let truncate = mode.contains('w');
        let append = mode.contains('a');
        let mut bytes = if truncate {
            Vec::new()
        } else {
            fs.read(&path).unwrap_or_default()
        };
        if read && !truncate && fs.read(&path).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "virtual file not found",
            ));
        }
        let mut cursor = Cursor::new(std::mem::take(&mut bytes));
        if append {
            cursor.set_position(cursor.get_ref().len() as u64);
        }
        if truncate {
            fs.write(&path, Vec::new())?;
        }
        let file = Self {
            backend: Backend::Virtual { fs, path, cursor },
            buf: Vec::new(),
            buffered: true,
            ident: None,
        };
        Ok(file)
    }

    pub(super) fn ident(&self) -> Option<(u64, u64)> {
        self.ident
    }
    pub(super) fn has_pending(&self) -> bool {
        !self.buf.is_empty()
    }

    pub(super) fn set_write_through(&mut self) {
        let _ = self.write_out();
        self.buffered = false;
    }

    pub(super) fn write_out(&mut self) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        let result = match &mut self.backend {
            Backend::Native(file) => file.write_all(&self.buf),
            Backend::Virtual { fs, path, cursor } => {
                let result = cursor
                    .write_all(&self.buf)
                    .and_then(|_| fs.write(path, cursor.get_ref().clone()));
                result
            }
        };
        self.buf.clear();
        result
    }
}

#[cfg(unix)]
fn regular_file_ident(file: &File) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    file.metadata()
        .ok()
        .filter(|m| m.file_type().is_file())
        .map(|m| (m.dev(), m.ino()))
}

#[cfg(not(unix))]
fn regular_file_ident(_file: &File) -> Option<(u64, u64)> {
    None
}

impl Write for SvFile {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if !self.buffered {
            return match &mut self.backend {
                Backend::Native(file) => file.write(data),
                Backend::Virtual { fs, path, cursor } => {
                    let written = cursor.write(data)?;
                    fs.write(path, cursor.get_ref().clone())?;
                    Ok(written)
                }
            };
        }
        if self.buf.len() + data.len() > SV_FILE_BUF_CAP {
            self.write_out()?;
            if data.len() >= SV_FILE_BUF_CAP {
                return match &mut self.backend {
                    Backend::Native(file) => file.write(data),
                    Backend::Virtual { fs, path, cursor } => {
                        let written = cursor.write(data)?;
                        fs.write(path, cursor.get_ref().clone())?;
                        Ok(written)
                    }
                };
            }
        }
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_out()?;
        match &mut self.backend {
            Backend::Native(file) => file.flush(),
            Backend::Virtual { .. } => Ok(()),
        }
    }
}

impl Read for SvFile {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.write_out()?;
        match &mut self.backend {
            Backend::Native(file) => file.read(out),
            Backend::Virtual { cursor, .. } => cursor.read(out),
        }
    }
}

impl Seek for SvFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.write_out()?;
        match &mut self.backend {
            Backend::Native(file) => file.seek(pos),
            Backend::Virtual { cursor, .. } => cursor.seek(pos),
        }
    }
}

impl Drop for SvFile {
    fn drop(&mut self) {
        let _ = self.write_out();
    }
}
