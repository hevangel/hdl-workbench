//! IEEE 1800-2017 §3.6 — a data declaration with a USER-DEFINED (class)
//! type at compilation-unit ($unit) scope, e.g. `rw_tr q[$];`.
//!
//! Previously xezim's $unit-scope parser guard only admitted builtin type
//! keywords (int/logic/…), `var`, and `const`. A user-defined type name
//! (class/typedef) led by an Identifier fell through to "unexpected token".
//! Module instances cannot legally exist at $unit scope (§23.3), so a
//! leading Identifier there is always a data declaration.
//!
//! Verified byte-for-byte against reference simulators.

use xezim::simulate;

const SRC: &str = r#"
class rw_tr;
  int addr;
  function new(int a); addr = a; endfunction
endclass

rw_tr q[$];

module top;
  int sz;
  int got;
  initial begin
    rw_tr t;
    t = new(42);
    q.push_back(t);
    sz  = q.size();
    got = q[0].addr;
  end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

#[test]
fn unit_scope_user_type_queue_var() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "sz"), 1, "queue must hold the one pushed object");
    assert_eq!(
        u(&sim, "got"),
        42,
        "q[0].addr must read back the stored value"
    );
}
