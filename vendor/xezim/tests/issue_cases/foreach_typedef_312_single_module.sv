
`ifndef SVTEST_DEFS_SVH
`define SVTEST_DEFS_SVH
`define SVTEST_INIT \
int failures = 0;
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

// =============================================================================
// Section 1: file-scope types, packages, classes and interfaces
// (verbatim from the individual survey cases, so results stay comparable)
// =============================================================================

// ----- T00 / T01: package-scope AA typedef with unpacked-struct elements -----
package foreach_aa_p;                       // c00
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
  aa_t   aa_via_typedef;                    // BROKEN: foreach iterates garbage
  elem_t aa_direct [string];                // control
endpackage

package c01_p;                              // c01
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
  aa_t   aa_via_typedef;
  elem_t aa_direct [string];
endpackage

// ----- T02: the SAME shape at module scope (control) -----
typedef struct { int x; } m_elem_t;
typedef m_elem_t m_aa_t [string];

// ----- T03: AA typedef as a class property (control) -----
package c03_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

class c03_bag;
  c03_p::aa_t   m_td;                       // typedef'd AA property
  c03_p::elem_t m_dir [string];             // control
  function new();
  endfunction
  function void fill();
    c03_p::elem_t e;
    e.x = 11; m_td["a"] = e;
    e.x = 22; m_td["b"] = e;
    e.x = 33; m_dir["a"] = e;
  endfunction
  // foreach INSIDE a class method
  function int td_num();
    return m_td.num();
  endfunction
  function int td_count();
    int n; n = 0; foreach (m_td[k]) n++; return n;
  endfunction
  function int td_sum();
    int s; s = 0; foreach (m_td[k]) s += m_td[k].x; return s;
  endfunction
  function int dir_num();
    return m_dir.num();
  endfunction
  function int dir_count();
    int n; n = 0; foreach (m_dir[k]) n++; return n;
  endfunction
  function int dir_sum();
    int s; s = 0; foreach (m_dir[k]) s += m_dir[k].x; return s;
  endfunction
endclass

// ----- T04 / T05 / T06: package-scope typedef'd queue / dynamic / fixed -----
package c04_p;
  typedef int q_t [$];
  q_t      q_via_typedef;                   // BROKEN?
  int      q_direct [$];                    // control
endpackage

package c05_p;
  typedef int d_t [];
  d_t      d_via_typedef;                   // BROKEN?
  int      d_direct [];                     // control
endpackage

package c06_p;
  typedef int f_t [0:3];
  f_t      f_via_typedef;                   // BROKEN?
  int      f_direct [0:3];                  // control
endpackage

// ----- T07: chained (2-hop) typedef at module scope -----
typedef int c07_base_t [0:3];
typedef c07_base_t c07_mid_t;

// ----- T08: chained + cross-package typedef, package-scope variables -----
package c08_src_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
  typedef aa_t   aa_alias_t;                // second hop
endpackage

package c08_use_p;
  c08_src_p::aa_t       aa_crosspkg;        // cross-package typedef, 1 hop
  c08_src_p::aa_alias_t aa_crosspkg_2hop;   // cross-package typedef, 2 hops
endpackage

// ----- T09: typedef in a package, VARIABLE at module scope (control) -----
package c09_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

// ----- T10: typedef'd AA as a STRUCT FIELD (package-scope struct) -----
package c10_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
  typedef struct {
    aa_t        m_td;                       // struct field whose type IS an AA typedef
    elem_t      m_dir [string];             // control field
    int         tag;
  } holder_t;
endpackage

// ----- T11: typedef'd AA as a ref formal and as a subroutine local (control) -----
package c11_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

// ----- T12: AA of AA, both levels via typedef -----
package c12_p;
  typedef int inner_t [string];
  typedef inner_t outer_t [int];
  outer_t nested_td;                        // BROKEN?
  int     nested_dir [int][string];         // control
endpackage

// ----- T13: the whole AA built-in API on a package-scope typedef'd AA -----
package c13_p;
  typedef int aa_t [string];
  aa_t td;
  int  dir [string];
endpackage

// ----- T14: 2-D AA typedef + int-keyed AA typedef -----
package c14_p;
  typedef int m2_t [int][string];
  typedef int ai_t [int];
  m2_t m_td;
  ai_t ai_td;                               // int-keyed AA via typedef
  int  ai_dir [int];                        // control
endpackage

// ----- T15: AA typedef at INTERFACE scope (control) -----
package c15_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

interface c15_if(input bit clk);
  c15_p::aa_t   aa_td;                      // AA typedef at interface scope
  c15_p::elem_t aa_dir [string];            // control
endinterface

// ----- T16: foreach from INSIDE a package subroutine + class static member -----
package c16_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
  aa_t tbl;

  function automatic int count_all();
    int n;
    n = 0;
    foreach (tbl[k]) n++;
    return n;
  endfunction

  function automatic int sum_all();
    int s;
    s = 0;
    foreach (tbl[k]) s += tbl[k].x;
    return s;
  endfunction
endpackage

class c16_static_holder;
  static c16_p::aa_t s_reg;                 // class STATIC member of an AA typedef type
  static function int count_static();
    int n;
    n = 0;
    foreach (s_reg[k]) n++;
    return n;
  endfunction
endclass

// ----- T17: 2-D AA foreach with NO typedef (separate bug, both scopes) -----
package c17_pkg_p;
  int m_pkg [int][string];
endpackage

// ----- T18: element-type sensitivity of the package typedef'd AA bug -----
package c18_p;
  typedef struct { int x; } elem_t;
  typedef int      aa_int_t  [string];
  typedef elem_t   aa_struct_t [string];
  typedef string   aa_str_t  [string];
  typedef bit [7:0] aa_bits_t [string];
  aa_int_t    a_int;
  aa_struct_t a_struct;
  aa_str_t    a_str;
  aa_bits_t   a_bits;
  int         d_int    [string];            // controls
  elem_t      d_struct [string];
endpackage

// ----- T19: MECHANISM probe — bogus trip count tracks the element width -----
package c19_p;
  typedef bit [3:0]   aa_w4_t  [string];
  typedef bit [31:0]  aa_w32_t [string];
  typedef bit [63:0]  aa_w64_t [string];
  typedef bit [127:0] aa_w128_t [string];
  aa_w4_t   a4;
  aa_w32_t  a32;
  aa_w64_t  a64;
  aa_w128_t a128;
  int       ctrl [string];
endpackage

// ----- T20: query functions and reduction methods on typedef'd collections -----
package c20_p;
  typedef int aa_t  [string];
  typedef int aai_t [int];      // $size is legal only on INTEGRAL-key AAs
  typedef int q_t   [$];
  typedef int d_t   [];
  aa_t  aa;
  aai_t aai;
  q_t   q;
  d_t   d;
  int   aa_dir  [string];
  int   aai_dir [int];
  int   q_dir   [$];
endpackage

// ----- T21: typedef'd KEY type (string key + narrow typed key) -----
package c21_p;
  typedef string                 key_t;
  typedef int                    val_t;
  typedef val_t                  aa_strkey_t [key_t];  // key via typedef
  typedef bit [3:0]              nib_t;
  typedef val_t                  aa_nibkey_t [nib_t];  // narrow typed key
  aa_strkey_t  a_strkey;
  aa_nibkey_t  a_nibkey;
  int          dir_strkey [string];         // controls
  int          dir_nibkey [bit [3:0]];
endpackage

// ----- T22: assignment-pattern initializers on a package-scope typedef'd AA -----
package c22_p;
  typedef int aa_t [string];
  aa_t  with_pattern = '{"a": 1, "b": 2};   // keyed pattern initializer
  aa_t  with_default = '{default: 0, "c": 7};
  aa_t  plain;                              // no initializer
  aa_t  aliased;                            // assigned from another AA
  int   dir_pattern [string] = '{"a": 1, "b": 2}; // control
endpackage

// ----- T23: AA typedef as a PORT and as a CHILD-INSTANCE variable (control) ---
package c23_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

// The child probes itself from its own initial block, gated on `go` so the
// parent has populated its own AA first, and reporting back with `done`.
//
// NOTE: the AA-as-PORT form (`module c23_child(c23_p::aa_t m_in, ...)`) is NOT
// legal SystemVerilog - xrun rejects it with QUENVR, and a `ref` port of an AA
// with ILLPID "Illegal port decl. [A.2.1.2]".  xezim accepts both: an
// over-acceptance bug.  The legal boundary is an ordinary int output.
module c23_child(input  bit go,
                 output int n_local, output bit done);
  c23_p::aa_t local_td;
  initial begin
    c23_p::elem_t e;
    done = 0; n_local = 0;
    wait (go == 1);                         // parent has filled its AA
    e.x = 1; local_td["x"] = e;
    e.x = 2; local_td["y"] = e;
    foreach (local_td[k])   n_local++;
    $display("[OBS] child local foreach count = %0d", n_local);
    done = 1;
    $fflush;
  end
endmodule

// ----- T24: typedef-dims PEEL DEPTH for AA typedefs at module scope -----
package c24_p;
  typedef struct { int x; } elem_t;
  typedef elem_t             aa1_t [string]; // 1 hop
endpackage

typedef c24_p::aa1_t         aa2_t;          // 2 hops
typedef aa2_t                aa3_t;          // 3 hops

// ----- T25: AA typedef as a struct FIELD declared at FILE scope -----
typedef struct { int x; } c25_elem_t;
typedef c25_elem_t c25_aa_t [string];
typedef struct {
  c25_aa_t      m_td;                        // struct field whose type IS an AA typedef
  c25_elem_t    m_dir [string];              // control field
  int           tag;
} c25_holder_t;

// =====================================================================
// Section 1b: the 15 follow-up PROBE cases (probe_new/p1..p15) -> T26..T40.
// Each probe keeps its own package (pNN_p); the file-scope 2-hop aliases are
// renamed pNN_alias_t so all 15 can share this one compilation unit.
// =====================================================================

// ----- T26 / T27: 2-hop typedef'd AA as a local in a function / method -----
package p1_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];   // 1 hop
endpackage
typedef p1_p::aa_t p1_alias_t;    // 2 hops

package p2_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p2_p::aa_t p2_alias_t;

class p2_c;
  function int count_local();
    p2_alias_t loc;               // 2-hop typedef'd AA local in a method
    p2_p::elem_t e;
    int n;
    e.x = 8; loc["z"] = e;
    n = 0;
    foreach (loc[k]) n++;
    return n;
  endfunction
endclass

// ----- T28: function RETURN type is a typedef'd AA (1 hop) -----
package p3_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

// ----- T29: input (by-value) FORMAL of typedef'd AA type, 1 hop -----
package p4_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

// ----- T30: typedef'd AA declared in a NAMED begin/end block -----
package p5_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p5_p::aa_t p5_alias_t;

// ----- T31: typedef'd AA declared in a fork/join block -----
package p6_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p6_p::aa_t p6_alias_t;

// ----- T32: 2-hop typedef'd AA at INTERFACE scope -----
package p7_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p7_p::aa_t p7_alias_t;

interface p7_if(input bit clk);
  p7_alias_t    aa_td2;           // 2-hop typedef at interface scope
  p7_p::aa_t    aa_td1;           // 1-hop control (c15 shape)
  p7_p::elem_t  aa_dir [string];  // direct control
endinterface

// ----- T33: STATIC local of typedef'd AA type in a task -----
package p8_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p8_p::aa_t p8_alias_t;

// ----- T34: package-scope typedef'd AA reached through a WILDCARD import -----
package p9_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
  aa_t   aa_via_typedef;
  elem_t aa_direct [string];
endpackage

// ----- T35: 2-hop typedef'd QUEUE and DYNAMIC ARRAY as task locals -----
package p10_p;
  typedef int q_t  [$];
  typedef int d_t  [];
endpackage
typedef p10_p::q_t p10_qalias_t;
typedef p10_p::d_t p10_dalias_t;

// ----- T36: class PROPERTY with a 2-hop typedef'd AA -----
package p11_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p11_p::aa_t p11_alias_t;

class p11_c;
  p11_alias_t   m_td2;            // 2-hop typedef'd AA property
  p11_p::aa_t   m_td1;            // 1-hop control (c03 shape)
  function int count2();
    int n = 0;
    foreach (m_td2[k]) n++;
    return n;
  endfunction
  function int count1();
    int n = 0;
    foreach (m_td1[k]) n++;
    return n;
  endfunction
endclass

// ----- T37: ref FORMAL with a 2-hop typedef'd AA type -----
package p12_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p12_p::aa_t p12_alias_t;

// ----- T38: typedef'd AA declared inside a GENERATE-FOR block -----
package p13_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage
typedef p13_p::aa_t p13_alias_t;

// ----- T39: OUTPUT formal of typedef'd AA type -----
package p14_p;
  typedef struct { int x; } elem_t;
  typedef elem_t aa_t [string];
endpackage

// ----- T40: package var with dims on BOTH the typedef and the declarator -----
package p15_p;
  typedef int fa_t [0:3];
  fa_t v [0:1];                   // 2-D: [0:1] outer, [0:3] from typedef
  int  v_dir [0:1][0:3];          // direct control
endpackage


// =============================================================================
// Section 2: testbench module - shared harness + all 41 cases
// =============================================================================
module tb_foreach_typedef_all;

`SVTEST_INIT
int    cur_case = 0;
string cur_name = "";

task automatic case_begin(int n, string nm);
   cur_case = n; cur_name = nm;
   $display("[%0t] CASE %0d BEGIN : %s", $time, n, nm);
   $fflush;
endtask

task automatic case_end();
   $display("[%0t] CASE %0d DONE  : %s", $time, cur_case, cur_name);
   $fflush;
endtask

task automatic chk(bit ok, string msg);
   if (!ok) begin
      failures++;
      $display("FAIL @%0t [CASE %0d %s] : %s", $time, cur_case, cur_name, msg);
      $fflush;
   end
endtask

// int compare that reports got/expected, routed through chk()
task automatic ck(string what, int got, int exp);
   chk(got === exp, $sformatf("%s: got %0d, expected %0d", what, got, exp));
endtask

// observation only (mechanism probes)
task automatic obs(string what, int got);
   $display("[OBS] [CASE %0d %s] %s = %0d", cur_case, cur_name, what, got);
   $fflush;
endtask

// ----- module-scope state for the module-scope cases -----
m_aa_t        t02_aa_td;                    // T02 control (module scope)
m_elem_t      t02_aa_dir [string];
c07_mid_t     t07_v_chained;                // T07 two typedef hops
c07_base_t    t07_v_onehop;                 // T07 one hop (control)
int           t07_v_direct [0:3];
c09_p::aa_t   t09_aa;                       // T09 control
int           t17_m_mod [int][string];      // T17 module scope
c24_p::aa1_t  t24_a1;                       // T24 1 hop
aa2_t         t24_a2;                       // T24 2 hops
aa3_t         t24_a3;                       // T24 3 hops

bit           t15_clk = 0;
c15_if        u_if(.clk(t15_clk));          // T15 interface instance

bit           t23_go;
bit           t23_done;
int           t23_n_local;
c23_p::aa_t   t23_drv;
c23_child     u_child(.go(t23_go), .n_local(t23_n_local), .done(t23_done));

// ----- T11 helpers: typedef'd AA as a ref formal and as a subroutine local -----
function automatic int t11_count_td(ref c11_p::aa_t m);
  int n; n = 0;
  foreach (m[k]) n++;
  return n;
endfunction

function automatic int t11_sum_td(ref c11_p::aa_t m);
  int s; s = 0;
  foreach (m[k]) s += m[k].x;
  return s;
endfunction

function automatic int t11_count_local();
  c11_p::aa_t   loc;
  c11_p::elem_t e;
  int n;
  e.x = 8; loc["z"] = e;
  n = 0;
  foreach (loc[k]) n++;
  return n;
endfunction

// ----- module-scope state for the probe cases (T26..T40) -----
import p9_p::*;                          // T34: wildcard import (bare names)

p7_if         p32_u_if(.clk(t15_clk));    // T32 interface instance

bit           p13_go;                     // T38 generate-block handshake
bit           p13_done;
genvar        p13_g;
generate
  for (p13_g = 0; p13_g < 1; p13_g = p13_g + 1) begin : p13_genblk
    p13_p::aa_t     loc1;                 // 1-hop generate local
    p13_alias_t     loc2;                 // 2-hop generate local
    p13_p::elem_t   aa_dir [string];      // direct control
    initial begin
      p13_p::elem_t e;
      int n;
      wait (p13_go == 1);                 // released by t38_p13_generate_local()
      e.x = 1; loc1["a"] = e;
      e.x = 2; loc2["a"] = e;
      e.x = 3; aa_dir["a"] = e;
      ck("gen 1hop num()", loc1.num(), 1);
      ck("gen 2hop num()", loc2.num(), 1);
      n = 0; foreach (loc1[k]) n++;
      ck("gen 1hop foreach", n, 1);
      n = 0; foreach (loc2[k]) n++;
      ck("gen 2hop foreach", n, 1);
      n = 0; foreach (aa_dir[k]) n++;
      ck("gen direct foreach", n, 1);
      p13_done = 1;
      $fflush;
    end
  end
endgenerate

// ----- T26 helpers: 2-hop typedef'd AA local in a plain function -----
function automatic int t26_count_local();
  p1_alias_t loc;                  // 2-hop typedef'd AA local
  p1_p::elem_t e;
  int n;
  e.x = 8; loc["z"] = e;
  n = 0;
  foreach (loc[k]) n++;
  return n;
endfunction

function automatic int t26_count_local_1hop();
  p1_p::aa_t loc;                  // control: 1-hop local (c11 shape)
  p1_p::elem_t e;
  int n;
  e.x = 8; loc["z"] = e;
  n = 0;
  foreach (loc[k]) n++;
  return n;
endfunction

// ----- T28 helper: function whose RETURN type is a typedef'd AA -----
function automatic p3_p::aa_t t28_make();
  p3_p::elem_t e;
  e.x = 1; t28_make["a"] = e;
  e.x = 2; t28_make["b"] = e;
endfunction

// ----- T29 helpers: input (by-value) and ref formals of typedef'd AA -----
function automatic int t29_count_val(input p4_p::aa_t a);
  int n;
  n = 0;
  foreach (a[k]) n++;
  return n;
endfunction

function automatic int t29_count_ref(ref p4_p::aa_t a);
  int n;
  n = 0;
  foreach (a[k]) n++;
  return n;
endfunction

// ----- T33 helpers: STATIC locals of typedef'd AA in a task -----
task automatic t33_fill_and_count1(output int n);
  static p8_p::aa_t loc;           // 1-hop STATIC local
  p8_p::elem_t e;
  e.x = 1; loc["a"] = e;
  n = 0;
  foreach (loc[k]) n++;
endtask

task automatic t33_fill_and_count2(output int n);
  static p8_alias_t loc;           // 2-hop STATIC local
  p8_p::elem_t e;
  e.x = 1; loc["a"] = e;
  n = 0;
  foreach (loc[k]) n++;
endtask

// ----- T35 helpers: 2-hop typedef'd queue / dynarray locals -----
function automatic int t35_count_q();
  p10_qalias_t loc;                // 2-hop typedef'd queue local
  int n;
  loc.push_back(7);
  n = 0;
  foreach (loc[i]) n++;
  return n;
endfunction

function automatic int t35_count_q1();
  p10_p::q_t loc;                  // 1-hop control
  int n;
  loc.push_back(7);
  n = 0;
  foreach (loc[i]) n++;
  return n;
endfunction

function automatic int t35_count_d();
  p10_dalias_t loc;                // 2-hop typedef'd dynarray local
  int n;
  loc = new[2];
  n = 0;
  foreach (loc[i]) n++;
  return n;
endfunction

function automatic int t35_count_d1();
  p10_p::d_t loc;                  // 1-hop control
  int n;
  loc = new[2];
  n = 0;
  foreach (loc[i]) n++;
  return n;
endfunction

// ----- T37 helpers: ref formals with 1-hop / 2-hop typedef'd AA -----
function automatic int t37_count_ref2(ref p12_alias_t m);
  int n = 0;
  foreach (m[k]) n++;
  return n;
endfunction

function automatic int t37_count_ref1(ref p12_p::aa_t m);
  int n = 0;
  foreach (m[k]) n++;
  return n;
endfunction

// ----- T39 helper: OUTPUT formal of typedef'd AA type -----
task automatic t39_make_aa(output p14_p::aa_t m);
  p14_p::elem_t e;
  e.x = 1; m["a"] = e;
  e.x = 2; m["b"] = e;
endtask


// -----------------------------------------------------------------------------
// T00 - c00: package-scope AA typedef, EMPTY AA.  THE REPORTED BUG.
// -----------------------------------------------------------------------------
task automatic t00_pkg_aa_empty();
   int n;
   case_begin(0, "c00 package AA typedef, EMPTY AA (REPORTED BUG)");
   ck("typedef'd AA num()", foreach_aa_p::aa_via_typedef.num(), 0);
   n = 0; foreach (foreach_aa_p::aa_via_typedef[k]) n++;
   ck("typedef'd AA foreach count", n, 0);
   ck("direct AA num()", foreach_aa_p::aa_direct.num(), 0);
   n = 0; foreach (foreach_aa_p::aa_direct[k]) n++;
   ck("direct AA foreach count", n, 0);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T01 - c01: POPULATED package-scope typedef'd AA.
// -----------------------------------------------------------------------------
task automatic t01_pkg_aa_populated();
   c01_p::elem_t e;
   int n, sum_td, sum_dir;
   case_begin(1, "c01 package AA typedef, populated");
   e.x = 0;
   e.x = 10; c01_p::aa_via_typedef["a"] = e;
   e.x = 20; c01_p::aa_via_typedef["b"] = e;
   e.x = 30; c01_p::aa_direct["a"] = e;
   e.x = 40; c01_p::aa_direct["b"] = e;

   ck("typedef'd num()",     c01_p::aa_via_typedef.num(), 2);
   ck("typedef'd exists(a)", c01_p::aa_via_typedef.exists("a"), 1);
   ck("typedef'd read b.x",  c01_p::aa_via_typedef["b"].x, 20);

   n = 0; sum_td = 0;
   foreach (c01_p::aa_via_typedef[k]) begin n++; sum_td += c01_p::aa_via_typedef[k].x; end
   ck("typedef'd foreach count", n, 2);
   ck("typedef'd foreach sum",   sum_td, 30);

   ck("direct num()", c01_p::aa_direct.num(), 2);
   n = 0; sum_dir = 0;
   foreach (c01_p::aa_direct[k]) begin n++; sum_dir += c01_p::aa_direct[k].x; end
   ck("direct foreach count", n, 2);
   ck("direct foreach sum",   sum_dir, 70);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T02 - c02: CONTROL, same typedef'd AA shape at MODULE scope.
// -----------------------------------------------------------------------------
task automatic t02_module_aa_typedef();
   m_elem_t e;
   int n, s;
   case_begin(2, "c02 CONTROL module-scope typedef'd AA");
   e.x = 5;  t02_aa_td["a"] = e;
   e.x = 7;  t02_aa_td["b"] = e;
   e.x = 9;  t02_aa_dir["a"] = e;

   ck("module typedef'd num()", t02_aa_td.num(), 2);
   n = 0; s = 0;
   foreach (t02_aa_td[k]) begin n++; s += t02_aa_td[k].x; end
   ck("module typedef'd foreach count", n, 2);
   ck("module typedef'd foreach sum",   s, 12);

   ck("module direct num()", t02_aa_dir.num(), 1);
   n = 0; s = 0;
   foreach (t02_aa_dir[k]) begin n++; s += t02_aa_dir[k].x; end
   ck("module direct foreach count", n, 1);
   ck("module direct foreach sum",   s, 9);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T03 - c03: CONTROL, typedef'd AA as a CLASS PROPERTY (+ foreach in a method).
// -----------------------------------------------------------------------------
task automatic t03_class_member_aa();
   c03_bag b;
   case_begin(3, "c03 CONTROL class property of an AA typedef");
   b = new();
   b.fill();
   ck("class typedef'd num()",       b.td_num(),    2);
   ck("class typedef'd foreach count", b.td_count(), 2);
   ck("class typedef'd foreach sum",  b.td_sum(),   33);
   ck("class direct num()",           b.dir_num(),  1);
   ck("class direct foreach count",   b.dir_count(), 1);
   ck("class direct foreach sum",     b.dir_sum(),  33);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T04 - c04: package-scope typedef'd QUEUE.
// -----------------------------------------------------------------------------
task automatic t04_pkg_queue_typedef();
   int n, s;
   case_begin(4, "c04 package typedef'd QUEUE [$]");
   c04_p::q_via_typedef.push_back(1);
   c04_p::q_via_typedef.push_back(2);
   c04_p::q_via_typedef.push_back(3);
   c04_p::q_direct.push_back(10);
   c04_p::q_direct.push_back(20);

   ck("typedef'd queue size", c04_p::q_via_typedef.size(), 3);
   n = 0; s = 0;
   foreach (c04_p::q_via_typedef[i]) begin n++; s += c04_p::q_via_typedef[i]; end
   ck("typedef'd queue foreach count", n, 3);
   ck("typedef'd queue foreach sum",   s, 6);

   ck("direct queue size", c04_p::q_direct.size(), 2);
   n = 0; s = 0;
   foreach (c04_p::q_direct[i]) begin n++; s += c04_p::q_direct[i]; end
   ck("direct queue foreach count", n, 2);
   ck("direct queue foreach sum",   s, 30);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T05 - c05: package-scope typedef'd DYNAMIC ARRAY.
// -----------------------------------------------------------------------------
task automatic t05_pkg_dynarray_typedef();
   int n, s;
   case_begin(5, "c05 package typedef'd DYNAMIC ARRAY []");
   c05_p::d_via_typedef = new[4];
   foreach (c05_p::d_via_typedef[i]) c05_p::d_via_typedef[i] = i + 1;
   c05_p::d_direct = new[2];
   foreach (c05_p::d_direct[i]) c05_p::d_direct[i] = (i + 1) * 10;

   ck("typedef'd dyn size", c05_p::d_via_typedef.size(), 4);
   n = 0; s = 0;
   foreach (c05_p::d_via_typedef[i]) begin n++; s += c05_p::d_via_typedef[i]; end
   ck("typedef'd dyn foreach count", n, 4);
   ck("typedef'd dyn foreach sum",   s, 10);

   ck("direct dyn size", c05_p::d_direct.size(), 2);
   n = 0; s = 0;
   foreach (c05_p::d_direct[i]) begin n++; s += c05_p::d_direct[i]; end
   ck("direct dyn foreach count", n, 2);
   ck("direct dyn foreach sum",   s, 30);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T06 - c06: package-scope typedef'd FIXED ARRAY.
// -----------------------------------------------------------------------------
task automatic t06_pkg_fixedarray_typedef();
   int n, s;
   case_begin(6, "c06 package typedef'd FIXED ARRAY [0:3]");
   foreach (c06_p::f_via_typedef[i]) c06_p::f_via_typedef[i] = i + 1;
   foreach (c06_p::f_direct[i])      c06_p::f_direct[i]      = (i + 1) * 10;

   ck("typedef'd fixed [2]", c06_p::f_via_typedef[2], 3);
   n = 0; s = 0;
   foreach (c06_p::f_via_typedef[i]) begin n++; s += c06_p::f_via_typedef[i]; end
   ck("typedef'd fixed foreach count", n, 4);
   ck("typedef'd fixed foreach sum",   s, 10);

   ck("direct fixed [2]", c06_p::f_direct[2], 30);
   n = 0; s = 0;
   foreach (c06_p::f_direct[i]) begin n++; s += c06_p::f_direct[i]; end
   ck("direct fixed foreach count", n, 4);
   ck("direct fixed foreach sum",   s, 100);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T07 - c07: CHAINED (2-hop) typedef at module scope.
// -----------------------------------------------------------------------------
task automatic t07_chained_typedef_module();
   int n, s;
   case_begin(7, "c07 chained 2-hop typedef, fixed array, module scope");
   foreach (t07_v_chained[i]) t07_v_chained[i] = i + 1;
   foreach (t07_v_onehop[i])  t07_v_onehop[i]  = i + 1;
   foreach (t07_v_direct[i])  t07_v_direct[i]  = i + 1;

   ck("chained [2]", t07_v_chained[2], 3);
   n = 0; s = 0;
   foreach (t07_v_chained[i]) begin n++; s += t07_v_chained[i]; end
   ck("chained foreach count", n, 4);
   ck("chained foreach sum",   s, 10);

   n = 0; s = 0;
   foreach (t07_v_onehop[i]) begin n++; s += t07_v_onehop[i]; end
   ck("onehop foreach count", n, 4);
   ck("onehop foreach sum",   s, 10);

   n = 0; s = 0;
   foreach (t07_v_direct[i]) begin n++; s += t07_v_direct[i]; end
   ck("direct foreach count", n, 4);
   ck("direct foreach sum",   s, 10);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T08 - c08: CHAINED + cross-package typedef, package-scope variables.
// -----------------------------------------------------------------------------
task automatic t08_chained_crosspkg();
   int n;
   case_begin(8, "c08 chained + cross-package typedef, package scope");
   ck("crosspkg 1hop num()", c08_use_p::aa_crosspkg.num(), 0);
   n = 0; foreach (c08_use_p::aa_crosspkg[k]) n++;
   ck("crosspkg 1hop foreach count", n, 0);

   ck("crosspkg 2hop num()", c08_use_p::aa_crosspkg_2hop.num(), 0);
   n = 0; foreach (c08_use_p::aa_crosspkg_2hop[k]) n++;
   ck("crosspkg 2hop foreach count", n, 0);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T09 - c09: CONTROL, typedef in a package, VARIABLE at module scope.
// -----------------------------------------------------------------------------
task automatic t09_pkgtypedef_var_at_module();
   c09_p::elem_t e;
   int n, s;
   case_begin(9, "c09 CONTROL pkg typedef, module-scope var");
   e.x = 4;  t09_aa["p"] = e;
   e.x = 6;  t09_aa["q"] = e;

   ck("pkg-typedef var num()", t09_aa.num(), 2);
   n = 0; s = 0;
   foreach (t09_aa[k]) begin n++; s += t09_aa[k].x; end
   ck("pkg-typedef var foreach count", n, 2);
   ck("pkg-typedef var foreach sum",   s, 10);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T10 - c10: typedef'd AA as a STRUCT FIELD of a package-scope struct.
// -----------------------------------------------------------------------------
task automatic t10_struct_member_aa();
   c10_p::holder_t h;
   c10_p::elem_t   e;
   int n, s;
   case_begin(10, "c10 AA typedef as struct field (package struct)");
   h.tag = 1;
   e.x = 2;  h.m_td["a"] = e;
   e.x = 3;  h.m_td["b"] = e;
   e.x = 4;  h.m_dir["a"] = e;

   ck("struct field typedef'd num()", h.m_td.num(), 2);
   n = 0; s = 0;
   foreach (h.m_td[k]) begin n++; s += h.m_td[k].x; end
   ck("struct field typedef'd foreach count", n, 2);
   ck("struct field typedef'd foreach sum",   s, 5);

   ck("struct field direct num()", h.m_dir.num(), 1);
   n = 0; s = 0;
   foreach (h.m_dir[k]) begin n++; s += h.m_dir[k].x; end
   ck("struct field direct foreach count", n, 1);
   ck("struct field direct foreach sum",   s, 4);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T11 - c11: CONTROL, typedef'd AA as a ref formal and as a subroutine local.
// -----------------------------------------------------------------------------
task automatic t11_subr_arg_and_local();
   c11_p::aa_t   m;
   c11_p::elem_t e;
   case_begin(11, "c11 CONTROL AA typedef as ref arg + local");
   e.x = 1;  m["a"] = e;
   e.x = 2;  m["b"] = e;
   e.x = 3;  m["c"] = e;

   ck("arg num()",                  m.num(),        3);
   ck("arg foreach via fn",         t11_count_td(m), 3);
   ck("arg sum via fn",             t11_sum_td(m),   6);
   ck("local typedef'd AA foreach", t11_count_local(), 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T12 - c12: AA of AA, both levels via typedef, nested foreach.
// -----------------------------------------------------------------------------
task automatic t12_aa_of_aa_typedef();
   int n, s;
   case_begin(12, "c12 AA of AA, both levels typedef'd");
   c12_p::nested_td[1]["a"] = 10;
   c12_p::nested_td[1]["b"] = 20;
   c12_p::nested_td[2]["c"] = 30;
   c12_p::nested_dir[1]["a"] = 10;
   c12_p::nested_dir[1]["b"] = 20;
   c12_p::nested_dir[2]["c"] = 30;

   ck("nested typedef outer num()", c12_p::nested_td.num(), 2);
   n = 0; s = 0;
   foreach (c12_p::nested_td[i, j]) begin n++; s += c12_p::nested_td[i][j]; end
   ck("nested typedef foreach count", n, 3);
   ck("nested typedef foreach sum",   s, 60);

   ck("nested direct outer num()", c12_p::nested_dir.num(), 2);
   n = 0; s = 0;
   foreach (c12_p::nested_dir[i, j]) begin n++; s += c12_p::nested_dir[i][j]; end
   ck("nested direct foreach count", n, 3);
   ck("nested direct foreach sum",   s, 60);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T13 - c13: the whole AA built-in API on a package-scope typedef'd AA.
// -----------------------------------------------------------------------------
task automatic t13_aa_api_on_typedef();
   int idx;
   string k;
   case_begin(13, "c13 whole AA built-in API on a typedef");
   c13_p::td["a"] = 1;  c13_p::td["b"] = 2;
   c13_p::dir["a"] = 1; c13_p::dir["b"] = 2;

   ck("td num()",       c13_p::td.num(), 2);
   ck("td size()",      c13_p::td.size(), 2);
   ck("td exists(a)",   c13_p::td.exists("a"), 1);
   ck("td exists(zz)",  c13_p::td.exists("zz"), 0);
   ck("td read a",      c13_p::td["a"], 1);

   idx = c13_p::td.first(k);
   ck("td first() ok", idx, 1);
   if (idx) ck("td first() key non-empty", (k.len() > 0) ? 1 : 0, 1);

   begin
     int tmp [string];
     tmp = c13_p::dir;
     ck("copy from direct: num()",   tmp.num(), 2);
     ck("copy from direct: read b",  tmp["b"], 2);
   end
   begin
     c13_p::aa_t cp;
     cp = c13_p::td;
     ck("copy of typedef'd: num()",  cp.num(), 2);
     ck("copy of typedef'd: read b", cp["b"], 2);
   end

   c13_p::td.delete();
   ck("td after delete num()", c13_p::td.num(), 0);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T14 - c14: 2-D AA typedef with TWO foreach indices + int-keyed AA typedef.
// -----------------------------------------------------------------------------
task automatic t14_2d_and_intkey();
   int n, s, ksum;
   case_begin(14, "c14 2-D AA typedef + int-key AA typedef");
   c14_p::m_td[7]["a"] = 1;  c14_p::m_td[7]["b"] = 2;  c14_p::m_td[9]["c"] = 3;
   n = 0; s = 0;
   foreach (c14_p::m_td[i, j]) begin n++; s += c14_p::m_td[i][j]; end
   ck("2-D typedef foreach count", n, 3);
   ck("2-D typedef foreach sum",   s, 6);

   c14_p::ai_td[10] = 100;  c14_p::ai_td[20] = 200;  c14_p::ai_td[30] = 300;
   c14_p::ai_dir[10] = 100; c14_p::ai_dir[20] = 200; c14_p::ai_dir[30] = 300;

   n = 0; ksum = 0;
   foreach (c14_p::ai_td[k]) begin n++; ksum += k; end
   ck("int-key typedef foreach count",  n, 3);
   ck("int-key typedef foreach keysum", ksum, 60);

   n = 0; ksum = 0;
   foreach (c14_p::ai_dir[k]) begin n++; ksum += k; end
   ck("int-key direct foreach count",  n, 3);
   ck("int-key direct foreach keysum", ksum, 60);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T15 - c15: CONTROL, typedef'd AA declared at INTERFACE scope.
// -----------------------------------------------------------------------------
task automatic t15_interface_aa();
   c15_p::elem_t e;
   int n, s;
   case_begin(15, "c15 CONTROL AA typedef at interface scope");
   e.x = 3;  u_if.aa_td["a"] = e;
   e.x = 4;  u_if.aa_td["b"] = e;
   e.x = 5;  u_if.aa_dir["a"] = e;

   ck("if typedef'd num()", u_if.aa_td.num(), 2);
   n = 0; s = 0;
   foreach (u_if.aa_td[k]) begin n++; s += u_if.aa_td[k].x; end
   ck("if typedef'd foreach count", n, 2);
   ck("if typedef'd foreach sum",   s, 7);

   ck("if direct num()", u_if.aa_dir.num(), 1);
   n = 0; s = 0;
   foreach (u_if.aa_dir[k]) begin n++; s += u_if.aa_dir[k].x; end
   ck("if direct foreach count", n, 1);
   ck("if direct foreach sum",   s, 5);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T16 - c16: foreach from INSIDE a package subroutine + class static member.
// -----------------------------------------------------------------------------
task automatic t16_pkg_subr_and_class_static();
   c16_p::elem_t e;
   case_begin(16, "c16 foreach inside a package subroutine + class static");
   e.x = 12; c16_p::tbl["m"] = e;
   e.x = 30; c16_p::tbl["n"] = e;
   ck("pkg-internal foreach count", c16_p::count_all(), 2);
   ck("pkg-internal foreach sum",   c16_p::sum_all(), 42);

   e.x = 1;  c16_static_holder::s_reg["p"] = e;
   e.x = 2;  c16_static_holder::s_reg["q"] = e;
   ck("class static foreach count", c16_static_holder::count_static(), 2);
   ck("class static num()",         c16_static_holder::s_reg.num(), 2);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T17 - c17: 2-D AA foreach with NO typedef (independent bug, both scopes).
// -----------------------------------------------------------------------------
task automatic t17_2d_aa_foreach_no_typedef();
   int n, s;
   case_begin(17, "c17 2-D AA foreach, NO typedef (module + package scope)");
   c17_pkg_p::m_pkg[1]["a"] = 10; c17_pkg_p::m_pkg[1]["b"] = 20;
   c17_pkg_p::m_pkg[2]["c"] = 30;
   t17_m_mod[1]["a"] = 10; t17_m_mod[1]["b"] = 20; t17_m_mod[2]["c"] = 30;

   n = 0; s = 0;
   foreach (t17_m_mod[i, j]) begin n++; s += t17_m_mod[i][j]; end
   ck("module 2-D AA foreach count", n, 3);
   ck("module 2-D AA foreach sum",   s, 60);

   n = 0; s = 0;
   foreach (c17_pkg_p::m_pkg[i, j]) begin n++; s += c17_pkg_p::m_pkg[i][j]; end
   ck("package 2-D AA foreach count", n, 3);
   ck("package 2-D AA foreach sum",   s, 60);

   n = 0; foreach (t17_m_mod[i]) n++;
   ck("module 2-D AA outer-only count", n, 2);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T18 - c18: element-type sensitivity of the package typedef'd AA bug.
// -----------------------------------------------------------------------------
task automatic t18_element_type_sensitivity();
   c18_p::elem_t e;
   case_begin(18, "c18 element-type sensitivity of the typedef'd AA bug");
   c18_p::a_int["k"]    = 42;
   e.x = 7; c18_p::a_struct["k"] = e;
   c18_p::a_str["k"]    = "hello";
   c18_p::a_bits["k"]   = 8'hAB;
   c18_p::d_int["k"]    = 42;
   e.x = 7; c18_p::d_struct["k"] = e;

   ck("int-elem    typedef num()",  c18_p::a_int.num(), 1);
   ck("int-elem    typedef read",   c18_p::a_int["k"], 42);
   ck("struct-elem typedef num()",  c18_p::a_struct.num(), 1);
   ck("struct-elem typedef read x", c18_p::a_struct["k"].x, 7);
   ck("string-elem typedef num()",  c18_p::a_str.num(), 1);
   ck("bits-elem   typedef num()",  c18_p::a_bits.num(), 1);
   ck("int-elem    direct num()",   c18_p::d_int.num(), 1);
   ck("struct-elem direct num()",   c18_p::d_struct.num(), 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T19 - c19: MECHANISM probe — bogus foreach count tracks element width.
// -----------------------------------------------------------------------------
task automatic t19_bogus_count_tracks_width();
   int n;
   case_begin(19, "c19 MECHANISM: bogus foreach count == element width");
   n = 0; foreach (c19_p::a4[k])   n++;  obs("elem width 4",   n);
   n = 0; foreach (c19_p::a32[k])  n++;  obs("elem width 32",  n);
   n = 0; foreach (c19_p::a64[k])  n++;  obs("elem width 64",  n);
   n = 0; foreach (c19_p::a128[k]) n++;  obs("elem width 128", n);
   n = 0; foreach (c19_p::ctrl[k]) n++;  obs("direct control", n);

   // the trip count must equal num() == 0 for every empty AA (LRM 12.7.3)
   n = 0; foreach (c19_p::a4[k])   n++;  ck("empty w4 AA foreach count",   n, 0);
   n = 0; foreach (c19_p::a32[k])  n++;  ck("empty w32 AA foreach count",  n, 0);
   n = 0; foreach (c19_p::a64[k])  n++;  ck("empty w64 AA foreach count",  n, 0);
   n = 0; foreach (c19_p::a128[k]) n++;  ck("empty w128 AA foreach count", n, 0);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T20 - c20: query functions and reduction methods on typedef'd collections.
// -----------------------------------------------------------------------------
task automatic t20_queries_and_reductions();
   case_begin(20, "c20 .num()/.sum()/.min()/.max()/$size on typedef'd collections");
   c20_p::aa["a"] = 5; c20_p::aa["b"] = 15;
   c20_p::aa_dir["a"] = 5; c20_p::aa_dir["b"] = 15;
   c20_p::aai[1] = 5; c20_p::aai[2] = 15;
   c20_p::aai_dir[1] = 5; c20_p::aai_dir[2] = 15;
   c20_p::q.push_back(1); c20_p::q.push_back(2);
   c20_p::q_dir.push_back(1); c20_p::q_dir.push_back(2);

   ck("AA typedef .num()",    c20_p::aa.num(), 2);
   ck("AA direct .num()",     c20_p::aa_dir.num(), 2);
   ck("AA typedef .sum()",    c20_p::aa.sum(), 20);
   ck("AA direct .sum()",     c20_p::aa_dir.sum(), 20);
   // .min()/.max() are 7.12.1 ARRAY-QUERY methods: legal on queues, NOT on
   // associative arrays (xrun TYCMPAT), and they RETURN A QUEUE.
   ck("queue typedef .min()", c20_p::q.min()[0], 1);
   ck("queue typedef .max()", c20_p::q.max()[0], 2);
   ck("AA typedef .exists()",  c20_p::aa.exists("a"), 1);
   ck("AA direct .exists()",   c20_p::aa_dir.exists("b"), 1);
   ck("queue typedef .sum()", c20_p::q.sum(), 3);
   ck("queue direct .sum()",  c20_p::q_dir.sum(), 3);
   ck("queue typedef .size()", c20_p::q.size(), 2);

   // $size on a NON-INTEGRAL-index AA is rejected by the reference (AANINT)
   ck("$size AA typedef",  $size(c20_p::aai), 2);
   ck("$size AA direct",   $size(c20_p::aai_dir), 2);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T21 - c21: typedef'd KEY type (string key + narrow typed key).
// -----------------------------------------------------------------------------
task automatic t21_typedef_key_type();
   int n, s;
   case_begin(21, "c21 typedef'd KEY type (string + narrow 4-bit key)");
   c21_p::a_strkey["one"]   = 1;
   c21_p::a_strkey["two"]   = 2;
   c21_p::dir_strkey["one"] = 1;
   c21_p::dir_strkey["two"] = 2;

   ck("typedef-key num()",  c21_p::a_strkey.num(), 2);
   ck("typedef-key read",   c21_p::a_strkey["one"], 1);
   n = 0; s = 0;
   foreach (c21_p::a_strkey[k]) begin n++; s += c21_p::a_strkey[k]; end
   ck("typedef-key foreach count", n, 2);
   ck("typedef-key foreach sum",   s, 3);

   n = 0; s = 0;
   foreach (c21_p::dir_strkey[k]) begin n++; s += c21_p::dir_strkey[k]; end
   ck("direct-key foreach count", n, 2);
   ck("direct-key foreach sum",   s, 3);

   // narrow typed key: index var must come back narrowed, not 32-bit
   c21_p::a_nibkey[3] = 30;  c21_p::a_nibkey[5] = 50;
   c21_p::dir_nibkey[3] = 30; c21_p::dir_nibkey[5] = 50;
   n = 0; s = 0;
   foreach (c21_p::a_nibkey[k]) begin n++; s += k; end
   ck("nib-key typedef foreach count", n, 2);
   ck("nib-key typedef keysum",        s, 8);
   n = 0; s = 0;
   foreach (c21_p::dir_nibkey[k]) begin n++; s += k; end
   ck("nib-key direct foreach count", n, 2);
   ck("nib-key direct keysum",        s, 8);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T22 - c22: assignment-pattern initializers on a package-scope typedef'd AA.
// -----------------------------------------------------------------------------
task automatic t22_pkg_aa_initializers();
   case_begin(22, "c22 '{...} initializers + whole-AA copy on a typedef'd AA");
   ck("pattern init num()",     c22_p::with_pattern.num(), 2);
   ck("pattern init read a",    c22_p::with_pattern["a"], 1);
   ck("pattern init read b",    c22_p::with_pattern["b"], 2);
   ck("default init num()",     c22_p::with_default.num(), 1);
   ck("default init read c",    c22_p::with_default["c"], 7);
   ck("plain num()",            c22_p::plain.num(), 0);
   ck("control pattern num()",  c22_p::dir_pattern.num(), 2);
   ck("control pattern read a", c22_p::dir_pattern["a"], 1);

   c22_p::plain["z"] = 9;
   c22_p::aliased = c22_p::plain;
   ck("alias num()",  c22_p::aliased.num(), 1);
   ck("alias read z", c22_p::aliased["z"], 9);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T23 - c23: CONTROL, typedef'd AA at module scope and as a CHILD-INSTANCE
// variable (the AA-as-port form is illegal SV - see the c23_child note).
// -----------------------------------------------------------------------------
task automatic t23_port_and_child_instance();
   c23_p::elem_t e;
   int n;
   case_begin(23, "c23 CONTROL module-scope AA typedef var + child-instance var");
   e.x = 1; t23_drv["a"] = e;
   e.x = 2; t23_drv["b"] = e;

   ck("driver num()",         t23_drv.num(), 2);
   n = 0; foreach (t23_drv[k]) n++;
   ck("driver foreach count", n, 2);

   t23_go = 1;                             // release the child probe
   $fflush;
   wait (t23_done == 1);
   ck("child local foreach count", t23_n_local, 2);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T24 - c24: typedef-dims PEEL DEPTH for AA typedefs at module scope.
// -----------------------------------------------------------------------------
task automatic t24_typedef_peel_depth();
   c24_p::elem_t e;
   int n, s;
   case_begin(24, "c24 1/2/3-hop typedef peel depth, module scope");
   e.x = 1; t24_a1["k"] = e;
   e.x = 2; t24_a2["k"] = e;
   e.x = 3; t24_a3["k"] = e;

   ck("1-hop num()", t24_a1.num(), 1);
   ck("2-hop num()", t24_a2.num(), 1);
   ck("3-hop num()", t24_a3.num(), 1);

   n = 0; s = 0; foreach (t24_a1[k]) begin n++; s += t24_a1[k].x; end
   ck("1-hop foreach count", n, 1);
   ck("1-hop foreach sum",   s, 1);
   n = 0; s = 0; foreach (t24_a2[k]) begin n++; s += t24_a2[k].x; end
   ck("2-hop foreach count", n, 1);
   ck("2-hop foreach sum",   s, 2);
   n = 0; s = 0; foreach (t24_a3[k]) begin n++; s += t24_a3[k].x; end
   ck("3-hop foreach count", n, 1);
   ck("3-hop foreach sum",   s, 3);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T25 - c25: AA typedef as a struct FIELD declared at FILE scope.
// -----------------------------------------------------------------------------
task automatic t25_filescope_struct_aa_field();
   c25_holder_t h;
   c25_elem_t   e;
   int n, s;
   case_begin(25, "c25 AA typedef as struct field (file-scope struct)");
   h.tag = 1;
   e.x = 2;  h.m_td["a"] = e;
   e.x = 3;  h.m_td["b"] = e;
   e.x = 4;  h.m_dir["a"] = e;

   ck("file-scope struct field typedef'd num()", h.m_td.num(), 2);
   n = 0; s = 0;
   foreach (h.m_td[k]) begin n++; s += h.m_td[k].x; end
   ck("file-scope struct field typedef'd foreach count", n, 2);
   ck("file-scope struct field typedef'd foreach sum",   s, 5);

   ck("file-scope struct field direct num()", h.m_dir.num(), 1);
   n = 0; s = 0;
   foreach (h.m_dir[k]) begin n++; s += h.m_dir[k].x; end
   ck("file-scope struct field direct foreach count", n, 1);
   ck("file-scope struct field direct foreach sum",   s, 4);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T26 - p1: 2-hop typedef'd AA as a LOCAL in a plain (non-class) function.
// -----------------------------------------------------------------------------
task automatic t26_p1_2hop_local_fn();
   case_begin(26, "p1 2-hop typedef'd AA local in a plain function");
   ck("1hop local foreach", t26_count_local_1hop(), 1);
   ck("2hop local foreach", t26_count_local(),      1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T27 - p2: 2-hop typedef'd AA as a LOCAL in a CLASS METHOD.
// -----------------------------------------------------------------------------
task automatic t27_p2_2hop_local_method();
   p2_c c = new();
   case_begin(27, "p2 2-hop typedef'd AA local in a class method");
   ck("method 2hop local foreach", c.count_local(), 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T28 - p3: function RETURN type is a typedef'd AA (1 hop).
// -----------------------------------------------------------------------------
task automatic t28_p3_fn_return_aa();
   p3_p::aa_t   v;
   p3_p::elem_t e;
   int n, s;
   case_begin(28, "p3 function RETURN type is a typedef'd AA");
   e.x = 5; v["a"] = e;            // control: direct local AA
   v = t28_make();
   ck("return num()", v.num(), 2);
   n = 0; s = 0;
   foreach (v[k]) begin n++; s += v[k].x; end
   ck("return foreach count", n, 2);
   ck("return foreach sum",   s, 3);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T29 - p4: input (by-value) FORMAL of typedef'd AA type, 1 hop.
// -----------------------------------------------------------------------------
task automatic t29_p4_input_formal_aa();
   p4_p::aa_t   m;
   p4_p::elem_t e;
   case_begin(29, "p4 input (by-value) formal of typedef'd AA type");
   e.x = 1; m["a"] = e;
   e.x = 2; m["b"] = e;
   ck("ref formal foreach",   t29_count_ref(m), 2);
   ck("input formal foreach", t29_count_val(m), 2);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T30 - p5: typedef'd AA declared in a NAMED begin/end block (1-hop, 2-hop).
// -----------------------------------------------------------------------------
task automatic t30_p5_block_local();
   p5_p::elem_t e;
   int n1, n2;
   case_begin(30, "p5 typedef'd AA declared in a named begin/end block");
   e.x = 1;
   begin : p5_blk1
     p5_p::aa_t loc;               // 1-hop block local
     loc["a"] = e;
     n1 = 0;
     foreach (loc[k]) n1++;
   end
   begin : p5_blk2
     p5_alias_t loc;               // 2-hop block local
     loc["a"] = e;
     n2 = 0;
     foreach (loc[k]) n2++;
   end
   ck("block 1hop foreach", n1, 1);
   ck("block 2hop foreach", n2, 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T31 - p6: typedef'd AA declared in a fork/join block (1-hop, 2-hop).
// -----------------------------------------------------------------------------
task automatic t31_p6_fork_local();
   p6_p::elem_t e;
   int n1, n2;
   case_begin(31, "p6 typedef'd AA declared in a fork/join block");
   e.x = 1;
   fork
     begin : p6_f1
       p6_p::aa_t loc;             // 1-hop fork local
       loc["a"] = e;
       n1 = 0;
       foreach (loc[k]) n1++;
     end
     begin : p6_f2
       p6_alias_t loc;             // 2-hop fork local
       loc["a"] = e;
       n2 = 0;
       foreach (loc[k]) n2++;
     end
   join
   ck("fork 1hop foreach", n1, 1);
   ck("fork 2hop foreach", n2, 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T32 - p7: 2-hop typedef'd AA at INTERFACE scope (c15 covered only 1 hop).
// -----------------------------------------------------------------------------
task automatic t32_p7_iface_2hop();
   p7_p::elem_t e;
   int n, s;
   case_begin(32, "p7 2-hop typedef'd AA at interface scope");
   e.x = 3;  p32_u_if.aa_td1["a"] = e;
   e.x = 4;  p32_u_if.aa_td2["a"] = e;
   e.x = 5;  p32_u_if.aa_dir["a"] = e;
   ck("if 1hop num()", p32_u_if.aa_td1.num(), 1);
   ck("if 2hop num()", p32_u_if.aa_td2.num(), 1);
   n = 0; s = 0; foreach (p32_u_if.aa_td1[k]) begin n++; s += p32_u_if.aa_td1[k].x; end
   ck("if 1hop foreach count", n, 1);
   n = 0; s = 0; foreach (p32_u_if.aa_td2[k]) begin n++; s += p32_u_if.aa_td2[k].x; end
   ck("if 2hop foreach count", n, 1);
   ck("if 2hop foreach sum",   s, 4);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T33 - p8: STATIC local of typedef'd AA type in a task (1-hop, 2-hop).
// -----------------------------------------------------------------------------
task automatic t33_p8_static_local();
   int n;
   case_begin(33, "p8 STATIC local of typedef'd AA type in a task");
   t33_fill_and_count1(n);
   ck("static 1hop foreach", n, 1);
   t33_fill_and_count2(n);
   ck("static 2hop foreach", n, 1);
   case_end();
endtask


// -----------------------------------------------------------------------------
// T34 - p9: package-scope typedef'd AA reached through a WILDCARD import.
// -----------------------------------------------------------------------------
task automatic t34_p9_wildcard_import();
   p9_p::elem_t e;
   int n;
   case_begin(34, "p9 package typedef'd AA via WILDCARD import (bare name)");
   e.x = 1; aa_via_typedef["a"] = e;
   e.x = 2; aa_direct["a"] = e;
   ck("wildcard typedef'd num()", aa_via_typedef.num(), 1);
   n = 0; foreach (aa_via_typedef[k]) n++;
   ck("wildcard typedef'd foreach", n, 1);
   n = 0; foreach (aa_direct[k]) n++;
   ck("wildcard direct foreach", n, 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T35 - p10: 2-hop typedef'd QUEUE and DYNAMIC ARRAY as task locals.
// -----------------------------------------------------------------------------
task automatic t35_p10_2hop_q_dyn_local();
   case_begin(35, "p10 2-hop typedef'd QUEUE and DYNAMIC ARRAY as task locals");
   ck("1hop queue local foreach", t35_count_q1(), 1);
   ck("2hop queue local foreach", t35_count_q(),  1);
   ck("1hop dyn local foreach",   t35_count_d1(), 2);
   ck("2hop dyn local foreach",   t35_count_d(),  2);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T36 - p11: class PROPERTY with a 2-hop typedef'd AA (module-scope typedefs).
// -----------------------------------------------------------------------------
task automatic t36_p11_class_prop_2hop();
   p11_c       c = new();
   p11_p::elem_t e;
   case_begin(36, "p11 class PROPERTY with a 2-hop typedef'd AA");
   e.x = 1; c.m_td1["a"] = e;
   e.x = 2; c.m_td2["a"] = e;
   ck("prop 1hop num()", c.m_td1.num(), 1);
   ck("prop 2hop num()", c.m_td2.num(), 1);
   ck("prop 1hop foreach", c.count1(), 1);
   ck("prop 2hop foreach", c.count2(), 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T37 - p12: ref FORMAL with a 2-hop typedef'd AA type.
// -----------------------------------------------------------------------------
task automatic t37_p12_ref_formal_2hop();
   p12_alias_t  v2;
   p12_p::aa_t  v1;
   p12_p::elem_t e;
   case_begin(37, "p12 ref FORMAL with a 2-hop typedef'd AA type");
   e.x = 1; v1["a"] = e;
   e.x = 2; v2["a"] = e;
   ck("ref 1hop foreach", t37_count_ref1(v1), 1);
   ck("ref 2hop foreach", t37_count_ref2(v2), 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T38 - p13: typedef'd AA declared inside a GENERATE-FOR block (1-hop, 2-hop).
// The generate-block initial above is released via the p13_go/p13_done
// handshake (same pattern as T23's child instance).
// -----------------------------------------------------------------------------
task automatic t38_p13_generate_local();
   case_begin(38, "p13 typedef'd AA declared inside a generate-for block");
   p13_go = 1;
   $fflush;
   wait (p13_done == 1);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T39 - p14: OUTPUT formal of typedef'd AA type (task writes, caller foreaches).
// -----------------------------------------------------------------------------
task automatic t39_p14_output_formal_aa();
   p14_p::aa_t v;
   int n, s;
   case_begin(39, "p14 OUTPUT formal of typedef'd AA type");
   t39_make_aa(v);
   ck("output formal num()", v.num(), 2);
   n = 0; s = 0;
   foreach (v[k]) begin n++; s += v[k].x; end
   ck("output formal foreach count", n, 2);
   ck("output formal foreach sum",   s, 3);
   case_end();
endtask

// -----------------------------------------------------------------------------
// T40 - p15: package var with dims on BOTH the typedef and the declarator.
// -----------------------------------------------------------------------------
task automatic t40_p15_pkg_both_dims();
   int n, s;
   case_begin(40, "p15 package var with dims on BOTH typedef and declarator");
   p15_p::v[0][0] = 1; p15_p::v[0][3] = 2;
   p15_p::v[1][0] = 3; p15_p::v[1][3] = 4;
   p15_p::v_dir[0][0] = 1; p15_p::v_dir[0][3] = 2;
   p15_p::v_dir[1][0] = 3; p15_p::v_dir[1][3] = 4;
   ck("pkg both-dims [1][3]", p15_p::v[1][3], 4);
   n = 0; s = 0;
   foreach (p15_p::v[i, j]) begin n++; s += p15_p::v[i][j]; end
   ck("pkg both-dims foreach count", n, 8);
   ck("pkg both-dims foreach sum",   s, 10);
   n = 0; s = 0;
   foreach (p15_p::v_dir[i, j]) begin n++; s += p15_p::v_dir[i][j]; end
   ck("pkg direct 2D foreach count", n, 8);
   ck("pkg direct 2D foreach sum",   s, 10);
   case_end();
endtask


// =============================================================================
// Main : run all 41 cases in order (each isolated; one failure never blocks rest)
// =============================================================================
initial begin
   $display("=== tb_foreach_typedef_all : 41 foreach-over-typedef cases ===");

   t00_pkg_aa_empty();                t01_pkg_aa_populated();
   t02_module_aa_typedef();           t03_class_member_aa();
   t04_pkg_queue_typedef();           t05_pkg_dynarray_typedef();
   t06_pkg_fixedarray_typedef();      t07_chained_typedef_module();
   t08_chained_crosspkg();            t09_pkgtypedef_var_at_module();
   t10_struct_member_aa();            t11_subr_arg_and_local();
   t12_aa_of_aa_typedef();            t13_aa_api_on_typedef();
   t14_2d_and_intkey();               t15_interface_aa();
   t16_pkg_subr_and_class_static();   t17_2d_aa_foreach_no_typedef();
   t18_element_type_sensitivity();    t19_bogus_count_tracks_width();
   t20_queries_and_reductions();      t21_typedef_key_type();
   t22_pkg_aa_initializers();         t23_port_and_child_instance();
   t24_typedef_peel_depth();          t25_filescope_struct_aa_field();
   t26_p1_2hop_local_fn();            t27_p2_2hop_local_method();
   t28_p3_fn_return_aa();             t29_p4_input_formal_aa();
   t30_p5_block_local();              t31_p6_fork_local();
   t32_p7_iface_2hop();               t33_p8_static_local();
   t34_p9_wildcard_import();          t35_p10_2hop_q_dyn_local();
   t36_p11_class_prop_2hop();         t37_p12_ref_formal_2hop();
   t38_p13_generate_local();          t39_p14_output_formal_aa();
   t40_p15_pkg_both_dims();

   $display("=== tb_foreach_typedef_all : all %0d cases executed ===", 41);
   `SVTEST_PASSFAIL
   $finish;
end

endmodule : tb_foreach_typedef_all


