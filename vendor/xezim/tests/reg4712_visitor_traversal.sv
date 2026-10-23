// Pure-SystemVerilog regression test for visitor traversal:
// uvm_visitor / uvm_bottom_up_visitor_adapter / uvm_top_down_visitor_adapter
//
// Tests tree traversal algorithms (top-down, bottom-up, by-level) that
// underlie the visitor pattern.  Uses parent-pointer tree representation
// and manual queue copying (xezim bug: queue-to-queue assignment `dst = src`
// inside automatic functions doesn't copy elements).

module top;
   string names[6];
   int parent[6];

   function automatic void get_children(int idx, ref int c[$]);
      foreach (parent[i])
         if (parent[i] == idx) c.push_back(i);
   endfunction

   function automatic void qcpy(ref int dst[$], ref int src[$]);
      dst.delete();
      foreach (src[i]) dst.push_back(src[i]);
   endfunction

   function automatic void accept_top_down(int s, ref string order[$], input bit is_root);
      int c[$];
      if (is_root) order.push_back("begin_v");
      order.push_back(names[s]);
      get_children(s, c);
      foreach (c[idx]) accept_top_down(c[idx], order, 0);
      if (is_root) order.push_back("end_v");
   endfunction

   function automatic void accept_bottom_up(int s, ref string order[$], input bit is_root);
      int c[$];
      if (is_root) order.push_back("begin_v");
      get_children(s, c);
      foreach (c[idx]) accept_bottom_up(c[idx], order, 0);
      order.push_back(names[s]);
      if (is_root) order.push_back("end_v");
   endfunction

   function automatic void accept_by_level(int s, ref string order[$], input bit is_root);
      int cur[$];
      int nxt[$];
      cur.push_back(s);
      if (is_root) order.push_back("begin_v");
      while (cur.size() > 0) begin
         nxt.delete();
         foreach (cur[idx]) begin
            int t[$];
            order.push_back(names[cur[idx]]);
            get_children(cur[idx], t);
            foreach (t[ti]) nxt.push_back(t[ti]);
         end
         qcpy(cur, nxt);
      end
      if (is_root) order.push_back("end_v");
   endfunction

   function automatic bit check(string got[$], string exp[$]);
      if (got.size() != exp.size()) return 0;
      foreach (exp[i])
         if (got[i] != exp[i]) return 0;
      return 1;
   endfunction

   initial begin
      names[0] = "root"; names[1] = "a";  names[2] = "b";
      names[3] = "a1";   names[4] = "a2"; names[5] = "b1";
      parent[0] = -1; parent[1] = 0; parent[2] = 0;
      parent[3] = 1;  parent[4] = 1; parent[5] = 2;
      test_all();
   end

   function automatic void test_all();
      string order[$];
      string exp[$];
      bit ok = 1;

      // Test 1: Top-down
      order.delete();
      accept_top_down(0, order, 1);
      exp.delete();
      exp = '{"begin_v", "root", "a", "a1", "a2", "b", "b1", "end_v"};
      if (check(order, exp)) begin
         $write("PASS: top-down\n");
      end else begin
         $write("FAIL: top-down\n");
         foreach (order[i]) $write("  [%0d] %s\n", i, order[i]);
         ok = 0;
      end

      // Test 2: Bottom-up
      order.delete();
      accept_bottom_up(0, order, 1);
      exp.delete();
      exp = '{"begin_v", "a1", "a2", "a", "b1", "b", "root", "end_v"};
      if (check(order, exp)) begin
         $write("PASS: bottom-up\n");
      end else begin
         $write("FAIL: bottom-up\n");
         foreach (order[i]) $write("  [%0d] %s\n", i, order[i]);
         ok = 0;
      end

      // Test 3: By-level
      order.delete();
      accept_by_level(0, order, 1);
      exp.delete();
      exp = '{"begin_v", "root", "a", "b", "a1", "a2", "b1", "end_v"};
      if (check(order, exp)) begin
         $write("PASS: by-level\n");
      end else begin
         $write("FAIL: by-level\n");
         foreach (order[i]) $write("  [%0d] %s\n", i, order[i]);
         ok = 0;
      end

      if (ok) $write("TAG_PASS\n");
      else   $write("TAG_FAIL\n");
   endfunction
endmodule