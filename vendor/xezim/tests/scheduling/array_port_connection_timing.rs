//! §23.3.3.3: variable array connections propagate through an implicit
//! continuous assignment. A blocking write is not visible at the child
//! port until that assignment runs; only net-to-net ports may collapse.

const SRC: &str = r#"
module net8(input wire logic [7:0] a[0:1]); endmodule
module var8(input var logic [7:0] a[0:1]); endmodule
module net1(input wire a[0:1]); endmodule
module net64(input wire [63:0] a[0:1]); endmodule
module net2d(input wire [7:0] a[0:1][0:1]); endmodule
module middle(input wire [7:0] a[0:1]); net8 child(a); endmodule
module top;
  logic [7:0] m[0:1];
  logic b[0:1];
  logic [63:0] w[0:1];
  logic [7:0] g[0:1][0:1];
  net8 un(m); var8 uv(m); net1 ub(b); net64 uw(w); net2d ug(g); middle um(m);
  initial begin
    m[0]=8'h11; m[1]=8'h22;
    b[0]=0; b[1]=0;
    w[0]=64'h1111111111111111; w[1]=0;
    g[0][0]=8'h11; g[0][1]=0; g[1][0]=0; g[1][1]=0;
    #1;
    m[0]=8'h55; b[0]=1; w[0]=64'h5555555555555555; g[0][0]=8'h55;
    $display("T|active %h %h %b %h %h %h", un.a[0], uv.a[0], ub.a[0], uw.a[0], ug.a[0][0], um.child.a[0]);
    #0;
    $display("T|inactive %h %h %b %h %h %h", un.a[0], uv.a[0], ub.a[0], uw.a[0], ug.a[0][0], um.child.a[0]);
    #1 $finish;
  end
endmodule
"#;

#[test]
fn variable_array_ports_preserve_connection_timing() {
    let sim = xezim::simulate(SRC, 1000).expect("array port simulation");
    let text = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("T|active 11 11 0 1111111111111111 11 11"),
        "{text}"
    );
    assert!(
        text.contains("T|inactive 55 55 1 5555555555555555 55 55"),
        "{text}"
    );
}
