//! String-RETURNING functions inline into compiled blocks: the result
//! register binds at width 0 (§6.16 — any resize truncates the FRONT of the
//! text, and `infer_lhs_width`'s 32-bit default did exactly that through a
//! string local), early `return` inside case arms compiles to result-move +
//! jump patched to the body end, and `$sformatf` counts as pure in the
//! purity walk so a get_csr_name-style helper (case of returns with a
//! formatted default) qualifies at all. Byte-verified against the
//! reference simulator.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("SFN"))
        .collect()
}

const SRC: &str = r#"
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  string decoded;
  logic [11:0] csr;
  logic [4:0] rd;
  logic [1:0] mode;

  function automatic string reg_name(input logic [11:0] addr);
    unique case (addr)
      12'd0:   return "ustatus";
      12'd4:   return "uie";
      12'd5:   return "utvec";
      12'd256: return "sstatus";
      12'd260: return "sie";
      12'd261: return "stvec";
      12'd768: return "mstatus";
      12'd769: return "misa";
      12'd772: return "mie";
      12'd773: return "mtvec";
      default: return $sformatf("0x%x", addr);
    endcase
  endfunction

  function automatic string early(input logic [4:0] v);
    if (v == 5'd0) return "zero";
    if (v < 5'd4) return $sformatf("small_%0d", v);
    return "big";
  endfunction

  function automatic void decode_csr(input string mnemonic);
    string nm;
    nm = reg_name(csr);
    decoded = $sformatf("%s\tx%0d,%s,%s", mnemonic, rd, nm, early(rd));
  endfunction

  always @(posedge clk) begin
    case (mode)
      2'd0: decode_csr("csrrw");
      2'd1: decoded = reg_name(csr);
      default: decoded = "idle";
    endcase
  end
  task check(input logic [1:0] m, input logic [11:0] c, input logic [4:0] r);
    mode = m; csr = c; rd = r; @(negedge clk); $display("SFN [%s]", decoded);
  endtask
  initial begin
    check(0, 12'd768, 5'd3);
    check(0, 12'd999, 5'd0);
    check(0, 12'd260, 5'd2);
    check(1, 12'd773, 5'd9);
    check(1, 12'd123, 5'd9);
    check(2, 12'd0, 5'd0);
    $finish;
  end
endmodule
"#;

#[test]
fn string_returning_fn_inlines_with_early_returns() {
    assert_eq!(
        notes(SRC),
        [
            "SFN [csrrw\tx3,mstatus,small_3]",
            "SFN [csrrw\tx0,0x3e7,zero]",
            "SFN [csrrw\tx2,sie,small_2]",
            "SFN [mtvec]",
            "SFN [0x07b]",
            "SFN [idle]",
        ]
    );
}
