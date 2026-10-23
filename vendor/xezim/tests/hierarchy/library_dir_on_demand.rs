//! `-y` library directories load `<module>.<ext>` ON DEMAND (§33.3 and every
//! other tool's behavior): nothing else in the directory is read, so an
//! unrelated broken file cannot poison the run. Previously the whole
//! directory was eagerly parsed — pointing `-y` at Verilator's test_regress
//! `t/` (~9400 files, many deliberately broken) produced hundreds of parse
//! warnings and exit 247 for a one-module test, and multi-GB RSS on big
//! trees. Name-mismatched libraries still resolve through the one-time
//! full-scan fallback. Exercised through the CLI since the harness API has
//! no -y knob.

#[test]
fn library_dir_loads_only_named_module() {
    let dir = std::env::temp_dir().join(format!("xz_ylib_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(
        dir.join("lib/wanted.v"),
        "module wanted(output logic [7:0] o); assign o = 8'hA5; endmodule\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/broken_unrelated.v"),
        "module broken_unrelated; this is not valid verilog @@@ ; endmodule\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("top.sv"),
        "module top;\n  logic [7:0] o;\n  wanted u(o);\n  initial #1 $display(\"NOTE: o=%h\", o);\nendmodule\n",
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "--no-cache",
            "-s",
            "top",
            "-y",
            dir.join("lib").to_str().unwrap(),
            "+libext+.v",
            dir.join("top.sv").to_str().unwrap(),
        ])
        .output()
        .expect("run xezim");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.status.success(), "run failed:\n{text}");
    assert!(text.contains("NOTE: o=a5"), "wrong output:\n{text}");
    assert!(
        !text.contains("broken_unrelated"),
        "unrelated library file was parsed:\n{text}"
    );
}
