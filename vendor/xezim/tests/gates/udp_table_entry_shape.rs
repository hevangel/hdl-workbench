//! §29.3.6: a combinational UDP table entry is `inputs : output` and a
//! sequential one `inputs : state : output`, with one symbol per input and a
//! single state and output symbol. Each rejected table is also rejected by
//! the reference simulator; the legal primitives print the same lines there.

use xezim::simulate;

#[test]
fn malformed_udp_table_entries() {
    for (hdr, row) in [
        ("output id_2, input id_1", "? 1 ? 0 0 0 : 0;"),
        ("output reg id_2, input id_1", "? 1 ? 0 0 0 : 0 : 0;"),
        ("output id_2, input id_1", "0 : 0 0;"),
        ("output reg id_2, input id_1", "0 : 0 0 : 0;"),
        ("output reg id_2, input id_1", "0 : 0 : 0 0;"),
        ("output reg id_2, input id_1", "0 : 1 : 0 : 1;"),
    ] {
        let src = format!(
            "primitive id_0({hdr});\n  table\n    {row}\n  endtable\nendprimitive\n\
             module top; wire o; reg i; id_0 u(o, i); endmodule\n"
        );
        assert!(simulate(&src, 10).is_err(), "accepted:\n{src}");
    }
}

#[test]
fn well_formed_udp_tables() {
    let src = r#"
primitive mux2 (out, ctl, in0, in1);
  output out;
  input ctl, in0, in1;
  table
    0 0 ? : 0;
    0 1 ? : 1;
    1 ? 0 : 0;
    1 ? 1 : 1;
    x 0 0 : 0;
    x 1 1 : 1;
  endtable
endprimitive
primitive dff (q, clk, d);
  output q; reg q;
  input clk, d;
  table
    (01) 0 : ? : 0;
    (01) 1 : ? : 1;
    (0?) 1 : 1 : 1;
    (0?) 0 : 0 : 0;
    (?0) ? : ? : -;
    ? (??) : ? : -;
  endtable
endprimitive
module top;
  reg ctl, a, b, clk, d;
  wire m, q;
  mux2 u0(m, ctl, a, b);
  dff u1(q, clk, d);
  initial begin
    ctl = 0; a = 1; b = 0; clk = 0; d = 1;
    #1 clk = 1;
    #1 $display("U|%b %b", m, q);
    ctl = 1; d = 0;
    #1 clk = 0;
    #1 clk = 1;
    #1 $display("U|%b %b", m, q);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("well-formed UDPs must run");
    let lines: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|o| o.message.strip_prefix("U|"))
        .collect();
    assert_eq!(lines, ["1 1", "0 0"], "{:?}", sim.output);
}
