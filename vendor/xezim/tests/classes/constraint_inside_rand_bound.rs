//! §11.4.13 / §18.5 — `inside` ranges whose BOUNDS read rand variables
//! (`dst inside {[BASE : TOP - size*4]}` with `solve size before dst`). The
//! joint solver left such a set to the evaluator, which can only judge it
//! once every operand is fixed, so a 32-bit `dst` was searched value by value
//! over its whole domain; each refuted value split the domain further and the
//! search spent seconds per call before giving up. Each range now propagates
//! as `lo <= x <= hi`, and the refutation work counts against the budget.
//! Counts cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class blk;
  rand logic [31:0] src_addr;
  rand logic [31:0] dst_addr;
  rand int transfer_size;
  constraint page_size { transfer_size inside {[1:1024]}; }
  constraint address_alignment { src_addr[1:0] == 0; dst_addr[1:0] == 0; }
endclass

class pair;
  rand bit [15:0] lo;
  rand bit [15:0] v;
  rand bit [7:0] n;
  constraint c { lo inside {[100:200]}; }
endclass

module top;
  initial begin
    blk b = new;
    pair p = new;
    int ok, bad, viol;
    ok = 0; bad = 0; viol = 0;
    for (int k = 0; k < 40; k++) begin
      if (!b.randomize() with {src_addr inside {[32'h0100_0000:32'h0100_FFFF]};
                               dst_addr inside {[32'h0103_0000:(32'h0104_0000 - (transfer_size*4))]};
                               transfer_size < 512;
                               solve transfer_size before dst_addr;}) bad++;
      else if (b.transfer_size < 1 || b.transfer_size >= 512
               || b.src_addr < 32'h0100_0000 || b.src_addr > 32'h0100_FFFF
               || b.dst_addr < 32'h0103_0000 || b.dst_addr > 32'h0104_0000 - b.transfer_size*4
               || b.src_addr[1:0] != 0 || b.dst_addr[1:0] != 0) viol++;
      else ok++;
    end
    $display("R1 ok=%0d fail=%0d viol=%0d", ok, bad, viol);
    ok = 0; bad = 0; viol = 0;
    for (int k = 0; k < 40; k++) begin
      if (!p.randomize() with { v inside {[lo + 1 : lo + 3], 16'h7000}; n inside {[lo[3:0] : 8'd20]}; }) bad++;
      else if (!((p.v >= p.lo + 1 && p.v <= p.lo + 3) || p.v == 16'h7000) || p.n < p.lo[3:0] || p.n > 20) viol++;
      else ok++;
    end
    $display("R2 ok=%0d fail=%0d viol=%0d", ok, bad, viol);
  end
endmodule
"#;

#[test]
fn inside_with_rand_dependent_bounds() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with('R'))
        .collect();
    assert_eq!(got, ["R1 ok=40 fail=0 viol=0", "R2 ok=40 fail=0 viol=0"]);
}
