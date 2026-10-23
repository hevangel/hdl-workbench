//! `assign {a,b} = ({N{hit}} & decode(w)) | ...` — the classic way-select
//! decode idiom — must COMPILE to bytecode, and the compiled path must be
//! bit-exact with the interpreter, X semantics included.
//!
//! Two one-line omissions kept it interpreted: neither continuous-assign
//! compile site handed the compiler the function table (so `compile_pure_call`
//! could not even look up the callee → `Expr_Call` bail), and the inliner's
//! purity walker had no `case` arm, branding every casez decode helper impure
//! (`Expr_Call_impure`). On a design-scaled MWE of a real profile hotspot the
//! two together were worth ~50x on top of the metadata-index fix; the user's
//! design spent 26.7s per assign per 180ns of sim on this shape.
//!
//! The compile is pinned through `XEZIM_COMPILE_FAIL_STATS`: a regression that
//! silently falls back to the (correct, slow) interpreter would pass a pure
//! value check, so the test also asserts no `Expr_Call*` failure is counted.

use std::process::Command;

const SRC: &str = r#"
`timescale 1ps/1ps
module tb;
  logic h0, h1;
  logic [31:0] w0, w1;
  logic [5:0] racyc; logic [2:0] rbcyc;
  logic [8:0] single;
  assign {racyc,rbcyc} = ({9{h0}} & rxcyc(w0)) | ({9{h1}} & rxcyc(w1));
  assign single = rxcyc(w0);
  function [8:0] rxcyc;
      input [31:0] inst;
      reg [5:0] ra; reg [2:0] rb;
      begin
         casez (inst)
           32'b0000_00??_????_????_????_????_????_????: begin ra = 6'd5;  rb = 3'd0; end
           32'b0001_????_????_????_????_????_????_????: begin ra = 6'd17; rb = 3'd0; end
           32'b0010_????_????_????_????_????_????_????: begin ra = inst[21:16]; rb = 3'd0; end
           default: begin ra = 6'bx; rb = 3'bx; end
         endcase
         rxcyc = {ra,rb};
      end
  endfunction
  initial begin
    h0 = 0; h1 = 1; w0 = 32'h0; w1 = 32'h1000_0000;
    #1 $display("NOTE: sel1 %0d %0d %b", racyc, rbcyc, single);
    h0 = 1; h1 = 0; w0 = 32'h202A_0000;
    #1 $display("NOTE: sel0 %0d %0d", racyc, rbcyc);
    h0 = 1'bx;
    #1 $display("NOTE: selx %b", {racyc, rbcyc});
    $finish;
  end
endmodule
"#;

#[test]
fn decode_helper_assign_compiles_and_is_bit_exact() {
    let mut sv = std::env::temp_dir();
    sv.push(format!("xezim_decode_{}.sv", std::process::id()));
    std::fs::write(&sv, SRC).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .env("XEZIM_COMPILE_FAIL_STATS", "1")
        .args(["--no-cache", "-s", "tb"])
        .arg(&sv)
        .output()
        .expect("run xezim");
    let _ = std::fs::remove_file(&sv);
    assert!(out.status.success(), "xezim failed: {:?}", out);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Values: way select, variable count field from the formal, X mask.
    assert!(stdout.contains("NOTE: sel1 17 0 000101000"), "{stdout}");
    assert!(stdout.contains("NOTE: sel0 42 0"), "{stdout}");
    // w0 was changed to 202A_0000 at sel0, so rxcyc(w0) is 101010_000 here;
    // {9{x}} & 101010000 -> only the 1-bits go x.
    assert!(stdout.contains("NOTE: selx x0x0x0000"), "{stdout}");

    // And it genuinely compiled: no continuous-assign call bail was counted.
    assert!(
        !stderr.contains("Expr_Call"),
        "decode assign fell back to the interpreter:\n{stderr}"
    );
}
