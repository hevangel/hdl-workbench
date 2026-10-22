//! §8.3 / §6.21: a block-local `struct` variable named `s` in one function
//! must not make a different class's plain `int s` member look struct-capable.
//!
//! The blocking-assignment fast path (`simple_ident_copy_ok`) classifies the
//! LHS by consulting the flat `var_decl_types` map, which is overwritten on the
//! fly by every local/block-local variable declaration and never restored on
//! scope exit. A block-local `struct s;` therefore polluted the name `s` for
//! the whole module, so a class write `s = 99` via a bare identifier was
//! diverts to the struct-decompose (member-spread) path, which then dropped
//! the scalar write (the member came back 0). This regression guards that the
//! class's own non-struct property wins over the flat-map collision.

use xezim::simulate;

#[test]
fn blocklocal_struct_does_not_pollute_class_int_member() {
    // Compiled from the first call (tier 0) in a child process, then again
    // here at the default tier.
    let _ = super::compiled_method_test_env::eager();
    let src = r#"
module test;
  typedef struct { int a; } mystruct;

  class C;
    int s;
    function void set(int v);
      s = v;
    endfunction
  endclass

  function void pollute(int x);
    begin : blk
      mystruct s;
      s.a = x;
    end
  endfunction

  initial begin
    C c = new;
    pollute(1);
    c.set(99);
    $display("C|%0d %0d %0d", c.s, c.s, c.s);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("runs");
    assert!(
        sim.output.iter().any(|o| o.message == "C|99 99 99"),
        "block-local struct named s must not hijack C.s: {:?}",
        sim.output
            .iter()
            .map(|o| o.message.as_str())
            .collect::<Vec<_>>()
    );
}
