//! §6.21 / §25.8: a `static` subroutine local is one persistent cell per
//! DECLARING subroutine instance, and a non-blocking write to it must land in
//! that cell.
//!
//! Two things went wrong together for an interface task:
//!
//!  * the `d == 0` NBA path resolved its target with `resolve_nba_target`
//!    BEFORE any static-local check, and that found a bare same-named SIGNAL —
//!    which every instance of the interface shares;
//!  * `assign_value`'s static redirect resolves the persistent key through
//!    `static_local_key_for`, which reads the CURRENT call frame. That is right
//!    while the statement executes and wrong in the NBA region, where the
//!    declaring frame is no longer on top.
//!
//! Two instances of one interface, each running `task Gen; static int count;
//! … count <= count + 1;`, therefore fought over a single cell: one instance's
//! counter never advanced at all while the other advanced at double rate. A
//! divider written that way never reaches its threshold, so the clock it drives
//! never toggles — the failure looks like a dead clock, not a shared variable.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

/// Each interface instance keeps its own `static` counter, so both dividers
/// toggle at the same rate.
#[test]
fn static_task_local_is_per_interface_instance() {
    let src = r#"
`timescale 1ns/1ps
interface divider_if(input bit clk);
  bit tog;
  task Gen(input int div);
    static int count = 0;
    forever begin
      @(posedge clk or negedge clk)
      if (count == (div-1)) begin
        count <= 0;
        tog   <= ~tog;
      end else begin
        count <= count + 1;
      end
    end
  endtask
endinterface
module top;
  bit clk = 0;
  always #5 clk = ~clk;
  divider_if u_a(clk);
  divider_if u_b(clk);
  int ta = 0, tb = 0;
  always @(posedge u_a.tog) ta++;
  always @(posedge u_b.tog) tb++;
  initial begin
    fork begin u_a.Gen(4); end join_none
    fork begin u_b.Gen(4); end join_none
    #500;
    $display("NOTE: a=%0d b=%0d", ta, tb);
    $finish;
  end
endmodule
"#;
    // One shared cell gave a=25 b=0 (or a=0 b=25, depending on process order).
    assert_eq!(notes(src), vec!["NOTE: a=12 b=12"]);
}

/// A `static` local still PERSISTS across calls of its own subroutine — the fix
/// must not turn it into an automatic.
#[test]
fn static_task_local_still_persists_across_calls() {
    let src = r#"
`timescale 1ns/1ps
module top;
  bit clk = 0;
  always #5 clk = ~clk;
  task Step();
    static int seen = 0;
    @(posedge clk);
    seen <= seen + 1;
    @(posedge clk);
    $display("NOTE: seen=%0d", seen);
  endtask
  initial begin
    Step();
    Step();
    Step();
    $finish;
  end
endmodule
"#;
    assert_eq!(
        notes(src),
        vec!["NOTE: seen=1", "NOTE: seen=2", "NOTE: seen=3"],
        "a static local must carry its value into the next call"
    );
}

/// An automatic local of the same shape keeps working — it was never broken,
/// and the new static path must not capture it.
#[test]
fn automatic_task_local_is_unaffected() {
    let src = r#"
`timescale 1ns/1ps
interface auto_if(input bit clk);
  bit tog;
  task Gen(input int div);
    int count = 0;
    forever begin
      @(posedge clk or negedge clk)
      if (count == (div-1)) begin count <= 0; tog <= ~tog; end
      else begin count <= count + 1; end
    end
  endtask
endinterface
module top;
  bit clk = 0;
  always #5 clk = ~clk;
  auto_if u_a(clk);
  auto_if u_b(clk);
  int ta = 0, tb = 0;
  always @(posedge u_a.tog) ta++;
  always @(posedge u_b.tog) tb++;
  initial begin
    fork begin u_a.Gen(4); end join_none
    fork begin u_b.Gen(4); end join_none
    #500;
    $display("NOTE: a=%0d b=%0d", ta, tb);
    $finish;
  end
endmodule
"#;
    assert_eq!(notes(src), vec!["NOTE: a=12 b=12"]);
}
