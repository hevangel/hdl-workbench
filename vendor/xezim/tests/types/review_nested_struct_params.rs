//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    xezim::simulate(src, 1_000_000)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn nested_array_parameter_member_runtime() {
    let got = output(
        r###"
typedef struct packed {
  logic       accept;
  logic [1:0] gpr;
} inner_t;
typedef struct packed {
  logic [31:0] instr;
  inner_t      rsp;
} outer_t;
module t;
  localparam inner_t INNERS[1] = '{'{accept: 1'b1, gpr: 2'b01}};
  localparam outer_t OUTERS[1] = '{'{instr: 32'h0400702b, rsp: '{accept: 1'b1, gpr: 2'b01}}};
  localparam int inner_arr = INNERS[0].accept;
  localparam int inner_bits = $bits(INNERS[0].accept);
  localparam int chain = OUTERS[0].rsp.accept;
  localparam int chain_bits = $bits(OUTERS[0].rsp.accept);
  localparam int whole = OUTERS[0].rsp;
initial begin $display("T|nested %0d %0d %0d",chain,chain_bits,whole); $finish; end
endmodule
"###,
    );
    assert!(got.contains("T|nested 1 1 5"), "{got}");
}
