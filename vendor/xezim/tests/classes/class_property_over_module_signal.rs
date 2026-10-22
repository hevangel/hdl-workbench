//! §8.1 — inside a class method a bare name the class declares is that
//! property, even when a module-scope signal has the same name. The type of
//! `b` in a constructor's `b = new(7)` came from the module's `outer b`, so
//! the constructor built an `outer`, whose constructor built this class
//! again: the simulation never returned. The expected line is the reference
//! simulator's output.

use xezim::simulate;

const SRC: &str = r#"
class leaf;
  int k;
  function new(int x); k = x; endfunction
endclass
class holder;
  leaf b;
  function new(); b = new(7); endfunction
endclass
class outer;
  holder h;
  function new(); h = new; endfunction
endclass
module top;
  outer b;
  initial begin
    b = new;
    $display("built k=%0d", b.h.b.k);
  end
endmodule
"#;

#[test]
fn class_property_wins_over_same_named_module_signal() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, vec!["built k=7"], "{out:?}");
}
