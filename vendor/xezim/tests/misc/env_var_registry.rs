//! `--show-env-avail` registry drift guard: every `"XEZIM_..."` string
//! literal read in xezim's own sources must appear in
//! `xezim::env_vars::ENV_VARS`, and every table entry must still be
//! referenced somewhere — so the printed list can't rot as variables are
//! added or removed. (xezim-core is scanned too when the sibling checkout
//! is present.)

use std::collections::BTreeSet;
use std::path::Path;

fn collect_vars(dir: &Path, out: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_vars(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let src = std::fs::read_to_string(&path).unwrap_or_default();
            let bytes = src.as_bytes();
            let mut i = 0;
            while let Some(off) = src[i..].find("\"XEZIM_") {
                let start = i + off + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_uppercase()
                        || bytes[end].is_ascii_digit()
                        || bytes[end] == b'_')
                {
                    end += 1;
                }
                if end < bytes.len() && bytes[end] == b'"' {
                    out.insert(src[start..end].to_string());
                }
                i = end;
            }
        }
    }
}

#[test]
fn env_var_registry_matches_sources() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut used = BTreeSet::new();
    collect_vars(&manifest.join("src"), &mut used);
    let core_src = manifest.join("../xezim-core/src");
    if core_src.is_dir() {
        collect_vars(&core_src, &mut used);
    }
    // Build-time-only variables read via env! in build scripts, not runtime.
    for buildtime in ["XEZIM_GIT_DATE", "XEZIM_GIT_HASH", "XEZIM_GIT_TAG"] {
        used.insert(buildtime.to_string());
    }

    let listed: BTreeSet<String> = xezim::env_vars::ENV_VARS
        .iter()
        .map(|(n, _)| n.to_string())
        .collect();

    let missing: Vec<_> = used.difference(&listed).collect();
    let stale: Vec<_> = listed.difference(&used).collect();
    assert!(
        missing.is_empty() && stale.is_empty(),
        "env-var registry drift.\n  read in source but not listed in env_vars.rs: {missing:?}\n  listed but no longer read anywhere: {stale:?}"
    );

    // Table stays sorted so the printout is scannable.
    let names: Vec<_> = xezim::env_vars::ENV_VARS.iter().map(|(n, _)| *n).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "ENV_VARS must be alphabetically sorted");
}
