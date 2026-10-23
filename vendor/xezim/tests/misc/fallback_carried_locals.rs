//! A compiled block keeps block-local declarations, `for (int i ...)`
//! counters, unrolled loop constants and an inlined task's formals and locals
//! in VM registers. A construct the compiler hands to the interpreter
//! (`$display`, `$time`, an associative-array `exists`/store, an
//! intra-assignment delay) used to force the whole block — or the whole task
//! call — onto the interpreter, because the interpreter cannot see a
//! register. Now the fallback carries the locals it names in a frame and
//! copies them back.
//!
//! These cases pin the edges of that mechanism against reference-simulator
//! output: output formals named like their actuals, locals named like module
//! variables a callee writes, declared signedness and `real` conversion of
//! register locals, loop variables read (and written) by a fallback,
//! `return`/`disable` inside an interpreted statement, `%m` and `$time` in an
//! inlined body, static tasks, and comb blocks.

use xezim::simulate;

fn lines(src: &str, max_time: u64) -> Vec<String> {
    let sim = simulate(src, max_time).expect("design must run");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn expect(src: &str, max_time: u64, want: &[&str]) {
    let got = lines(src, max_time);
    let want: Vec<String> = want.iter().map(|s| s.to_string()).collect();
    assert_eq!(got, want, "\n got: {got:#?}\nwant: {want:#?}");
}

#[test]
fn inlined_tasks_with_interpreted_statements_keep_their_formals_and_locals() {
    // An inlined task whose `$display` or associative-array access runs on the
    // interpreter; an output formal named like the caller's actual (`o`); a
    // caller local named like the module variable the callee increments
    // (`cnt`).
    let src = r#"
module sub (input logic clk, input int cyc);
  int cnt;
  logic [63:0] mem [logic [9:0]];
  function automatic int cnt_mod(); return cnt; endfunction
  task automatic t1(input int ia, output int o);
    int loc;
    loc = ia * 3;
    $display("T| t1 ia=%0d loc=%0d", ia, loc);
    o = loc + 1;
  endtask
  task automatic bump();
    cnt = cnt + 1;
  endtask
  task automatic mw(input logic [9:0] key, input logic [63:0] data, input logic [7:0] dm);
    logic [63:0] cur;
    if (mem.exists(key)) cur = mem[key];
    else cur = {64{1'bx}};
    for (int bt = 0; bt < 8; bt++)
      if (dm[bt] === 1'b1)
        cur[bt*8 +: 8] = data[bt*8 +: 8];
    mem[key] = cur;
  endtask
  task automatic mr(input logic [9:0] key, output logic [63:0] data);
    if (mem.exists(key)) data = mem[key];
    else data = {64{1'bx}};
  endtask
  always @(posedge clk) begin
    int o, cnt;
    logic [63:0] d;
    cnt = 100;
    t1(cyc, o);
    mw(10'(cyc % 2), {8{8'(cyc)}}, 8'hA5);
    mr(10'(cyc % 2), d);
    bump();
    $display("T| o=%0d d=%h cnt=%0d mod=%0d", o, d, cnt, cnt_mod());
    mr(10'd900, d);
    $display("T| miss=%h", d);
  end
endmodule
module tb;
  logic clk = 0;
  int cyc = 0;
  sub u (.clk(clk), .cyc(cyc));
  initial begin
    repeat (3) begin #5 clk = 1; cyc++; #5 clk = 0; end
    $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &[
            "T| t1 ia=1 loc=3",
            "T| o=4 d=01xx01xxxx01xx01 cnt=100 mod=1",
            "T| miss=xxxxxxxxxxxxxxxx",
            "T| t1 ia=2 loc=6",
            "T| o=7 d=02xx02xxxx02xx02 cnt=100 mod=2",
            "T| miss=xxxxxxxxxxxxxxxx",
            "T| t1 ia=3 loc=9",
            "T| o=10 d=03xx03xxxx03xx03 cnt=100 mod=3",
            "T| miss=xxxxxxxxxxxxxxxx",
        ],
    );
}

#[test]
fn register_locals_keep_declared_signedness_and_real_type() {
    // `int s = u` with an unsigned `u` is negative; `real` locals and formals
    // convert integral values; both are read by an interpreted `$display`.
    let src = r#"
module sub (input logic clk);
  logic [31:0] u = 32'hFFFF_FFFB;
  logic [7:0] ub = 8'hF0;
  int r1, r2, r3, r4;
  real acc = 0.0;
  task automatic scale(input string what, input real dt, input real lim);
    acc = acc + dt / 2 + lim;
    $display("T| %s dt=%0.2f lim=%0.2f acc=%0.2f", what, dt, lim, acc);
  endtask
  always @(posedge clk) begin
    int s;
    byte sb;
    logic [7:0] lu;
    real rr, rh;
    s = u;
    sb = ub;
    lu = sb;
    r1 <= (s < 0);
    r2 <= (sb < 0);
    r3 <= (lu > 8'd3) ? 1 : 0;
    r4 <= s / 2;
    rh = 3;
    rr = rh / 2;
    $display("T| s=%0d sb=%0d lu=%0d neg=%0d rr=%0.3f rh=%0.3f", s, sb, lu, s < 0, rr, rh);
    scale("a", 5, 2);
    scale("b", rr, s);
  end
  always @(negedge clk) if ($time > 0) $display("T| r1=%0d r2=%0d r3=%0d r4=%0d", r1, r2, r3, r4);
endmodule
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  initial begin
    repeat (2) begin #5 clk = 1; #5 clk = 0; end
    $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &[
            "T| s=-5 sb=-16 lu=240 neg=1 rr=1.500 rh=3.000",
            "T| a dt=5.00 lim=2.00 acc=4.50",
            "T| b dt=1.50 lim=-5.00 acc=0.25",
            "T| r1=1 r2=1 r3=1 r4=-2",
            "T| s=-5 sb=-16 lu=240 neg=1 rr=1.500 rh=3.000",
            "T| a dt=5.00 lim=2.00 acc=4.75",
            "T| b dt=1.50 lim=-5.00 acc=0.50",
        ],
    );
}

#[test]
fn loop_variables_are_carried_into_interpreted_statements() {
    // A register loop counter written inside an interpreted statement, an
    // unrolled constant and a `foreach` index read by one.
    let src = r#"
module sub (input logic clk);
  int aa [int];
  logic [7:0] arr [0:3];
  int hits;
  initial begin
    aa[1] = 10; aa[3] = 30; aa[5] = 50;
    for (int k = 0; k < 4; k++) arr[k] = 8'(k * 7);
  end
  always @(posedge clk) begin
    int n, acc;
    n = 0; acc = 0;
    // loop variable written inside an interpreted statement
    for (int j = 0; j < 6; j++) begin
      if (aa.exists(j)) begin aa[j] = aa[j] + j; j++; end
      n++;
    end
    // unrolled loop constant read by an interpreted statement
    for (int k = 0; k < 3; k++)
      if (aa.exists(k * 2 + 1)) acc += aa[k * 2 + 1];
    foreach (arr[i]) if (arr[i] > 8'd6) $display("T| arr[%0d]=%0d", i, arr[i]);
    // a register loop counter carried into an interpreted statement
    for (int i = 0; i < 3; i++) $display("T| i=%0d", i);
    $display("T| n=%0d acc=%0d a5=%0d", n, acc, aa[5]);
  end
endmodule
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  initial begin
    repeat (2) begin #5 clk = 1; #5 clk = 0; end
    $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &[
            "T| arr[1]=7",
            "T| arr[2]=14",
            "T| arr[3]=21",
            "T| i=0",
            "T| i=1",
            "T| i=2",
            "T| n=4 acc=99 a5=55",
            "T| arr[1]=7",
            "T| arr[2]=14",
            "T| arr[3]=21",
            "T| i=0",
            "T| i=1",
            "T| i=2",
            "T| n=4 acc=108 a5=60",
        ],
    );
}

#[test]
fn control_transfers_inside_interpreted_statements_stay_correct() {
    // `return` and an outer `break` inside a statement that must run on the
    // interpreter keep the enclosing unit interpreted; a `disable` of a block
    // inside the statement is carried; recursion and a `ref` actual stay calls.
    let src = r#"
module sub (input logic clk);
  int aa [int];
  logic [7:0] cv = 0;
  int cres;
  initial begin aa[1] = 10; aa[3] = 30; aa[5] = 50; end
  // `return` inside a statement that runs on the interpreter
  task automatic find(input int want, output int pos);
    int n;
    n = 0;
    pos = -1;
    foreach (aa[k]) begin
      n++;
      if (aa[k] == want) begin pos = k; return; end
    end
    pos = -100 - n;
  endtask
  // a `disable` that stays inside the interpreted statement
  task automatic sum_to(input int lim, output int s);
    int t;
    t = 0;
    begin : blk
      foreach (aa[k]) begin
        if (aa[k] > lim) disable blk;
        t += aa[k];
      end
    end
    s = t;
  endtask
  // recursion stays a call; a `ref` actual is a caller local
  task automatic fact(input int n, output int f);
    int sub;
    if (n <= 1) f = 1;
    else begin
      fact(n - 1, sub);
      f = n * sub;
    end
  endtask
  task automatic incr(ref int v);
    v = v + 5;
  endtask
  always @(posedge clk) begin
    int p, q, f, v, s;
    find(30, p);
    find(99, q);
    sum_to(35, s);
    fact(5, f);
    v = 10;
    incr(v);
    $display("T| p=%0d q=%0d s=%0d f=%0d v=%0d", p, q, s, f, v);
  end
  // comb block: the whole `if` runs on the interpreter and holds a `break`
  always_comb begin
    int i, s2;
    s2 = 0;
    for (i = 0; i < 8; i++) begin
      if (aa.exists(i)) break;
      s2 += 1;
    end
    cres = s2 * 100 + i + cv;
  end
  always @(posedge clk) begin
    cv <= cv + 1;
    if (cv > 0) $display("T| cres=%0d", cres);
  end
endmodule
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  initial begin
    repeat (2) begin #5 clk = 1; #5 clk = 0; end
    $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &[
            "T| p=3 q=-103 s=40 f=120 v=15",
            "T| p=3 q=-103 s=40 f=120 v=15",
            "T| cres=102",
        ],
    );
}

#[test]
fn scope_names_and_time_in_inlined_bodies() {
    // `%m` inside an inlined task names the task; a task declared under another
    // timescale is not inlined, so its `$time` stays in its own unit.
    let src = r#"
`timescale 1ns/1ps
module sub (input logic clk);
  task automatic whereami(input int v);
    $display("T| whereami v=%0d at %m", v);
  endtask
  task automatic tick(input int v);
    $display("T| tick v=%0d t=%0t rt=%0.3f", v, $time, $realtime);
  endtask
  always @(posedge clk) begin
    int v;
    v = 5;
    whereami(v);
    begin : inner
      tick(v + 2);
    end
  end
endmodule
`timescale 1us/1ns
module slow;
  task automatic stamp(input int v);
    $display("T| slow stamp v=%0d t=%0t", v, $time);
  endtask
endmodule
`timescale 1ns/1ps
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  slow us ();
  always @(negedge clk) if ($time > 0) begin
    int q;
    q = 40;
    us.stamp(q);
  end
  initial begin
    repeat (2) begin #5 clk = 1; #5 clk = 0; end
    #1 $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &[
            "T| whereami v=5 at tb.u.whereami",
            "T| tick v=7 t=5000 rt=5.000",
            "T| slow stamp v=40 t=0",
            "T| whereami v=5 at tb.u.whereami",
            "T| tick v=7 t=15000 rt=15.000",
            "T| slow stamp v=40 t=0",
        ],
    );
}

#[test]
fn select_stores_into_register_locals() {
    // Bit and part-select stores into register locals — constant, unrolled,
    // dynamic and out of range — including a function (`mr_value`) that now
    // inlines because of them.
    let src = r#"
module sub (input logic clk);
  function automatic logic [13:0] mr_value(input int n);
    logic [13:0] v;
    logic [3:0] c;
    v = '0;
    c = 4'hB;
    case (n)
      0: begin v[6:4] = c[3:1]; v[2] = c[0]; v[11:9] = 3'd5; end
      1: v[0] = 1'b1;
      2: v[5 +: 3] = 3'b110;
      default: v[13 -: 2] = 2'b10;
    endcase
    return v;
  endfunction
  logic [13:0] ddr_a;
  logic [63:0] q;
  int nn = 0;
  always @(posedge clk) begin
    int mrn;
    logic [63:0] cur;
    logic [3:0] idx;
    mrn = nn;
    ddr_a <= mr_value(mrn);
    cur = {64{1'bx}};
    for (int bt = 0; bt < 8; bt++)
      if (bt[0]) cur[bt*8 +: 8] = 8'(bt * 17);
    idx = 4'(nn + 3);
    cur[idx] = 1'b0;
    cur[idx + 4'd8 +: 4] = 4'h5;
    cur[70 +: 4] = 4'hF;
    q <= cur;
    nn <= nn + 1;
  end
  always @(negedge clk) if ($time > 0) $display("T| ddr_a=%b q=%h", ddr_a, q);
endmodule
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  initial begin
    repeat (4) begin #5 clk = 1; #5 clk = 0; end
    $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &[
            "T| ddr_a=00101001010100 q=77xx55xx33xx29xX",
            "T| ddr_a=00000000000001 q=77xx55xx33xx51Xx",
            "T| ddr_a=00000011000000 q=77xx55xx33xXb1Xx",
        ],
    );
}

#[test]
fn deferred_and_call_arguments_of_interpreted_statements() {
    // `$strobe` reads its arguments at the end of the time step, when a
    // carried frame is gone: a block whose `$strobe` names a local stays on
    // the interpreter. An unrolled loop constant passed to an input formal
    // of a call that stays interpreted (recursive) is carried read-only.
    let src = r#"
module sub (input logic clk);
  int aa [int];
  int log_n;
  initial begin aa[0] = 5; aa[2] = 7; end
  // not inlinable (recursive), called with an unrolled loop constant
  task automatic note(input int k, input int depth);
    int d;
    d = depth;
    if (d > 0) note(k, d - 1);
    else log_n = log_n + k;
  endtask
  always @(posedge clk) begin
    int x;
    x = 3;
    $strobe("T| strobe x=%0d", x);
    x = 4;
    for (int k = 0; k < 3; k++)
      if (aa.exists(k)) note(k, 1);
    $display("T| log_n=%0d", log_n);
  end
endmodule
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  initial begin
    repeat (2) begin #5 clk = 1; #5 clk = 0; end
    $finish;
  end
endmodule
"#;
    expect(
        src,
        1_000_000,
        &["T| log_n=2", "T| strobe x=4", "T| log_n=4", "T| strobe x=4"],
    );
}

#[test]
fn real_locals_written_inside_interpreted_statements_stay_real() {
    // `r = k` with an integral `k` inside a statement the interpreter runs:
    // the carried `real` local converts, so `r / 2` is a real division —
    // and so does a task's own `real` local on the interpreter path.
    let src = r#"
module sub (input logic clk);
  int aa [int];
  initial begin aa[3] = 30; aa[5] = 50; end
  task automatic half(input int v, output real h);
    real t;
    foreach (aa[k]) if (k == v) begin t = k; h = t / 2; end
  endtask
  always @(posedge clk) begin
    real r, q;
    int n;
    r = 1.5;
    n = 0;
    foreach (aa[k]) begin r = k; q = r / 2; n = n + 1; end
    half(5, q);
    $display("T| r=%0.2f q=%0.2f n=%0d", r, q, n);
  end
endmodule
module tb;
  logic clk = 0;
  sub u (.clk(clk));
  initial begin
    #5 clk = 1; #5 clk = 0;
    #1 $finish;
  end
endmodule
"#;
    expect(src, 1_000_000, &["T| r=5.00 q=2.50 n=2"]);
}

#[test]
fn interpreted_task_calls_convert_integral_actuals_to_real_formals() {
    // §13.3.1 on the interpreter's task path, which the compiled inline
    // already honours: `addhalf(5, 1)` binds `dt` to 5.0.
    let src = r#"
module tb;
  real acc = 0.0;
  task automatic addhalf(input real dt, input int k);
    acc = acc + dt / 2;
    $display("T| %m dt=%0.2f half=%0.2f k=%0d acc=%0.2f", dt, dt / 2, k, acc);
  endtask
  initial begin
    addhalf(5, 1);
    addhalf(-3, 2);
    addhalf(2.5, 3);
  end
endmodule
"#;
    expect(
        src,
        100,
        &[
            "T| tb.addhalf dt=5.00 half=2.50 k=1 acc=2.50",
            "T| tb.addhalf dt=-3.00 half=-1.50 k=2 acc=1.00",
            "T| tb.addhalf dt=2.50 half=1.25 k=3 acc=2.25",
        ],
    );
}
