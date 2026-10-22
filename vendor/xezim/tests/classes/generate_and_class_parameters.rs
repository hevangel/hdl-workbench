//! Parameter corners in generate blocks and parameterized classes, found by a
//! probe sweep and reference-validated:
//!
//! 1. §27.4: a generate-scope localparam used as an INSTANCE PARAMETER
//!    OVERRIDE (`#(.W(MYW))`) resolved to nothing — the per-iteration rename
//!    rewrote port connections but not parameter overrides, so the child
//!    elaborated with 0-width parameters (and could panic downstream).
//! 2. §27.6: a generate-block localparam read hierarchically
//!    (`gb[2].MYIDX`) returned 0 — no scoped alias was registered.
//! 3. A class-body localparam sized from a class parameter
//!    (`localparam int MYW = W`) baked in the value computed with an EMPTY
//!    parameter map (0), and every specialization read that.
//! 4. `$bits(logic [W-1:0])` and `$bits(T)` inside a parameterized class
//!    method collapsed to 1 — module parameters carry no class specialization,
//!    and a type parameter's default AST was dropped (only a lossy text
//!    fragment survived, `logic [W-1:0]` → `logic`).

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

#[test]
fn generate_localparam_as_instance_override() {
    let src = r#"
module leafm #(parameter int IDX = 0, parameter int W = 4) ();
  localparam int SEEN = IDX + W;
  int seen;
  initial seen = SEEN;
endmodule
module tb;
  genvar g;
  generate
    for (g = 0; g < 2; g++) begin : gb
      localparam int MYIDX = g * 3;
      localparam int MYW   = 8;
      leafm #(.IDX(MYIDX), .W(MYW)) u();
    end
  endgenerate
  int s0, s1, hier;
  initial begin
    #1;
    s0 = gb[0].u.seen;      // 0 + 8
    s1 = gb[1].u.seen;      // 3 + 8
    hier = gb[1].MYIDX;     // hierarchical read of a generate localparam
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "s0"),
        8,
        "generate localparam must reach the override"
    );
    assert_eq!(u(&sim, "s1"), 11);
    assert_eq!(
        u(&sim, "hier"),
        3,
        "generate localparam is visible hierarchically"
    );
}

#[test]
fn class_body_localparam_per_specialization() {
    let src = r#"
class simple #(parameter int W = 8);
  localparam int MYW = W;
  function int getw(); return MYW; endfunction
  function int getbits(); return $bits(logic [W-1:0]); endfunction
endclass
module tb;
  simple #(16) a;
  simple #(4)  b;
  simple #()   c;
  int r_a, r_b, r_c, bits_a;
  initial begin
    a = new(); b = new(); c = new();
    r_a = a.getw(); r_b = b.getw(); r_c = c.getw();
    bits_a = a.getbits();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "r_a"),
        16,
        "body localparam follows the specialization"
    );
    assert_eq!(u(&sim, "r_b"), 4);
    assert_eq!(u(&sim, "r_c"), 8, "unspecialized keeps the default");
    assert_eq!(
        u(&sim, "bits_a"),
        16,
        "$bits of a class-parameter-sized type"
    );
}

#[test]
fn type_parameter_default_sized_by_value_parameter() {
    let src = r#"
class pclass #(parameter int W = 8, parameter type T = logic [W-1:0]);
  localparam int MYW = $bits(T);
  function int getw(); return MYW; endfunction
endclass
module tb;
  pclass #(16) c16;
  pclass #(.W(4)) c4;
  pclass #() cdef;
  int w16, w4, wdef;
  initial begin
    c16 = new(); c4 = new(); cdef = new();
    w16 = c16.getw(); w4 = c4.getw(); wdef = cdef.getw();
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "w16"),
        16,
        "type-param default re-sizes per specialization"
    );
    assert_eq!(u(&sim, "w4"), 4);
    assert_eq!(u(&sim, "wdef"), 8);
}
