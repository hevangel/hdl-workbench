//! Time-0 activation order of `initial` / `always` processes across the
//! instance hierarchy. IEEE 1800 §4.4.2/§9.2 leave it unspecified; xezim
//! follows the reference simulator:
//!
//! * each top-level root runs as one block, roots in elaboration order;
//! * within a root, processes follow the depth-first source order of the
//!   instance tree (a child's processes at its instantiation, a bound
//!   instance's after all of its host's own items);
//! * except that the `initial` blocks that cannot suspend (no delay, event
//!   control, `wait` or blocking task call) run as one group, at the position
//!   of the first of them.
//!
//! xezim used to run every process-backed `always` first, then each inlined
//! instance's `initial` blocks, then the top module's — reordering an AVIP's
//! time-0 banner messages. Every expectation below was cross-checked against
//! the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

/// Nested instances: every `initial` runs where its instance is instantiated,
/// depth first — not the children after (or before) the parent.
#[test]
fn initials_follow_instance_source_order() {
    let src = r#"
module leaf #(parameter int ID = 0);
  initial $display("leaf%0d A", ID);
  always @* $display("leaf%0d never", ID);
  initial $display("leaf%0d B", ID);
endmodule
module mid #(parameter int ID = 0);
  initial $display("mid%0d 1", ID);
  leaf #(ID*10+1) l1();
  initial $display("mid%0d 2", ID);
  leaf #(ID*10+2) l2();
  initial $display("mid%0d 3", ID);
endmodule
module top;
  initial $display("top 1");
  mid #(1) m1();
  initial $display("top 2");
  mid #(2) m2();
  initial $display("top 3");
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "top 1", "mid1 1", "leaf11 A", "leaf11 B", "mid1 2", "leaf12 A", "leaf12 B", "mid1 3",
            "top 2", "mid2 1", "leaf21 A", "leaf21 B", "mid2 2", "leaf22 A", "leaf22 B", "mid2 3",
            "top 3",
        ]
    );
}

/// Interfaces, generate blocks and a bound instance (after all of its host's
/// own items); `always` blocks that start by printing follow the non-suspending
/// `initial` group.
#[test]
fn bound_generate_and_interface_instances() {
    let src = r#"
module leaf #(parameter int ID = 0);
  initial $display("leaf%0d init", ID);
  always begin $display("leaf%0d always", ID); #100; end
endmodule
interface ifc #(parameter int ID = 0);
  initial $display("ifc%0d init", ID);
endinterface
module mid #(parameter int ID = 0);
  always begin $display("mid%0d always-first", ID); #100; end
  initial $display("mid%0d 1", ID);
  leaf #(ID*10+1) l1();
  ifc #(ID*10+5) i1();
  initial $display("mid%0d 2", ID);
  generate if (1) begin : g
    initial $display("mid%0d gen", ID);
    leaf #(ID*10+7) lg();
  end endgenerate
  initial $display("mid%0d 3", ID);
endmodule
module bmod;
  initial $display("bound init");
endmodule
module top;
  initial $display("top 1");
  mid #(1) m1();
  initial $display("top 2");
  bind mid bmod bm();
  always begin $display("top always"); #100; end
  initial #1 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "top 1",
            "mid1 1",
            "leaf11 init",
            "ifc15 init",
            "mid1 2",
            "mid1 gen",
            "leaf17 init",
            "mid1 3",
            "bound init",
            "top 2",
            "mid1 always-first",
            "leaf11 always",
            "leaf17 always",
            "top always",
        ]
    );
}

/// One module, suspending processes only: plain source order, visible in
/// the order of the #10 wakeups.
#[test]
fn single_module_keeps_source_order() {
    let src = r#"
module top;
  always begin $display("%0t A0", $time); #10; end
  initial $display("%0t I1", $time);
  always #10 $display("%0t A1", $time);
  initial #10 $display("%0t I2", $time);
  always begin #10; $display("%0t A2", $time); end
  initial begin #10; $display("%0t I3", $time); end
  initial #15 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        ["0 A0", "0 I1", "10 A0", "10 A1", "10 I2", "10 A2", "10 I3",]
    );
}

/// A child's suspending `initial` and `always` blocks run at the position of
/// its instantiation, between the parent's own blocks.
#[test]
fn child_processes_at_their_instantiation() {
    let src = r#"
module child;
  initial begin #10; $display("%0t child I", $time); end
  always #10 $display("%0t child A", $time);
endmodule
module top;
  bit clk;
  always #10 clk = ~clk;
  initial begin #10; $display("%0t top I1 clk=%0d", $time, clk); end
  child c();
  always @(posedge clk) $display("%0t top posedge", $time);
  initial begin #10; $display("%0t top I2 clk=%0d", $time, clk); end
  initial #15 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "10 top I1 clk=1",
            "10 child I",
            "10 child A",
            "10 top I2 clk=1",
            "10 top posedge",
        ]
    );
}

/// A root `always` ahead of the first non-suspending `initial` runs before the
/// group; later ones run after it.
#[test]
fn always_before_the_group_runs_first() {
    let src = r#"
module child;
  always begin $display("child A0"); #100; end
  initial $display("child I1");
  always begin $display("child A2"); #100; end
endmodule
module top;
  always begin $display("top A0"); #100; end
  initial $display("top I1");
  child c();
  initial $display("top I2");
  always begin $display("top A3"); #100; end
  initial #1 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "top A0", "top I1", "child I1", "top I2", "child A0", "child A2", "top A3",
        ]
    );
}

/// `always` blocks at any depth after the first non-suspending `initial` run
/// after the whole group, depth first.
#[test]
fn always_blocks_after_the_group() {
    let src = r#"
module leaf;
  always begin $display("leaf A"); #100; end
  initial $display("leaf I");
endmodule
module mid;
  initial $display("mid I1");
  leaf l();
  always begin $display("mid A"); #100; end
  initial $display("mid I2");
endmodule
module top;
  initial $display("top I1");
  mid m();
  always begin $display("top A"); #100; end
  initial $display("top I2");
  always begin $display("top A2"); #100; end
  initial #1 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "top I1", "mid I1", "leaf I", "mid I2", "top I2", "leaf A", "mid A", "top A", "top A2",
        ]
    );
}

/// Generate-block and interface-instance processes take their source slot.
#[test]
fn generate_block_and_interface_always() {
    let src = r#"
interface ifc;
  always begin $display("ifc A"); #100; end
  initial $display("ifc I");
endinterface
module top;
  initial $display("top I1");
  if (1) begin : g
    always begin $display("g A"); #100; end
    initial $display("g I");
  end
  ifc i();
  initial $display("top I2");
  always begin $display("top A"); #100; end
  initial $display("top I3");
  initial #1 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "top I1", "g I", "ifc I", "top I2", "top I3", "g A", "ifc A", "top A",
        ]
    );
}

/// Delay-first child blocks queue their wakeups in instantiation order.
#[test]
fn child_delay_first_blocks_keep_their_slot() {
    let src = r#"
module leaf;
  initial begin #10; $display("%0t leaf I", $time); end
  always #10 $display("%0t leaf A", $time);
  always begin #10; $display("%0t leaf A2", $time); end
endmodule
module top;
  initial begin #10; $display("%0t top I1", $time); end
  leaf l();
  initial begin #10; $display("%0t top I2", $time); end
  initial #15 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "10 top I1",
            "10 leaf I",
            "10 leaf A",
            "10 leaf A2",
            "10 top I2",
        ]
    );
}

/// The group sits at its FIRST member: a suspending `initial` and an
/// `always` ahead of it run first.
#[test]
fn group_sits_at_the_first_member() {
    let src = r#"
module top;
  initial begin $display("I1"); #0; end
  always begin $display("A2"); #10; end
  initial $display("I3");
  initial $display("I4");
  always begin $display("A5"); #10; end
  initial #1 $finish;
endmodule
"#;
    assert_eq!(lines(src), ["I1", "A2", "I3", "I4", "A5",]);
}

/// `initial` blocks with a delay or an event control run after the
/// non-suspending ones.
#[test]
fn suspending_initials_follow_the_group() {
    let src = r#"
module top;
  initial $display("I1");
  initial begin $display("I2"); #0; end
  initial $display("I3");
  initial begin #0 $display("I4"); end
  initial $display("I5");
  initial begin $display("I6"); @(top.x); end
  int x;
  initial $display("I7");
endmodule
"#;
    assert_eq!(lines(src), ["I1", "I3", "I5", "I7", "I2", "I6", "I4",]);
}

/// A `fork ... join_none` or a function call does not make an `initial`
/// suspending; a call to a task with a delay does.
#[test]
fn blocking_task_call_suspends() {
    let src = r#"
module child;
  initial begin $display("c R1"); #0; end
  initial $display("c T1");
endmodule
module top;
  always begin $display("t A1"); #10; end
  child c();
  initial begin $display("t R2"); fork #0; join_none end
  initial $display("t T3");
  task automatic tk(); #0; endtask
  initial begin $display("t R4"); tk(); end
  function void fn(); endfunction
  initial begin $display("t T5"); fn(); end
  initial #1 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        ["t A1", "c R1", "c T1", "t R2", "t T3", "t T5", "t R4",]
    );
}

/// Each top-level module runs as one block, in root order.
#[test]
fn each_top_runs_as_one_block() {
    let src = r#"
module child #(parameter string N = "");
  always begin $display("%s.c A", N); #100; end
  initial $display("%s.c I", N);
endmodule
module ta;
  always begin $display("ta A"); #100; end
  initial $display("ta I1");
  child #("ta") c();
  initial $display("ta I2");
endmodule
module tb;
  initial $display("tb I1");
  child #("tb") c();
  always begin $display("tb A"); #100; end
  initial $display("tb I2");
  initial #1 $finish;
endmodule
"#;
    assert_eq!(
        lines(src),
        [
            "ta A", "ta I1", "ta.c I", "ta I2", "ta.c A", "tb I1", "tb.c I", "tb I2", "tb.c A",
            "tb A",
        ]
    );
}

/// Several binds into one host run after the host's own blocks, in REVERSE
/// bind order — whether the binds sit at compilation-unit scope or inside a
/// module.
#[test]
fn bound_instances_run_in_reverse_bind_order() {
    let src = r#"
module mon; initial $display("mon"); endmodule
module chkA; initial $display("A"); endmodule
module chkB; initial $display("B"); endmodule
module chkC; initial $display("C"); endmodule
module top; mon m(); initial $display("top"); endmodule
bind mon chkA z1();
bind mon chkB a2();
bind mon chkC m3();
"#;
    assert_eq!(lines(src), ["mon", "C", "B", "A", "top"]);
    let src = r#"
module mon; initial $display("mon"); endmodule
module chkA; initial $display("A"); endmodule
module chkB; initial $display("B"); endmodule
module agent;
  mon u_mon();
  bind mon chkA z1();
  initial $display("agent");
  bind mon chkB a2();
endmodule
module top; agent ag(); initial $display("top"); endmodule
"#;
    assert_eq!(lines(src), ["mon", "B", "A", "agent", "top"]);
}
