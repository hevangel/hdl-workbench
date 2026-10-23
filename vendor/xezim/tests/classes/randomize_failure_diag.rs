//! `XEZIM_RAND_DIAG=1`: when `randomize()` fails, report on stderr which
//! class failed, which variables and constraint blocks were switched off
//! (`rand_mode` / `constraint_mode`), and which constraint items the trials
//! left unsatisfied, with their source text and location. Nothing is
//! printed without the variable.

use std::process::Command;

const SRC: &str = "\
class item;
  rand int x;
  rand int id;
  rand int y;
  constraint c_lo { x < 5; }
  constraint c_hi { x > 10; }
  constraint c_off { y == 1; }
  constraint c_y { y inside {[0:3]}; }
endclass

module top;
  initial begin
    item it = new();
    it.id.rand_mode(0);
    it.c_off.constraint_mode(0);
    if (!it.randomize()) $display(\"randomize failed\");
  end
endmodule
";

fn run(diag: bool) -> (String, String) {
    run_sources("single", &[("rand_diag.sv", SRC)], diag)
}

fn run_sources(tag: &str, sources: &[(&str, &str)], diag: bool) -> (String, String) {
    let dir = std::env::temp_dir().join(format!(
        "xezim_rand_diag_{}_{}_{}",
        std::process::id(),
        tag,
        diag
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let files: Vec<_> = sources
        .iter()
        .map(|(name, source)| {
            let file = dir.join(name);
            std::fs::write(&file, source).unwrap();
            file
        })
        .collect();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    cmd.arg("--simulate").arg("-s").arg("top").args(&files);
    if diag {
        cmd.env("XEZIM_RAND_DIAG", "1");
    } else {
        cmd.env_remove("XEZIM_RAND_DIAG");
    }
    let out = cmd.output().expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn diag_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| l.starts_with("[rand-diag]"))
        .collect()
}

#[test]
fn a_failed_randomize_reports_the_unsatisfied_constraints() {
    let (stdout, stderr) = run(true);
    assert!(stdout.contains("randomize failed"), "{stdout}");
    let lines = diag_lines(&stderr);
    let has = |s: &str| lines.iter().any(|l| l.contains(s));
    assert!(has("randomize() failed: class item"), "{lines:#?}");
    assert!(has("rand_mode off: id"), "{lines:#?}");
    assert!(has("constraint_mode off: c_off"), "{lines:#?}");
    assert!(
        lines
            .iter()
            .any(|l| l.contains("c_lo") && l.contains("rand_diag.sv:5") && l.contains("x < 5")),
        "{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("c_hi") && l.contains("rand_diag.sv:6") && l.contains("x > 10")),
        "{lines:#?}"
    );
    // Always satisfied and shares no variable with them: listed as an
    // active block, but neither unsatisfied nor related.
    assert!(has("constraint blocks: c_hi, c_lo, c_y"), "{lines:#?}");
    assert!(
        !lines
            .iter()
            .any(|l| l.contains("rand_diag.sv:") && l.contains("c_y")),
        "{lines:#?}"
    );
}

#[test]
fn nothing_is_reported_without_the_variable() {
    let (stdout, stderr) = run(false);
    assert!(stdout.contains("randomize failed"), "{stdout}");
    assert!(diag_lines(&stderr).is_empty(), "{stderr}");
}

#[test]
fn named_constraints_use_their_defining_file() {
    const DECOY: &str = "\
class unused_item;
  rand int value;
  constraint bounds { value < 2; value > 20; }
endclass
";
    const ACTUAL: &str = "\
class item;
  rand int value;
  constraint bounds { value < 5; value > 10; }
endclass
module top;
  initial begin
    item it = new();
    if (!it.randomize()) $display(\"failed as expected\");
  end
endmodule
";
    let (stdout, stderr) = run_sources(
        "named_multifile",
        &[("decoy_source.sv", DECOY), ("named_source.sv", ACTUAL)],
        true,
    );
    assert!(stdout.contains("failed as expected"), "{stdout}");
    let lines = diag_lines(&stderr);
    assert!(
        lines.iter().any(|line| {
            line.contains("bounds")
                && line.contains("named_source.sv:3")
                && line.contains("value < 5")
        }),
        "{lines:#?}"
    );
    assert!(
        !lines.iter().any(|line| line.contains("decoy_source.sv")),
        "{lines:#?}"
    );
}

#[test]
fn inline_constraints_use_their_call_site_file() {
    const DECOY: &str = "\
module unused_top;
  int padding_that_keeps_this_source_long_enough_for_matching_offsets;
endmodule
";
    const ACTUAL: &str = "\
class item;
  rand int value;
endclass
module top;
  initial begin
    item it = new();
    if (!it.randomize() with { value < 5; value > 10; })
      $display(\"failed as expected\");
  end
endmodule
";
    let (stdout, stderr) = run_sources(
        "inline_multifile",
        &[("unused_source.sv", DECOY), ("inline_source.sv", ACTUAL)],
        true,
    );
    assert!(stdout.contains("failed as expected"), "{stdout}");
    let lines = diag_lines(&stderr);
    assert!(
        lines.iter().any(|line| {
            line.contains("with {...}")
                && line.contains("inline_source.sv:7")
                && line.contains("value < 5")
        }),
        "{lines:#?}"
    );
    assert!(
        !lines.iter().any(|line| line.contains("(item ")),
        "{lines:#?}"
    );
}
