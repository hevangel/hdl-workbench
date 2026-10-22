// IEEE 1800-2023 §21.2.1.7 -- `%m` prints the hierarchical name of the
// scope containing the format string. Inside a module instance that is the
// instance path (`top.u_alpha`), whatever KIND of process the `%m` sits in:
// initial, always_comb, always @(edge), a task, or a named block.
//
// A simulator that renders `%m` from the top module name alone reports
// `pct_m_hier_scope` for every instance, which makes per-instance log
// correlation impossible in a multiply-instantiated design.

`timescale 1ns/1ps

module m_leaf (input logic clk);
   logic a;

   string m_initial = "";
   string m_comb    = "";
   string m_edge    = "";
   string m_task    = "";
   string m_named   = "";

   initial m_initial = $sformatf("%m");

   always_comb begin
      a     = clk;
      m_comb = $sformatf("%m");
   end

   always @(posedge clk) m_edge = $sformatf("%m");

   task automatic grab_t;
      m_task = $sformatf("%m");
   endtask

   initial begin : nb
      m_named = $sformatf("%m");
   end

   initial #1 grab_t();
endmodule

module pct_m_hier_scope;

   logic clk = 1'b0;

   m_leaf u_alpha (.clk(clk));
   m_leaf u_beta  (.clk(clk));

   int n_checks = 0;
   int n_errors = 0;

   function automatic bit has_sub(string hay, string needle);
      int hl = hay.len();
      int nl = needle.len();
      if (nl == 0)  return 1'b1;
      if (nl > hl)  return 1'b0;
      for (int i = 0; i <= hl - nl; i++)
         if (hay.substr(i, i + nl - 1) == needle) return 1'b1;
      return 1'b0;
   endfunction

   // `%m` formatting differs slightly between tools (separator, leading
   // path), so require the INSTANCE name to appear rather than an exact
   // string. That is the property the design engineer actually needs.
   task automatic chk_scope(string what, string got, string inst);
      n_checks++;
      if (!has_sub(got, inst)) begin
         n_errors++;
         $display("  FAIL  %-30s %%m=\"%s\"  (expected it to name \"%s\")", what, got, inst);
      end
   endtask

   initial begin
      #2 clk = 1'b1;
      #2;

      $display("TEST pct_m_hier_scope");
      $display("  u_alpha initial     : %s", u_alpha.m_initial);
      $display("  u_alpha always_comb : %s", u_alpha.m_comb);
      $display("  u_alpha always@edge : %s", u_alpha.m_edge);
      $display("  u_alpha task        : %s", u_alpha.m_task);
      $display("  u_alpha named block : %s", u_alpha.m_named);
      $display("  u_beta  always_comb : %s", u_beta.m_comb);
      $display("  u_beta  always@edge : %s", u_beta.m_edge);

      chk_scope("alpha initial",     u_alpha.m_initial, "u_alpha");
      chk_scope("alpha always_comb", u_alpha.m_comb,    "u_alpha");
      chk_scope("alpha always@edge", u_alpha.m_edge,    "u_alpha");
      chk_scope("alpha task",        u_alpha.m_task,    "u_alpha");
      chk_scope("alpha named block", u_alpha.m_named,   "u_alpha");

      chk_scope("beta initial",      u_beta.m_initial,  "u_beta");
      chk_scope("beta always_comb",  u_beta.m_comb,     "u_beta");
      chk_scope("beta always@edge",  u_beta.m_edge,     "u_beta");
      chk_scope("beta task",         u_beta.m_task,     "u_beta");
      chk_scope("beta named block",  u_beta.m_named,    "u_beta");

      $display("TEST pct_m_hier_scope: %0d checks, %0d errors -> %s",
               n_checks, n_errors, (n_errors == 0) ? "PASS" : "FAIL");
      $finish;
   end
endmodule
