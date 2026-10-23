//! IEEE 1800-2023 §21.2.1.7: `%m` is the hierarchical name of the scope
//! holding the format string. Inside a module instance that is the instance
//! path, whatever KIND of process the `%m` sits in.
//!
//! `current_scope` is installed by `run_process`, which only ever runs
//! initial/fork/delay-always processes. A sensitivity-driven block
//! (`always_comb`, `always @(edge)`) is a CombEntry or a compiled edge block
//! and never passes through there, so `%m` in one reported the bare TOP
//! module name — identically for every instance, which makes per-instance log
//! correlation impossible in a multiply-instantiated design.
//!
//! Two further gaps this covers:
//!   - a comb block's scope was INFERRED from its read/write sets, which
//!     returns the first scope that fits, so with two instances both blocks
//!     claimed the first one (`u_beta`'s block reported `u_alpha`);
//!   - the scheduler flattens a process's outermost begin/end, so a named
//!     `initial begin : lbl` never reached the arm that pushes its label.
//!
//! The checks assert the instance name APPEARS in `%m` rather than matching
//! the whole string, since the exact separator/prefix is not what a design
//! engineer depends on here.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z, expected a defined value", n))
}

const SRC: &str = r#"
module m_leaf #(parameter string SELF = "?") (input logic clk);
   logic a;

   string m_initial = "";
   string m_comb    = "";
   string m_edge    = "";
   string m_task    = "";
   string m_named   = "";

   initial m_initial = $sformatf("%m");
   always_comb begin a = clk; m_comb = $sformatf("%m"); end
   always @(posedge clk) m_edge = $sformatf("%m");

   task automatic grab_t;
      m_task = $sformatf("%m");
   endtask

   initial begin : nb
      m_named = $sformatf("%m");
   end

   initial #1 grab_t();
endmodule

module tb;
   logic clk = 0;

   m_leaf #("u_alpha") u_alpha (.clk(clk));
   m_leaf #("u_beta")  u_beta  (.clk(clk));

   int ok_a_init, ok_a_comb, ok_a_edge, ok_a_task, ok_a_named;
   int ok_b_init, ok_b_comb, ok_b_edge, ok_b_task, ok_b_named;
   int a_named_has_label;

   function automatic bit has_sub(string hay, string needle);
      int hl = hay.len();
      int nl = needle.len();
      if (nl == 0) return 1'b1;
      if (nl > hl) return 1'b0;
      for (int i = 0; i <= hl - nl; i++)
         if (hay.substr(i, i + nl - 1) == needle) return 1'b1;
      return 1'b0;
   endfunction

   initial begin
      #2 clk = 1'b1;
      #2;
      ok_a_init  = has_sub(u_alpha.m_initial, "u_alpha");
      ok_a_comb  = has_sub(u_alpha.m_comb,    "u_alpha");
      ok_a_edge  = has_sub(u_alpha.m_edge,    "u_alpha");
      ok_a_task  = has_sub(u_alpha.m_task,    "u_alpha");
      ok_a_named = has_sub(u_alpha.m_named,   "u_alpha");

      ok_b_init  = has_sub(u_beta.m_initial,  "u_beta");
      ok_b_comb  = has_sub(u_beta.m_comb,     "u_beta");
      ok_b_edge  = has_sub(u_beta.m_edge,     "u_beta");
      ok_b_task  = has_sub(u_beta.m_task,     "u_beta");
      ok_b_named = has_sub(u_beta.m_named,    "u_beta");

      a_named_has_label = has_sub(u_alpha.m_named, "nb");
   end
endmodule
"#;

#[test]
fn pct_m_names_the_instance_in_every_process_kind() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    for (inst, tag) in [("a", "u_alpha"), ("b", "u_beta")] {
        for (kind, what) in [
            ("init", "initial block"),
            ("comb", "always_comb"),
            ("edge", "always @(posedge)"),
            ("task", "task"),
            ("named", "named begin/end"),
        ] {
            assert_eq!(
                u(&sim, &format!("ok_{inst}_{kind}")),
                1,
                "%m inside a {what} must name the enclosing instance {tag}; \
                 a sensitivity-driven block that reports only the top module \
                 makes every instance's log line identical"
            );
        }
    }
}

#[test]
fn pct_m_keeps_the_named_block_label() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "a_named_has_label"),
        1,
        "%m inside `initial begin : nb` must include the block label — the \
         outermost begin/end is flattened into the process, so the label has \
         to be carried per-pid rather than pushed by the block arm"
    );
}
