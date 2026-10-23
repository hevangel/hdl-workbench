//! #261: `randomize()` on a class with a rand variable wider than 64 bits.
//! The joint solver keeps its domains in 64-bit intervals and refused the
//! whole class, so constraints the trial loop cannot repair (`(x >> 8) ==
//! 0`, `x[N-1:8] == 0`) failed even on the narrow members:
//!
//! - a wide variable no constraint reads (or reads only through `size()`)
//!   now stays out of the solve and is drawn bit by bit (§18.4);
//! - a wide variable a constraint reads is solved as segments of at most 64
//!   bits: shifts by a constant, constant masks, part and bit selects,
//!   struct fields, `==`/`!=`, `inside`, and the orderings (lexicographic
//!   over the segments, the sign bit first for a signed compare) (§11.4.4,
//!   §11.4.5, §11.4.10, §11.4.13, §11.5.1, §7.2.1).
//!
//! The reference simulator solves every class below. Random values differ
//! between simulators, so each line reports the `randomize()` result and
//! whether the constraint holds afterwards.

use xezim::simulate;

const SRC: &str = r#"
class narrow_only;
  rand bit [31:0] x; rand bit [63:0] y;
  constraint k { (x >> 8) == 0; }
endclass
class with_wide;
  rand bit [31:0] x; rand bit [64:0] y;
  constraint k { (x >> 8) == 0; }
endclass
class with_wide_mask;
  rand bit [31:0] x; rand bit [64:0] y;
  constraint k { (x & 32'hffff_ff00) == 0; }
endclass
class with_wide_less;
  rand bit [31:0] x; rand bit [64:0] y;
  constraint k { x < 256; }
endclass

`define T(NAME, N, C) class NAME; rand bit [N-1:0] x, z; rand bit [7:0] n; constraint k { C; } function int check(); return (C); endfunction endclass
`T(w128_lt256, 128, x < 256)
`T(w128_inr, 128, x inside {[100:200]})
`T(w128_eqc, 128, x == 'h1234)
`T(w128_eqz, 128, x == z)
`T(w128_ne0, 128, x != 0)
`T(w128_ltz, 128, x < z)
`T(w128_ltn, 128, x < n)
`T(w128_mask, 128, (x & 'hff) == 0)
`T(w128_ps, 128, x[3:0] == 5)
`T(w128_bit0, 128, x[0] == 1)
`T(w128_shr, 128, (x >> 8) == 0)
`T(w128_hps, 128, x[127:8] == 0)
`T(w256_lt256, 256, x < 256)
`T(w256_inr, 256, x inside {[100:200]})
`T(w256_eqc, 256, x == 'h1234)
`T(w256_eqz, 256, x == z)
`T(w256_ne0, 256, x != 0)
`T(w256_ltz, 256, x < z)
`T(w256_ltn, 256, x < n)
`T(w256_mask, 256, (x & 'hff) == 0)
`T(w256_ps, 256, x[3:0] == 5)
`T(w256_bit0, 256, x[0] == 1)
`T(w256_shr, 256, (x >> 8) == 0)
`T(w256_hps, 256, x[255:8] == 0)
`T(w65_shr, 65, (x >> 8) == 0)
`T(w1024_shr, 1024, (x >> 8) == 0)
`T(w1024_top, 1024, x[1023:1016] == 8'ha5 && x[7:0] == 8'h5a)
`T(w256_shl, 256, (x << 8) == 0)
`T(w256_orm, 256, (x | 'hff) == 'hff)
`T(w128_gtc, 128, x > 'h1_0000_0000_0000_0000 && x < 'h1_0000_0000_0000_0100)
`T(w128_ins, 128, x inside {'h5, 'h7, [10:12]})
`T(w128_nin, 128, !(x inside {[0:'hffff_ffff_ffff_ffff]}))

typedef struct packed { bit [63:0] hi; bit [7:0] tag; bit [63:0] lo; } s136_t;
typedef struct packed { bit [99:0] d; bit [3:0] k; } s104_t;
class sgn;
  rand bit signed [127:0] s; rand bit signed [255:0] t;
  constraint c { s < 0; s > -5; t inside {[-3:3]}; t != 0; }
  function int check(); return s < 0 && s > -5 && t >= -3 && t <= 3 && t != 0; endfunction
endclass
class dynw;
  int data_width = 32;
  rand bit [1023:0] data[];
  rand bit [127:0] strobe[];
  constraint c { data.size() == 4; strobe.size() == data.size();
    foreach (data[i]) (data[i] >> data_width) == 0;
    foreach (strobe[i]) (strobe[i] >> 4) == 0; }
  function int check(); int ok = data.size() == 4 && strobe.size() == 4;
    foreach (data[i]) ok &= (data[i] >> data_width) == 0;
    foreach (strobe[i]) ok &= (strobe[i] >> 4) == 0; return ok; endfunction
endclass
class qw;
  rand bit [255:0] q[$];
  rand bit [3:0] len;
  constraint c { len inside {[1:4]}; q.size() == len; foreach (q[i]) { q[i][255:8] == 0; q[i][7:0] > i; } }
  function int check(); int ok = q.size() == len && len >= 1 && len <= 4;
    foreach (q[i]) ok &= q[i][255:8] == 0 && q[i][7:0] > i; return ok; endfunction
endclass
class fixw;
  rand bit [127:0] fa[3];
  constraint c { foreach (fa[i]) { fa[i][127:120] == i; (fa[i] & 128'hffff) == 'h1234; } }
  function int check(); int ok = 1; foreach (fa[i]) ok &= fa[i][127:120] == i && fa[i][15:0] == 'h1234; return ok; endfunction
endclass
class stw;
  rand s136_t s; rand s104_t u;
  constraint c { s.tag == 8'h5a; s.hi < 100; s.lo[3:0] == 0; u.d[99:90] == 0; u.k == 3; }
  function int check(); return s.tag == 8'h5a && s.hi < 100 && s.lo[3:0] == 0 && u.d[99:90] == 0 && u.k == 3; endfunction
endclass
class softw;
  rand bit [7:0] a; rand bit [99:0] y; rand bit [99:0] fy[2];
  constraint c { soft a < 10; }
  function int check(); return a < 10; endfunction
endclass
typedef struct packed { bit [3:0] num; bit [3:0] first; } h_t;
typedef struct packed { h_t h; bit [99:0] pad; bit [31:0] cnt; } r_t;
class nst;
  rand r_t r; rand bit [1:0] pre;
  constraint c { r.h.num == (4'hf >> pre); r.h.first == 0; r.pad[99:96] == r.h.num; (r.pad >> 96) == r.h.num;
                 r.cnt[15:0] dist { [32:63] :/ 25, 64 :/ 50, [65:192] :/ 25 }; }
  function int check(); return r.h.num == (4'hf >> pre) && r.h.first == 0 && r.pad[99:96] == r.h.num
      && (r.pad >> 96) == r.h.num && r.cnt[15:0] >= 32 && r.cnt[15:0] <= 192; endfunction
endclass
class mixw;
  rand bit [7:0] a; rand bit [3:0] b; rand bit [199:0] w;
  constraint c { a + b == 20; w[199:196] == b; (w >> 196) == b; }
  function int check(); return a + b == 20 && w[199:196] == b && (w >> 196) == b; endfunction
endclass

`define R(NAME) begin NAME o; o = new(); for (int t = 0; t < 3; t++) begin int r; r = o.randomize(); $display("%s r=%0d sat=%0d", `"NAME`", r, o.check()); end end
module top;
  initial begin
    narrow_only a = new(); with_wide b = new(); with_wide_mask c = new(); with_wide_less d = new();
    softw sw = new();
    bit [99:0] py; int ch;
    $display("narrow_only r=%0d sat=%0d", a.randomize(), a.x < 'h100);
    $display("with_wide r=%0d sat=%0d", b.randomize(), b.x < 'h100);
    $display("with_wide_mask r=%0d sat=%0d", c.randomize(), c.x < 'h100);
    $display("with_wide_less r=%0d sat=%0d", d.randomize(), d.x < 'h100);
    `R(w128_lt256) `R(w128_inr) `R(w128_eqc) `R(w128_eqz) `R(w128_ne0) `R(w128_ltz)
    `R(w128_ltn) `R(w128_mask) `R(w128_ps) `R(w128_bit0) `R(w128_shr) `R(w128_hps)
    `R(w256_lt256) `R(w256_inr) `R(w256_eqc) `R(w256_eqz) `R(w256_ne0) `R(w256_ltz)
    `R(w256_ltn) `R(w256_mask) `R(w256_ps) `R(w256_bit0) `R(w256_shr) `R(w256_hps)
    `R(w65_shr) `R(w1024_shr) `R(w1024_top) `R(w256_shl) `R(w256_orm) `R(w128_gtc)
    `R(w128_ins) `R(w128_nin)
    `R(sgn) `R(dynw) `R(qw) `R(fixw) `R(stw) `R(nst) `R(mixw)
    // An unconstrained wide member keeps being drawn over its full width.
    for (int t = 0; t < 3; t++) begin
      py = sw.y;
      $display("softw r=%0d sat=%0d", sw.randomize(), sw.check());
      ch = (sw.y != py) && (sw.y[99:64] != 0) && (sw.fy[0] != sw.fy[1]);
      $display("softw drawn=%0d", ch);
    end
  end
endmodule
"#;

const CLASSES: &[&str] = &[
    "w128_lt256",
    "w128_inr",
    "w128_eqc",
    "w128_eqz",
    "w128_ne0",
    "w128_ltz",
    "w128_ltn",
    "w128_mask",
    "w128_ps",
    "w128_bit0",
    "w128_shr",
    "w128_hps",
    "w256_lt256",
    "w256_inr",
    "w256_eqc",
    "w256_eqz",
    "w256_ne0",
    "w256_ltz",
    "w256_ltn",
    "w256_mask",
    "w256_ps",
    "w256_bit0",
    "w256_shr",
    "w256_hps",
    "w65_shr",
    "w1024_shr",
    "w1024_top",
    "w256_shl",
    "w256_orm",
    "w128_gtc",
    "w128_ins",
    "w128_nin",
    "sgn",
    "dynw",
    "qw",
    "fixw",
    "stw",
    "nst",
    "mixw",
];

#[test]
fn issue_261_wide_rand_variables_randomize() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let mut expected: Vec<String> = [
        "narrow_only",
        "with_wide",
        "with_wide_mask",
        "with_wide_less",
    ]
    .iter()
    .map(|c| format!("{c} r=1 sat=1"))
    .collect();
    for c in CLASSES {
        for _ in 0..3 {
            expected.push(format!("{c} r=1 sat=1"));
        }
    }
    for _ in 0..3 {
        expected.push("softw r=1 sat=1".to_string());
        expected.push("softw drawn=1".to_string());
    }
    assert_eq!(out, expected, "{out:?}");
}

/// Arithmetic across the segments of a wide variable is not modelled: that
/// class alone is left to the trial loop, and `randomize()` never reports
/// success with the constraint violated. The reference simulator solves it
/// (`r=1`); either way the line reads `ok=1`.
#[test]
fn issue_261_wide_arithmetic_not_modelled() {
    let src = r#"
class wadd;
  rand bit [127:0] x;
  constraint k { x + 1 == 'h100; }
endclass
class wshr;
  rand bit [127:0] x; rand bit [127:0] y;
  constraint k { (x >> 8) == 0; }
endclass
module top;
  initial begin
    wadd a = new();
    wshr b = new();
    int r;
    r = a.randomize();
    $display("wadd ok=%0d", r == 0 || a.x == 'hff);
    r = b.randomize();
    $display("wshr r=%0d sat=%0d", r, b.x < 'h100);
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, vec!["wadd ok=1", "wshr r=1 sat=1"], "{out:?}");
}
