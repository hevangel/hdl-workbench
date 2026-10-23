//! String casts retain byte content across execution tiers (§6.16, §6.24.3).
use std::path::{Path, PathBuf};
use std::process::Command;

fn directory(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("string_cast_{name}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(source: &str, dir: &Path, compiled: bool, cache: Option<&Path>) -> Vec<String> {
    let input = dir.join("case.sv");
    std::fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "--no-cache", "-s", "top"])
        .arg(&input)
        .env("XEZIM_METHOD_TIER", "0")
        .env("XEZIM_COMPILE_METHODS", if compiled { "1" } else { "0" })
        .env("XEZIM_PROC_FSM", if compiled { "1" } else { "0" })
        .env("XEZIM_TWO_STATE", if compiled { "1" } else { "0" })
        .env(
            "XEZIM_METHOD_CACHE",
            cache.unwrap_or_else(|| Path::new("0")),
        )
        .current_dir(dir)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{text}");
    assert!(!text.lines().any(|line| line.starts_with("FAIL")), "{text}");
    let mut rows: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("T|"))
        .map(str::to_owned)
        .collect();
    rows.sort();
    rows
}

fn cached_methods(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bin"))
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

#[test]
fn string_cast_method_cache_preserves_results_when_warm() {
    let dir = directory("methods");
    let cache = dir.join("methods");
    let source = include_str!("../issue_cases/issue391_string_method_cache.sv");
    assert_eq!(run(source, &dir, true, Some(&cache)), ["T|cache|1 5"]);
    let cold = cached_methods(&cache);
    assert!(
        cold.iter().any(|(_, bytes)| bytes.len() > 4),
        "no compiled method cached"
    );
    assert_eq!(run(source, &dir, true, Some(&cache)), ["T|cache|1 5"]);
    assert_eq!(cached_methods(&cache), cold);
}

const EXPECTED: [&str; 5] = [
    "T|class|1 2",
    "T|concat|1 1 1",
    "T|high|2 233",
    "T|nul|1 2 0 1",
    "T|once|1 1 2",
];

#[test]
fn compiled_string_casts_preserve_bytes_and_aliases() {
    assert_eq!(
        run(
            include_str!("../issue_cases/issue391_string_cast_neighbors.sv"),
            &directory("compiled"),
            true,
            None
        ),
        EXPECTED
    );
}

#[test]
fn interpreted_string_casts_preserve_bytes_and_aliases() {
    assert_eq!(
        run(
            include_str!("../issue_cases/issue391_string_cast_neighbors.sv"),
            &directory("interpreted"),
            false,
            None
        ),
        EXPECTED
    );
}
