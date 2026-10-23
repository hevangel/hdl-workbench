//! §13.3 / §6.21: a subroutine's formal shadows everything outside it. A
//! class constructor's formal `name` and an interface's `string name` in
//! the design are different variables; string methods on the formal
//! (`name.len()`, `name.substr()`, `name.getc()`) used to resolve the
//! receiver by name and read the interface's string instead. UVM's
//! resource-name check (`uvm_resource#(T)::new`) iterated the wrong string,
//! so its `UVM/RSRC/NOREGEX` warning never fired on AVIPs whose BFM
//! interfaces declare `string name`. Expected values are the reference
//! simulator's.

use xezim::simulate;

#[test]
fn string_methods_on_a_formal_ignore_a_same_named_instance_signal() {
    let sim = simulate(
        r#"
interface bfm;
  string name = "AXI4_SLAVE_DRIVER_BFM";
endinterface
class res;
  function new(string name = "");
    int hits = 0;
    for (int i = 0; i < name.len(); i++)
      if (name.getc(i) inside {"[", "."}) hits++;
    $display("SM plain=%s len=%0d first=%s hits=%0d", name, name.len(), name.substr(0, 2), hits);
  endfunction
endclass
module top;
  bfm u_bfm();
  initial begin
    automatic res r = new("cfg[0]");
  end
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(
        o.iter()
            .any(|l| l == "SM plain=cfg[0] len=6 first=cfg hits=1"),
        "{o:?}"
    );
}
