//! Two user-testbench defects behind "the DUT lags its reference model" and
//! "the queue never drains", both reference-validated, both instance-only.
//!
//! 1. **§12.7.1 / §6.21 — for-init loop variables are loop-scoped.** They were
//!    stored globally by bare name, so a comb block's `for (int i...)` executed
//!    by a settle triggered MID-ITERATION of another block's same-named loop
//!    (blocking assigns settle synchronously) clobbered the interrupted
//!    counter — that loop silently exited after ONE iteration.
//! 2. **Instance comb sensitivity through a port copy.** A port named like its
//!    connected signal resolves at runtime to the instance's `dut.grant` copy,
//!    which updates a settle-cascade step later than the top-level signal; the
//!    dep graph only listed the top name, so the comb read one NBA application
//!    behind and the whole datapath lagged the TB model by a clock.
//! 3. **Queue of unpacked structs inside an instance.** The inlined queue
//!    never registered its ELEMENT type, so `push_back(item)` collapsed to one
//!    packed x; member writes (`q[i].latency--`) fell through every lvalue arm
//!    and vanished; and a signed member (`integer`) read back unsigned, so
//!    `q[0].latency <= 0` went false once the count went negative.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able (x/z?)", n))
}

/// The interrupted-loop clobber, distilled: an initial's for loop whose body's
/// first blocking assign triggers a settle that runs an always_comb containing
/// a same-named `for (int i...)`.
#[test]
fn settle_does_not_clobber_an_interrupted_loops_counter() {
    let src = r#"
module tb;
  logic [3:0] lvl [4];
  logic [3:0] nxt [4];
  int f0, f1, f2, f3;
  always_comb begin
    for (int i = 0; i < 4; i++) nxt[i] = lvl[i];
  end
  initial begin
    for (int i = 0; i < 4; i++) lvl[i] = i + 1;
    #1;
    f0 = nxt[0]; f1 = nxt[1]; f2 = nxt[2]; f3 = nxt[3];
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert_eq!(u(&sim, "f0"), 1);
    assert_eq!(
        u(&sim, "f1"),
        2,
        "iteration 2 ran (loop var survived settle)"
    );
    assert_eq!(u(&sim, "f2"), 3);
    assert_eq!(u(&sim, "f3"), 4);
}

/// FIFO level tracker vs inline model: the instance comb must see the port
/// value applied THIS timestep, so the DUT matches the model cycle-for-cycle
/// (`0 - 1` wraps to 15 in 4 bits on both sides, same edge).
#[test]
fn instance_comb_tracks_port_written_by_nba() {
    let src = r#"
module duty(input logic clk, input logic rst, input logic grant,
            input logic [15:0] sel);
  logic bq_rd [16];
  logic [3:0] level [16];
  logic [3:0] level_nxt [16];
  always_comb begin
    for (int i = 0; i < 16; i++) begin
      bq_rd[i] = grant && sel[i];
      level_nxt[i] = level[i] - bq_rd[i];
    end
  end
  always_ff @(posedge clk) begin
    if (rst) begin
      for (int i = 0; i < 16; i++) level[i] <= '0;
    end else begin
      for (int i = 0; i < 16; i++)
        if (bq_rd[i]) level[i] <= level_nxt[i];
    end
  end
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic rst, grant;
  logic [15:0] sel;
  logic [3:0] exp_level [16];
  int mismatches = 0;
  duty dut(.clk(clk), .rst(rst), .grant(grant), .sel(sel));
  always @(posedge clk) begin
    if (rst) begin
      for (int i = 0; i < 16; i++) exp_level[i] <= '0;
    end else begin
      for (int i = 0; i < 16; i++) begin
        bit rd;
        rd = grant && sel[i];
        exp_level[i] <= exp_level[i] - rd;
      end
    end
  end
  always @(posedge clk)
    if (!rst)
      for (int i = 0; i < 16; i++)
        if (!(dut.level[i] === exp_level[i])) mismatches++;
  int l12;
  initial begin
    rst = 1; grant = 0; sel = '0;
    repeat (3) @(posedge clk);
    rst = 0;
    @(negedge clk);
    grant <= 1; sel <= 16'h1000;
    @(negedge clk);
    grant <= 0; sel <= '0;
    repeat (3) @(posedge clk);
    l12 = dut.level[12];
  end
endmodule
"#;
    let sim = simulate(src, 200).expect("simulate failed");
    assert_eq!(
        u(&sim, "mismatches"),
        0,
        "DUT must match the model every edge"
    );
    assert_eq!(u(&sim, "l12"), 15, "0 - 1 wraps to 15 in 4 bits");
}

/// Queue-of-structs router inside an instance: push with member values, decay
/// the (signed) latency past zero, pop everything back out in order.
#[test]
fn instance_queue_of_structs_drains() {
    let src = r#"
module router(input logic clk, input logic rst_n, input logic vld,
              output logic lane_valid, output logic [15:0] oseq);
  typedef struct {
    logic [15:0] seq;
    integer latency;
  } item_t;
  item_t q[$];
  integer i;
  logic [15:0] nseq;
  always_ff @(posedge clk) begin
    if (!rst_n) begin
      q.delete();
      lane_valid <= 0;
      nseq <= 0;
    end else begin
      lane_valid <= 0;
      if (vld) begin
        item_t item;
        item.seq = nseq;
        item.latency = 3 + nseq[1:0];   // varying, so some go NEGATIVE first
        q.push_back(item);
        nseq <= nseq + 1;
      end
      for (i = 0; i < q.size(); i++)
        q[i].latency--;
      if (q.size() > 0 && q[0].latency <= 0) begin
        lane_valid <= 1;
        oseq <= q[0].seq;
        q.pop_front();
      end
    end
  end
endmodule
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  logic rst_n = 0, vld = 0;
  logic lane_valid;
  logic [15:0] oseq;
  router u(.clk(clk), .rst_n(rst_n), .vld(vld), .lane_valid(lane_valid), .oseq(oseq));
  int got = 0, in_order = 1, qs;
  logic [15:0] expect_seq = 0;
  always @(posedge clk) if (lane_valid) begin
    if (oseq !== expect_seq) in_order = 0;
    expect_seq <= expect_seq + 1;
    got++;
  end
  initial begin
    repeat (3) @(posedge clk);
    rst_n = 1;
    @(negedge clk); vld = 1;
    repeat (4) @(negedge clk);
    vld = 0;
    repeat (30) @(posedge clk);
    qs = u.q.size();
  end
endmodule
"#;
    let sim = simulate(src, 500).expect("simulate failed");
    assert_eq!(u(&sim, "got"), 4, "all four items must drain");
    assert_eq!(u(&sim, "qs"), 0, "queue empty at the end");
    assert_eq!(u(&sim, "in_order"), 1, "FIFO order preserved");
}
