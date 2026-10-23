//! §6.16 + §6.21 — a character-INDEX write into a FUNCTION-LOCAL (frame)
//! string must replace a character byte, not bit-bash the packed container.
//!
//! UVM's `uvm_packer::unpack_string` rebuilds a string one character at a time
//! with `unpack_string = {unpack_string, " "}; unpack_string[i] = ...;`.
//! The index write was routed through the frame-local unpacked-struct MEMBER
//! bit-slice arm (`s[i] = v` treated the string as a packed bit vector and
//! `set_bit` a single bit of the 128-char placeholder container), so every
//! character read back 0/space and a packed string decoded to garbage
//! ("zPU% |Ko" came back "  $( â�"). Module-level strings were unaffected
//! because they are stored in the signal table and take a different arm.
//!
//! Distilled from the report-message element-table regression where a string
//! field packed/unpacked through UVM miscompared. Verified byte-identical to
//! reference simulators.

use xezim::simulate;

/// An indexed write into a FUNCTION-LOCAL string (the UVM unpack_string idiom
/// of grow-by-concat then overwrite each character) must round-trip.
#[test]
fn char_index_write_into_function_local_string() {
    let src = r#"
module top;
  string SRC = "zPU% |Ko";
  function string f();
    string sx;
    int i;
    sx = "";
    i = 0;
    while (i < 8) begin
      sx = {sx, " "};
      sx[i] = SRC[i];
      ++i;
    end
    return sx;
  endfunction
  function string g();   // implicit-return variant, exactly uvm_packer
    int i;
    g = "";
    i = 0;
    while (i < 8) begin
      g = {g, " "};
      g[i] = SRC[i];
      ++i;
    end
  endfunction
  initial begin
    string a;
    a = f();
    $display("TAG1 %0s '%s'", (a == "zPU% |Ko") ? "PASS" : "FAIL", a);
    a = g();
    $display("TAG2 %0s '%s'", (a == "zPU% |Ko") ? "PASS" : "FAIL", a);
  end
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    let out = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n");
    for tag in ["TAG1", "TAG2"] {
        let line = out
            .lines()
            .find(|l| l.starts_with(tag))
            .unwrap_or_else(|| panic!("no {tag} line:\n{out}"));
        assert!(
            line.contains("PASS"),
            "expected {tag} PASS, got:\n{line}\n\nfull:\n{out}"
        );
    }
}
