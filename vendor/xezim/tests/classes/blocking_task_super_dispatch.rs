//! Declaring-class context across nested blocking task dispatch.

use xezim::simulate;

fn value(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("top.{name}")))
        .unwrap_or_else(|| panic!("signal not found: {name}"))
        .to_u64()
        .expect("signal contains x/z")
}

/// IEEE 1800-2017 §8.15: `super` binds from the running method's defining class.
#[test]
fn nested_blocking_dispatch_keeps_defining_class() {
    let source = r#"
`timescale 1ns/1ns
module top;
  class root;
    int result;

    virtual task work();
    endtask

    task launch();
      #1;
      work();
    endtask
  endclass

  class middle extends root;
    task prepare();
      result = 41;
    endtask
  endclass

  class leaf extends middle;
    virtual task work();
      super.prepare();
      #1;
      result++;
    endtask
  endclass

  leaf item;
  int observed;

  initial begin
    item = new();
    item.launch();
    observed = item.result;
  end
endmodule
"#;

    let sim = simulate(source, 20).expect("simulation failed");
    assert_eq!(value(&sim, "observed"), 42);
}
