// Regression test: `new(...)` must not be dispatched as a static method.
// In xezim, `ClassName::new(args)` reached `exec_static_method` first,
// which ran the constructor with `this`=0 (null). Property writes like
// `m_leaf_name = name` went to the null handle, producing a component
// with empty name/type/inst_id=0 — the "ghost child" that caused
// infinite recursion in UVM bottomup-phase traversal.
// The fix: `exec_static_method` returns None for method_name=="new",
// letting call-sites fall through to `instantiate_class` which allocates
// a real object and dispatches the constructor with the correct handle.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(&format!("top.{}", n))
        .or_else(|| sim.get_signal(n))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

const SRC: &str = r#"
class base;
  string m_name;
  int inst_id;
  static int s_next_id = 1;
  function new(string name);
    m_name = name;
    inst_id = s_next_id;
    s_next_id = s_next_id + 1;
  endfunction
  function string get_name(); return m_name; endfunction
  function int get_inst_id(); return inst_id; endfunction
endclass

class child extends base;
  function new(string name);
    super.new(name);
  endfunction
endclass

module top;
  int result;
  initial begin
    child c;
    c = new("hello");
    // If new() were dispatched as a static method with this=0, the
    // constructor's property writes (m_name, inst_id) would be lost and
    // get_name() would return "" and get_inst_id() would return 0.
    if (c.get_name() == "hello" && c.get_inst_id() != 0)
      result = 1;
    else
      result = 0;
  end
endmodule
"#;

#[test]
fn constructor_new_not_static_dispatch() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    assert_eq!(
        u(&sim, "result"),
        1,
        "constructor must run on real instance"
    );
}
