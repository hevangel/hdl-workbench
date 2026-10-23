//! Class property inline initializers with bare `new` (no parens).
//!
//! Verifies that class properties declared with in-class initializers like
//! `my_class obj = new;` are instantiated during class allocation, rather
//! than remaining null (`X`), allowing methods on the object to be invoked.

use xezim::simulate;

fn messages(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

const BARE_NEW_INIT_SRC: &str = r#"
module top;
  class item;
    int val = 100;
    function int get_val();
      return val;
    endfunction
  endclass

  class container;
    item it = new;
    function int check();
      if (it != null) return it.get_val();
      return -1;
    endfunction
  endclass

  initial begin
    container c = new;
    int res = c.check();
    if (res == 100) $display("TAG_PASS");
    else $display("TAG_FAIL res=%0d", res);
  end
endmodule
"#;

#[test]
fn property_bare_new_initializer() {
    let sim = simulate(BARE_NEW_INIT_SRC, 100).expect("simulate failed");
    let msgs = messages(&sim);
    assert!(
        msgs.iter().any(|m| m == "TAG_PASS"),
        "class property `it = new` should construct non-null item; got {:?}",
        msgs
    );
}
