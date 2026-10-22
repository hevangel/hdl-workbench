//! §6.24.1 / §7.10.2 — `void'(q.pop_front())` as a statement inside a
//! compiled always block. Reference-validated.
//!
//! The parser lowers `void'(expr)` to `Paren(expr)` (the cast is a pure
//! discard). The bytecode compiler's expression-statement arm treated any
//! parenthesised expression as side-effect-free and compiled it to a NO-OP,
//! so the pop never ran: the queue kept its length and every later `q[0]`
//! read the same head. The bare `q.pop_front();` form fell through to the
//! AST fallback and worked, as did the same void'() statement in an initial
//! block — which is exactly how the difference stayed hidden.
//!
//! Field symptom: a scoreboard comparing `dut_data == expected[0]` with
//! `void'(expected.pop_front())` matched transaction 1 forever and reported
//! every later transaction as a mismatch.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
module tb;
  logic clk = 0;
  always #5 clk = ~clk;

  longint unsigned q[$];
  longint unsigned h1, h2, h3;
  int sz_after, done;

  always @(posedge clk) begin
    if (!done) begin
      h1 = q[0];
      void'(q.pop_front());
      h2 = q[0];               // must see the NEW head, same activation
      void'(q.pop_front());
      h3 = q[0];
      done = 1;
    end
  end

  initial begin
    done = 0;
    q.push_back(64'h11);
    q.push_back(64'h22);
    q.push_back(64'h33);
    repeat (2) @(posedge clk);
    #1;
    sz_after = q.size();
  end
endmodule
"#;

#[test]
fn void_cast_pop_front_executes_in_always_block() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "h1"), 0x11, "head before any pop");
    assert_eq!(
        u(&sim, "h2"),
        0x22,
        "head after first void'(pop_front) — stale 0x11 means the pop was dropped"
    );
    assert_eq!(u(&sim, "h3"), 0x33, "head after second pop");
    assert_eq!(
        u(&sim, "sz_after"),
        1,
        "two pops must actually shrink the queue"
    );
}
