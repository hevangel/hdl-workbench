//! Pins for the ivtest round-55 fixes (each shape reference-verified).
//!
//! 1. §6.11.1: a function RETURN VARIABLE carries the return type's
//!    signedness — `function integer f; f = x; f >>>= 3;` must shift
//!    arithmetically. The frame-write signedness stamp reads
//!    `signed_signals`, and return vars/for-init locals must register
//!    there (this escaped to main once: ivtest cfunc_assign_op_vec).
//! 2. §13.4: returning a subroutine-LOCAL dynamic array survives frame
//!    teardown, and a cast of the call packs element 0 most significant
//!    (ivtest sv_darray_function).
//! 3. §3.14.2.3: a class declared under a compilation-unit `timeunit`
//!    keeps that scope's unit for delays and $time regardless of the
//!    caller's module timescale (ivtest br1003a).
//! 4. §6.8 + §10.3.1: `output p; wire p = 1'b1;` — the net decl's
//!    initializer drives the port, and port/net signing merges
//!    (ivtest br_gh540).

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn signed_return_var_arithmetic_shift() {
    let msgs = out(r#"
module test;
function integer asr3(input integer x);
begin
  asr3 = x;
  asr3 >>>= 3;
end
endfunction
localparam m25 = asr3(-25);
initial $display("T|lp=%0d rt=%0d eq=%0d", m25, asr3(-25), m25 === asr3(-25));
endmodule
"#);
    assert!(
        msgs.iter().any(|m| m == "T|lp=-4 rt=-4 eq=1"),
        "signed return var must arithmetic-shift, const and runtime agreeing; got {:?}",
        msgs
    );
}

#[test]
fn dyn_array_function_return_and_cast_pack() {
    let msgs = out(r#"
module main;
typedef logic[7:0] byte_array [];
typedef logic[23:0] byte_vector;
function byte_array inc_array(byte_array inp);
    byte_array tmp;
    tmp = new[$size(inp)];
    for(int i = 0; i < $size(inp); ++i) tmp[i] = inp[i] + 1;
    return tmp;
endfunction
initial begin
    byte_array a, b;
    byte_vector c;
    a = new[3];
    a[0] = 10; a[1] = 11; a[2] = 12;
    b = inc_array(a);
    c = byte_vector'(inc_array(b));
    $display("T|sz=%0d b0=%0d c=%h", $size(b), b[0], c);
end
endmodule
"#);
    assert!(
        msgs.iter().any(|m| m == "T|sz=3 b0=11 c=0c0d0e"),
        "dyn-array return + cast pack; got {:?}",
        msgs
    );
}

#[test]
fn cu_scope_class_timeunit() {
    let msgs = out(r#"
timeunit 100ps / 10ps;
class tc_t;
  task delay(output [63:0] t);
    #5ns t = $time;
  endtask
endclass
module top;
timeunit 1ns / 1ps;
tc_t tc;
reg [63:0] t1, t2;
initial begin
  tc = new;
  tc.delay(t1);
  t2 = $time;
  $display("T|t1=%0d t2=%0d", t1, t2);
end
endmodule
"#);
    assert!(
        msgs.iter().any(|m| m == "T|t1=50 t2=5"),
        "class-scope $time in 100ps units, module $time in 1ns; got {:?}",
        msgs
    );
}

#[test]
fn port_net_decl_init_and_signing_merge() {
    let msgs = out(r#"
module top(p, q, r);
  output p;
  output signed q;
  output r;
  wire p = 1'b1;
  wire q = 1'b1;
  wire signed r = 1'b1;
  reg [1:0] vp, vq, vr;
  initial begin
    #1;
    vp = p; vq = q; vr = r;
    $display("T|p=%b q=%b r=%b", vp, vq, vr);
  end
endmodule
"#);
    assert!(
        msgs.iter().any(|m| m == "T|p=01 q=11 r=11"),
        "net-decl init drives the port; port/net signing merges either way; got {:?}",
        msgs
    );
}

/// §5.7.1: an UNSIZED based literal with an x/z leading digit extends with
/// x/z to the assignment context; a 0/1 leading digit zero-extends; a SIZED
/// literal extends inside its own size. Reference-verified (ivtest
/// sv_packed_port2 — zero-extension made undriven struct elements drive
/// ~00=ff where the reference keeps x).
#[test]
fn unsized_based_literal_xz_extension() {
    let msgs = out(r#"
module test;
  reg [63:0] a, b, c;
  reg [15:0] d;
  initial begin
    a = 'hx3x2x1x0;
    b = 'hz3z2z1z0;
    c = 'h13121110;
    d = 16'hx;
    $display("T|%h %h %h %h", a, b, c, d);
  end
endmodule
"#);
    assert!(
        msgs.iter()
            .any(|m| m == "T|xxxxxxxxx3x2x1x0 zzzzzzzzz3z2z1z0 0000000013121110 xxxx"),
        "unsized x/z-lead literals extend with x/z; got {:?}",
        msgs
    );
}
