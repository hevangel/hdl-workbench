//! §25.10 / §8.4 — arrays of virtual interfaces, and the null default of an
//! unassigned vif variable. Reference-validated.
//!
//! Element-wise binding was never recorded in either scope: `varr[0] = b0;` on
//! a module-scope vif array silently became a value copy (only the WHOLE-array
//! form registered aliases), and a bare `va[i] = bus;` inside a class method
//! had no binding arm at all — so every later `varr[0].data` access read x /
//! wrote nowhere, in both directions, for both holders.
//!
//! Separately, an unassigned module/block-scope `virtual <iface>` variable was
//! registered as an ordinary 4-state signal defaulting to x, so the standard
//! guard `if (vif == null) ...` evaluated x and took the else branch — the
//! "not connected" check silently passed. A handle defaults to null (§8.4);
//! class properties already did, which is why only the variable form
//! misbehaved.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Module-scope vif array: element binds, drives, and reads back — each
/// element to its own instance.
#[test]
fn module_scope_vif_array_elements() {
    let src = r#"
interface bus_if;
  logic [7:0] data;
endinterface
module tb;
  bus_if b0(), b1();
  virtual bus_if varr [0:1];
  int r_b0, r_b1, r_v0, r_v1;
  initial begin
    varr[0] = b0;
    varr[1] = b1;
    varr[0].data = 8'h55;
    varr[1].data = 8'h99;
    #1;
    r_b0 = b0.data;      r_b1 = b1.data;
    r_v0 = varr[0].data; r_v1 = varr[1].data;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "r_b0"), u(&sim, "r_b1")),
        (0x55, 0x99),
        "writes land on the instances"
    );
    assert_eq!(
        (u(&sim, "r_v0"), u(&sim, "r_v1")),
        (0x55, 0x99),
        "and read back through the elements"
    );
}

/// A class property that is an ARRAY of vifs, bound bare inside a method.
#[test]
fn class_property_vif_array_elements() {
    let src = r#"
interface bus_if;
  logic [7:0] data;
endinterface
class Agent;
  virtual bus_if va [0:1];
  function void bind_ifs(virtual bus_if x0, virtual bus_if x1);
    va[0] = x0;
    va[1] = x1;
  endfunction
  function void drive();
    va[0].data = 8'hA0;
    va[1].data = 8'hA1;
  endfunction
  function int rd(int i);
    return va[i].data;
  endfunction
endclass
module tb;
  bus_if b0(), b1();
  Agent a;
  int r_b0, r_b1, r_m0, r_m1;
  initial begin
    a = new();
    a.bind_ifs(b0, b1);
    a.drive();
    #1;
    r_b0 = b0.data; r_b1 = b1.data;
    r_m0 = a.rd(0); r_m1 = a.rd(1);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        (u(&sim, "r_b0"), u(&sim, "r_b1")),
        (0xA0, 0xA1),
        "per-element binds reach their instances"
    );
    assert_eq!(
        (u(&sim, "r_m0"), u(&sim, "r_m1")),
        (0xA0, 0xA1),
        "method reads through a variable index"
    );
}

/// The unconnected-vif guard: null before assignment, non-null after,
/// null again after clearing.
#[test]
fn unassigned_vif_variable_is_null() {
    let src = r#"
interface bus_if;
  logic [7:0] data;
endinterface
module tb;
  bus_if bi();
  virtual bus_if v;
  int before_assign, after_assign, after_clear;
  initial begin
    before_assign = (v == null);
    v = bi;
    after_assign  = (v == null);
    v = null;
    after_clear   = (v == null);
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "before_assign"),
        1,
        "an unassigned vif variable is null, not x"
    );
    assert_eq!(u(&sim, "after_assign"), 0);
    assert_eq!(u(&sim, "after_clear"), 1);
}
