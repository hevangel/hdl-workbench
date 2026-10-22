//! §25.9 (issue #113, partial): a VIRTUAL INTERFACE must survive a
//! parameterized-class STATIC set/get round-trip (the uvm_config_db shape)
//! and bind the receiving class property so writes reach the interface.
//! Reference-validated: writes through the returned vif land on the real
//! instance.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> String {
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn vif_survives_parameterized_static_store() {
    let src = r#"
interface pif;
  logic [7:0] data;
endinterface

class Holder;
  virtual pif vif;
endclass

class DB #(type T = int);
  static T store;
  static bit has;
  static function void set_val(T v);
    // The uvm_resource::write shape: an unbound-vs-bound guard must be
    // FALSE (binding compare, not x-value compare) or the store is skipped.
    if (store == v) return;
    store = v;
    has = 1;
  endfunction
  static function bit get_val(inout T v);
    if (!has) return 0;
    v = store;
    return 1;
  endfunction
endclass

module top;
  pif i0();
  Holder h;
  bit ok;
  initial begin
    h = new;
    DB#(virtual pif)::set_val(i0);
    ok = DB#(virtual pif)::get_val(h.vif);
    h.vif.data = 8'hA7;
    $display("T|ok=%0d through=%h direct=%h", ok, h.vif.data, i0.data);
  end
endmodule
"#;
    let out = outs(&simulate(src, 10).expect("sim"));
    assert!(
        out.contains("T|ok=1 through=a7 direct=a7"),
        "vif must round-trip the static store and bind (writes reach the instance):\n{out}"
    );
}
