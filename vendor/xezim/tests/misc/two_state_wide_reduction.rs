//! §11.4.9 reduction operators over WIDE (>64-bit) operands in the two-state
//! path.
//!
//! Registers wider than 64 bits live in the wide plane file (`wregs`), not
//! the u64 file. `ReduceOr` lowered to the narrow `RedOr` regardless of
//! source width, so `|wide_bus` read `regs[s]` — a slot the wide load never
//! wrote, i.e. whatever the PREVIOUS block's evaluation left there (zero in
//! a fresh file). On the C910 SoC the first `|128-bit` control reduction on
//! the LSU store path evaluated to the wrong value and the core wedged after
//! exactly 250 retired instructions, with the AXI bus idle.
//!
//! The values below are deliberately chosen so a stale-or-zero narrow read
//! cannot accidentally produce the right answer: the interesting bits sit
//! ONLY in the high word.

use xezim::simulate;

fn msgs(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn reduce_or_high_word_only() {
    let out = msgs(
        r#"
module top;
  logic [127:0] v;
  logic y;
  always_comb y = |v;
  initial begin
    v = 128'h0;          #1 $display("A_%b", y);
    v = 128'h1 << 100;   #1 $display("B_%b", y);  // ONLY the high word is set
    v = 128'h1;          #1 $display("C_%b", y);  // only the low word
    v = 128'h0;          #1 $display("D_%b", y);
  end
endmodule
"#,
    );
    assert!(out.contains(&"A_0".to_string()), "{out:?}");
    assert!(out.contains(&"B_1".to_string()), "{out:?}");
    assert!(out.contains(&"C_1".to_string()), "{out:?}");
    assert!(out.contains(&"D_0".to_string()), "{out:?}");
}

#[test]
fn reduce_and_wide() {
    let out = msgs(
        r#"
module top;
  logic [99:0] v;   // non-word-aligned width: the high-word mask matters
  logic y;
  always_comb y = &v;
  initial begin
    v = {100{1'b1}};              #1 $display("E_%b", y);
    v = {100{1'b1}} & ~(100'h1 << 90); #1 $display("F_%b", y); // one 0 in the high word
    v = {100{1'b1}} & ~100'h1;    #1 $display("G_%b", y);      // one 0 in the low word
    v = {100{1'b1}};              #1 $display("H_%b", y);
  end
endmodule
"#,
    );
    assert!(out.contains(&"E_1".to_string()), "{out:?}");
    assert!(out.contains(&"F_0".to_string()), "{out:?}");
    assert!(out.contains(&"G_0".to_string()), "{out:?}");
    assert!(out.contains(&"H_1".to_string()), "{out:?}");
}
