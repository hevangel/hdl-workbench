//! §8.23 / §8.25 / §13.4.1 / §15: static class references retain
//! specialization, lexical scope, and suspended synchronization state.

use std::process::Command;

fn check(source: &str, mode: &[(&str, &str)], passes: usize) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_static_382_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--no-cache",
            "-s",
            if passes == 88 { "tb_top" } else { "top" },
        ])
        .arg(&path)
        .env_remove("XEZIM_METHOD_TIER")
        .env_remove("XEZIM_COMPILE_METHODS")
        .env_remove("XEZIM_PROC_FSM")
        .envs(mode.iter().copied())
        .output()
        .expect("run static class regressions");
    std::fs::remove_dir_all(&dir).unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{text}");
    assert!(!text.contains("FAIL"), "{text}");
    assert_eq!(
        text.lines()
            .filter(|line| line.trim_start_matches(['|', ' ']).starts_with("PASS"))
            .count(),
        passes,
        "{text}"
    );
    text
}

const ORIGINAL: &str = include_str!("../issue_cases/static_class_members_382.sv");
const SCOPES: &str = include_str!("../issue_cases/module_class_scopes_382.sv");

#[test]
fn static_class_members() {
    check(ORIGINAL, &[], 88);
}
#[test]
fn static_class_members_eager() {
    check(ORIGINAL, &[("XEZIM_METHOD_TIER", "0")], 88);
}
#[test]
fn static_class_members_interpreted() {
    check(
        ORIGINAL,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        88,
    );
}
#[test]
fn module_class_scopes() {
    check(SCOPES, &[], 13);
}
#[test]
fn module_class_scopes_eager() {
    check(SCOPES, &[("XEZIM_METHOD_TIER", "0")], 13);
}
#[test]
fn module_class_scopes_interpreted() {
    check(
        SCOPES,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        13,
    );
}

const TYPE_RECEIVERS: &str = include_str!("../issue_cases/module_type_receivers_382.sv");
const EVENTS: &str = include_str!("../issue_cases/static_event_groups_382.sv");
#[test]
fn module_type_receivers() {
    check(TYPE_RECEIVERS, &[], 5);
}
#[test]
fn module_type_receivers_eager() {
    check(TYPE_RECEIVERS, &[("XEZIM_METHOD_TIER", "0")], 5);
}
#[test]
fn module_type_receivers_interpreted() {
    check(
        TYPE_RECEIVERS,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        5,
    );
}
#[test]
fn static_event_groups() {
    check(EVENTS, &[], 8);
}
#[test]
fn static_event_groups_eager() {
    check(EVENTS, &[("XEZIM_METHOD_TIER", "0")], 8);
}
#[test]
fn static_event_groups_interpreted() {
    check(
        EVENTS,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        8,
    );
}

#[test]
fn module_class_type_names_retain_their_instance_paths() {
    let text = check(SCOPES, &[], 13);
    let mut got: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("T|type "))
        .collect();
    got.sort();
    assert_eq!(
        got,
        [
            "T|type top.u1 class top/u1/holder",
            "T|type top.u2 class top/u2/holder",
            "T|type top.u3 class top/u3/holder",
            "T|type top.u4 class top/u4/holder",
            "T|type top.u5 class top/u5/holder",
            "T|type top.u6 class top/u6/holder",
        ]
    );
}

const ALIAS_INITIALIZERS: &str =
    include_str!("../issue_cases/module_class_static_initializers_382.sv");
#[test]
fn module_class_aliases_initialize_once() {
    check(ALIAS_INITIALIZERS, &[], 7);
}
#[test]
fn module_class_aliases_initialize_once_eager() {
    check(ALIAS_INITIALIZERS, &[("XEZIM_METHOD_TIER", "0")], 7);
}
#[test]
fn module_class_aliases_initialize_once_interpreted() {
    check(
        ALIAS_INITIALIZERS,
        &[("XEZIM_COMPILE_METHODS", "0"), ("XEZIM_PROC_FSM", "0")],
        7,
    );
}

#[test]
fn warm_design_cache_keeps_module_class_initializers_single() {
    let dir =
        std::env::temp_dir().join(format!("xezim_class_init_cache_382_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("top.sv");
    std::fs::write(&source, ALIAS_INITIALIZERS).unwrap();
    let cache = dir.join("cache");
    for warm in [false, true] {
        let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
            .arg("--cache-dir")
            .arg(&cache)
            .args(["--verbose", "-s", "top"])
            .arg(&source)
            .env_remove("XEZIM_METHOD_TIER")
            .env_remove("XEZIM_COMPILE_METHODS")
            .env_remove("XEZIM_PROC_FSM")
            .env_remove("XEZIM_ALLOW_IMPLICIT_STATIC")
            .output()
            .expect("run cached static class initializers");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success() && !text.contains("FAIL"), "{text}");
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("PASS "))
                .count(),
            7,
            "{text}"
        );
        assert_eq!(text.contains("CACHE] hit"), warm, "{text}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
