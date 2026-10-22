//! §11.6.1 / §11.8.2: `==` is a SIGNED comparison only when BOTH operands are
//! signed. If either is unsigned the propagated type is unsigned and the
//! narrower operand is ZERO-extended to the common width — extending it by its
//! OWN signedness instead made a negative-looking narrow signed value compare
//! unequal to the same bit pattern in a wider unsigned operand:
//!
//! ```systemverilog
//! byte b = 8'hFE;      // signed, -2
//! b == 32'hFE          // must be TRUE  (b zero-extends to 0x000000FE)
//! b == 32'hFFFFFFFE    // must be FALSE (b does NOT sign-extend)
//! b == -2              // must be TRUE  (both signed -> signed compare)
//! ```
//!
//! Every expectation below is reference-simulator verified. The same rule
//! lives twice — `Value::is_equal` and the VM fast path `vm_is_equal` — and
//! `vm_fastpath_tests::binary_ops_match_value_methods` pins them together.

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
fn narrow_signed_vs_wider_unsigned_zero_extends() {
    let out = msgs(
        r#"
module top;
  byte     b = 8'hFE;
  shortint s = 16'hFFFE;
  initial begin
    $display("A_%b", b == 32'hFE);
    $display("B_%b", b == 32'hFFFFFFFE);
    $display("C_%b", b == -2);
    $display("D_%b", s == 32'hFFFE);
    $display("E_%b", s == 32'hFFFFFFFE);
    $display("F_%b", b != 32'hFE);
  end
endmodule
"#,
    );
    assert!(out.contains(&"A_1".to_string()), "{out:?}"); // zero-extended
    assert!(out.contains(&"B_0".to_string()), "{out:?}"); // NOT sign-extended
    assert!(out.contains(&"C_1".to_string()), "{out:?}"); // both signed
    assert!(out.contains(&"D_1".to_string()), "{out:?}");
    assert!(out.contains(&"E_0".to_string()), "{out:?}");
    assert!(out.contains(&"F_0".to_string()), "{out:?}"); // != agrees with ==
}

#[test]
fn context_sized_constant_still_sign_extends() {
    // A constant's width is context-determined, so `-1` widens to the
    // expression width as a signed value and matches an all-ones unsigned
    // operand. Guards the fix above from over-reaching.
    let out = msgs(
        r#"
module top;
  logic [63:0] u64 = 64'hFFFFFFFFFFFFFFFF;
  logic [31:0] u32 = 32'hFFFFFFFF;
  time         t   = -1;
  initial begin
    $display("G_%b", u64 == -1);
    $display("H_%b", u32 == -1);
    $display("I_%b", t == -1);
  end
endmodule
"#,
    );
    assert!(out.contains(&"G_1".to_string()), "{out:?}");
    assert!(out.contains(&"H_1".to_string()), "{out:?}");
    assert!(out.contains(&"I_1".to_string()), "{out:?}");
}

#[test]
fn unsized_based_literal_keeps_its_32_bit_self_determined_width() {
    // The bug above was previously "fixed" by shrinking unsized based literals
    // to their natural width, which broke self-determined contexts. `'hfe` is
    // 32 bits wide on its own; only the COMPARISON rule needed changing.
    let out = msgs(
        r#"
module top;
  byte b = 'hfe;
  logic [31:0] r;
  initial begin
    $display("J_%0d", $bits('hfe));
    r = 'hfe << 24;  $display("K_%0h", r);
    r = ~'hfe;       $display("L_%0h", r);
    $display("M_%b", b == 'hfe);
  end
endmodule
"#,
    );
    assert!(out.contains(&"J_32".to_string()), "{out:?}");
    assert!(out.contains(&"K_fe000000".to_string()), "{out:?}");
    assert!(out.contains(&"L_ffffff01".to_string()), "{out:?}");
    assert!(out.contains(&"M_1".to_string()), "{out:?}");
}
