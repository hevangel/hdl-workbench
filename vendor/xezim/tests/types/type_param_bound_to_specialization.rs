// IEEE 1800-2017 §8.25: when a class TYPE parameter is specialized to a
// SPECIALIZED class name (e.g. `C#(int)`), the parameter binding must record
// the full `base#(args)` form — not just the bare base, and not the bare
// trailing type argument. The binding drives later construction: a method
// body `T obj; obj = new(name)` where `T` resolves to `wrap#(int)` must
// instantiate `wrap#(int)`, returning a non-null object whose constructor ran.
//
// Before the fix, the specialization argument `wrap#(int)` was classified as
// a VALUE argument (the specialized-name expression wasn't recognized as a
// definite type), so positional parameter binding broke: the type parameter
// went unbound and silently fell back to its declared default (`int`). The
// later `obj = new(name)` then had nothing valid to construct and returned
// null. The embedded SV self-checks for a non-null handle and the correct
// field value; without the fix it prints TAG_FAIL (null handle), with it
// TAG_PASS.

#[test]
fn type_param_bound_to_specialized_class_constructs_correctly() {
    let src = r#"
module top;
   // A parameterized class that will be used as a TYPE ARGUMENT to another
   // parameterized class. Its constructor records a field so we can verify
   // the *right* specialization was built.
   class wrap #(type VT=int);
      int v;
      function new(string n="");
         v = 42;
      endfunction
   endclass

   // A registry whose type parameter T is itself specialized when the
   // registry is instantiated. `create_object` does `T obj; obj = new(name)`:
   // T must resolve to the FULL specialization, not the default `int`.
   class registry #(type T=int);
      virtual function T create_object(string name="");
         T obj;
         obj = new(name);
         return obj;
      endfunction
   endclass

   initial begin : tb
      // T binds to the SPECIALIZED class `wrap#(int)`.
      automatic registry#(wrap#(int)) r;
      automatic wrap#(int) c;
      r = new;
      c = r.create_object("hello");
      if (c == null)
         $display("TAG_FAIL: create_object returned null");
      else if (c.v != 42)
         $display("TAG_FAIL: wrong val v=%0d", c.v);
      else
         $display("TAG_PASS");
   end
endmodule
"#;

    let sim = xezim::simulate(src, 50).expect("simulate");
    assert!(
        sim.output.iter().any(|line| line.message == "TAG_PASS"),
        "a type parameter specialized to a specialized class name must record \
         the full `base#(args)` form so a later `T obj; obj = new(name)` \
         constructs the specialized class. Output:\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
