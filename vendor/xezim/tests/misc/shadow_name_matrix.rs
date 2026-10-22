//! Shadow-name sibling matrix for the round-69 dependency fix: a TB signal
//! and an instance port sharing one name must behave identically wherever
//! the name is read — cont-assign member fanout (pinned in svtb_suite),
//! always_comb, @*-self-ref (edge-routed), whole-struct copy, compiled
//! arithmetic, a shadowed CLOCK, hierarchical waiters on the scoped copy,
//! output-side shadowing, two-level nesting, parameterized copies, and
//! single-bit member writes from waiter continuations. Every expected value
//! reference-verified.

use std::process::Command;

const PKG: &str = "package sp;
  typedef struct packed { logic [9:0] f1; logic [9:0] f0; } duo_t;
endpackage
";

fn run(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join(format!("xezim_snm_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.sv"));
    std::fs::write(&path, format!("{PKG}{body}")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "test",
            path.to_str().unwrap(),
            "--no-cache",
        ])
        .output()
        .expect("run xezim");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text
}

#[test]
fn shadowed_clock_flop_samples_correctly() {
    let text = run(
        "clkshadow",
        r#"module flopper import sp::*; (clk, sync_in, q);
  input  logic clk;
  input  duo_t sync_in;
  output duo_t q;
  always_ff @(posedge clk) q <= sync_in;
endmodule
module test;
  import sp::*;
  logic clk = 0; always #5 clk = ~clk;
  duo_t sync_in; duo_t q;
  flopper u_f (.clk(clk), .sync_in(sync_in), .q(q));
  initial begin
    sync_in = '0;
    @(negedge clk) sync_in.f0 = 10'h0AA;
    @(posedge clk); #1 $display("T|t=%0t q=%h", $time, q);
    @(negedge clk) sync_in.f0 = 10'h0BB;
    @(posedge clk); #1 $display("T|t=%0t q=%h", $time, q);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|t=16 q=000aa"), "{text}");
    assert!(text.contains("T|t=26 q=000bb"), "{text}");
}

#[test]
fn hierarchical_waiter_on_scoped_shadow_wakes() {
    let text = run(
        "hierwait",
        r#"module fanout import sp::*; (sync_in, fan);
  input  duo_t sync_in;
  output logic [1:0][9:0] fan;
  assign fan[0] = sync_in.f0;
  assign fan[1] = sync_in.f1;
endmodule
module test;
  import sp::*;
  duo_t sync_in;
  wire [1:0][9:0] fan;
  logic clk = 0; always #2.5 clk = ~clk;
  integer wakes = 0;
  fanout u_f (.sync_in(sync_in), .fan(fan));
  initial begin : watcher
    forever begin @(u_f.sync_in) wakes = wakes + 1; end
  end
  initial begin
    sync_in = '0;
    #10; @(posedge clk);
    sync_in.f0 = 10'h1A5;
    @(posedge clk);
    sync_in.f1 = 10'h25B;
    #10 $display("T|wakes=%0d fan0=%h fan1=%h", wakes, fan[0], fan[1]); $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|wakes=3 fan0=1a5 fan1=25b"), "{text}");
}

#[test]
fn output_side_shadow_member_reads() {
    let text = run(
        "outshadow",
        r#"module producer import sp::*; (clk, sel, sync_out);
  input logic clk; input logic sel;
  output duo_t sync_out;
  always_ff @(posedge clk) begin
    sync_out.f0 <= sel ? 10'h111 : 10'h0F0;
    sync_out.f1 <= sel ? 10'h222 : 10'h0E0;
  end
endmodule
module test;
  import sp::*;
  duo_t sync_out;
  logic sel = 0;
  logic [9:0] m0;
  logic clk = 0; always #2.5 clk = ~clk;
  producer u_p (.clk(clk), .sel(sel), .sync_out(sync_out));
  assign m0 = sync_out.f0;
  initial begin
    #10; @(negedge clk) sel = 1;
    @(posedge clk); #1 $display("T|t=%0t m0=%h f1=%h", $time, m0, sync_out.f1);
    $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|t=16 m0=111 f1=222"), "{text}");
}

#[test]
fn two_level_shadow_chain_propagates() {
    let text = run(
        "twolevel",
        r#"module leaf import sp::*; (sync_in, fan);
  input  duo_t sync_in;
  output logic [1:0][9:0] fan;
  assign fan[0] = sync_in.f0;
  assign fan[1] = sync_in.f1;
endmodule
module mid import sp::*; (sync_in, fan);
  input  duo_t sync_in;
  output logic [1:0][9:0] fan;
  leaf u_leaf (.sync_in(sync_in), .fan(fan));
endmodule
module test;
  import sp::*;
  duo_t sync_in;
  wire [1:0][9:0] fan;
  logic clk = 0; always #2.5 clk = ~clk;
  mid u_mid (.sync_in(sync_in), .fan(fan));
  initial begin
    sync_in = '0;
    #10; @(posedge clk);
    sync_in.f0 = 10'h1A5; sync_in.f1 = 10'h25B;
    #10 $display("T|fan0=%h fan1=%h", fan[0], fan[1]); $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|fan0=1a5 fan1=25b"), "{text}");
}

#[test]
fn bit_select_member_write_propagates_through_shadow() {
    let text = run(
        "bitwrite",
        r#"module fanout import sp::*; (sync_in, fan);
  input  duo_t sync_in;
  output logic [1:0][9:0] fan;
  assign fan[0] = sync_in.f0;
  assign fan[1] = sync_in.f1;
endmodule
module test;
  import sp::*;
  duo_t sync_in;
  wire [1:0][9:0] fan;
  logic clk = 0; always #2.5 clk = ~clk;
  fanout u_f (.sync_in(sync_in), .fan(fan));
  initial begin
    sync_in = '0;
    #10; @(posedge clk);
    sync_in.f0[3] = 1'b1;
    @(posedge clk);
    sync_in.f1[9] = 1'b1;
    #10 $display("T|fan0=%h fan1=%h", fan[0], fan[1]); $finish;
  end
endmodule
"#,
    );
    assert!(text.contains("T|fan0=008 fan1=200"), "{text}");
}
