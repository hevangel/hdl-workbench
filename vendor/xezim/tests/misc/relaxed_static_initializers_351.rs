//! §6.21: accepting an implicit initializer retains static lifetime, while
//! explicit automatic declarations and automatic subroutines start afresh.

use std::process::Command;

fn check(source: &str, mode: &[(&str, &str)], relax: bool) -> String {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "xezim_static_351_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_xezim"));
    command
        .args(["--no-cache", "-s", "top"])
        .arg(&path)
        .env_remove("XEZIM_ALLOW_IMPLICIT_STATIC")
        .env_remove("XEZIM_METHOD_TIER")
        .env_remove("XEZIM_COMPILE_METHODS");
    if relax {
        command.arg("--relax-implicit-static");
    }
    let output = command
        .envs(mode.iter().copied())
        .output()
        .expect("run simulator");
    std::fs::remove_dir_all(&dir).unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{text}");
    assert!(
        text.contains("ALL PASS") && !text.contains("FAIL"),
        "{text}"
    );
    text
}

const ORIGINAL: &str = include_str!("../issue_cases/implicit_static_init_351.sv");
const NEIGHBORS: &str = include_str!("../issue_cases/implicit_static_neighbors_351.sv");

#[test]
fn relaxed_static_original() {
    check(ORIGINAL, &[], true);
}
#[test]
fn relaxed_static_original_eager() {
    check(ORIGINAL, &[("XEZIM_METHOD_TIER", "0")], true);
}
#[test]
fn relaxed_static_original_interpreted() {
    check(ORIGINAL, &[("XEZIM_COMPILE_METHODS", "0")], true);
}
#[test]
fn relaxed_static_neighbors() {
    check(NEIGHBORS, &[], true);
}
#[test]
fn relaxed_static_neighbors_eager() {
    check(NEIGHBORS, &[("XEZIM_METHOD_TIER", "0")], true);
}
#[test]
fn relaxed_static_neighbors_interpreted() {
    check(NEIGHBORS, &[("XEZIM_COMPILE_METHODS", "0")], true);
}
#[test]
fn environment_relaxation_has_static_lifetime() {
    check(ORIGINAL, &[("XEZIM_ALLOW_IMPLICIT_STATIC", "1")], false);
}
#[test]
fn unrelaxed_initialized_block_policy_is_preserved() {
    check(
        r#"module top;
      initial begin
        int seen;
        for (int i = 0; i < 2; i++) begin int n = 0; n++; seen = n; end
        if (seen == 1) $display("ALL PASS"); else $display("FAIL");
      end
    endmodule"#,
        &[],
        false,
    );
}

#[test]
fn design_cache_separates_static_lifetime_modes() {
    let dir = std::env::temp_dir().join(format!("xezim_static_cache_351_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("top.sv");
    std::fs::write(
        &source,
        r#"module top;
      initial begin
        int seen;
        repeat (2) begin int n = 0; n++; seen = n; end
        $display("T|seen %0d", seen);
      end
    endmodule"#,
    )
    .unwrap();
    let cache = dir.join("cache");
    for (flag, environment, want, hit) in [
        (false, false, 1, false),
        (true, false, 2, false),
        (true, false, 2, true),
        (false, false, 1, true),
        (false, true, 2, true),
    ] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
        cmd.args(["--verbose", "--cache-dir"])
            .arg(&cache)
            .arg(&source)
            .env_remove("XEZIM_ALLOW_IMPLICIT_STATIC");
        if flag {
            cmd.arg("--relax-implicit-static");
        }
        if environment {
            cmd.env("XEZIM_ALLOW_IMPLICIT_STATIC", "1");
        }
        let output = cmd.output().expect("run cached lifetime regression");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(
            stdout.contains(&format!("T|seen {want}")),
            "{stdout}\n{stderr}"
        );
        assert!(
            stderr.contains(if hit { "CACHE] hit" } else { "CACHE] miss" }),
            "{stderr}"
        );
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
