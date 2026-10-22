//! IEEE 1800-2020 §8.25 / §6.20.2 — a DERIVED class whose `extends` is a
//! TYPEDEF ALIAS of a PARAMETERIZED class must inherit the base's concrete
//! specialization args for the base method's `this_type` static resolution.
//!
//! `class D extends simple_lib` where `simple_lib = typedef base#(bit)
//! simple_lib;` stores the extends as the alias name. Every ancestor walk that
//! rebinds the base's type params (`static_receiver_spec` / `static_prop_key`)
//! must carry the alias's `#(bit)` args; otherwise a base method resolving
//! `this_type::m` reads the DEFAULT specialization (`base#(bit default)`)
//! — a per-spec cell disjoint from the concrete `base#(bit)` cell the static
//! write populated — so the derived object observed 0 elements while the base
//! broadcaster stored 2.
//!
//! Mirrors the UVM `uvm_sequence_library #(type REQ, RSP=REQ)` + `this_type`
//! macro pattern that initializes type-wide sequence registries per concrete
//! library subclass.
//!
//! Verified byte-for-byte against reference simulators.

use xezim::simulate;

const SRC: &str = r#"
module top;
  class item; endclass

  class seqlib #(type REQ = item, type RSP = REQ);
    static bit m_typewide[$];
    typedef seqlib#(REQ, RSP) this_type;
    bit sequences[$];
    static function void add_typewide(bit v);
      m_typewide.push_back(v);
    endfunction
    // Base method reads the per-spec static through `this_type` — this must
    // hit base#(item,item), NOT the default base#(item,item) with REQ unbound.
    function void init_sequence_library();
      foreach (this_type::m_typewide[i])
        sequences.push_back(this_type::m_typewide[i]);
    endfunction
    function new();
      init_sequence_library();
    endfunction
  endclass

  typedef seqlib#(item) simple_seq_lib;
  // `extends` is the TYPEDEF ALIAS `simple_seq_lib` (a parameterized-class
  // alias), not a bare class name.
  class seqlib_RST extends simple_seq_lib;
    function new();
      super.new();
    endfunction
  endclass

  int got;
  initial begin
    seqlib_RST rst;
    seqlib#(item)::add_typewide(1);
    seqlib#(item)::add_typewide(2);
    rst = new();
    got = rst.sequences.size();
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(&format!("top.{}", n))
        .or_else(|| sim.get_signal(n))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

#[test]
fn typedef_param_base_inherits_spec_arg() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    // The derived `RST` object's base `init_sequence_library` must see the two
    // elements stored on the concrete `seqlib#(item)` static. Before the fix,
    // the ancestor walk dropped the typedef's `#(item)` args and resolved
    // `this_type::m` against the unbound/default specialization -> got 0.
    assert_eq!(
        u(&sim, "got"),
        2,
        "derived base method read wrong per-spec static cell"
    );
}
