//! §9.4.2 — `always @(posedge <expression>)` where the term is COMPUTED.
//! Reference-validated.
//!
//! Sensitivity was built from the identifier names appearing in the event
//! expression, so a computed term (`a & 1'b1`, `~b`, `c ? x : y`) contributed
//! no names at all: the block armed on nothing and never ran, with no
//! diagnostic. A plain signal and a bit-select both worked, which is what made
//! it look like a corner case.
//!
//! Sensitizing the leaf identifiers instead would be wrong, not merely
//! incomplete: `posedge (~b)` must fire when `~b` RISES — that is, when `b`
//! FALLS — so the edge has to be evaluated against the expression's own value.
//! Each computed term now gets a hidden 1-bit net driven by a continuous
//! assign, and the edge is placed on that net. Truncating it to one bit also
//! gives the LSB tracking §9.4.2 specifies for a multi-bit expression.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The originally reported shapes, against a plain signal and a bit-select
/// control.
#[test]
fn computed_edge_terms_fire() {
    let src = r#"
module tb;
  logic a, b;
  logic [1:0] w;
  int n_plain, n_and, n_or, n_not, n_bit;
  always @(posedge a)          n_plain++;
  always @(posedge (a & 1'b1)) n_and++;
  always @(posedge (a | 1'b0)) n_or++;
  always @(posedge (~b))       n_not++;
  always @(posedge w[0])       n_bit++;
  initial begin
    a = 0; b = 1; w = 2'b00;
    n_plain=0; n_and=0; n_or=0; n_not=0; n_bit=0;
    #1 a = 1;
    #1 b = 0;
    #1 w[0] = 1;
    #1;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "n_plain"), 1, "a plain signal still works");
    assert_eq!(u(&sim, "n_bit"), 1, "a bit-select still works");
    assert_eq!(u(&sim, "n_and"), 1, "AND term");
    assert_eq!(u(&sim, "n_or"), 1, "OR term");
    assert_eq!(
        u(&sim, "n_not"),
        1,
        "an inversion fires when the OPERAND falls"
    );
}

/// Polarity, multi-bit LSB tracking, a mixed plain/computed list, an `iff`
/// guard, and a conditional term — and no spurious fires.
#[test]
fn computed_edge_polarity_vectors_and_iff() {
    let src = r#"
module tb;
  logic a, b, en;
  logic [3:0] v;
  int n_not, n_and2, n_neg, n_vec, n_mixed, n_iff, n_cond;
  always @(posedge (~b))              n_not++;
  always @(posedge (a & b))           n_and2++;
  always @(negedge (a | b))           n_neg++;
  always @(posedge (v + 4'd1))        n_vec++;
  always @(posedge a or posedge (~b)) n_mixed++;
  always @(posedge (a & b) iff en)    n_iff++;
  always @(posedge (en ? a : b))      n_cond++;
  initial begin
    a = 0; b = 1; en = 0; v = 4'd0;
    n_not=0; n_and2=0; n_neg=0; n_vec=0; n_mixed=0; n_iff=0; n_cond=0;
    #1 b = 0;
    #1 a = 1;
    #1 b = 1;
    #1 en = 1;
    #1 a = 0;
    #1 a = 1;
    #1 v = 4'd1;
    #1 v = 4'd2;
    #1;
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    // Counts are taken from the reference, not hand-derived: a term's net also
    // takes an x->1 transition when it first settles, which is itself a posedge.
    assert_eq!(u(&sim, "n_not"), 1, "negation fires when its OPERAND falls");
    assert_eq!(u(&sim, "n_and2"), 2, "AND of two varying operands");
    assert_eq!(u(&sim, "n_neg"), 1, "negedge of a computed term");
    assert_eq!(u(&sim, "n_vec"), 2, "a multi-bit term tracks its LSB");
    assert_eq!(
        u(&sim, "n_mixed"),
        3,
        "plain and computed terms in one list"
    );
    assert_eq!(
        u(&sim, "n_iff"),
        1,
        "the iff guard still gates the computed term"
    );
    assert_eq!(u(&sim, "n_cond"), 3, "a conditional term");
}

/// A computed edge in a MID-BLOCK event control — inside a fork arm, a loop, or
/// a branch — not just an always header. The rewrite originally covered only
/// the header, so every one of these was still armed on nothing.
#[test]
fn computed_edge_in_a_mid_block_event_control() {
    let src = r#"
module tb;
  logic a, b, c;
  int t_plain, t_and, t_not, t_loop, t_if;
  initial begin
    a = 0; b = 1; c = 0;
    t_plain = -1; t_and = -1; t_not = -1; t_loop = -1; t_if = -1;
    fork
      begin @(posedge a);          t_plain = $time; end
      begin @(posedge (a & 1'b1)); t_and   = $time; end
      begin @(posedge (~b));       t_not   = $time; end
      begin
        for (int i = 0; i < 1; i++) begin
          @(posedge (a | 1'b0));   t_loop = $time;
        end
      end
      begin
        if (1) begin @(posedge (c ^ 1'b0)); t_if = $time; end
      end
    join_none
    #1 a = 1;
    #1 b = 0;
    #1 c = 1;
    #1;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "t_plain"), 1, "plain signal");
    assert_eq!(u(&sim, "t_and"), 1, "computed term in a fork arm");
    assert_eq!(
        u(&sim, "t_not"),
        2,
        "inversion fires when its operand falls"
    );
    assert_eq!(u(&sim, "t_loop"), 1, "computed term inside a loop body");
    assert_eq!(u(&sim, "t_if"), 3, "computed term inside a branch");
}
