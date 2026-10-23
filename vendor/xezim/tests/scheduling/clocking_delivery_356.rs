//! IEEE 1800 §14.4 and §14.13: all always forms waiting on a clocking
//! event observe this edge's samples after its NBA updates.
use xezim::simulate;

fn messages(source: &str) -> Vec<String> {
    simulate(source, 1_000)
        .expect("simulate clocking delivery")
        .output
        .iter()
        .map(|line| line.message.clone())
        .collect()
}

#[test]
fn always_clocking_controls_resume_after_sampling_and_nba() {
    let output = messages(ORIGINAL);
    assert_eq!(output, ["RESULT always=0 always_begin=0 initial=0"]);
}

#[test]
fn clocking_delivery_preserves_control_forms_skews_and_scopes() {
    let output = messages(NEIGHBORS);
    assert_eq!(
        output,
        [
            "T|direct 10 0",
            "T|body 10 0",
            "T|bare 10 0",
            "T|zero 10 0",
            "T|guard 10 0",
            "T|negedge 10 0",
            "T|interface 10 0",
            "T|initial 10 0",
        ]
    );
}

const ORIGINAL: &str = r#"`timescale 1ns/1ns
module top;
  logic clk = 0, d = 0, q = 0, qn = 0;
  int k = 0, fa = 0, fb = 0, fi = 0;
  always #5 clk = ~clk;
  always @(negedge clk) begin d <= k[0] ^ k[2]; k++; qn = q; end
  always_ff @(posedge clk) q <= d;
  clocking cb @(posedge clk); input q; endclocking
  // §14.13: cb.q is q from before the edge (qn, taken at the falling edge),
  // and a process waiting on cb resumes after the edge's NBA update (q == d).
  always @(cb)
    if (cb.q !== qn || q !== d) begin
      fa++;
      $display("always @(cb)       t=%0t cb.q=%b (want %b) q=%b (want %b)", $time, cb.q, qn, q, d);
    end
  always begin
    @(cb);
    if (cb.q !== qn || q !== d) fb++;
  end
  initial forever begin
    @(cb);
    if (cb.q !== qn || q !== d) fi++;
  end
  initial #103 begin
    $display("RESULT always=%0d always_begin=%0d initial=%0d", fa, fb, fi);
    $finish;
  end
endmodule
"#;

const NEIGHBORS: &str = r#"`timescale 1ns/1ns
interface sample_bus(input logic clk, q);
  clocking cb @(posedge clk); input q; endclocking
endinterface
module top;
  logic clk=0,d=0,q=0,qn=0,fall_q=0,fall_old=0,fall_d=0;
  int k=0;
  int na=0,nb=0,nc=0,nd=0,ne=0,nf=0,ng=0,ni=0;
  int fa=0,fb=0,fc=0,fd=0,fe=0,ff=0,fg=0,fi=0;
  always #5 clk=~clk;
  always @(negedge clk) begin d<=k[0]^k[2]; k++; qn=q; end
  always @(posedge clk) begin q<=d; fall_old=fall_q; fall_d=d; end
  always @(negedge clk) fall_q<=d;
  clocking cb @(posedge clk); input q; endclocking
  clocking cb0 @(posedge clk); input #0 q; endclocking
  clocking cbn @(negedge clk); input fall_q; endclocking
  sample_bus bus(clk,q);
  always @(cb) begin na++; if(cb.q!==qn || q!==d) fa++; end
  always begin @(cb); nb++; if(cb.q!==qn || q!==d) fb++; end
  always @cb begin nc++; if(cb.q!==qn || q!==d) fc++; end
  always @(cb0) begin nd++; if(cb0.q!==q || q!==d) fd++; end
  always @(cb iff(q===d)) begin ne++; if(cb.q!==qn || q!==d) fe++; end
  always @(cbn) begin nf++; if(cbn.fall_q!==fall_old || fall_q!==fall_d) ff++; end
  always @(bus.cb) begin ng++; if(bus.cb.q!==qn || q!==d) fg++; end
  initial forever begin @(cb); ni++; if(cb.q!==qn || q!==d) fi++; end
  initial #103 begin
    $display("T|direct %0d %0d",na,fa);
    $display("T|body %0d %0d",nb,fb);
    $display("T|bare %0d %0d",nc,fc);
    $display("T|zero %0d %0d",nd,fd);
    $display("T|guard %0d %0d",ne,fe);
    $display("T|negedge %0d %0d",nf,ff);
    $display("T|interface %0d %0d",ng,fg);
    $display("T|initial %0d %0d",ni,fi);
    $finish;
  end
endmodule
"#;
