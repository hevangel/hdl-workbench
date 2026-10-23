// Self-test: a value parameter referenced in a static-call specialization
// (`Inner#(Name)::get()`) must be resolved to the enclosing class's concrete
// binding — NOT left as a symbolic bare name.
//
// Before the fix, xezim left `Name` symbolic in the specialization, so when
// `Inner`'s static method read the bare value param `Name`,
// `resolve_value_param_from_spec` re-entered itself with the identical
// specialization `(Inner, "Name")` and recursed infinitely, overflowing the
// stack. (This shape appears in UVM's factory: a registry class specializes
// its base by passing its own `Tname` param through. Several tests crashed
// via such parameterized-inheritance chains.)
//
// Two cooperating fixes:
//  (1) `resolve_call_spec_params` now resolves a top-level value-param
//      fragment against the enclosing specialization (was only handled for
//      nested `Class#(args)` fragments) → returns the concrete value.
//  (2) `resolve_value_param_from_spec` has a cycle guard so any residual
//      self/cyclic value-param reference falls back to the declared default
//      instead of overflowing the stack.
//
// Reference (reference simulator): r=hello.

use xezim::simulate;

const SRC: &str = r#"
class Inner #(string Name = "inner-default");
  static function string get();
    return Name;   // bare value-param read in a static method
  endfunction
endclass

class Mid #(type T = int, string Name = "mid-default");
  static function string probe();
    // Specialize Inner with Mid's own `Name`. The value param must resolve
    // to Mid's binding ("hello"), not stay symbolic.
    return Inner#(Name)::get();
  endfunction
endclass

module top;
  string r;
  initial begin
    r = Mid#(int, "hello")::probe();
  end
endmodule
"#;

#[test]
fn value_param_resolved_not_symbolic() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // r should be "hello" (Mid's Name binding), NOT the default
    // "inner-default", and the simulation must not stack-overflow.
    let val = sim
        .get_signal("top.r")
        .or_else(|| sim.get_signal("r"))
        .expect("signal top.r not found");
    let bytes: Vec<u8> = (0..5)
        .map(|j| {
            (0..8)
                .map(|k| {
                    if val.get_bit_code(j * 8 + k) == 1 {
                        1u8 << k
                    } else {
                        0
                    }
                })
                .sum()
        })
        .collect::<Vec<u8>>()
        .into_iter()
        .rev() // string stored MSB-first: first char at the high end
        .collect();
    let s = String::from_utf8_lossy(&bytes);
    assert!(
        s.starts_with("hello"),
        "expected r to start with \"hello\", got {:?}",
        s
    );
}
