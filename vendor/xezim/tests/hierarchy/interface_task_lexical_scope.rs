//! §23.8 / §26.3 — names inside an interface task resolve in the
//! interface's LEXICAL scope, never in the scope of whoever called the task
//! through a virtual interface. The reference simulator prints
//! `WHO=package`: the unqualified `who()` binds to the package function
//! imported into the interface, not to the calling class's method.
//!
//! This is the shape behind UVM's `uvm_info` inside a BFM task called from a
//! driver proxy: the macro's unqualified report calls must reach the global
//! reporter. xezim used to resolve them dynamically (`WHO=class`), so such
//! messages were attributed to the calling component.

use xezim::simulate;

#[test]
fn interface_task_calls_resolve_lexically() {
    let sim = simulate(
        r#"
package p;
  function automatic string who(); return "package"; endfunction
endpackage
interface bfm;
  import p::*;
  task report(); $display("WHO=%s", who()); endtask
endinterface
class comp;
  virtual bfm v;
  function string who(); return "class"; endfunction
  task run(); v.report(); endtask
endclass
module tb;
  bfm b();
  initial begin
    automatic comp c = new;
    c.v = b;
    c.run();
  end
endmodule
"#,
        1000,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(o.iter().any(|l| l == "WHO=package"), "{o:?}");
}
