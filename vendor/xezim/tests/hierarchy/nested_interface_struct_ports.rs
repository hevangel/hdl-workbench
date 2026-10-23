//! §7.2 / §23.10 — whole-struct writes to interface members when the interface
//! lives in a NON-top module. Reference-validated.
//!
//! Two coupled defects, both invisible at top level:
//!
//! 1. A port connection naming a SIBLING instance (`h_leaf lf(li.mp);` where
//!    `li` is the same module's interface instance) is not a module-scope
//!    name, so the inlining rewrite left it unprefixed — the leaf's port bound
//!    to a phantom top-level `li` instead of `h.li`, and a whole-struct write
//!    through the port vanished. Member-wise writes survived via runtime
//!    scope-hint fallbacks, which made this look like a struct-copy bug.
//! 2. A DIRECT whole-struct write inside the holder (`li.us2 = t;`) reached
//!    the struct-copy branch with the child's own unprefixed spelling; the
//!    branch's type lookup bypassed the scope hint that rescues plain reads,
//!    so the copy silently declined and every member stayed x.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The full i9h shape: member write and whole-struct write through a
/// modport-qualified port, with the interface at top AND inside a holder.
#[test]
fn whole_struct_write_through_a_nested_interface_port() {
    let src = r#"
typedef struct { logic [7:0] a; logic [15:0] b; } h_us_t;
interface h_bus_if;
  h_us_t us;
  h_us_t us2;
  modport mp (output us, output us2);
endinterface
module h_leaf(h_bus_if.mp p);
  h_us_t t;
  initial begin
    #1;
    t.a = 8'h11; t.b = 16'h2233;
    p.us.a = 8'h44;
    p.us2  = t;
  end
endmodule
module h_holder;
  h_bus_if li();
  h_leaf lf(li.mp);
endmodule
module tb;
  h_bus_if ti();
  h_leaf lt(ti.mp);
  h_holder h();
  int t_usa, t_us2a, t_us2b, n_usa, n_us2a, n_us2b;
  initial begin
    #2;
    t_usa = ti.us.a;    t_us2a = ti.us2.a;    t_us2b = ti.us2.b;
    n_usa = h.li.us.a;  n_us2a = h.li.us2.a;  n_us2b = h.li.us2.b;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!(u(&sim, "t_usa"), 0x44, "top: member write");
    assert_eq!(
        (u(&sim, "t_us2a"), u(&sim, "t_us2b")),
        (0x11, 0x2233),
        "top: whole-struct write"
    );
    assert_eq!(u(&sim, "n_usa"), 0x44, "nested: member write");
    assert_eq!(
        (u(&sim, "n_us2a"), u(&sim, "n_us2b")),
        (0x11, 0x2233),
        "nested: the whole-struct write must not vanish"
    );
}

/// The direct form, no port involved: a whole-struct write to the holder's own
/// interface member.
#[test]
fn direct_whole_struct_write_in_a_nested_holder() {
    let src = r#"
typedef struct { logic [7:0] a; logic [15:0] b; } h_us_t;
interface h_bus_if;
  h_us_t us2;
  modport mp (output us2);
endinterface
module h_holder;
  h_bus_if li();
  h_us_t t;
  initial begin
    #1;
    t.a = 8'h11; t.b = 16'h2233;
    li.us2 = t;
  end
endmodule
module tb;
  h_holder h();
  int a, b;
  initial begin
    #2;
    a = h.li.us2.a; b = h.li.us2.b;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    assert_eq!((u(&sim, "a"), u(&sim, "b")), (0x11, 0x2233));
}
