//! ANSI port-list ambiguity from a user DRAM-BFM testbench that "worked only
//! in the lenient simulator": `output v0vJ[t-1:0],` — an IMPLICIT-typed port
//! whose NAME carries an unpacked dimension. The parser matched
//! `Identifier [` as a typedef-with-packed-dims and then died on the comma
//! where the port name should be.
//!
//! Disambiguation: look past the balanced bracket group(s) — an IDENTIFIER
//! there means the first token really was a type (`typedef_t [7:0] name`);
//! a comma / `)` / `=` means it was the port name.
//!
//! (The strict-LRM note: an implicit ANSI output is a NET, and the reference
//! simulator rejects procedural writes to it outright. xezim
//! keeps its existing leniency and runs such designs, matching the lenient
//! vendor the testbench was written for.)

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// Implicit-typed ports with unpacked dims on the name, next to every shape
/// that already worked.
#[test]
fn implicit_port_name_with_unpacked_dimension() {
    let src = r#"
typedef logic [7:0] byte_t;
module duT #(parameter t = 2) (
  output v_out[t-1:0],
  output [15:0] w_out[t-1:0],
  input  clk,
  input  [31:0] a_in[t-1:0],
  byte_t [1:0] typed_port,       // a REAL type + packed dims + name: must
                                 // still parse as a type
  input  plain
);
  assign v_out[0] = plain;
  assign v_out[1] = ~plain;
  assign w_out[0] = a_in[0][15:0];
  assign w_out[1] = a_in[1][31:16];
  assign typed_port = 16'hA55A;
endmodule
module tb;
  logic clk = 0, p = 1;
  logic [31:0] a[1:0];
  wire v[1:0];
  wire [15:0] w[1:0];
  wire [15:0] tp;
  duT #(.t(2)) u (.v_out(v), .w_out(w), .clk(clk), .a_in(a), .typed_port(tp), .plain(p));
  int v0, v1, w0, w1, tpv;
  initial begin
    a[0] = 32'h1234_ABCD; a[1] = 32'hCAFE_0000;
    #1;
    v0 = v[0]; v1 = v[1]; w0 = w[0]; w1 = w[1]; tpv = tp;
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(
        u(&sim, "v0"),
        1,
        "implicit 1-bit port, unpacked dim on the name"
    );
    assert_eq!(u(&sim, "v1"), 0);
    assert_eq!(
        u(&sim, "w0"),
        0xABCD,
        "ranged port with unpacked dim on the name"
    );
    assert_eq!(u(&sim, "w1"), 0xCAFE);
    assert_eq!(
        u(&sim, "tpv"),
        0xA55A,
        "typedef [dims] name still parses as a TYPE"
    );
}
