//! §11.5.1 zero-width indexed part-select: the check must only fire when the
//! width really is a resolvable constant.
//!
//! `const_eval_i64_with_params` answers `Some(0)` for references it cannot
//! actually resolve — a struct-typed parameter member (`Cfg.NrEntries`)
//! routes through `eval_const_expr_val`, whose Value defaults to zero. Fed
//! straight into the check, that read as "width 0" and REJECTED legal RTL:
//! every cva6 (4 configs) and black-parrot (6 configs) elaboration in
//! sv-tests failed with 66 of these on `[base +: $clog2(CVA6Cfg.<field>)]`,
//! which cannot be resolved without instance context (the parameter is
//! registered per instance, e.g. `u.Cfg`). The width is now trusted only
//! when every leaf resolves; a genuinely constant zero still errors.

use xezim::simulate;

const LEGAL: &str = r#"
package cfg_pkg;
  typedef struct packed {
    int unsigned NrEntries;
    int unsigned DataWidth;
  } cfg_t;
  localparam cfg_t DefaultCfg = '{ NrEntries: 16, DataWidth: 32 };
endpackage

module sub import cfg_pkg::*; #(parameter cfg_t Cfg = cfg_pkg::DefaultCfg) (
  input  logic [63:0] din,
  output logic [$clog2(Cfg.NrEntries)-1:0] idx
);
  assign idx = din[8 +: $clog2(Cfg.NrEntries)];
endmodule

module top;
  logic [63:0] din; logic [3:0] idx;
  sub u(din, idx);
  initial begin din = 64'hDEADBEEF12345678; #1 $display("NOTE: idx=%h", idx); $finish; end
endmodule
"#;

const ILLEGAL: &str = r#"
module top;
  localparam int W = 0;
  logic [31:0] d; logic [7:0] o;
  assign o = d[4 +: W];
  initial begin d = 32'h1234; #1 $display("NOTE: o=%h", o); $finish; end
endmodule
"#;

#[test]
fn struct_param_width_is_not_mistaken_for_zero() {
    let sim = simulate(LEGAL, 1_000_000).expect("legal design must elaborate");
    let notes: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect();
    // Reference simulator gives idx=6 for these inputs.
    assert_eq!(notes, ["NOTE: idx=6"]);
}

#[test]
fn genuinely_constant_zero_width_still_errors() {
    let err = simulate(ILLEGAL, 1_000_000)
        .err()
        .expect("a constant zero-width part-select must still be rejected");
    assert!(
        format!("{err}").contains("zero width"),
        "unexpected error text: {err}"
    );
}
