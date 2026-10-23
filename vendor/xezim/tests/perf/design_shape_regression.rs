//! End-to-end guards for the two benchmark-shaped workloads: a RISC-style
//! pipeline (dense case decoder, wildcard casez, byte-masked RAM writes,
//! constant CSR readback mux, tracer-style void decode helper with string
//! formal + $sformatf) and a cipher-style one (128-bit state, case-based
//! sbox function inlined 16x per round through an argument-carrying task
//! FSM). Both checksums are byte-verified against the reference simulator.
//!
//! Each test asserts three things: the ANSWER (a rotating checksum folds
//! every path in, so a mid-run wrong turn cannot cancel out), that the whole
//! design COMPILES (fallbacks=0 — any construct dropping back to the AST
//! interpreter fails here first), and an executed-instruction CEILING (a
//! lost fusion or dead code shows up as work). Re-baseline the ceilings
//! downward when an optimization lands.

fn run_profiled(name: &str) -> String {
    let path = format!("{}/tests/perf/{}", env!("CARGO_MANIFEST_DIR"), name);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "top",
            &path,
            "--no-cache",
            "--max-time",
            "2000000",
        ])
        .env("XEZIM_PROFILE_TIMING", "1")
        .output()
        .expect("run profiled design");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "profiled design failed:\n{text}");
    text
}

fn stat(text: &str, key: &str) -> u64 {
    text.lines()
        .filter(|l| l.contains("edge_detect="))
        .find_map(|l| {
            l.split_whitespace()
                .find_map(|tok| tok.strip_prefix(key))
                .and_then(|n| n.parse::<u64>().ok())
        })
        .unwrap_or_else(|| panic!("missing `{key}` stat:\n{text}"))
}

#[test]
fn riscv_shape_compiles_and_matches_reference() {
    let out = run_profiled("riscv_shape_regression.sv");
    assert!(
        out.contains("IBX acc=940fc24b chk=8efa3394 dec=[nop\tr15,0xc248]"),
        "wrong answer:\n{out}"
    );
    assert_eq!(
        stat(&out, "fallbacks="),
        0,
        "AST fallbacks crept in:\n{out}"
    );
    assert!(
        stat(&out, "insns=") <= 10_000_000,
        "instruction count regressed:\n{out}"
    );
}

#[test]
fn cipher_shape_compiles_and_matches_reference() {
    let out = run_profiled("cipher_shape_regression.sv");
    assert!(
        out.contains("AESM digest=41570f4d35f01fe3878b8a131109d7b1 blocks=2000"),
        "wrong answer:\n{out}"
    );
    assert_eq!(
        stat(&out, "fallbacks="),
        0,
        "AST fallbacks crept in:\n{out}"
    );
    assert!(
        stat(&out, "insns=") <= 10_000_000,
        "instruction count regressed:\n{out}"
    );
}
