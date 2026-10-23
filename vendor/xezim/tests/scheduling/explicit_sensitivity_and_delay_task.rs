//! Two silent-corruption bugs found while debugging a vendor register-file
//! cell model whose self-checking testbench read back all-X.
//!
//! 1. §9.4.2 — an explicit `always @(trig)` fired on signals its body merely
//!    READS. All-level explicit lists were routed to the comb-settle path,
//!    which discards the `@()` list and re-derives sensitivity from the read
//!    set. A timing-check notifier block (`always @(notifier) ... else if
//!    (flag == 0) corrupt_memory;`) therefore re-ran on every change of the
//!    signals in its body and clobbered the memory with X.
//!    A list carrying a select (`@(v[idx])`) still needs the read-set path so
//!    it re-evaluates when the INDEX changes — see tests/prtest/pr2011429.v.
//!
//! 2. §13.3 — a parenless task enable was silently DROPPED when the task body
//!    reached a `#delay` (even indirectly, through another task). The
//!    synchronous statement path does implement `#delay`; only genuinely
//!    suspending constructs (event control, `wait`, `fork…join`, `forever`)
//!    cannot run there. The cell enabled its memory-write task from an
//!    `always @(wclk)` block and that task reached a `#0` two levels down, so
//!    every write was discarded.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// The notifier-block shape: the trigger reg never changes after init, while
/// the signals read in the body do. Only the listed `trig` may fire the block.
const UNLISTED_READS: &str = r#"
module leafcell (output reg q);
  initial q = 1'b0;
endmodule

module tb;
  reg trig, sa, sb, sc;
  reg [7:0] corrupt_count;
  leafcell u_leaf ();

  task corrupt;
    begin corrupt_count = corrupt_count + 8'h01; end
  endtask

  initial corrupt_count = 8'h00;

  always @(trig) begin
    if (sa && !sb) begin
      u_leaf.q <= 1'bx;
    end else begin
      if (sc == 1'b0) begin
        corrupt;
      end
    end
  end

  initial begin
    trig = 1'b0; sa = 1'b0; sb = 1'b0; sc = 1'b0;
    #1 sa = 1'b1;   // unlisted read -> must NOT fire
    #1 sb = 1'b1;   // unlisted read -> must NOT fire
    #1 sc = 1'b1;   // unlisted read -> must NOT fire
    #1 trig = 1'b1; // listed -> fires, but sa&&!sb is 0 and sc is 1: no bump
    #1;
  end
endmodule
"#;

#[test]
fn explicit_sensitivity_ignores_unlisted_body_reads() {
    let sim = simulate(UNLISTED_READS, 100).expect("simulate failed");
    // Exactly one bump: the t=0 X->0 initialisation of `trig`.
    assert_eq!(get(&sim, "corrupt_count") & 0xFF, 1);
}

/// A parenless enable of a task that reaches a `#0` through a second task must
/// still execute; it used to be dropped whole.
const DELAY_TASK_ENABLE: &str = r#"
module tb;
  reg wclk;
  reg [7:0] witness;

  initial begin
    witness = 8'h00;
    wclk    = 1'b0;
  end

  task inner_with_delay;
    begin
      #0 witness = 8'hA5;
    end
  endtask

  task outer_enable;
    begin
      inner_with_delay;   // transitively reaches a #0
    end
  endtask

  always @(wclk) begin
    outer_enable;         // parenless enable
  end

  initial begin
    #1 wclk = 1'b1;
    #2;
  end
endmodule
"#;

#[test]
fn parenless_enable_of_delay_only_task_runs() {
    let sim = simulate(DELAY_TASK_ENABLE, 100).expect("simulate failed");
    assert_eq!(get(&sim, "witness") & 0xFF, 0xA5);
}

/// An edge block that produces a clock edge inside its own `#0` window. The
/// delay re-enters the edge machinery through the synchronous
/// `TimingControl::Delay` arm; the downstream flop must still see the posedge.
///
/// The edge pass used to `mem::take` the block list, so a nested pass ran
/// against an EMPTY list: every block index failed the bounds check and was
/// dropped silently, while the prev-value snapshot still advanced — consuming
/// the edge so no pass ever delivered it. On a vendor register-file cell this
/// killed all 20 read-data flops (0 executions vs 1680 in a reference
/// simulator) and the whole memory read back X.
const NESTED_EDGE_CLOCK: &str = r#"
module tb;
  reg trig, clk_int;
  reg [7:0] fires;
  initial begin fires = 8'h00; clk_int = 1'b0; trig = 1'b0; end

  always @(posedge trig) begin
    clk_int = 1'b0;
    #0 clk_int = 1'b1;     // posedge made DURING a nested delay window
  end

  always @(posedge clk_int) begin
    fires = fires + 8'h01;
  end

  initial begin
    #1 trig = 1'b1;
    #1 trig = 1'b0;
    #1 trig = 1'b1;
    #1;
  end
endmodule
"#;

#[test]
fn edge_made_inside_nested_delay_is_delivered() {
    let sim = simulate(NESTED_EDGE_CLOCK, 100).expect("simulate failed");
    assert_eq!(get(&sim, "fires") & 0xFF, 2);
}

/// §9.4.2 — `always @(v[0])` must wake only on bit 0. Edge sensitivity is
/// tracked per SIGNAL, so a block watching `datain[0]` also woke when
/// `datain[3]` changed. A constant bit-select term now narrows the wake to its
/// own bit; a non-constant index (`@(v[i])`) keeps whole-signal sensitivity,
/// since that block must also wake when the INDEX moves.
const BITSEL_SENSITIVITY: &str = r#"
module tb;
  reg [3:0] datain;
  reg [7:0] hits_b0, hits_b3;

  initial begin datain = 4'b0000; hits_b0 = 8'h00; hits_b3 = 8'h00; end

  always @(datain[0]) hits_b0 = hits_b0 + 8'h01;
  always @(datain[3]) hits_b3 = hits_b3 + 8'h01;

  initial begin
    #1 datain[3] = 1'b1;   // only bit 3 -> hits_b3 only
    #1 datain[3] = 1'b0;   // only bit 3 -> hits_b3 only
    #1 datain[0] = 1'b1;   // only bit 0 -> hits_b0 only
    #1;
  end
endmodule
"#;

#[test]
fn bit_select_sensitivity_wakes_only_on_its_own_bit() {
    let sim = simulate(BITSEL_SENSITIVITY, 100).expect("simulate failed");
    // One init fire (x->0) + one real change on bit 0.
    assert_eq!(get(&sim, "hits_b0") & 0xFF, 2);
    // One init fire + two real changes on bit 3.
    assert_eq!(get(&sim, "hits_b3") & 0xFF, 3);
}
