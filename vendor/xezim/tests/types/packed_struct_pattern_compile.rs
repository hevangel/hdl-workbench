//! §10.9.2: an assignment pattern applied to a PACKED-struct target is member
//! concatenation, first declared field in the MSBs.
//!
//! Patterns bailed the enclosing block to the AST interpreter wholesale (the
//! expression form is context-free — its meaning depends on the target type,
//! which compile_expr cannot see). The assign arms now install the
//! destination's field layout around the rvalue compile, so named, default-
//! filled and ordered patterns onto a packed struct compile to a Concat with
//! each member evaluated at its OWN field width. ibex's CSR write logic
//! rebuilds mstatus_d/mcause_d this way every cycle — 11% of the 3M-cycle
//! bench ran interpreted for this alone. All four forms reference-verified.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module top;
  typedef struct packed {
    logic       mie;
    logic       mpie;
    logic [1:0] mpp;
    logic       mprv;
    logic       tw;
  } status_t;
  status_t s1, s2, s3, s4;
  logic clk = 0;
  always #5 clk = ~clk;
  logic a = 1, b = 0;
  always_comb s1 = '{mie: a, mpie: b, mpp: 2'b11, mprv: a, tw: b};
  always_comb s2 = '{mie: a, default: 1'b0};
  always_comb s3 = '{a, b, 2'b10, b, a};
  always_ff @(posedge clk) s4 <= '{mie: b, mpie: a, mpp: 2'b01, mprv: b, tw: a};
  initial begin
    #12;
    $display("NOTE: s1=%b", s1);
    $display("NOTE: s2=%b", s2);
    $display("NOTE: s3=%b", s3);
    $display("NOTE: s4=%b", s4);
    $finish;
  end
endmodule
"#;

#[test]
fn packed_struct_patterns_compile_and_match_reference() {
    assert_eq!(
        notes(SRC),
        vec![
            "NOTE: s1=101110",
            "NOTE: s2=100000",
            "NOTE: s3=101001",
            "NOTE: s4=010101",
        ]
    );
}
