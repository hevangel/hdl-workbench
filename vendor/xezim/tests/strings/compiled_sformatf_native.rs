//! `$sformatf` with a literal template and specs in the native subset
//! (d/b/h/x/o/s/c, `-` flag, widths) compiles to a template parsed once and
//! filled from register Values — and a VOID function called as a statement
//! inlines like a task (§13.4.1), with `string` formals bound unresized and
//! `string` signal stores skipping the placeholder-width resize (§6.16).
//! Every expectation below is byte-verified against the reference simulator.
//! The MWE mirrors the RISC-V tracer shape that motivated it: decode helpers
//! taking a mnemonic string and formatting register fields.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("FMT"))
        .collect()
}

const SRC: &str = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  string decoded;
  logic [4:0] rd, rs1, rs2;
  logic [31:0] imm;
  logic [31:0] insn;

  function automatic void decode_r(input string mnemonic);
    decoded = $sformatf("%s\tx%0d,x%0d,x%0d", mnemonic, rd, rs1, rs2);
  endfunction
  function automatic void decode_i(input string mnemonic);
    decoded = $sformatf("%s\tx%0d,x%0d,%0d", mnemonic, rd, rs1, $signed(imm));
  endfunction
  function automatic void decode_h(input string mnemonic);
    decoded = $sformatf("%s\tx%0d,0x%0x h=%h b=%b o=%o c=%c s5=%5s", mnemonic, rd, imm, imm, rd, rd, 8'h41, "ab");
  endfunction

  always @(posedge clk) begin
    case (insn[1:0])
      2'd0: decode_r("add");
      2'd1: decode_i("addi");
      2'd2: decode_h("lui");
      default: decoded = "unknown";
    endcase
  end
  task check(input logic [31:0] i);
    insn = i; @(negedge clk); $display("FMT [%s]", decoded);
  endtask
  initial begin
    rd = 5'd3; rs1 = 5'd11; rs2 = 5'd31; imm = 32'hffff_fff6;
    check(0);
    check(1);
    check(2);
    check(3);
    rd = 5'd0; imm = 32'h0000_002a;
    check(0);
    check(1);
    check(2);
    $finish;
  end
endmodule
"#;

#[test]
fn native_sformatf_matches_reference() {
    assert_eq!(
        notes(SRC),
        [
            "FMT [add\tx3,x11,x31]",
            "FMT [addi\tx3,x11,-10]",
            "FMT [lui\tx3,0xfffffff6 h=fffffff6 b=00011 o=03 c=A s5=   ab]",
            "FMT [unknown]",
            "FMT [add\tx0,x11,x31]",
            "FMT [addi\tx0,x11,42]",
            "FMT [lui\tx0,0x2a h=0000002a b=00000 o=00 c=A s5=   ab]",
        ]
    );
}
