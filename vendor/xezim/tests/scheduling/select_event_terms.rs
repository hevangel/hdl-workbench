//! §9.4.2 event terms that select bits of a vector.
//!
//! `always @(v[3])` wakes only when that bit changes, `@(v[7:4])` only when
//! one of those bits does, and an explicit event list never becomes
//! "whatever the body reads". Port substitution hands the simulator selects
//! OF selects (`@(dq[1])` on `inout [3:0] dq` wired to `bus[7:4]` arrives as
//! `bus[7:4][1]`); those resolve to the net bit they read and compile as
//! edge blocks, like a local `@(loc[1])`.
//!
//! Expected values are the reference simulator's.
use std::path::PathBuf;
use std::process::Command;

fn run(name: &str, src: &str, env: &[(&str, &str)]) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("select_event_terms");
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join(format!("{name}.sv"));
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd
        .args(["--simulate", "-s", "tb", "--no-cache", sv.to_str().unwrap()])
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "run failed:\n{text}");
    text
}

fn t_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|l| l.find("T|").map(|i| &l[i..]))
        .collect()
}

/// §9.4.2 Table 9-2: all sixteen four-state transitions agree across
/// direct signals, bit aliases and dynamic-select value waiters.
#[test]
fn four_state_event_transition_table() {
    let o = run(
        "edge_transition_table",
        r#"
`timescale 1ns/1ns
module observer(inout [3:0] link);
  int changed = 0, rising = 0, falling = 0, edged = 0;
  always @(link[2]) changed++;
  always @(posedge link[2]) rising++;
  always @(negedge link[2]) falling++;
  always @(edge link[2]) edged++;
endmodule
module tb;
  logic level;
  logic [15:0] drive;
  wire [15:0] bus;
  int pick = 10;
  int changed = 0, rising = 0, falling = 0, edged = 0;
  int waited = 0, wait_rising = 0, wait_falling = 0, wait_edged = 0;
  assign bus = drive;
  observer u(.link(bus[11:8]));
  always @(level) changed++;
  always @(posedge level) rising++;
  always @(negedge level) falling++;
  always @(edge level) edged++;
  always @(bus[pick]) waited++;
  always @(posedge bus[pick]) wait_rising++;
  always @(negedge bus[pick]) wait_falling++;
  always @(edge bus[pick]) wait_edged++;
  function automatic logic state_value(input int code);
    case (code)
      0: return 1'b0;
      1: return 1'b1;
      2: return 1'bx;
      default: return 1'bz;
    endcase
  endfunction
  initial begin
    drive = 0;
    for (int from_code = 0; from_code < 4; from_code++) begin
      for (int to_code = 0; to_code < 4; to_code++) begin
        #2 level = state_value(from_code); drive[10] = state_value(from_code);
        #2 changed = 0; rising = 0; falling = 0; edged = 0;
        u.changed = 0; u.rising = 0; u.falling = 0; u.edged = 0;
        waited = 0; wait_rising = 0; wait_falling = 0; wait_edged = 0;
        #2 level = state_value(to_code); drive[10] = state_value(to_code);
        #2 $display("T| %0d>%0d direct=%0d%0d%0d%0d alias=%0d%0d%0d%0d waiter=%0d%0d%0d%0d",
          from_code, to_code, changed, rising, falling, edged,
          u.changed, u.rising, u.falling, u.edged,
          waited, wait_rising, wait_falling, wait_edged);
      end
    end
    $finish;
  end
endmodule
"#,
        &[],
    );
    let mut expected = Vec::new();
    for from in 0..4 {
        for to in 0..4 {
            let changed = usize::from(from != to);
            let rising = usize::from((from == 0 && to != 0) || (from != 1 && to == 1));
            let falling = usize::from((from == 1 && to != 1) || (from != 0 && to == 0));
            let edged = usize::from(rising != 0 || falling != 0);
            let counts = format!("{changed}{rising}{falling}{edged}");
            expected.push(format!(
                "T| {from}>{to} direct={counts} alias={counts} waiter={counts}"
            ));
        }
    }
    assert_eq!(t_lines(&o), expected, "{o}");
}

/// §9.4.2: high bits and ascending aliases preserve the transition table;
/// a part-select spanning two storage words observes changes in both words.
#[test]
fn wide_ascending_alias_transition_table() {
    let o = run(
        "wide_alias_table",
        r#"
`timescale 1ns/1ns
module observer(inout [7:0] link);
  int changed = 0, rising = 0, falling = 0, edged = 0;
  always @(link[2]) changed++;
  always @(posedge link[2]) rising++;
  always @(negedge link[2]) falling++;
  always @(edge link[2]) edged++;
endmodule
module tb;
  logic [127:0] drive;
  logic [64:191] ascending;
  wire [127:0] bus;
  wire [64:191] ascending_bus;
  int part_changes = 0;
  assign bus = drive;
  assign ascending_bus = ascending;
  observer high_bit(.link(bus[79:72]));
  observer ascending_bit(.link(ascending_bus[136:143]));
  observer boundary_bit(.link(bus[69:62]));
  always @(bus[65:62]) part_changes++;
  function automatic logic state_value(input int code);
    case (code)
      0: return 1'b0;
      1: return 1'b1;
      2: return 1'bx;
      default: return 1'bz;
    endcase
  endfunction
  initial begin
    drive = 0; ascending = 0;
    for (int from_code = 0; from_code < 4; from_code++) begin
      for (int to_code = 0; to_code < 4; to_code++) begin
        #2 drive[74] = state_value(from_code); drive[64] = state_value(from_code);
        ascending[141] = state_value(from_code);
        #2 high_bit.changed = 0; high_bit.rising = 0; high_bit.falling = 0; high_bit.edged = 0;
        ascending_bit.changed = 0; ascending_bit.rising = 0; ascending_bit.falling = 0; ascending_bit.edged = 0;
        boundary_bit.changed = 0; boundary_bit.rising = 0; boundary_bit.falling = 0; boundary_bit.edged = 0;
        part_changes = 0;
        #2 drive[74] = state_value(to_code); drive[64] = state_value(to_code);
        ascending[141] = state_value(to_code);
        #2 $display("T| %0d>%0d high=%0d%0d%0d%0d ascending=%0d%0d%0d%0d boundary=%0d%0d%0d%0d part=%0d",
          from_code, to_code, high_bit.changed, high_bit.rising, high_bit.falling, high_bit.edged,
          ascending_bit.changed, ascending_bit.rising, ascending_bit.falling, ascending_bit.edged,
          boundary_bit.changed, boundary_bit.rising, boundary_bit.falling, boundary_bit.edged, part_changes);
      end
    end
    #2 drive = 0;
    #2 part_changes = 0;
    #2 drive[63] = 1'b1;
    #2 drive[63] = 1'b0;
    #2 drive[64] = 1'b1;
    #2 drive[64] = 1'b0;
    #2 $display("T| cross_word changes=%0d", part_changes);
    $finish;
  end
endmodule
"#,
        &[],
    );
    let mut expected = Vec::new();
    for from in 0..4 {
        for to in 0..4 {
            let changed = usize::from(from != to);
            let rising = usize::from((from == 0 && to != 0) || (from != 1 && to == 1));
            let falling = usize::from((from == 1 && to != 1) || (from != 0 && to == 0));
            let edged = usize::from(rising != 0 || falling != 0);
            let counts = format!("{changed}{rising}{falling}{edged}");
            expected.push(format!(
                "T| {from}>{to} high={counts} ascending={counts} boundary={counts} part={changed}"
            ));
        }
    }
    expected.push("T| cross_word changes=4".to_string());
    assert_eq!(t_lines(&o), expected, "{o}");
}

/// §9.4.2: aliased bit events distinguish all four values, ignore sibling
/// changes, and retain both the known-to-unknown and unknown-to-known edges.
#[test]
fn aliased_bit_events_preserve_four_state_transitions() {
    let o = run(
        "port_four_state",
        r#"
`timescale 1ns/1ns
module observer(inout [3:0] link);
  int changes = 0, rises = 0, falls = 0;
  always @(link[2]) changes++;
  always @(posedge link[2]) rises++;
  always @(negedge link[2]) falls++;
endmodule
module tb;
  logic [15:0] drive;
  wire [15:0] bus;
  assign bus = drive;
  observer u(.link(bus[11:8]));
  initial begin
    #1 drive = 0;
    #1 u.changes = 0; u.rises = 0; u.falls = 0;
    #1 drive[9] = 1'bx;
    #1 drive[9] = 1'bz;
    #1 drive[15] = 1'b1;
    #1 $display("T| siblings changes=%0d rises=%0d falls=%0d", u.changes, u.rises, u.falls);
    drive[10] = 1'bx;
    #1 drive[10] = 1'bx;
    #1 drive[9] = 1'b0;
    #1 drive[10] = 1'bz;
    #1 drive[10] = 1'bz;
    #1 drive[10] = 1'b1;
    #1 drive[10] = 1'bx;
    #1 drive[10] = 1'b0;
    #1 drive[10] = 1'bz;
    #1 drive[10] = 1'b0;
    #1 drive[10] = 1'b1;
    #1 drive[10] = 1'b0;
    #1 $display("T| selected changes=%0d rises=%0d falls=%0d", u.changes, u.rises, u.falls);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(
        t_lines(&o),
        [
            "T| siblings changes=0 rises=0 falls=0",
            "T| selected changes=9 rises=4 falls=4",
        ],
        "{o}"
    );
}

/// §9.4.2: high physical bits and ascending port actuals keep exact value
/// waits when they cannot be represented by the edge dispatch mask.
#[test]
fn aliased_bit_events_fall_back_for_wide_and_ascending_nets() {
    let o = run(
        "port_fallback",
        r#"
`timescale 1ns/1ns
module observer(inout [7:0] link);
  int changes = 0;
  always @(link[2]) changes++;
endmodule
module tb;
  logic [127:0] descending;
  logic [64:191] ascending;
  wire [127:0] down_bus;
  wire [64:191] up_bus;
  assign down_bus = descending;
  assign up_bus = ascending;
  observer down(.link(down_bus[79:72]));
  observer up(.link(up_bus[136:143]));
  initial begin
    #1 descending = 0; ascending = 0;
    #1 down.changes = 0; up.changes = 0;
    #1 descending[73] = 1'bx; ascending[137] = 1'bx;
    #1 $display("T| siblings down=%0d up=%0d", down.changes, up.changes);
    descending[74] = 1'bx; ascending[141] = 1'bx;
    #1 descending[78] = 1'b1; ascending[136] = 1'bz;
    #1 descending[74] = 1'bz; ascending[141] = 1'bz;
    #1 descending[74] = 1'b1; ascending[141] = 1'b1;
    #1 $display("T| selected down=%0d up=%0d", down.changes, up.changes);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(
        t_lines(&o),
        ["T| siblings down=0 up=0", "T| selected down=3 up=3"],
        "{o}"
    );
}

/// §9.4.2: a whole-vector edge term does not turn a neighboring bit's
/// any-change term into an any-change term for every bit of the vector.
#[test]
fn mixed_bit_and_whole_edge_terms_keep_the_bit_mask() {
    let o = run(
        "mixed_edge_masks",
        r#"
`timescale 1ns/1ns
module tb;
  logic [3:0] bus;
  int rising = 0, falling = 0, edged = 0, whole = 0;
  always @(bus[1] or posedge bus) rising++;
  always @(bus[1] or negedge bus) falling++;
  always @(bus[1] or edge bus) edged++;
  always @(bus[1] or bus) whole++;
  initial begin
    #1 bus = 0;
    #1 rising = 0; falling = 0; edged = 0; whole = 0;
    #1 bus[3] = 1'b1;
    #1 $display("T| siblings rising=%0d falling=%0d edged=%0d whole=%0d", rising, falling, edged, whole);
    bus[0] = 1'b1;
    #1 bus[3] = 1'b0;
    #1 bus[0] = 1'bx;
    #1 bus[0] = 1'bz;
    #1 bus[1] = 1'b1;
    #1 bus[0] = 1'b0;
    #1 $display("T| selected rising=%0d falling=%0d edged=%0d whole=%0d", rising, falling, edged, whole);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(
        t_lines(&o),
        [
            "T| siblings rising=0 falling=0 edged=0 whole=1",
            "T| selected rising=2 falling=3 edged=4 whole=7",
        ],
        "{o}"
    );
}

/// Bit and part selects of local vectors: ascending and non-zero-based
/// ranges read through their declared labels, two bits of one vector, a
/// bit next to the whole vector, a parameter index, `+:`, an element of a
/// packed 2-D vector, a bit past 64, and an edge on a part-select (its LSB).
#[test]
fn local_select_terms_wake_on_their_bits() {
    let o = run(
        "local",
        r#"
`timescale 1ns/1ns
module tb;
  localparam int K = 2;
  logic [0:3] a;
  logic [8:1] b;
  logic [7:0] c;
  logic [3:0][1:0] pk;
  logic [99:0] w;
  int na = 0, nb = 0, nc = 0, nd = 0, ne = 0, nf = 0, ng = 0, nh = 0, ni = 0, nj = 0, nk = 0, nl = 0, nm = 0;
  always @(a[1]) na++;
  always @(b[1]) nb++;
  always @(c[1] or c[2]) nc++;
  always @(posedge c[1] or negedge c[2]) nd++;
  always @(c[1] or c) ne++;
  always @(c[6:5]) nf++;
  always @(c[K +: 2]) ng++;
  always @(c[K]) nh++;
  always @(pk[1]) ni++;
  always @(w[70]) nj++;
  always @(posedge c[7:6]) nk++;
  always @(a[1:2]) nl++;
  always @(c[1] or posedge c) nm++;
  initial begin
    #1 a = 0; b = 0; c = 0; pk = 0; w = 0;
    for (int i = 0; i < 16; i++) begin #1 a = i; b = i * 3; c = i * 5; pk = i * 7; w = {i[3:0], 70'b0} | i; end
    #1 $display("T| na=%0d nb=%0d nc=%0d nd=%0d ne=%0d nf=%0d ng=%0d nh=%0d ni=%0d nj=%0d nk=%0d nl=%0d nm=%0d", na, nb, nc, nd, ne, nf, ng, nh, ni, nj, nk, nl, nm);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(
        t_lines(&o),
        ["T| na=4 nb=16 nc=16 nd=9 ne=16 nf=3 ng=16 nh=13 ni=16 nj=16 nk=1 nl=8 nm=16"],
        "{o}"
    );
}

/// An explicit list with a select never takes the body's read set as its
/// sensitivity: the blocks call tasks and read `te` and `s`, and wake only
/// on `a[1]` / `a[3:2]` (the read-set routing woke them on every `s` and
/// `te` change and ran `viol` 8 times).
#[test]
fn select_list_ignores_body_reads() {
    let o = run(
        "b2",
        r#"
`timescale 1ps/1ps
module tb;
  logic [3:0] a = 0;
  logic       s = 0;
  real        te = -100.0;
  int         nv = 0, nw = 0, nx = 0;
  task automatic viol(); nv++; endtask
  task automatic hit_w(); nw++; endtask
  task automatic hit_x(); nx++; endtask
  always @(s) te = $realtime;
  // Explicit lists: each block wakes on its own bits only, never on what
  // its body reads (te, s) or on the other bits of `a`.
  always @(a[1]) begin
    hit_w();
    if ($realtime - te < 1.0) viol();
  end
  always @(a[3:2]) begin
    hit_x();
    if (s) viol();
  end
  initial begin
    #5;
    repeat (4) #10 s = ~s;
    repeat (3) #10 a[0] = ~a[0];
    #10 a = 4'b0110;
    #10 s = ~s;
    #10 a = 4'b1000;
    #10 $display("T| viol=%0d nw=%0d nx=%0d", nv, nw, nx);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(t_lines(&o), ["T| viol=1 nw=2 nx=2"], "{o}");
}

/// Bit events on ports bound to selects of a parent net: inout part-selects
/// (`[7:4]`, `+:` from a generate, `-:`), a concatenation, a non-zero-based
/// net, two levels of ports, an output and an input; level, posedge and
/// negedge on one bit, and the whole port.
#[test]
fn port_bit_events_follow_the_connection() {
    let o = run(
        "ports",
        r#"
`timescale 1ns/1ns
module dev #(parameter string TAG="x") (inout [3:0] dq);
  int n1 = 0, np = 0, nn = 0, nw = 0;
  logic v1;
  always @(dq[1]) begin n1++; v1 = dq[1]; end
  always @(posedge dq[2]) np++;
  always @(negedge dq[3]) nn++;
  always @(dq) nw++;
  final $display("T| %s n1=%0d v1=%b np=%0d nn=%0d nw=%0d dq=%b", TAG, n1, v1, np, nn, nw, dq);
endmodule
module devo (output [3:0] oq, input [3:0] src);
  int n1 = 0;
  assign oq = src;
  always @(oq[1]) n1++;
  final $display("T| out n1=%0d", n1);
endmodule
module devi (input [3:0] iq);
  int n1 = 0, np = 0;
  always @(iq[1]) n1++;
  always @(posedge iq[3]) np++;
  final $display("T| in n1=%0d np=%0d", n1, np);
endmodule
module mid (inout [7:0] m);
  dev #("multi") u (.dq(m[7:4]));
endmodule
module tb;
  logic [15:0] v;
  wire [15:0] bus;  assign bus = v;
  wire [8:1] nz;    assign nz = v[15:8];
  wire [1:0] a2, b2; assign a2 = v[1:0]; assign b2 = v[3:2];
  wire [15:0] bus2; assign bus2 = v;
  wire [3:0] oo;
  dev  #("slice74")  u0 (.dq(bus[7:4]));
  for (genvar d = 0; d < 2; d++) begin : g
    dev #("plus") u (.dq(bus[d*4+8 +: 4]));
  end
  dev  #("minus")    um0 (.dq(bus[11 -: 4]));
  dev  #("concat")   u1 (.dq({a2, b2}));
  dev  #("nonzero")  u4 (.dq(nz[6:3]));
  dev  #("whole")    u5 (.dq(bus[3:0]));
  mid  um (.m(bus2[15:8]));
  devo uo (.oq(oo), .src(v[7:4]));
  devi ui (.iq(bus[11:8]));
  initial begin
    #1 v = 0;
    for (int i = 0; i < 16; i++) begin #1 v = 16'h1 << i; end
    for (int i = 0; i < 16; i++) begin #1 v = 16'h1111 * i; end
    #1 v = 16'hffff; #1 v = 16'h0; #1 v = 16'ha5a5; #1 v = 16'h5a5a; #1 v = 16'h1248;
    #1 $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(
        t_lines(&o),
        [
            "T| slice74 n1=13 v1=0 np=4 nn=4 nw=25 dq=0100",
            "T| plus n1=12 v1=1 np=4 nn=4 nw=25 dq=0010",
            "T| plus n1=13 v1=0 np=4 nn=4 nw=25 dq=0001",
            "T| minus n1=12 v1=1 np=4 nn=4 nw=25 dq=0010",
            "T| concat n1=6 v1=1 np=10 nn=7 nw=25 dq=0010",
            "T| nonzero n1=7 v1=0 np=10 nn=7 nw=25 dq=0100",
            "T| whole n1=13 v1=0 np=4 nn=3 nw=25 dq=1000",
            "T| multi n1=13 v1=0 np=4 nn=4 nw=25 dq=0001",
            "T| out n1=13",
            "T| in n1=12 np=3",
        ],
        "{o}"
    );
}

/// `@(dq[g])` on an inout port bit compiles as an edge block watching the
/// parent net (it ran as an interpreted process parked on the whole bus),
/// exactly like the local `@(loc[g])` beside it.
#[test]
fn port_bit_events_are_edge_blocks() {
    let o = run(
        "vw1",
        r#"
`timescale 1ps/1ps
module dev (inout [3:0] dq, input [3:0] din);
  int n = 0, m = 0;
  logic [3:0] loc;
  assign loc = din;
  genvar g;
  for (g = 0; g < 4; g++) begin : g_dq
    always @(dq[g]) n++;      // inout port bit
    always @(loc[g]) m++;     // local net bit
  end
endmodule
module tb;
  wire [7:0] bus;
  logic [7:0] v = 0;
  assign bus = v;
  for (genvar d = 0; d < 2; d++) begin : g_dev
    dev u (.dq(bus[d*4 +: 4]), .din(v[d*4 +: 4]));
  end
  initial begin repeat (3) #10 v = v + 8'h11; #5000; end
endmodule
"#,
        &[("XEZIM_DUMP_EDGE_SENS", "1")],
    );
    let edge: Vec<&str> = o.lines().filter(|l| l.starts_with("[EDGE-SENS]")).collect();
    assert_eq!(edge.len(), 16, "{o}");
    assert_eq!(
        edge.iter().filter(|l| l.ends_with("sens=[bus]")).count(),
        8,
        "{o}"
    );
}

/// §9.4.2: high-bit and cross-word level selects stay on native edge
/// dispatch. A union of low and high terms must retain both masks.
#[test]
fn wide_selects_are_narrowed_edge_blocks() {
    let o = run(
        "wide_native",
        r#"
module observer(inout [3:0] link);
  int hits = 0;
  always @(link[2]) hits++;
endmodule
module tb;
  logic [127:0] drive;
  wire [127:0] bus = drive;
  int combined = 0, boundary = 0, whole = 0;
  observer u(.link(bus[71:68]));
  always @(bus[0] or bus[70]) combined++;
  always @(bus[65:62]) boundary++;
  always @(bus[70] or bus) whole++;
  initial begin
    #1 drive = 0;
    #1 u.hits = 0; combined = 0; boundary = 0; whole = 0;
    #1 drive[1] = 1;
    #1 drive[70] = 1;
    #1 drive[0] = 1;
    #1 drive[63] = 1;
    #1 drive[64] = 1;
    #1 drive[70] = 1'bx;
    #1 drive[70] = 1'bz;
    #1 drive[70] = 0;
    #1 $display("T|high=%0d union=%0d boundary=%0d whole=%0d", u.hits, combined, boundary, whole);
    $finish;
  end
endmodule
"#,
        &[("XEZIM_DUMP_EDGE_SENS", "1")],
    );
    assert_eq!(t_lines(&o), ["T|high=4 union=5 boundary=2 whole=8"], "{o}");
    assert_eq!(
        o.lines().filter(|l| l.starts_with("[EDGE-SENS]")).count(),
        4,
        "{o}"
    );
}

/// §7.4.1 part-selects of ASCENDING vectors that are nets or live inside an
/// instance read the declared labels (`v[0:3]` is the four MSBs). They read
/// the bits mirrored, while bit-selects and top-level variables were right;
/// a level event on such a part-select and a port bound to one follow.
#[test]
fn ascending_part_selects_in_instances_and_nets() {
    let o = run(
        "asc",
        r#"
`timescale 1ns/1ns
module ch (input [7:0] src);
  logic [0:7] av;
  wire  [0:7] aw;
  logic [3:10] ao;
  assign aw = src;
  always @* begin av = src; ao = src; end
  final $display("T| ch av03=%b av47=%b aw03=%b aw47=%b av25=%b aw1=%b ao36=%b ao4=%b aw_up=%b aw_dn=%b",
                 av[0:3], av[4:7], aw[0:3], aw[4:7], av[2:5], aw[1], ao[3:6], ao[4], aw[2 +: 3], aw[6 -: 2]);
endmodule
module dev (inout [3:0] dq);
  int n1 = 0;
  always @(dq[1]) n1++;
  final $display("T| dev n1=%0d dq=%b", n1, dq);
endmodule
module tb;
  logic [7:0] v;
  wire [0:7] abus; assign abus = v;
  int na = 0;
  ch u (.src(v));
  dev d0 (.dq(abus[0:3]));
  always @(abus[2:3]) na++;
  initial begin
    #1 v = 0;
    for (int i = 0; i < 16; i++) begin #1 v = 8'h11 * i; end
    #1 v = 8'b0001_0010;
    #1 $display("T| tb abus03=%b abus47=%b na=%0d", abus[0:3], abus[4:7], na);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(
        t_lines(&o),
        [
            "T| tb abus03=0001 abus47=0010 na=17",
            "T| ch av03=0001 av47=0010 aw03=0001 aw47=0010 av25=0100 aw1=0 ao36=0001 ao4=0 aw_up=010 aw_dn=01",
            "T| dev n1=9 dq=0001",
        ],
        "{o}"
    );
}

/// A select list stays a combinational block when that is unobservable: the
/// body only assigns and reads the listed signals within the listed bits
/// (`@(a[1:0] or b) case (a[1:0])`, the generated-netlist shape). A body
/// reading other bits of a listed signal (`@(a[2]) z = a[3:0]`) must not
/// follow them.
#[test]
fn comb_style_select_lists() {
    let o = run(
        "comb",
        r#"
`timescale 1ns/1ns
module tb;
  logic [7:0] a = 0, b = 0;
  logic [3:0] y, z;
  int nw = 0;
  // comb-style select lists: body reads only the listed bits
  always @( a[1:0] or b) begin
    case (a[1:0])
      2'b00: y = b[3:0];
      2'b01: y = b[7:4];
      default: y = 4'hf;
    endcase
  end
  // body reads an unlisted bit of a listed signal: must not wake on it
  always @(a[2]) z = a[3:0];
  initial begin
    for (int i = 0; i < 40; i++) begin #1 a = a + 8'h3; b = b + 8'h11; end
    #1 $display("T| y=%h z=%h", y, z);
    $finish;
  end
endmodule
"#,
        &[],
    );
    assert_eq!(t_lines(&o), ["T| y=8 z=8"], "{o}");
    let o = run(
        "comb_sens",
        r#"
`timescale 1ns/1ns
module tb;
  logic [7:0] a = 0, b = 0;
  logic [3:0] y, z;
  int nw = 0;
  // comb-style select lists: body reads only the listed bits
  always @( a[1:0] or b) begin
    case (a[1:0])
      2'b00: y = b[3:0];
      2'b01: y = b[7:4];
      default: y = 4'hf;
    endcase
  end
  // body reads an unlisted bit of a listed signal: must not wake on it
  always @(a[2]) z = a[3:0];
  initial begin
    for (int i = 0; i < 40; i++) begin #1 a = a + 8'h3; b = b + 8'h11; end
    #1 $display("T| y=%h z=%h", y, z);
    $finish;
  end
endmodule
"#,
        &[("XEZIM_DUMP_COMB_SENS", "1")],
    );
    assert_eq!(
        o.lines().filter(|l| l.starts_with("[COMB-SENS]")).count(),
        1,
        "{o}"
    );
}

/// The 1-bit nets synthesized to watch an edge of a select (`posedge dq[2]`
/// through a port, `posedge v[1]`) are no design objects: a waveform dump
/// leaves them out.
#[test]
fn edge_alias_nets_stay_out_of_dumps() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("select_event_terms");
    std::fs::create_dir_all(&dir).unwrap();
    let vcd = dir.join("alias.vcd");
    let _ = std::fs::remove_file(&vcd);
    let src = r#"
`timescale 1ns/1ns
module dev (inout [3:0] dq);
  int np = 0;
  always @(posedge dq[2]) np++;
endmodule
module tb;
  logic [7:0] v = 0;
  wire [7:0] bus; assign bus = v;
  int nl = 0;
  dev u (.dq(bus[7:4]));
  always @(posedge v[1]) nl++;
  initial begin
    $dumpfile("VCD"); $dumpvars(0, tb);
    repeat (20) #1 v = v + 8'h13;
    $display("T| np=%0d nl=%0d", u.np, nl);
    $finish;
  end
endmodule
"#
    .replace("VCD", vcd.to_str().unwrap());
    let sv = dir.join("alias.sv");
    std::fs::write(&sv, src).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "--wave",
            "-s",
            "tb",
            "--no-cache",
            sv.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(output.status.success(), "{text}");
    assert_eq!(t_lines(&text), ["T| np=3 nl=5"], "{text}");
    let dump = std::fs::read_to_string(&vcd).unwrap();
    let vars: Vec<&str> = dump.lines().filter(|l| l.contains("$var")).collect();
    assert!(!vars.is_empty(), "{dump}");
    assert!(vars.iter().all(|l| !l.contains("__xz_")), "{vars:?}");
}

/// Generated-netlist select lists through ports and on wide vectors stay
/// combinational: a booth-encoder child whose port reads arrive as selects
/// of the connection (`A[W-1]` over `.A(m[32:0])`, `code[2:0]` over
/// `.code(d[4:2])`), a block listing one bit of a port and reading only it,
/// and a 128-bit select list. Two encoder instances with two blocks each
/// and the wide mux make five combinational entries.
#[test]
fn port_and_wide_select_lists_stay_combinational() {
    let src = r#"
module enc (input [32:0] A, input [2:0] code, output reg [32:0] product, output reg sn);
  parameter W = 33;
  always @( A[32:0] or code[2:0]) begin
    case (code[2:0])
      3'b000, 3'b111: product[W-1:0] = {W{1'b0}};
      3'b001, 3'b010: product[W-1:0] = {A[W-1:0]};
      3'b011: product[W-1:0] = {A[W-2:0], 1'b0};
      3'b100: product[W-1:0] = {~A[W-2:0], 1'b0};
      default: product[W-1:0] = ~A[W-1:0];
    endcase
  end
  always @( A[32] or code[2:0]) begin
    case (code[2:0])
      3'b000, 3'b111: sn = 1'b1;
      3'b001, 3'b010, 3'b011: sn = ~A[W-1];
      default: sn = A[W-1];
    endcase
  end
endmodule
module tb;
  reg [32:0] m = 0;
  reg [6:0] d = 0;
  reg [127:0] w0 = 0, w1 = 0;
  reg [1:0] sel = 0;
  reg [127:0] q;
  wire [32:0] p0, p1; wire s0, s1;
  enc e0 (.A(m[32:0]), .code(d[2:0]), .product(p0), .sn(s0));
  enc e1 (.A(m[32:0]), .code(d[4:2]), .product(p1), .sn(s1));
  always @( w1[127:0] or w0[127:0] or sel[1:0]) begin
    case (sel[1:0]) 2'b10: q[127:0] = w0[127:0]; 2'b01: q[127:0] = w1[127:0]; default: q[127:0] = 128'hx; endcase
  end
  initial begin
    repeat (21) begin
      #1 m = m * 3 + 7; d = d + 5; w0 = {w0[126:0], ~w0[127]}; w1 = w1 + 128'h1_0000_0000_0000_0001; sel = sel + 1;
      #1 $display("T| %h %h %b %b %h", p0, p1, s0, s1, q);
    end
    $finish;
  end
endmodule
"#;
    let o = run("comb_ports", src, &[]);
    assert_eq!(
        t_lines(&o),
        [
            "T| 1fffffff8 000000007 0 1 00000000000000010000000000000001",
            "T| 00000001c 00000001c 1 1 00000000000000000000000000000003",
            "T| 000000000 0000000b6 1 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 1fffffdce 1fffffee7 0 0 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 00000034f 1fffffcb0 1 0 00000000000000050000000000000005",
            "T| 1fffff60b 000000000 0 1 0000000000000000000000000000003f",
            "T| 000003bc6 000000000 1 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 000000000 0000059b0 1 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 1fffef2e8 000021a2e 0 1 00000000000000090000000000000009",
            "T| 00003274c 1fff9b166 1 0 000000000000000000000000000003ff",
            "T| 000000000 1fff68a14 1 0 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 1ffc73c6e 000000000 0 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 00055255f 000000000 1 1 000000000000000d000000000000000d",
            "T| 1ff008fdb 000ff7024 0 1 00000000000000000000000000003fff",
            "T| 005fca0e6 002fe5073 1 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 000000000 1ee0a1d3e 1 0 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 1e50f2bd8 1e50f2bd8 0 0 00000000000000110000000000000011",
            "T| 050d27c7c 1af2d8383 1 0 0000000000000000000000000003ffff",
            "T| 000000000 000000000 1 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 051333f0e 0d7666078 0 1 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "T| 08633216f 08633216f 1 1 00000000000000150000000000000015",
        ],
        "{o}"
    );
    let o = run("comb_ports_sens", src, &[("XEZIM_DUMP_COMB_SENS", "1")]);
    assert_eq!(
        o.lines().filter(|l| l.starts_with("[COMB-SENS]")).count(),
        5,
        "{o}"
    );
}
