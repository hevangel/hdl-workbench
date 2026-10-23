//! §7.4.6 / §11.5.1: reading an unpacked-array element with an out-of-range
//! or x index yields x at the ELEMENT width. Three paths produced a 1-bit x
//! that the store then zero-extended (`0000000X`): the fused memory-read
//! flop (`q <= mem[i]`), a blocking read in a process (`a = mem[i]`), and
//! the same read through a task's output. Wide (>64-bit) elements take the
//! two-state executor's element loads and must agree bit for bit.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn out_of_range_element_reads_are_all_x_at_element_width() {
    let msgs = messages(
        "module tb;
  logic [31:0] nm [0:3]; logic [31:0] a, b, c, nq, nq2; logic [3:0] i; logic clk = 0;
  task automatic rd(output logic [31:0] o); o = nm[i]; endtask
  always_comb nq = nm[i];
  always @(posedge clk) nq2 <= nm[i];
  initial begin
    for (int k = 0; k < 4; k++) nm[k] = 32'h5;
    i = 9; #1 clk = 1; #1;
    a = nm[i];
    rd(b);
    c = nm[i] + 32'h0;
    $display(\"OOB %h %h %h %h %h\", a, b, c, nq, nq2);
    i = 2; #1 clk = 0; #1 clk = 1; #1;
    a = nm[i];
    rd(b);
    $display(\"IN %h %h %h %h\", a, b, nq, nq2);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter()
            .any(|m| m == "OOB xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx"),
        "{msgs:?}"
    );
    assert!(
        msgs.iter()
            .any(|m| m == "IN 00000005 00000005 00000005 00000005"),
        "{msgs:?}"
    );
}

/// 256-bit elements: an in-range read, a read that walks out of range, and
/// elements holding x, all in one clocked block that runs two-state until
/// an element read aborts it.
#[test]
fn wide_element_reads_match_four_state() {
    let msgs = messages(
        "module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [255:0] mem [0:7];
  logic [255:0] q, q2, acc;
  logic [2:0] ra; logic [3:0] rb; int cyc = 0;
  always @(posedge clk) begin
    q <= mem[ra];
    q2 <= mem[rb];
    acc <= acc ^ {mem[ra][31:0], mem[ra][255:224]};
    ra <= ra + 1; rb <= rb + 1;
    cyc <= cyc + 1;
  end
  initial begin
    for (int i = 0; i < 6; i++) mem[i] = {8{32'h1000_0000 * i + 32'hcafe}} ^ (256'h1 << i);
    mem[6] = 'x; mem[7][3:0] = 4'bxxxx;
    ra = 0; rb = 0; acc = 0;
    repeat (12) @(posedge clk);
    #1 $display(\"W %h %h %h %h %h %0d\", q[63:0], q[255:224], q2[63:0], acc[63:0], acc[255:192], cyc);
    repeat (16) @(posedge clk);
    #1 $display(\"W2 %h %h %h %h %h %0d\", q[63:0], q[255:224], q2[63:0], acc[63:0], acc[255:192], cyc);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter().any(|m| m
            == "W 3000cafe3000caf6 3000cafe xxxxxxxxxxxxxxxx xxxxxxxxxxxxxxxx 0000000000000000 12"),
        "{msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m == "W2 3000cafe3000caf6 3000cafe xxxxxxxxxxxxxxxx xxxxxxxxxxxxxxxx 0000000000000000 28"),
        "{msgs:?}"
    );
}
