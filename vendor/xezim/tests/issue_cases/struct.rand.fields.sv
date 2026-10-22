// ============================================================================
// mwe_struct_rand_fields.sv -- consolidated MWE: randomize() over struct
// FIELDS (xezim 0.11.0, git f8418467).  Merges the original member-dist
// hang MWE with the cousin-gap probe.
//
// GAP 1 (correctness -- the trigger):
//   A `dist` / `inside` constraint whose target is a MEMBER of a `rand`
//   packed-struct class property -- `s.FIELD3`, a nested `s.FIELD1.FIELD1`,
//   or a part-select `s.FIELD2[31:16]` -- does not shape the draw.  The
//   solver draws the whole struct as ONE uniformly-random variable
//   (XEZIM_RAND_DBG=1 prints `RPROP <prop> w=<struct width>`), and the
//   member-level item is only enforced by the post-draw acceptance check.
//   Per-trial acceptance odds are range_size / 2^member_width, so a narrow
//   range exhausts all 1000 trials and randomize() returns 0.
//   Source: `solve_forced`'s ConstraintItem::Inside arm gates the
//   dist-weighted re-pick on `rand_lvalue_name`, which recognises only a
//   BARE rand property name -- never an aggregate sub-field.  (The
//   `rand_member_target` path that makes `s.FIELD3 == const` equality work
//   is not consulted for dist/inside items.)
//
// GAP 2 (cousins -- same root cause, other constraint forms):
//   - relational `<` on a member, either operand order ([3] A, B)
//   - EQUALITY on a part-select of a member ([3] C) and on a 2-level
//     nested member ([3] D) -- the 1-level equality control passes
//   - dist on an UNPACKED struct member ([3] L)
//   - foreach over an unpacked struct's array member ([3] M) -- returns 1
//     with the constraint silently dropped
//   - `unique` on a 2-D fixed array: pigeonhole-UNSAT not detected,
//     returns 1 with values drawn ignoring uniqueness ([3] G)
//
// GAP 3 (storage views -- 2-level member access is split):
//   A HANDLE write (`obj.s.FIELD1.FIELD2` from module scope) is not
//   visible to a METHOD read (`return s.FIELD1.FIELD2;` reads 0), and a
//   method write is not visible to a handle read.  Both views must alias
//   the same storage ([4]).
//
// GAP 4 (amplifier -- the hang):
//   When a class whose randomize() fails this way is randomized through a
//   `rand` OBJECT HANDLE (IEEE 1800-2017 sec. 18.4 nested randomize), the
//   outer solve re-runs the inner 1000-trial solve on EVERY outer trial:
//   1000 x 1000 trials.  Every trial check re-computes packed layouts for
//   each member select, so a register-file-sized struct (~100 members)
//   costs ~0.7 s per failed inner solve here -- the nested call in [6]
//   spins ~13 minutes before returning 0.  (In the reported testbench the
//   struct is ~20k bits with hundreds of constraint items and three rand
//   handles, so the same call spins for hours -- the reported hang.)
//
// Sections:
//   [1] controls that pass today (a fix must keep them passing)
//   [2] the member dist/inside gaps (each fails after 1000 trials, ~0.05 s)
//   [3] the cousin gaps (A/B/C/D/G/L/M fail; E/F/H/H0/I/J/K/N pass)
//   [4] the storage-view split, all four write/read directions
//   [5] direct randomize of the register-file-shaped class (fails, ~0.7 s)
//   [6] THE HANG: nested rand-handle randomize -- announced, runs LAST.
//       Unfixed solver: re-runs the inner 1000-trial solve on every outer
//       trial (~13 minutes here; hours on the reported testbench) --
//       timeout-kill it, or wait for the FAIL.
//       Fixed solver: completes in well under a second and prints PASS.
//
// Run:   xezim --no-cache -s mwe mwe_struct_rand_fields.sv
//        XEZIM_RAND_DBG=1 xezim --no-cache -s mwe mwe_struct_rand_fields.sv
//          (adds per-solve diagnostics: `RPROP ...`, `... randomize FAILED
//           after 1000 trials`, and the failing constraint items)
// ============================================================================

// ----- self-check macros (tests/classes SVTEST_* style, inlined) -----
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH

// `static` spells out the implicit default: an initializer on an
// implicitly-static initial-block variable warns on strict compilers.
`define SVTEST_INIT \
static int failures = 0;

`define SVTEST_CHECK(expr, msg) \
if (!(expr)) begin \
  failures++; \
  $display("FAIL @%0t : %s", $time, msg); \
end

`define SVTEST_PASSFAIL \
if (failures == 0) begin \
  $display("TEST_PASS"); \
end else begin \
  $display("TEST_FAIL count=%0d", failures); \
  $fatal(1); \
end

`endif


package rsp;

  // 2-level nested member target: a 32-bit field (nested equality target)
  // and an 8-bit field (storage-split target).
  typedef struct packed {
    bit [31:0] FIELD1;
    bit [7:0]  FIELD2;
  } sub_t;

  // Small register file -- fast checks in [1]..[4].
  typedef struct packed {
    sub_t      FIELD1;   // nested 2-level member target
    bit [31:0] FIELD2;   // part-select target
    bit [31:0] FIELD3;   // dist / relational target
  } regs_small_t;

  // 12 x 4-bit fields: `unique` over struct members.
  typedef struct packed {
    bit [3:0] FIELD1;  bit [3:0] FIELD2;  bit [3:0] FIELD3;  bit [3:0] FIELD4;
    bit [3:0] FIELD5;  bit [3:0] FIELD6;  bit [3:0] FIELD7;  bit [3:0] FIELD8;
    bit [3:0] FIELD9;  bit [3:0] FIELD10; bit [3:0] FIELD11; bit [3:0] FIELD12;
  } uniq_t;

  // Unpacked structs: dist / foreach targets.
  // Unpacked structs -- LRM 1800-2017 §18.4: a member of an unpacked
  // structure is made random by a `rand` modifier in the declaration of its
  // type; `rand` on the struct property then solves those random members.
  typedef struct { rand int FIELD1; } u_t;
  typedef struct { rand int FIELD1[4]; } ua_t;

  // Register-file-shaped: ~100 members with the constrained ones deep
  // inside, so every trial's member-select check recomputes a real layout
  // (this is what makes one failed inner solve cost ~0.7 s).
  typedef struct packed {
    bit [31:0] FIELD01;  bit [31:0] FIELD02;  bit [31:0] FIELD03;
    bit [31:0] FIELD04;  bit [31:0] FIELD05;  bit [31:0] FIELD06;
    bit [31:0] FIELD07;  bit [31:0] FIELD08;  bit [31:0] FIELD09;
    bit [31:0] FIELD10;  bit [31:0] FIELD11;  bit [31:0] FIELD12;
    bit [31:0] FIELD13;  bit [31:0] FIELD14;  bit [31:0] FIELD15;
    bit [31:0] FIELD16;  bit [31:0] FIELD17;  bit [31:0] FIELD18;
    bit [31:0] FIELD19;  bit [31:0] FIELD20;  bit [31:0] FIELD21;
    bit [31:0] FIELD22;  bit [31:0] FIELD23;  bit [31:0] FIELD24;
    bit [31:0] FIELD25;  bit [31:0] FIELD26;  bit [31:0] FIELD27;
    bit [31:0] FIELD28;  bit [31:0] FIELD29;  bit [31:0] FIELD30;
    bit [31:0] FIELD31;  bit [31:0] FIELD32;  bit [31:0] FIELD33;
    bit [31:0] FIELD34;  bit [31:0] FIELD35;  bit [31:0] FIELD36;
    bit [31:0] FIELD37;  bit [31:0] FIELD38;  bit [31:0] FIELD39;
    bit [31:0] FIELD40;  bit [31:0] FIELD41;  bit [31:0] FIELD42;
    bit [31:0] FIELD43;  bit [31:0] FIELD44;  bit [31:0] FIELD45;
    bit [31:0] FIELD46;  bit [31:0] FIELD47;  bit [31:0] FIELD48;
    bit [31:0] FIELD49;  bit [31:0] FIELD50;  bit [31:0] FIELD51;
    bit [31:0] FIELD52;  bit [31:0] FIELD53;  bit [31:0] FIELD54;
    bit [31:0] FIELD55;  bit [31:0] FIELD56;  bit [31:0] FIELD57;
    bit [31:0] FIELD58;  bit [31:0] FIELD59;  bit [31:0] FIELD60;
    bit [31:0] FIELD61;  bit [31:0] FIELD62;  bit [31:0] FIELD63;
    bit [31:0] FIELD64;  bit [31:0] FIELD65;  bit [31:0] FIELD66;
    bit [31:0] FIELD67;  bit [31:0] FIELD68;  bit [31:0] FIELD69;
    bit [31:0] FIELD70;  bit [31:0] FIELD71;  bit [31:0] FIELD72;
    bit [31:0] FIELD73;  bit [31:0] FIELD74;  bit [31:0] FIELD75;
    bit [31:0] FIELD76;  bit [31:0] FIELD77;  bit [31:0] FIELD78;
    bit [31:0] FIELD79;  bit [31:0] FIELD80;  bit [31:0] FIELD81;
    bit [31:0] FIELD82;  bit [31:0] FIELD83;  bit [31:0] FIELD84;
    bit [31:0] FIELD85;  bit [31:0] FIELD86;  bit [31:0] FIELD87;
    bit [31:0] FIELD88;  bit [31:0] FIELD89;  bit [31:0] FIELD90;
    bit [31:0] FIELD91;  bit [31:0] FIELD92;  bit [31:0] FIELD93;
    bit [31:0] FIELD94;  bit [31:0] FIELD95;  bit [31:0] FIELD96;
    sub_t      FIELD97;   // nested struct member
    bit [575:0] FIELD98;  // 32 clients x 18 bits
    bit [31:0] FIELD99;
    bit [31:0] FIELD100;
    bit [31:0] FIELD101;
  } regs_t;

endpackage : rsp


module mwe;

  import rsp::*;

  // ---------- [1] controls: these pass today ----------
  class CtlScalarDist;                      // dist on a plain scalar rand var
    rand bit [31:0] v;
    constraint c { v dist { [7:7] :/ 1 }; }
  endclass

  class CtlScalarRel;                       // relational on a plain scalar
    rand bit [31:0] v;
    constraint c { v < 32'd5; }
  endclass

  class CtlMemberEq;                        // equality on a struct member
    rand regs_small_t s;
    constraint c { s.FIELD3 == 7; }
    function int dln(); return s.FIELD3; endfunction
  endclass

  class CtlFullRangeDist;                   // full-range dist on a member
    rand regs_small_t s;
    constraint c { s.FIELD3 dist { [0:32'hffffffff] :/ 1 }; }
  endclass

  class CtlMemberLeq;                       // <= between two members
    rand regs_small_t s;
    constraint c { s.FIELD2 <= s.FIELD3; }
  endclass

  // ---------- [2] the member dist/inside gaps: these fail today ----------
  class BugMemberDist;                      // dist on a struct member
    rand regs_small_t s;
    constraint c { s.FIELD3 dist { [7:7] :/ 1 }; }
    function int dln(); return s.FIELD3; endfunction
  endclass

  class BugMemberInside;                    // inside on a struct member
    rand regs_small_t s;
    constraint c { s.FIELD3 inside { [7:7] }; }
    function int dln(); return s.FIELD3; endfunction
  endclass

  class BugPartSelectDist;                  // dist on a part-select
    rand regs_small_t s;
    constraint c { s.FIELD2[15:0] dist { [7:7] :/ 1 }; }
    function int bank_lo(); return s.FIELD2[15:0]; endfunction
  endclass

  class BugIfElseDist;                      // dist inside if/else branches
    rand regs_small_t s;
    bit sel;
    constraint c {
      if (sel) { s.FIELD3 dist { [7:7] :/ 1 }; }
      else     { s.FIELD3 dist { [7:7] :/ 1 }; }
    }
    function int dln(); return s.FIELD3; endfunction
  endclass



  // ---------- [3] cousin gaps around the same solver paths ----------
  // A: relational < on a struct member (FAILS today)
  class A;
    rand regs_small_t s;
    constraint c { s.FIELD3 < 32'd5; }
    function int dln(); return s.FIELD3; endfunction
  endclass

  // B: mirrored relational, rand member on the right (FAILS today)
  class B;
    rand regs_small_t s;
    constraint c { 32'd5 > s.FIELD3; }
    function int dln(); return s.FIELD3; endfunction
  endclass

  // C: equality on part-selects of a member (FAILS today)
  class C;
    rand regs_small_t s;
    constraint c { s.FIELD2[15:0]  == 16'd7;
                   s.FIELD2[31:16] == 16'd8; }
    function int bank(); return s.FIELD2; endfunction
  endclass

  // D: equality on a 2-level nested member (FAILS today)
  class D;
    rand regs_small_t s;
    constraint c { s.FIELD1.FIELD1 == 32'd7; }
    function int f(); return s.FIELD1.FIELD1; endfunction
  endclass

  // E: direct element constraint outside foreach (passes today)
  class E1;  rand int arr[4]; constraint c { arr[0] inside { [7:7] }; } endclass
  class E2;  rand int arr[4]; constraint c { arr[0] == 32'd7; } endclass

  // F: unique over a rand dynamic array, pigeonhole UNSAT (passes: r == 0)
  class F;
    rand bit [3:0] da[];
    constraint c { da.size() == 32; unique { da }; }
  endclass

  // G: unique over a 2-D fixed array, pigeonhole UNSAT (FAILS today:
  // returns 1 with non-unique values -- constraint silently dropped)
  class G;  rand bit [3:0] m[6][6]; constraint c { unique { m }; } endclass

  // H: unique over struct members, 12 x 4-bit (passes today via the joint
  // CSP fallback -- a control proving all-different itself is modelled)
  class H;
    rand uniq_t s;
    constraint c { unique { s.FIELD1, s.FIELD2, s.FIELD3, s.FIELD4, s.FIELD5,
                             s.FIELD6, s.FIELD7, s.FIELD8, s.FIELD9, s.FIELD10,
                             s.FIELD11, s.FIELD12 }; }
    function bit dup();
      bit [3:0] vals [0:11];
      int i, j;
      vals[0]=s.FIELD1;  vals[1]=s.FIELD2;  vals[2]=s.FIELD3;  vals[3]=s.FIELD4;
      vals[4]=s.FIELD5;  vals[5]=s.FIELD6;  vals[6]=s.FIELD7;  vals[7]=s.FIELD8;
      vals[8]=s.FIELD9;  vals[9]=s.FIELD10; vals[10]=s.FIELD11; vals[11]=s.FIELD12;
      for (i = 0; i < 12; i = i + 1)
        for (j = i + 1; j < 12; j = j + 1)
          if (vals[i] == vals[j]) return 1;
      return 0;
    endfunction
  endclass

  // H0 control: the same unique over 12 plain scalars
  class H0;
    // `c2`, not `c`: a rand variable and a constraint block share the class
    // scope namespace, so a variable `c` collides with `constraint c`.
    rand bit [3:0] a, b, c2, d, e, f2, g2, h2, i2, j2, k2, l2;
    constraint c { unique { a, b, c2, d, e, f2, g2, h2, i2, j2, k2, l2 }; }
  endclass

  // I: dist inside randomize() with {} (passes today)
  class Leaf;  rand bit [31:0] v; endclass

  // J/K: outer constraint on a rand-handle member (passes today)
  class OuterIn;
    rand Leaf leaf;
    function new(); leaf = new(); endfunction
    constraint c { leaf.v inside { [7:7] }; }
  endclass
  class OuterRel;
    rand Leaf leaf;
    function new(); leaf = new(); endfunction
    constraint c { leaf.v < 32'd5; }
  endclass
  class OuterEq;
    rand Leaf leaf;
    function new(); leaf = new(); endfunction
    constraint c { leaf.v == 32'd7; }
  endclass

  // L: dist on an UNPACKED struct member (FAILS today)
  class L;
    rand u_t u;
    constraint c { u.FIELD1 dist { [7:7] :/ 1 }; }
    function int f(); return u.FIELD1; endfunction
  endclass

  // M: foreach over an unpacked struct's array member (FAILS today:
  // returns 1 with the constraint silently dropped)
  class M;
    rand ua_t u;
    constraint c { foreach (u.FIELD1[i]) u.FIELD1[i] inside { [7:7] }; }
    function int a0(); return u.FIELD1[0]; endfunction
  endclass

  // N: forced member value visible to both read views (passes today)
  class N;
    rand regs_small_t s;
    constraint c { s.FIELD3 == 32'hDEADBEEF; }
    function int dln(); return s.FIELD3; endfunction
  endclass

  // ---------- [4] storage-view split: 2-level member access ----------
  class ViewSplit;
    regs_small_t s;
    function int rd_mand(); return s.FIELD1.FIELD2; endfunction
    function void wr_mand(input bit [7:0] x); s.FIELD1.FIELD2 = x; endfunction
  endclass

  // ---------- [5] register-file-shaped config (the inner class) ----------
  class RegCfg;
    rand regs_t ctrl_r;
    bit cov_tweaks;
    bit [31:0] clients_iterator;
    constraint c_dln {
      ctrl_r.FIELD100 > 0;
      if (cov_tweaks) {
        ctrl_r.FIELD100 dist {
          0 :/ 2, [32'h1:32'h100] :/ 1, [32'h101:32'hfffffffe] :/ 1, 32'hffffffff :/ 2 };
      } else {
        ctrl_r.FIELD100 dist { [1:10] :/ 1, [11:100] :/ 9, [101:1000] :/ 30 };
      }
    }
    constraint c_targets {
      foreach (clients_iterator[i]) {
        ctrl_r.FIELD98[(i*18) +  8] dist { 1 := 9, 0 := 1 };
        ctrl_r.FIELD98[(i*18) + 17] dist { 1 := 9, 0 := 1 };
      }
    }
    constraint c_bank {
      ctrl_r.FIELD99[15:0] dist { [32:63] :/ 25, 64 :/ 50, [65:192] :/ 25 };
      ctrl_r.FIELD99[31:16] dist { [32:63] :/ 25, 64 :/ 50, [65:192] :/ 25 };
      ctrl_r.FIELD101[8:0] dist { 0 :/ 1, [1:100] :/ 5, [101:200] :/ 11, [201:511] :/ 3 };
      ctrl_r.FIELD101[24:16] dist { 0 :/ 1, [1:100] :/ 5, [101:200] :/ 11, [201:511] :/ 3 };
    }
    constraint c_counters {
      ctrl_r.FIELD97.FIELD2 <= ctrl_r.FIELD97.FIELD1;
    }
  endclass

  // ---------- [6] outer class holding the rand handle ----------
  class RegCfgSet;
    rand RegCfg cfg;
    function new(); cfg = new(); endfunction
  endclass

  // ---------- test driver ----------
  initial begin
    // Declarations carry no initializers: an initial block is static by
    // default, and an initializer on an implicitly-static variable warns on
    // strict compilers (vlog-2244).  Everything is constructed explicitly
    // below instead.  All declarations precede the first statement, as the
    // LRM requires for procedural blocks.
    `SVTEST_INIT
    int r;
    CtlScalarDist     ctl1;
    CtlScalarRel      ctl2;
    CtlMemberEq       ctl3;
    CtlFullRangeDist  ctl4;
    CtlMemberLeq      ctl5;
    BugMemberDist     bug1;
    BugMemberInside   bug2;
    BugPartSelectDist bug3;
    BugIfElseDist     bug4;
    A      a;      B    b;     C   cc;    D   d;
    E1     e1;     E2   e2;
    F      f;      G    g;     H   h;     H0  h0;
    Leaf   obj;    Leaf obj2;  Leaf obj3;
    OuterIn oi;    OuterRel orel;  OuterEq oe;
    L      l;      M    m;     N   n;
    ViewSplit vw;   ViewSplit vw2;
    RegCfg            cfg;
    RegCfgSet         set;

    ctl1 = new();  ctl2 = new();  ctl3 = new();
    ctl4 = new();  ctl5 = new();
    bug1 = new();  bug2 = new();  bug3 = new();  bug4 = new();
    a  = new();  b  = new();  cc = new();  d  = new();
    e1 = new();  e2 = new();
    f  = new();  g  = new();  h  = new();  h0 = new();
    obj  = new();  obj2 = new();  obj3 = new();
    oi   = new();  orel = new();  oe   = new();
    l  = new();  m  = new();  n  = new();
    vw = new();  vw2 = new();
    cfg = new();
    set = new();

    // [1] controls
    r = ctl1.randomize();
    `SVTEST_CHECK(r && (ctl1.v == 7), "1a scalar rand var dist picks from its range")
    r = ctl2.randomize();
    `SVTEST_CHECK(r && (ctl2.v < 5), "1b scalar rand var relational")
    r = ctl3.randomize();
    `SVTEST_CHECK(r && (ctl3.dln() == 7), "1c member equality forces the field")
    r = ctl4.randomize();
    `SVTEST_CHECK(r == 1, "1d full-range dist on a member passes")
    r = ctl5.randomize();
    `SVTEST_CHECK(r == 1, "1e member <= member passes")

    // [2] the member dist/inside gaps
    r = bug1.randomize();
    `SVTEST_CHECK(r && (bug1.dln() == 7), "2a dist on a struct member")
    r = bug2.randomize();
    `SVTEST_CHECK(r && (bug2.dln() == 7), "2b inside on a struct member")
    r = bug3.randomize();
    `SVTEST_CHECK(r && (bug3.bank_lo() == 7), "2c dist on a part-select of a member")
    r = bug4.randomize();
    `SVTEST_CHECK(r && (bug4.dln() == 7), "2d dist inside if/else branches")

    // [3] the cousin gaps
    r = a.randomize();
    `SVTEST_CHECK(r && (a.dln() < 5), "3a member relational <")
    r = b.randomize();
    `SVTEST_CHECK(r && (b.dln() < 5), "3b mirrored member relational")
    r = cc.randomize();
    `SVTEST_CHECK(r && (cc.bank() == 32'h0008_0007), "3c part-select equality")
    r = d.randomize();
    `SVTEST_CHECK(r && (d.f() == 7), "3d nested (2-level) member equality")
    r = e1.randomize();
    `SVTEST_CHECK(r && (e1.arr[0] == 7), "3e element inside outside foreach")
    r = e2.randomize();
    `SVTEST_CHECK(r && (e2.arr[0] == 7), "3f element equality outside foreach")
    r = f.randomize();
    `SVTEST_CHECK(r == 0, "3g unique on dyn array (UNSAT must return 0)")
    r = g.randomize();
    `SVTEST_CHECK(r == 0, "3h unique on 2-D array (UNSAT must return 0)")
    r = h.randomize();
    `SVTEST_CHECK(r && !h.dup(), "3i unique over struct members")
    r = h0.randomize();
    `SVTEST_CHECK(r == 1, "3j unique over scalars (control)")

    r = obj.randomize()  with { v dist { [7:7] :/ 1 }; };
    `SVTEST_CHECK(r && (obj.v == 7), "3k with{} bare-receiver dist")
    r = obj2.randomize() with { obj2.v dist { [7:7] :/ 1 }; };
    `SVTEST_CHECK(r && (obj2.v == 7), "3l with{} prefixed-receiver dist")
    r = obj3.randomize() with { obj3.v == 32'd7; };
    `SVTEST_CHECK(r && (obj3.v == 7), "3m with{} prefixed equality (control)")

    r = oi.randomize();
    `SVTEST_CHECK(r && (oi.leaf.v == 7), "3n inside on rand-handle member")
    r = orel.randomize();
    `SVTEST_CHECK(r && (orel.leaf.v < 5), "3o relational on rand-handle member")
    r = oe.randomize();
    `SVTEST_CHECK(r && (oe.leaf.v == 7), "3p equality on rand-handle member (control)")

    r = l.randomize();
    `SVTEST_CHECK(r && (l.f() == 7), "3q dist on unpacked struct member")
    r = m.randomize();
    `SVTEST_CHECK(r && (m.a0() == 7), "3r foreach over struct array member")

    r = n.randomize();
    `SVTEST_CHECK(r && (n.dln() == 32'hDEADBEEF), "3s forced member visible to method read")
    `SVTEST_CHECK(r && (n.s.FIELD3 == 32'hDEADBEEF), "3t forced member visible to handle read")

    // [4] storage-view split, all four write/read directions
    vw.s.FIELD1.FIELD2 = 8'h5A;
    `SVTEST_CHECK(vw.s.FIELD1.FIELD2 == 8'h5A, "4a handle write then handle read (2-level member)")
    `SVTEST_CHECK(vw.rd_mand() == 8'h5A, "4b handle write then method read (must be same storage)")
    vw2.wr_mand(8'h3C);
    `SVTEST_CHECK(vw2.rd_mand() == 8'h3C, "4c method write then method read")
    `SVTEST_CHECK(vw2.s.FIELD1.FIELD2 == 8'h3C, "4d method write then handle read (must be same storage)")

    // [5] direct randomize of the register-file-shaped config
    r = cfg.randomize();
    `SVTEST_CHECK(r == 1, "5 direct randomize of the register-file config")

    // [6] THE HANG -- nested rand-handle randomize
    $display("[*] 6: nested rand-handle randomize - on an unfixed solver this");
    $display("    re-runs the inner 1000-trial solve on every outer trial");
    $display("    (~13 minutes here; hours on the reported testbench).");
    $display("    Timeout-kill it, or wait for the final FAIL.");
    r = set.randomize();
    `SVTEST_CHECK(r == 1, "6 nested rand-handle randomize")

    `SVTEST_PASSFAIL
  end

endmodule : mwe
