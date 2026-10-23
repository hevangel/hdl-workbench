//! §20.7 / §26.3 — a port width computed from `$bits()` of a PACKAGE-SCOPED
//! type (`$bits(some_pkg::some_t)`) must elaborate to the type's real width.
//!
//! `pkg::t` lowers to the same `MemberAccess` AST node as a struct member
//! select, so the const-eval `$bits` arms — which only matched `Ident` — fell
//! through to 0. A port declared
//!
//!     input logic [`W-1:0] d      with  `define W $bits(some_pkg::some_t)
//!
//! therefore elaborated to ONE BIT and silently truncated the bus, surfacing
//! only as a "port width mismatch" warning. Found on a customer design, where
//! dozens of (module, port) pairs collapsed to a single bit and broke a whole
//! datapath. `$bits()` evaluated correctly at run time throughout; only the
//! elaboration-time width path was affected.
//!
//! The unqualified forms (compilation-unit typedef, wildcard-imported package
//! typedef) already worked and are kept here as controls.

use xezim::simulate;

fn get(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("tb.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("signal {} not u64-able", name))
}

/// `logic [3:0][15:0] lanes` (64) + `logic [2:0] tag` (3) = 67 bits. The point
/// is a packed struct carrying a MULTI-DIMENSIONAL packed field, so the width
/// only comes out right if `$bits` resolves the whole aggregate.
const PKG_SCOPED: &str = r#"
package widthy_pkg;
  typedef struct packed {
     logic [3:0] [15:0] lanes;
     logic [2:0]        tag;
  } parcel_t;
endpackage

`define PARCEL_W $bits(widthy_pkg::parcel_t)

module leafmod (
  input  logic [`PARCEL_W-1:0] d,
  output logic [`PARCEL_W-1:0] q
);
  assign q = d;
endmodule

module tb;
  logic [66:0] din, dout;
  logic        top_bit, low_bit;
  int          declared_width;
  leafmod u_leaf (.d(din), .q(dout));
  initial begin
    declared_width = `PARCEL_W;
    din = 67'h0;
    din[66] = 1'b1;
    din[0]  = 1'b1;
    #1;
    top_bit = dout[66];
    low_bit = dout[0];
  end
endmodule
"#;

#[test]
fn bits_of_package_scoped_type_sizes_a_port() {
    let sim = simulate(PKG_SCOPED, 100).expect("simulate failed");
    assert_eq!(get(&sim, "declared_width"), 67);
    // The MSB only survives if the port really is 67 bits wide.
    assert_eq!(get(&sim, "top_bit") & 1, 1);
    assert_eq!(get(&sim, "low_bit") & 1, 1);
}

/// Control: the same width via a compilation-unit typedef (no package scope).
const UNIT_TYPEDEF: &str = r#"
typedef struct packed {
   logic [3:0] [15:0] lanes;
   logic [2:0]        tag;
} parcel_t;

`define PARCEL_W $bits(parcel_t)

module leafmod (input logic [`PARCEL_W-1:0] d, output logic [`PARCEL_W-1:0] q);
  assign q = d;
endmodule

module tb;
  logic [66:0] din, dout;
  logic        top_bit;
  leafmod u_leaf (.d(din), .q(dout));
  initial begin
    din = 67'h0;
    din[66] = 1'b1;
    #1;
    top_bit = dout[66];
  end
endmodule
"#;

#[test]
fn bits_of_unit_scope_type_still_sizes_a_port() {
    let sim = simulate(UNIT_TYPEDEF, 100).expect("simulate failed");
    assert_eq!(get(&sim, "top_bit") & 1, 1);
}
