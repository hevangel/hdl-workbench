//! §20.5: `$rtoi` truncates toward zero to a signed 32-bit `integer`: the low
//! 32 bits of the truncated value as a 64-bit integer. A value outside the
//! 64-bit signed range has no such integer and gives 0, as the reference
//! simulator does; saturating it gave -1. Run-time and constant (parameter)
//! evaluation agree. The expected lines are the reference simulator's.

use std::process::Command;

/// Run `src` (top module `top`) and return its display lines: everything on
/// stdout but the banner and the end-of-run note.
fn run_lines(tag: &str, src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("xezim_{}_{}_{}", "rtoi", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("top.sv");
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .args(["--no-cache", "-s", "top"])
        .arg(&sv)
        .current_dir(&dir)
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .filter(|l| {
            !(l.starts_with('[')
                || l.starts_with("=== xezim")
                || l.starts_with("git ")
                || l.starts_with("Simulation finished")
                || l.starts_with("$finish"))
        })
        .map(str::to_string)
        .collect()
}

/// Both method tiers: interpreted, and compiled on the first call.
fn check(tag: &str, src: &str, want: &[&str]) {
    assert_eq!(run_lines(tag, src, &[]), want, "{tag}");
    assert_eq!(
        run_lines(&format!("{tag}_t0"), src, &[("XEZIM_METHOD_TIER", "0")]),
        want,
        "{tag} (XEZIM_METHOD_TIER=0)"
    );
}

#[test]
fn rtoi_of_out_of_range_reals() {
    let src = r#"module top;
  real r; longint l; integer g; int i;
  localparam integer P1 = $rtoi(1.0e19);
  localparam integer P2 = $rtoi(2.0**40 + 3.7);
  localparam integer P3 = $rtoi(-(2.0**63));
  initial begin
    r = 2.0**62 + 2.0**12 + 5.0; $display("T|rtoi 2^62+4101 = %0d", $rtoi(r));
    r = 2.0**40 + 3.7;  $display("T|rtoi 2^40+3.7 = %0d", $rtoi(r));
    r = -(2.0**40) - 3.7; $display("T|rtoi -2^40-3.7 = %0d", $rtoi(r));
    r = -(2.0**63);     $display("T|rtoi -2^63 = %0d", $rtoi(r));
    r = -(2.0**63) - 4096.0; $display("T|rtoi -2^63-4096 = %0d", $rtoi(r));
    r = 2.0**63 - 1024.0; $display("T|rtoi 2^63-1024 = %0d", $rtoi(r));
    r = 9.3e18;         $display("T|rtoi 9.3e18 = %0d", $rtoi(r));
    r = -9.3e18;        $display("T|rtoi -9.3e18 = %0d", $rtoi(r));
    r = 1.0e19;         $display("T|rtoi 1e19 = %0d", $rtoi(r));
    r = -1.0e10;        $display("T|rtoi -1e10 = %0d", $rtoi(r));
    r = 4.0e9;          $display("T|rtoi 4e9 = %0d", $rtoi(r));
    r = 2.0**31;        $display("T|rtoi 2^31 = %0d", $rtoi(r));
    r = 1.0e300;        $display("T|rtoi 1e300 = %0d", $rtoi(r));
    l = $rtoi(-1.0e10); $display("T|asg l rtoi(-1e10) = %0d", l);
    l = $rtoi(2.0**63); $display("T|asg l rtoi(2^63) = %0d", l);
    g = $rtoi(-1.0e30); $display("T|asg integer rtoi(-1e30) = %0d", g);
    $display("T|param %0d %0d %0d", P1, P2, P3);
  end
endmodule
"#;
    check(
        "rtoi_of_out_of_range_reals",
        src,
        &[
            "T|rtoi 2^62+4101 = 4096",
            "T|rtoi 2^40+3.7 = 3",
            "T|rtoi -2^40-3.7 = -3",
            "T|rtoi -2^63 = 0",
            "T|rtoi -2^63-4096 = 0",
            "T|rtoi 2^63-1024 = -1024",
            "T|rtoi 9.3e18 = 0",
            "T|rtoi -9.3e18 = 0",
            "T|rtoi 1e19 = 0",
            "T|rtoi -1e10 = -1410065408",
            "T|rtoi 4e9 = -294967296",
            "T|rtoi 2^31 = -2147483648",
            "T|rtoi 1e300 = 0",
            "T|asg l rtoi(-1e10) = -1410065408",
            "T|asg l rtoi(2^63) = 0",
            "T|asg integer rtoi(-1e30) = 0",
            "T|param 0 3 0",
        ],
    );
}
