//! §18.5.10: without `solve … before`, every solution of the constraint set
//! is equally likely. An antecedent whose value selects the SIZE of the
//! consequent's solution space must therefore be drawn in proportion to
//! that size. `sel inside {0,1,2}` with two narrow address windows and one
//! complement window gave one third each: `sel` was drawn uniformly and the
//! address repaired afterwards. Both the reference simulator and the LRM
//! put the complement case in the clear majority.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn antecedent_is_weighted_by_consequent_solution_space() {
    let msgs = messages(
        "class router_packet;
  rand bit [1:0]  sel; rand bit [31:0] paddr;
  constraint c_set { sel inside {0, 1, 2}; }
  constraint c_route {
    (sel == 0) -> (paddr inside {[32'h200 : 32'h3FF]});
    (sel == 1) -> (paddr inside {[32'h400 : 32'h5FF]});
    (sel == 2) -> !(paddr inside {[32'h200 : 32'h5FF]});
  }
endclass
module tb;
  int b0 = 0, b1 = 0, b2 = 0, bad = 0;
  initial begin
    router_packet p = new();
    repeat (400) begin
      if (!p.randomize()) bad++;
      case (p.sel)
        0: begin b0++; if (!(p.paddr >= 32'h200 && p.paddr < 32'h400)) bad++; end
        1: begin b1++; if (!(p.paddr >= 32'h400 && p.paddr < 32'h600)) bad++; end
        2: begin b2++; if (p.paddr >= 32'h200 && p.paddr < 32'h600) bad++; end
        default: bad++;
      endcase
    end
    $display(\"DIST b0=%0d b1=%0d b2=%0d bad=%0d majority=%0d\", b0, b1, b2, bad, b2 > 200);
  end
endmodule",
    );
    let line = msgs
        .iter()
        .find(|m| m.starts_with("DIST "))
        .expect("no DIST line");
    assert!(line.ends_with("bad=0 majority=1"), "{line}");
}

#[test]
fn solve_before_keeps_the_antecedent_uniform() {
    // With `solve sel before paddr` the antecedent is drawn on its own
    // (§18.5.10.1): the distribution must NOT be weighted.
    let msgs = messages(
        "class pkt;
  rand bit [1:0] sel; rand bit [31:0] paddr;
  constraint c_set { sel inside {0, 1, 2}; }
  constraint c_route {
    (sel == 0) -> (paddr inside {[32'h200 : 32'h3FF]});
    (sel == 1) -> (paddr inside {[32'h400 : 32'h5FF]});
    (sel == 2) -> !(paddr inside {[32'h200 : 32'h5FF]});
  }
  constraint c_order { solve sel before paddr; }
endclass
module tb;
  int b0 = 0, b1 = 0, b2 = 0, bad = 0;
  initial begin
    pkt p = new();
    repeat (600) begin
      if (!p.randomize()) bad++;
      case (p.sel)
        0: begin b0++; if (!(p.paddr >= 32'h200 && p.paddr < 32'h400)) bad++; end
        1: begin b1++; if (!(p.paddr >= 32'h400 && p.paddr < 32'h600)) bad++; end
        2: begin b2++; if (p.paddr >= 32'h200 && p.paddr < 32'h600) bad++; end
        default: bad++;
      endcase
    end
    $display(\"DIST b0=%0d b1=%0d b2=%0d bad=%0d spread=%0d\", b0, b1, b2, bad, b0 > 100 && b1 > 100 && b2 > 100);
  end
endmodule",
    );
    let line = msgs
        .iter()
        .find(|m| m.starts_with("DIST "))
        .expect("no DIST line");
    assert!(line.ends_with("bad=0 spread=1"), "{line}");
}
