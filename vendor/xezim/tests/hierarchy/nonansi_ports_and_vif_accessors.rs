//! Three clusters from user testbenches + ivtest, all reference-validated.
//!
//! 1. **§7.4.1 non-ANSI PORT shape registrations.** A port declared
//!    non-ANSI (`(o, x); output logic [3:0][9:0] o;`) never joined
//!    `packed_signal_elem_widths` / `packed_full_dims` /
//!    `packed_struct_fields` — the DataDeclaration arm has always registered
//!    those, the port arms (top and inlined-submodule) did not. So
//!    `assign o[1] = v;` degraded to a bit-window write (only the low bits
//!    landed) and struct-port member assigns lost the field layout. Surfaced
//!    by a CDC design whose loopback (`struct member -> input port ->
//!    packed-element assign -> output`) returned x.
//! 2. **§13.3 non-ANSI task/function ports with a SEPARATE type
//!    declaration** (`input x;  int x;`): the second line parsed as an
//!    ordinary local that shadowed the 1-bit implicit port. The parser now
//!    merges such a declaration's type into the matching implicit-typed port
//!    (ivtest `task_nonansi_int1`/`enum1`).
//! 3. **§25.9 virtual interfaces with MODPORTS through class accessors**:
//!    `virtual bus_if #(4).driver` as a function RETURN type did not parse;
//!    a module-scope `virtual req_if #(4).driver rd;` declaration parsed as
//!    a parameterized INSTANTIATION; `rv = rb.driver` (a modport VIEW as the
//!    bound actual) bound nothing, so writes landed on a phantom copy; and a
//!    vif returned FROM a function carried only its sentinel value, losing
//!    the binding. All four fixed — the manager-class pattern
//!    (`mgr.get_req_interface(...)`) now drives the real interface.
//!
//! KNOWN GAP (pre-existing, not from the merge): struct MEMBER reads on a
//! task/function formal (`formal.field`) read 0 in BOTH ANSI and non-ANSI
//! styles — ivtest `task_nonansi_struct1/2`, `task_nonansi_parray1` still
//! fail on that; the width half of those tests is fixed.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Non-ANSI packed-2D output port, element assigns — whole and per element.
#[test]
fn nonansi_packed_2d_port_element_assigns() {
    let src = r#"
module d4 (o2d, x);
  output logic [3:0][9:0] o2d;
  input logic [9:0] x;
  assign o2d[0] = x;
  assign o2d[1] = 10'h111;
  assign o2d[2] = 10'h222;
  assign o2d[3] = 10'h333;
endmodule
module tb;
  logic [9:0] v;
  wire [3:0][9:0] pt;
  d4 u (.o2d(pt), .x(v));
  int p0, p1, p3;
  logic [39:0] whole;
  initial begin
    v = 10'h0AA;
    #1;
    p0 = pt[0]; p1 = pt[1]; p3 = pt[3]; whole = pt;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "p0"), 0x0AA);
    assert_eq!(u(&sim, "p1"), 0x111);
    assert_eq!(u(&sim, "p3"), 0x333);
    assert_eq!(u(&sim, "whole"), 0xCCE22444AA, "all four elements packed");
}

/// Non-ANSI struct ports through a package type: member propagation both ways.
#[test]
fn nonansi_struct_ports_propagate_members() {
    let src = r#"
package PK;
  typedef struct packed { logic [9:0] f3; logic [9:0] f2; logic [9:0] f1; logic [9:0] f0; } st_t;
endpackage
module dut import PK::* ;
  (out4, gin);
  output logic [3:0][9:0] out4;
  input  st_t gin;
  assign out4[0] = gin.f0;
  assign out4[2] = gin.f2;
endmodule
module tb;
  import PK::*;
  st_t g;
  wire [3:0][9:0] pt;
  dut u (.out4(pt), .gin(g));
  int p0, p2;
  initial begin
    g.f0 = '0; g.f2 = '0;
    #1;
    g.f0 = 10'h1A5;
    g.f2 = 10'h25B;
    #1;
    p0 = pt[0]; p2 = pt[2];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "p0"),
        0x1A5,
        "member write reaches through the port"
    );
    assert_eq!(u(&sim, "p2"), 0x25B);
}

/// §13.3: `input x; int x;` merges the type into the port.
#[test]
fn nonansi_task_port_type_merges() {
    let src = r#"
module tb;
  int got_v, got_b;
  task t;
    input x;
    int x;
    begin
      got_v = x;
      got_b = $bits(x);
    end
  endtask
  initial t(10);
endmodule
"#;
    let sim = simulate(src, 20).expect("simulate failed");
    assert_eq!(u(&sim, "got_v"), 10, "the full value, not 1 bit");
    assert_eq!(u(&sim, "got_b"), 32, "$bits sees the merged int type");
}

/// §25.9: the interface-manager pattern — modport-typed vifs stored in a
/// class, retrieved through accessor functions, driving the real interface.
#[test]
fn vif_modport_accessors_bind_the_instance() {
    let src = r#"
interface req_if #(parameter N = 4);
  logic [N-1:0] req;
  modport driver (output req);
endinterface
interface sc_if #(parameter W = 8, parameter N = 2);
  logic [W-1:0] counter[N];
  modport receiver (input counter);
endinterface
class mgr_c;
  virtual req_if #(4).driver req_vif;
  virtual sc_if #(8,2).receiver sync_vif;
  function new(virtual req_if #(4).driver r, virtual sc_if #(8,2).receiver s);
    this.req_vif = r;
    this.sync_vif = s;
  endfunction
  function automatic virtual req_if #(4).driver get_req();
    return req_vif;
  endfunction
  function automatic virtual sc_if #(8,2).receiver get_sync();
    return sync_vif;
  endfunction
endclass
module tb;
  req_if #(4) rb();
  sc_if #(8,2) sb();
  mgr_c m;
  virtual req_if #(4).driver rd;
  virtual sc_if #(8,2).receiver sr;
  int through_iface, through_vif, c0, c1;
  initial begin
    m = new(rb.driver, sb.receiver);
    rd = m.get_req();
    sr = m.get_sync();
    rd.req = 4'b1010;
    sb.counter[0] = 8'd21;
    sb.counter[1] = 8'd99;
    #1;
    through_iface = rb.req;      // the write reached the REAL interface
    through_vif   = rd.req;
    c0 = sr.counter[0];          // array member read through the vif
    c1 = sr.counter[1];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "through_iface"),
        0b1010,
        "vif write aliases the instance"
    );
    assert_eq!(u(&sim, "through_vif"), 0b1010);
    assert_eq!(
        u(&sim, "c0"),
        21,
        "unpacked member through an accessor-returned vif"
    );
    assert_eq!(u(&sim, "c1"), 99);
}
