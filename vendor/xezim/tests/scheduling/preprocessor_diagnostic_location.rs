use std::process::Command;

fn xezim_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xezim")
}

#[test]
fn parse_mode_reports_reserved_macro_file_and_line() {
    let unique = format!(
        "xezim-cli-pp-location-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("create temp directory");
    let source = dir.join("diagsrc.sv");
    std::fs::write(
        &source,
        "module before_redefinition; endmodule\n\
         `define __FILE__ disabled\n\
         `define __LINE__ 0\n\
         module after_redefinition; endmodule\n",
    )
    .expect("write source");

    let strict = Command::new(xezim_bin())
        .arg("--parse")
        .arg(&source)
        .output()
        .expect("run strict parser");
    assert!(!strict.status.success());
    let stderr = String::from_utf8_lossy(&strict.stderr);
    assert!(
        stderr.contains(&format!("{}:2:1: error: `__FILE__", source.display())),
        "missing __FILE__ location:\n{stderr}"
    );
    assert!(
        stderr.contains(&format!("{}:3:1: error: `__LINE__", source.display())),
        "missing __LINE__ location:\n{stderr}"
    );

    let lenient = Command::new(xezim_bin())
        .arg("--parse")
        .arg("--no-strict")
        .arg(&source)
        .output()
        .expect("run lenient parser");
    assert!(
        lenient.status.success(),
        "--no-strict should accept the source:\n{}",
        String::from_utf8_lossy(&lenient.stderr)
    );

    std::fs::remove_dir_all(dir).expect("remove temp directory");
}

#[test]
fn parse_mode_accepts_guarded_reserved_macro_fallbacks_in_strict_mode() {
    let unique = format!(
        "xezim-cli-pp-guarded-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("create temp directory");
    let source = dir.join("amb_types.h");
    std::fs::write(
        &source,
        "`ifndef __FILE__\n\
         `define __FILE__ 0\n\
         `endif\n\
         `ifndef __LINE__\n\
         `define __LINE__ 0\n\
         `endif\n\
         module guarded_reserved_fallbacks; endmodule\n",
    )
    .expect("write source");

    let strict = Command::new(xezim_bin())
        .arg("--parse")
        .arg(&source)
        .output()
        .expect("run strict parser");
    assert!(
        strict.status.success(),
        "strict parse should skip guarded __FILE__/__LINE__ fallbacks:\n{}",
        String::from_utf8_lossy(&strict.stderr)
    );

    std::fs::remove_dir_all(dir).expect("remove temp directory");
}
