//! §6.18 typedef-base and §7.4 array-dimension lints must use the
//! DEFINITION's own type namespace, not just the elaborated top's.
//!
//! A library module (possibly never instantiated) declaring
//! `parameter type req_data_t = logic;` and then
//! `typedef req_data_t wbuf_data_t; typedef wbuf_data_t ...` — the
//! hpdcache/cva6 and black-parrot idiom — was reported as "base type not
//! declared" on every link of the chain, because the check consulted only
//! `elab.typedefs` (which knows nothing about library-module locals). The
//! same unreliable const-eval also rejected array dimensions derived from
//! struct-parameter members as "size 0". After the fix all four cva6 and
//! all six black-parrot sv-tests configurations elaborate with zero errors.
//! Genuinely wrong declarations must still be rejected — both directions
//! are pinned here.

use xezim::simulate;

const LEGAL: &str = r#"
module cache_core #(
  parameter type req_data_t = logic,
  parameter type req_be_t   = logic
) (
  input  req_data_t din,
  output req_data_t dout
);
  typedef req_data_t wbuf_data_t;
  typedef wbuf_data_t wbuf_data_buf_t;
  typedef req_be_t  wbuf_be_t;
  wbuf_data_buf_t buffer_q;
  always_comb begin buffer_q = din; end
  assign dout = buffer_q;
endmodule

module top;
  logic [31:0] din, dout;
  cache_core #(.req_data_t(logic [31:0]), .req_be_t(logic [3:0])) u(din, dout);
  initial begin din = 32'hCAFE_F00D; #1 $display("NOTE: dout=%h", dout); $finish; end
endmodule
"#;

const ILLEGAL_TYPEDEF: &str = r#"
module top;
  typedef never_declared_t chained_t;
  chained_t x;
  initial $display("NOTE: %0d", x);
endmodule
"#;

const ILLEGAL_DIM: &str = r#"
module top;
  // Literal form: a parameter-valued zero dimension was never caught by this
  // lint even before the confidence gate (verified against the pre-gate
  // build), so the guard pins the case that has always fired.
  logic [7:0] mem [0];
  initial $display("NOTE: %0d", mem[0]);
endmodule
"#;

#[test]
fn typedef_chain_through_type_params_elaborates() {
    let sim = simulate(LEGAL, 1_000_000).expect("legal library-module typedef chain");
    let notes: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect();
    // Reference simulator prints the same value.
    assert_eq!(notes, ["NOTE: dout=cafef00d"]);
}

#[test]
fn undeclared_typedef_base_still_errors() {
    let err = simulate(ILLEGAL_TYPEDEF, 1_000_000)
        .err()
        .expect("undeclared base type must still be rejected");
    assert!(format!("{err}").contains("not declared"), "{err}");
}

#[test]
fn zero_array_dimension_still_errors() {
    let err = simulate(ILLEGAL_DIM, 1_000_000)
        .err()
        .expect("a zero array dimension must still be rejected");
    assert!(format!("{err}").contains("greater than zero"), "{err}");
}
