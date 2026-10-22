//! randomize() and the WIDTH of what the solver forces (IEEE 1800-2017
//! §11.6.1, §18.5), plus the dist-frequency and width-boundary guards around
//! it. The consolidated MWE runs verbatim in `issue_cases_runner`
//! (`rand.width.dist.sv`); each section is restated here on its own.
//!
//! A, C, D: a value forced into a rand variable was stored at the forcing
//! expression's width, not the variable's — `ma == 40'h... * tot - 1` put a
//! 40-bit value in an `int unsigned`, `b == 32'd300` put 300 in a `byte`, and
//! a `!=` re-pick drawn at the other operand's 40 bits landed in a 1-bit
//! `bit`. The check then compared the oversized value and accepted
//! unsatisfiable sets. B: a sum of members of a >64-bit rand struct
//! (`{1'b0, a} + {1'b0, b} <= K`) had no repair, so randomize() exhausted its
//! trials.

use xezim::simulate;

fn tagged(src: &str) -> Vec<String> {
    simulate(src, 100_000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T "))
        .collect()
}

/// Failed forcing restores existing values; disabling the offending block
/// then allows the next call to solve at the declared width.
#[test]
fn inherited_width_failure_rolls_back_and_next_call_recovers() {
    let out = tagged(
        r#"
class narrow_base;
  rand bit [3:0] token;
endclass
class narrow_child extends narrow_base;
  rand logic [127:0] padding;
  constraint limit_block { token == 32'd31; }
endclass
module probe;
  initial begin
    narrow_child obj = new();
    int result;
    obj.token = 9;
    obj.padding = 128'h112233445566778899aabbccddeeff00;
    result = obj.randomize();
    $display("T failed=%0d token=%0d restored=%0d", result, obj.token,
             obj.padding == 128'h112233445566778899aabbccddeeff00);
    obj.limit_block.constraint_mode(0);
    result = obj.randomize() with { token == 6; };
    $display("T recovered=%0d token=%0d", result, obj.token);
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        ["T failed=0 token=9 restored=1", "T recovered=1 token=6"]
    );
}

/// Reversed operands and strict bounds take the same sum-repair path.
#[test]
fn mirrored_member_sum_bounds_keep_pinned_terms() {
    let out = tagged(
        r#"
typedef struct packed {
  bit [7:0] first;
  bit [7:0] second;
  bit [127:0] padding;
} bundle_t;
class sum_probe;
  rand bundle_t bundle;
  constraint bounds {
    bundle.first == 2;
    3 >= ({1'b0, bundle.first} + {1'b0, bundle.second});
    3 > ({1'b0, bundle.first} + {1'b0, bundle.second});
  }
endclass
module probe;
  initial begin
    sum_probe obj = new();
    int failures, invalid;
    repeat (64) begin
      if (!obj.randomize()) failures++;
      else if (obj.bundle.first != 2 || obj.bundle.second != 0) invalid++;
    end
    $display("T failures=%0d invalid=%0d", failures, invalid);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T failures=0 invalid=0"]);
}

#[test]
fn impossible_zero_sum_strict_bound_restores_the_bundle() {
    let out = tagged(
        r#"
typedef struct packed {
  bit [3:0] first;
  bit [3:0] second;
  bit [127:0] padding;
} saved_bundle_t;
class impossible_sum;
  rand saved_bundle_t bundle;
  constraint bounds { ({1'b0, bundle.first} + {1'b0, bundle.second}) < 0; }
endclass
module probe;
  initial begin
    impossible_sum obj = new();
    saved_bundle_t saved;
    int result;
    saved = {4'd3, 4'd5, 128'habcdef0123456789};
    obj.bundle = saved;
    result = obj.randomize();
    $display("T result=%0d restored=%0d", result, obj.bundle === saved);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T result=0 restored=1"]);
}

/// Section A: `==` with a right-hand side wider than the variable. With
/// `tot == 0` the right side is 2^40-1, which no 32-bit `ma` equals once
/// both are extended to 40 bits — randomize() must fail and leave `ma`.
#[test]
fn equality_wider_than_the_variable_is_unsatisfiable() {
    let out = tagged(
        r#"
class EqOv;
  rand int unsigned tot;
  rand int unsigned ma;
  constraint c_tot { tot == 0; }
  constraint c_ma  { ma == ((40'h800_0000 * tot) - 1); }
endclass
class EqFits;
  rand int unsigned tot;
  rand int unsigned ma;
  constraint c_tot { tot == 2; }
  constraint c_ma  { ma == ((40'h800_0000 * tot) - 1); }
endclass
module top;
  initial begin
    EqOv a = new(); EqFits b = new(); int r;
    r = a.randomize(); $display("T overflow r=%0d ma=%0d", r, a.ma);
    r = b.randomize(); $display("T fits r=%0d ma=%0h", r, b.ma);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T overflow r=0 ma=0", "T fits r=1 ma=fffffff"]);
}

/// Section D: `==` with a 32-bit literal a signed `byte` cannot hold; a
/// representable one still solves, negative ones included.
#[test]
fn equality_literal_out_of_a_byte_range_is_unsatisfiable() {
    let out = tagged(
        r#"
class EqN;
  rand byte b;
  rand logic [127:0] wide;
  constraint c { b == 32'd300; }
endclass
class EqOk;
  rand byte b;
  rand logic [127:0] wide;
  constraint c { b == -8'sd5; }
endclass
module top;
  initial begin
    EqN n = new(); EqOk k = new(); int r;
    r = n.randomize(); $display("T 300 r=%0d b=%0d", r, n.b);
    r = k.randomize(); $display("T -5 r=%0d b=%0d", r, k.b);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T 300 r=0 b=0", "T -5 r=1 b=-5"]);
}

/// Section C: `b0 != 40'd1` on a 1-bit variable forces `b0 = 0`; the
/// re-pick must never leave the variable's width.
#[test]
fn inequality_repick_stays_in_the_variables_width() {
    let out = tagged(
        r#"
class NeqW;
  rand bit b0;
  rand logic [127:0] wide;
  constraint c_neq { b0 != 40'd1; }
endclass
module top;
  initial begin
    NeqW c = new(); int fails, ones, wide;
    for (int i = 0; i < 200; i++) begin
      if (!c.randomize()) fails++;
      if (c.b0 == 1) ones++;
      if ($bits(c.b0) != 1 || c.b0 > 1) wide++;
    end
    $display("T fails=%0d ones=%0d wide=%0d", fails, ones, wide);
  end
endmodule
"#,
    );
    assert_eq!(out, ["T fails=0 ones=0 wide=0"]);
}

/// Section B: a >64-bit rand struct (the joint solver does not take it)
/// whose members are tied by sums. Only `nhosts_pre == 3` is feasible
/// (`NUM_HOSTS == 4'hf >> nhosts_pre` must leave `NUM_HOSTS + FIRST_HOST <=
/// 1`), and every other summed member must be 0. The same set on scalars
/// and a wide struct with plain member equalities are the contrasts.
#[test]
fn wide_struct_member_sums_are_solved() {
    let out = tagged(
        r#"
typedef logic [3:0] u4_t;
typedef logic [1:0] u2_t;
typedef struct packed {
  bit [1:0] INTLV; bit [1:0] FETCH; bit [1:0] TYPE; bit [1:0] GATING;
  bit [3:0] NUM_HOSTS; bit [3:0] FIRST_HOST; bit [3:0] SPARE_A; bit [3:0] SPARE_B;
} mode_t;
typedef struct packed {
  mode_t MODE;
  bit [31:0] REG_A; bit [31:0] REG_B; bit [31:0] REG_C; bit [31:0] REG_D;
  bit [31:0] REG_E; bit [31:0] REG_F; bit [31:0] REG_G; bit [31:0] REG_H;
} regs_wide_t;
class BaseCfg;
  rand regs_wide_t regs;
  integer RUNARG_FIRST_HOST = -1;
  constraint runarg {
    if (RUNARG_FIRST_HOST != -1) { regs.MODE.FIRST_HOST == RUNARG_FIRST_HOST; }
  }
endclass
class ClientCfg extends BaseCfg;
  rand u4_t nhosts_pre;
  constraint c_num_hosts {
    nhosts_pre dist { 0 :/ 15, 1 :/ 7, 2 :/ 2, 3 :/ 1 };
    regs.MODE.NUM_HOSTS == (4'hf >> nhosts_pre);
  }
  constraint c_mode_cons {
    ({1'b0, regs.MODE.NUM_HOSTS} + {1'b0, regs.MODE.FIRST_HOST}) <= (2 - 1);
    regs.MODE.INTLV == 0;
    ({1'b0, regs.MODE.FETCH} + {1'b0, regs.MODE.TYPE})   <= (2 - 2);
    ({1'b0, regs.MODE.GATING} + {1'b0, regs.MODE.INTLV}) <= (2 - 2);
    ({1'b0, regs.MODE.TYPE} + {1'b0, regs.MODE.GATING})  <= (2 - 2);
    ({1'b0, regs.MODE.SPARE_A} + {1'b0, regs.MODE.SPARE_B}) <= (2 - 2);
  }
endclass
class ScalarCfg;
  rand u4_t nhosts_pre; rand u4_t NUM_HOSTS; rand u4_t FIRST_HOST;
  rand u2_t INTLV; rand u2_t FETCH; rand u2_t TYPE; rand u2_t GATING;
  constraint c_num_hosts {
    nhosts_pre dist { 0 :/ 15, 1 :/ 7, 2 :/ 2, 3 :/ 1 };
    NUM_HOSTS == (4'hf >> nhosts_pre);
  }
  constraint c_mode_cons {
    ({1'b0, NUM_HOSTS} + {1'b0, FIRST_HOST}) <= (2 - 1);
    INTLV == 0;
    ({1'b0, FETCH} + {1'b0, TYPE})   <= (2 - 2);
    ({1'b0, GATING} + {1'b0, INTLV}) <= (2 - 2);
  }
endclass
class Sum3;
  rand regs_wide_t regs;
  constraint c { regs.MODE.FIRST_HOST + regs.MODE.SPARE_A + regs.MODE.SPARE_B < 5; }
endclass
module top;
  initial begin
    ClientCfg c = new(); ScalarCfg s = new(); Sum3 t = new();
    int cfail, cbad, sfail, sbad, tfail, tbad;
    for (int i = 0; i < 20; i++) begin
      if (!c.randomize()) cfail++;
      else if (!(c.nhosts_pre == 3 && c.regs.MODE.NUM_HOSTS == 1 && c.regs.MODE.FIRST_HOST == 0
                 && c.regs.MODE.INTLV == 0 && c.regs.MODE.FETCH == 0 && c.regs.MODE.TYPE == 0
                 && c.regs.MODE.GATING == 0 && c.regs.MODE.SPARE_A == 0
                 && c.regs.MODE.SPARE_B == 0)) cbad++;
      if (!s.randomize()) sfail++;
      else if (!(s.nhosts_pre == 3 && s.NUM_HOSTS == 1 && s.FIRST_HOST == 0)) sbad++;
    end
    for (int i = 0; i < 200; i++) begin
      if (!t.randomize()) tfail++;
      else if (t.regs.MODE.FIRST_HOST + t.regs.MODE.SPARE_A + t.regs.MODE.SPARE_B >= 5) tbad++;
    end
    $display("T struct fails=%0d bad=%0d", cfail, cbad);
    $display("T scalars fails=%0d bad=%0d", sfail, sbad);
    $display("T three-term fails=%0d bad=%0d", tfail, tbad);
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        [
            "T struct fails=0 bad=0",
            "T scalars fails=0 bad=0",
            "T three-term fails=0 bad=0",
        ]
    );
}

/// Section E: dist frequencies over 10000 randomize() calls each — `:=`
/// weights, `:/` and `:=` over a range, signed items, a narrow variable,
/// dist with a relational and with `inside`, and a dist on a member of a
/// >64-bit struct. Counts are per mille with >= 12-sigma tolerances.
#[test]
fn dist_frequencies_hold() {
    let out = tagged(
        r#"
class DistW; rand int unsigned v; constraint c { v dist { 0 := 1, 1 := 2, 2 := 3, 3 := 4 }; } endclass
class DistR; rand int unsigned v; constraint c { v dist { 1 := 1, [2:3] :/ 1 }; } endclass
class DistE; rand int unsigned v; constraint c { v dist { 1 := 1, [2:3] := 1 }; } endclass
class DistS; rand int s; constraint c { s dist { -1 := 1, 1 := 1 }; } endclass
class DistN; rand bit [3:0] x; constraint c { x dist { [0:15] := 1 }; } endclass
class DistC; rand int unsigned v; constraint c { v dist { 5 := 1, 6 := 3 }; v >= 6; } endclass
class DistI; rand int unsigned v; constraint c { v inside { [0:9] }; v dist { 0 := 1, [1:9] :/ 1 }; } endclass
class DistM;
  typedef struct packed { bit [3:0] f; bit [127:0] pad; } st_t;
  rand st_t s;
  constraint c { s.f dist { 0 := 1, 15 := 1 }; }
endclass
module top;
  localparam int N = 2000;
  function automatic bit near(int count, int pm, int tol);
    return 1000 * count / N >= pm - tol && 1000 * count / N <= pm + tol;
  endfunction
  initial begin
    DistW w = new(); DistR r = new(); DistE e = new(); DistS s = new();
    DistN n = new(); DistC c = new(); DistI d = new(); DistM m = new();
    int cw[4], cr[4], ce[4], neg, pos, cn[16], c6, cother, cd[10], mz, mf, fails;
    bit ok;
    for (int i = 0; i < N; i++) begin
      if (!w.randomize()) fails++; else if (w.v < 4) cw[w.v]++;
      if (!r.randomize()) fails++; else if (r.v inside {[1:3]}) cr[r.v]++;
      if (!e.randomize()) fails++; else if (e.v inside {[1:3]}) ce[e.v]++;
      if (!s.randomize()) fails++; else begin if (s.s == -1) neg++; if (s.s == 1) pos++; end
      if (!n.randomize()) fails++; else cn[n.x]++;
      if (!c.randomize()) fails++; else if (c.v == 6) c6++; else cother++;
      if (!d.randomize()) fails++; else if (d.v <= 9) cd[d.v]++;
      if (!m.randomize()) fails++; else begin if (m.s.f == 0) mz++; if (m.s.f == 15) mf++; end
    end
    $display("T fails=%0d", fails);
    ok = 1; foreach (cw[k]) ok &= near(cw[k], 100 * (k + 1), 80);
    $display("T := weights %0d", ok);
    $display("T :/ range %0d", near(cr[1], 500, 80) && near(cr[2], 250, 80) && near(cr[3], 250, 80));
    $display("T := range %0d", near(ce[1], 333, 80) && near(ce[2], 333, 80) && near(ce[3], 333, 80));
    $display("T signed %0d", near(neg, 500, 80) && near(pos, 500, 80));
    ok = 1; foreach (cn[k]) ok &= near(cn[k], 62, 45);
    $display("T narrow %0d", ok);
    $display("T dist+relational six=%0d other=%0d", c6, cother);
    ok = near(cd[0], 500, 80); for (int k = 1; k <= 9; k++) ok &= near(cd[k], 55, 45);
    $display("T dist+inside %0d", ok);
    $display("T wide member %0d", near(mz, 500, 80) && near(mf, 500, 80));
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        [
            "T fails=0",
            "T := weights 1",
            "T :/ range 1",
            "T := range 1",
            "T signed 1",
            "T narrow 1",
            "T dist+relational six=2000 other=0",
            "T dist+inside 1",
            "T wide member 1",
        ]
    );
}

/// Section F: the forcing arms that already clamp to the variable's width —
/// an unreachable relational bound, signed `inside` ranges, an affine `==`
/// with and without a representable solution, an `inside` range wider than
/// the variable, and a dist item the variable cannot hold.
#[test]
fn width_boundaries_of_the_other_forcing_arms() {
    let out = tagged(
        r#"
class W1C; rand bit [3:0] x; rand logic [127:0] wide; constraint c { x > 32'hFFFFFFF0; } endclass
class W2C; rand byte sb; rand logic [127:0] wide; constraint c { sb inside { [-128:-120], [120:127] }; } endclass
class W3C; rand byte sb; rand logic [127:0] wide; constraint c { sb + 5 == 32'd300; } endclass
class W4C; rand byte sb; rand logic [127:0] wide; constraint c { sb + 5 == 32'd100; } endclass
class W5C; rand bit [3:0] x; rand logic [127:0] wide; constraint c { x inside { [0:100] }; } endclass
class W6C; rand bit [3:0] x; rand logic [127:0] wide; constraint c { x dist { 20 := 1, 5 := 1 }; } endclass
module top;
  initial begin
    W1C a = new(); W2C b = new(); W3C c = new(); W4C d = new(); W5C e = new(); W6C f = new();
    int r, five, other, fails;
    r = a.randomize(); $display("T unreachable bound r=%0d", r);
    r = b.randomize();
    $display("T signed inside r=%0d ok=%0d", r, (b.sb >= -128 && b.sb <= -120) || (b.sb >= 120 && b.sb <= 127));
    r = c.randomize(); $display("T affine unrepresentable r=%0d", r);
    r = d.randomize(); $display("T affine r=%0d sb=%0d", r, d.sb);
    r = e.randomize(); $display("T wide inside r=%0d", r);
    for (int i = 0; i < 1000; i++) begin
      if (!f.randomize()) fails++; else if (f.x == 5) five++; else other++;
    end
    $display("T unreachable dist item fails=%0d five=%0d other=%0d", fails, five, other);
  end
endmodule
"#,
    );
    assert_eq!(
        out,
        [
            "T unreachable bound r=0",
            "T signed inside r=1 ok=1",
            "T affine unrepresentable r=0",
            "T affine r=1 sb=95",
            "T wide inside r=1",
            "T unreachable dist item fails=0 five=1000 other=0",
        ]
    );
}
