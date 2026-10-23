//! UVM 00no_macros / 4030 root cause (part 1): assigning a value WIDER than a
//! function/method-LOCAL `byte`/`shortint`/`int` must truncate to the declared
//! width (IEEE 1800 §11.6.1). UVM `pre_randomize` does `byte b; b = $urandom;
//! aa[b] = ele;` — if the 32-bit `$urandom` is not narrowed, the assoc key
//! becomes sign-extended garbage. xezim's local-assignment path was
//! NARROW-ONLY (only widened narrow literals; left a wider RHS as-is), so
//! `b = r` kept a 32-bit int. The reference truncates; this asserts xezim does
//! the same.

use std::process::Command;

#[test]
fn byte_local_narrows_wide_assignment() {
    let src = r#"
module top;
  initial begin
    byte b; int r; longint l; int x; int s32; shortint s16;
    b = 32'hDEAD00AB; $display("B0 %h", b);   // must be byte 0xAB
    r = 32'h12345678; b = r;    $display("B1 %h", b);  // -> 0x78
    l = 64'h123456_89ABCDEF; x = l; $display("I0 %h", x); // int low 32
    s32 = 32'hFFFE0007;  s16 = s32; $display("S0 %h", s16);// -> 0x0007
  end
endmodule
"#;
    let dir = std::env::temp_dir().join(format!("xezim_byte_narrow_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sv_path = dir.join("byte_narrow.sv");
    std::fs::write(&sv_path, src).unwrap();

    let bin = env!("CARGO_BIN_EXE_xezim");
    let out = Command::new(bin)
        .arg("--simulate")
        .arg("-s")
        .arg("top")
        .arg(sv_path.to_str().unwrap())
        .output()
        .expect("failed to run xezim");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!(
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("B0 ab"),
        "byte = 32'hDEAD00AB must truncate to low 8 bits 'ab'.\n{combined}"
    );
    assert!(
        stdout.contains("B1 78"),
        "int local 0x12345678 into a byte must give 0x78.\n{combined}"
    );
    assert!(
        stdout.contains("I0 89abcdef"),
        "longint into int must keep the low 32 bits.\n{combined}"
    );
    assert!(
        stdout.contains("S0 0007"),
        "int into shortint must keep the low 16 bits.\n{combined}"
    );
}
