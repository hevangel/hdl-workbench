// IEEE 1800-2017 §8.20 / §8.25: a NON-virtual method (which includes every
// `local` method, since `local` methods cannot be overridden — a subclass
// same-name method HIDES rather than overrides) binds STATICALLY to the
// lexical class that owns the call site, NOT the runtime class of `this`.
//
// Before the fix, xezim virtualized the unqualified call `f()` inside a class
// method: when `base::g()` (reached via `super` from `ext`) called `f()`, it
// dispatched to `ext::f()` because `this` was an `ext` object. The embedded
// SV collects the call order into a queue and self-checks it; without the fix
// it prints TAG_FAIL (base's body never runs for a derived instance), with it
// TAG_PASS.

#[test]
fn local_method_binds_statically_through_super() {
    let src = r#"
module top;
   // Ordered call log; populated by the class methods, checked at the end.
   string trace[$];

   // ------------------------------------------------------------------
   // Scenario A: `local` method (the motivating trigger)
   // ------------------------------------------------------------------
   class base;
      virtual function void g();
         f();                       // MUST bind to base::f (f is local)
      endfunction
      local function void f();
         trace.push_back("A:base.f");
      endfunction
   endclass

   class ext extends base;
      virtual function void g();
         super.g();                 // -> base::g -> base::f (A:base.f)
         f();                       // ext's own local f (A:ext.f)
      endfunction
      local function void f();
         trace.push_back("A:ext.f");
      endfunction
   endclass

   // ------------------------------------------------------------------
   // Scenario B: plain NON-virtual method (same §8.20 rule)
   // ------------------------------------------------------------------
   class p_base;
      virtual function void g();
         f();                       // MUST bind to p_base::f (non-virtual)
      endfunction
      function void f();            // non-virtual, NOT local
         trace.push_back("B:p_base.f");
      endfunction
   endclass

   class p_ext extends p_base;
      virtual function void g();
         super.g();                 // -> p_base::g -> p_base::f
         f();                       // p_ext's own f (hides, doesn't override)
      endfunction
      function void f();
         trace.push_back("B:p_ext.f");
      endfunction
   endclass

   function automatic bit check(ref string got[$], ref string exp[$]);
      if (got.size() != exp.size()) return 1'b0;
      foreach (exp[i])
         if (got[i] != exp[i]) return 1'b0;
      return 1'b1;
   endfunction

   initial begin : tb
      automatic ext    e  = new;
      automatic base   b  = new;
      automatic p_ext  pe = new;
      string exp[$];

      e.g();    // A:base.f, A:ext.f
      b.g();    // A:base.f
      pe.g();   // B:p_base.f, B:p_ext.f

      exp.push_back("A:base.f");
      exp.push_back("A:ext.f");
      exp.push_back("A:base.f");
      exp.push_back("B:p_base.f");
      exp.push_back("B:p_ext.f");

      if (check(trace, exp)) begin
         $display("TAG_PASS");
      end else begin
         $display("TAG_FAIL call-order mismatch (expected %0d, got %0d)",
                 exp.size(), trace.size());
         foreach (trace[i]) $display("  got[%0d] = %s", i, trace[i]);
         foreach (exp[i])   $display("  exp[%0d] = %s", i, exp[i]);
         $fatal(1);
      end
   end
endmodule
"#;

    let sim = xezim::simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|line| line.message == "TAG_PASS"),
        "local/non-virtual method must bind statically to the lexical class, \
         not dispatch virtually on `this`. Output:\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
