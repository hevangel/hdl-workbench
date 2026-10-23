//! `-uvm` with `XEZIM_UVM_DIR`: add the UVM library to the design.
//!
//! With `-uvm` (or `--uvm`) on the command line or in a `-f`/`-F` args file,
//! the UVM source tree that `XEZIM_UVM_DIR` names is added: its `src`
//! directory is appended to the include path and its `uvm_pkg.sv` becomes
//! the first source file, unless the file list already has a `uvm_pkg.sv`.
//! This holds for every mode (`--preprocess`, `--parse`, `--compile` and
//! simulation), so a UVM testbench needs only `-uvm` and its own files.
//! Without `-uvm`, nothing is added, whatever the variable says.
//!
//! The variable may name:
//! - the `src` directory itself (it holds `uvm_pkg.sv`);
//! - a UVM release root (it holds `src/uvm_pkg.sv`);
//! - a checkout holding several releases as subdirectories, such as
//!   https://github.com/nitronis/UVM (`1.1d`, `1.2`, `1800.2-2017`,
//!   `1800.2-2020`). `XEZIM_UVM_VERSION` picks one; otherwise the newest
//!   present is used.
//!
//! Pure path logic (no simulator state), so tests/misc/uvm_dir_env.rs
//! includes this file directly.

use std::path::{Path, PathBuf};

/// Release subdirectories of a multi-release checkout, newest first.
pub(crate) const UVM_VERSIONS: &[&str] = &["1800.2-2020", "1800.2-2017", "1.2", "1.1d"];

/// The `src` directory (the one holding `uvm_pkg.sv`) that `dir` names, or
/// an error that lists where it looked.
pub(crate) fn resolve_uvm_src(dir: &Path, version: Option<&str>) -> Result<PathBuf, String> {
    let has_pkg = |d: &Path| d.join("uvm_pkg.sv").is_file();
    if let Some(v) = version {
        let d = dir.join(v).join("src");
        return if has_pkg(&d) {
            Ok(d)
        } else {
            Err(format!("no uvm_pkg.sv in {}", d.display()))
        };
    }
    if has_pkg(dir) {
        return Ok(dir.to_path_buf());
    }
    let src = dir.join("src");
    if has_pkg(&src) {
        return Ok(src);
    }
    for v in UVM_VERSIONS {
        let d = dir.join(v).join("src");
        if has_pkg(&d) {
            return Ok(d);
        }
    }
    Err(format!(
        "no uvm_pkg.sv in {}, {}/src or {}/<{}>/src",
        dir.display(),
        dir.display(),
        dir.display(),
        UVM_VERSIONS.join("|")
    ))
}

/// Whether the file list already supplies the UVM package.
pub(crate) fn supplies_uvm_pkg(source_files: &[String]) -> bool {
    source_files
        .iter()
        .any(|f| Path::new(f).file_name().is_some_and(|n| n == "uvm_pkg.sv"))
}

/// What `-uvm` did, for the caller to report.
#[derive(Debug, PartialEq)]
pub(crate) enum UvmDirAction {
    /// No `-uvm`: nothing added.
    NotRequested,
    /// `-uvm`, but the file list already has a `uvm_pkg.sv`.
    AlreadySupplied,
    /// Added `src` to the include path and `src/uvm_pkg.sv` as the first
    /// source file.
    Added(PathBuf),
    /// `-uvm`, but `XEZIM_UVM_DIR` is unset, empty, or names no UVM source
    /// tree. The message says which.
    Error(String),
}

/// Apply `-uvm` (`requested`) with `XEZIM_UVM_DIR` = `uvm_dir` and
/// `XEZIM_UVM_VERSION` = `version` to the file list and include path.
pub(crate) fn apply_uvm_dir(
    requested: bool,
    uvm_dir: Option<&str>,
    version: Option<&str>,
    source_files: &mut Vec<String>,
    include_dirs: &mut Vec<String>,
) -> UvmDirAction {
    if !requested {
        return UvmDirAction::NotRequested;
    }
    if supplies_uvm_pkg(source_files) {
        return UvmDirAction::AlreadySupplied;
    }
    let Some(dir) = uvm_dir.filter(|d| !d.is_empty()) else {
        return UvmDirAction::Error(
            "-uvm needs XEZIM_UVM_DIR set to a UVM source tree".to_string(),
        );
    };
    let version = version.filter(|v| !v.is_empty());
    let src = match resolve_uvm_src(Path::new(dir), version) {
        Ok(s) => s,
        Err(e) => return UvmDirAction::Error(format!("-uvm: XEZIM_UVM_DIR has {}", e)),
    };
    // Appended, so directories the user gave are searched first.
    if !include_dirs.iter().any(|d| Path::new(d) == src.as_path()) {
        include_dirs.push(src.to_string_lossy().into_owned());
    }
    source_files.insert(0, src.join("uvm_pkg.sv").to_string_lossy().into_owned());
    UvmDirAction::Added(src)
}
