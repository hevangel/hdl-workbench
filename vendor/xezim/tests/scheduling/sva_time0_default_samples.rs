//! IEEE 1800-2023 §16.5.1: the sampled value of a variable in time slot 0
//! is its default sampled value: its declaration initializer, else its
//! type's default (x for a four-state variable, 0 for `int`). A clock that
//! rises at time 0 starts an attempt there (§9.4.2), and that attempt reads
//! these values, not what the slot's continuous assignments, `always_comb`
//! blocks and `initial` blocks write; a net has not been driven yet either.
//! §16.9.3: `$past` at the next tick returns the time-0 sample. xezim took
//! the time-0 samples after the time-0 settle, so the operands of continuous
//! assignments and `always_comb` read their driven values and those attempts
//! passed.

use xezim::simulate;

const SRC: &str = r#"
module top;
  logic clk;
  logic r;                         // never written: x
  logic ri = 1'b1;                 // declaration initializer
  int   n;                         // 2-state default: 0
  wire  w = 1'b1;                  // net declaration assignment
  logic rn = 1'b0;
  logic c; assign c = ~rn;         // continuous assignment
  logic g; always_comb g = ri;     // always_comb
  logic f; initial f = 1'b1;       // initial block
  logic m [2]; always_comb m[0] = ri;  // an array cell
  initial begin clk = 1; forever #5 clk = ~clk; end
  a_r:  assert property (@(posedge clk) r === 1'bx) $display("T|r|pass|%0t", $time); else $display("T|r|fail|%0t", $time);
  a_ri: assert property (@(posedge clk) ri === 1'b1) $display("T|ri|pass|%0t", $time); else $display("T|ri|fail|%0t", $time);
  a_n:  assert property (@(posedge clk) n === 0) $display("T|n|pass|%0t", $time); else $display("T|n|fail|%0t", $time);
  a_w:  assert property (@(posedge clk) w === 1'b1) $display("T|w|pass|%0t", $time); else $display("T|w|fail|%0t", $time);
  a_c:  assert property (@(posedge clk) c === 1'b1) $display("T|c|pass|%0t", $time); else $display("T|c|fail|%0t", $time);
  a_g:  assert property (@(posedge clk) g === 1'b1) $display("T|g|pass|%0t", $time); else $display("T|g|fail|%0t", $time);
  a_f:  assert property (@(posedge clk) f === 1'b1) $display("T|f|pass|%0t", $time); else $display("T|f|fail|%0t", $time);
  a_m:  assert property (@(posedge clk) m[0] === 1'b1) $display("T|m|pass|%0t", $time); else $display("T|m|fail|%0t", $time);
  // At 10 ns, $past reads the time-0 samples.
  a_p:  assert property (@(posedge clk) $time != 10 || 0)
          else $display("T|past|%0t|%b%b", $time, $past(c), $past(ri));
  initial #12 $finish;
endmodule
"#;

#[test]
fn time0_attempt_reads_default_sampled_values() {
    let sim = simulate(SRC, 100).expect("simulation failed");
    let mut got: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("T|"))
        .collect();
    got.sort();
    let mut want = vec![
        "T|c|fail|0",
        "T|c|pass|10",
        "T|f|fail|0",
        "T|f|pass|10",
        "T|g|fail|0",
        "T|g|pass|10",
        "T|m|fail|0",
        "T|m|pass|10",
        "T|n|pass|0",
        "T|n|pass|10",
        "T|past|10|x1",
        "T|r|pass|0",
        "T|r|pass|10",
        "T|ri|pass|0",
        "T|ri|pass|10",
        "T|w|fail|0",
        "T|w|pass|10",
    ];
    want.sort();
    assert_eq!(got, want);
}
