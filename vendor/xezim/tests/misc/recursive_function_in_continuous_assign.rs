//! A RECURSIVE function called from a continuous assignment must simulate,
//! not abort.
//!
//! `fn_is_pure_in`'s `Call` arm recurses into the callee's purity to decide
//! whether a helper can be inlined; for a self-recursive function that walk
//! never terminated and the process died with a stack overflow (SIGABRT).
//! Only the COMPILED path consults purity, which is why the same function
//! called from an initial block worked and `assign w = fact(6);` crashed —
//! the shape ivtest's recursive_func1/2 use. The walk is now depth-capped;
//! exceeding the cap answers "not pure", which just keeps the call on the
//! AST interpreter where recursion already worked. Values match the
//! reference simulator.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const SRC: &str = r#"
module top;
  function automatic [15:0] fact;
    input [15:0] n;
    begin
      fact = (n > 1) ? fact(n - 1) * n : n;
    end
  endfunction
  // Mutual recursion too: the depth cap must cover a cycle through a second
  // function, not just the direct self-call.
  function automatic [15:0] is_odd(input [15:0] n);
    return (n == 0) ? 16'd0 : is_even(n - 1);
  endfunction
  function automatic [15:0] is_even(input [15:0] n);
    return (n == 0) ? 16'd1 : is_odd(n - 1);
  endfunction

  wire [15:0] w6, w8, odd7;
  assign w6   = fact(6);
  assign w8   = fact(8);
  assign odd7 = is_odd(16'd7);
  initial begin
    #1 $display("NOTE: %0d %0d %0d", w6, w8, odd7);
    $finish;
  end
endmodule
"#;

#[test]
fn recursive_function_under_continuous_assign_does_not_abort() {
    assert_eq!(notes(SRC), ["NOTE: 720 40320 1"]);
}
