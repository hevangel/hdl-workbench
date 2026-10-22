//! A block-local declaration inside a COMPILED always block must stay visible
//! to any statement in that block that falls back to the AST interpreter.
//!
//! `compile_stmt_strict`'s `VarDecl` arm gives a block-local variable a VM
//! REGISTER. A statement the bytecode compiler cannot handle instead becomes an
//! `Insn::StmtFallback`, re-run by the interpreter — which never executed the
//! declaration and so has no storage for that name at all. The two halves of
//! one block then read two different variables: the compiled statements see the
//! register, the fallback sees a non-existent signal (x).
//!
//! The visible damage is a whole-object read. Member WRITES auto-vivify
//! per-member interpreter storage, so `item.f = 1` followed by
//! `$display(item.f)` looked correct — but `q.push_back(item)`, which reads the
//! object as a whole, pushed an all-x copy. A queue fed that way never
//! satisfies its drain condition, so a design accepts traffic forever and
//! completes none of it.
//!
//! `reg_var_loop_depth` already guarded the same failure for a register-backed
//! FOR-LOOP counter; this is the block-local-declaration case. Both resolve the
//! same way: refuse the per-statement fallback so the block rolls back to one
//! AST-interpreted unit.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// The distilled case: a block-local scalar written by a compiled statement and
/// read by a fallback statement in the same block.
#[test]
fn block_local_scalar_survives_a_fallback_read() {
    let src = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  int bag[$];
  always_ff @(posedge clk) begin
    integer local_v;
    local_v = 42;
    bag.push_back(local_v);
    $display("NOTE: local=%0d head=%0d", local_v, bag[0]);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: local=42 head=42"]);
}

/// The shape that actually broke: a block-local STRUCT filled member-by-member
/// and then pushed whole. Member reads always looked right; the whole-object
/// read was x.
#[test]
fn block_local_struct_pushes_its_real_value() {
    let src = r#"
module top;
  typedef struct {
    logic [15:0] tag;
    integer      countdown;
  } entry_t;
  logic clk = 0;
  always #5 clk = ~clk;
  entry_t bag[$];
  always_ff @(posedge clk) begin
    entry_t entry;
    entry.tag = 16'h00A5;
    entry.countdown = 3;
    bag.push_back(entry);
    $display("NOTE: tag=%0h countdown=%0d", bag[0].tag, bag[0].countdown);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: tag=a5 countdown=3"]);
}

/// A queue fed from a block-local struct must actually DRAIN — the end-to-end
/// symptom was traffic accepted forever and none completed.
#[test]
fn a_queue_fed_from_a_block_local_struct_drains() {
    let src = r#"
module top;
  typedef struct {
    logic [7:0] tag;
    integer     countdown;
  } entry_t;
  logic clk = 0;
  always #5 clk = ~clk;
  entry_t bag[$];
  int drained = 0;
  int pushed  = 0;
  always_ff @(posedge clk) begin
    if (pushed < 4) begin
      entry_t entry;
      entry.tag = pushed;
      entry.countdown = 2;
      bag.push_back(entry);
      pushed++;
    end
    for (int k = 0; k < bag.size(); k++) bag[k].countdown--;
    if (bag.size() > 0 && bag[0].countdown <= 0) begin
      drained++;
      void'(bag.pop_front());
    end
  end
  initial begin
    #400;
    $display("NOTE: pushed=%0d drained=%0d left=%0d", pushed, drained, bag.size());
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: pushed=4 drained=4 left=0"]);
}

/// The guard must not disturb a block with no block-local declaration — that
/// block still compiles and may still use per-statement fallback.
#[test]
fn a_block_without_locals_is_unaffected() {
    let src = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  int bag[$];
  int seed = 7;
  always_ff @(posedge clk) begin
    bag.push_back(seed);
    $display("NOTE: head=%0d", bag[0]);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: head=7"]);
}
