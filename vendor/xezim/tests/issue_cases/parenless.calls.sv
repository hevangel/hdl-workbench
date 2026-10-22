`timescale 1ns/1ps

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

// ============================================================================
// mwe_enum_parenless_svtest.sv — SVTEST-macro version of
// mwe_enum_parenless.sv (same coverage, tests/classes SVTEST_* check style):
// consolidated MWE for LRM §13.5 / §13.5.5 parenless ("optional argument
// list") zero-argument calls — enum methods AND the other user-defined /
// built-in forms.
//
// IEEE 1800 §13.5.5 / BNF `tf_call` ::= ps_or_hierarchical_tf_identifier
// [ ( list_of_arguments ) ] — the empty parentheses of a zero-argument call
// may be omitted, for user-defined functions/tasks AND built-in methods.
//
// PART 1 — enum methods (`e.first` is exactly `e.first()`).  xezim evaluates
// the WITH-parens form correctly in every context, but the PARENLESS form
// never reaches the enum-method machinery:
//
//   * module/initial/always scope: the parser yields a flat 2-segment hier
//     Ident (`Ident([e, first])`); the hier-read path only falls back to
//     `eval_builtin_method` for elaborated module signals, so a procedural
//     LOCAL reads as an unknown hierarchical name -> X.
//   * function/task/final/class-method bodies: the parser yields
//     `MemberAccess{Ident(e), first}`; `eval_expr_member_access` has no
//     enum-method arm, so it falls through to the object-property tail and
//     silently returns 0.
//   * `.name` is broken even for MODULE-scope variables (returns "").
//
// Real-testbench manifestation (a UVM coverage interface): initial-block
// locals walk `cenum = cenum.next;` inside `while(1)` with the only exit
// `if (cenum == cenum.last) break;` — X never equals X, so the loop never
// exits and the simulator hangs.
//
// PART 2 — the same §13.5.5 rule for NON-enum calls; these fail SILENTLY
// (return 0, or "" for strings):
//   A. bare free zero-arg function in EXPRESSION position (`r = getv;`) —
//      initial / function / task / class-method / final bodies;
//   B. package function parenless — imported bare (`pkf`) and qualified
//      (`vp::pkf`), at module scope and inside a function body;
//   C. string built-in methods `.len` / `.toupper` / `.tolower` (module and
//      local receivers);
//   D. class-property queue `pop_front` / `pop_back` inside a class method
//      (return 0 and do not pop; `.size` works);
//   E. interface function `u_ifn.ifn` (the with-parens form works).
//
// ============================================================================

package vp;
  typedef enum logic [5:0] { C0 = 6'd0, C1 = 6'd1, CC = 6'd63 } clients_enum_t;
  `SVTEST_INIT   // package-scope `failures` — shared by interface and module
  function automatic int pkf(); return 7; endfunction   // PART 2: package fn
endpackage

// The failing shape: an INTERFACE initial block (a coverage-interface walk).
interface cov_if;
  import vp::*;
  initial begin : iface_walk
    clients_enum_t cenum;
    int steps;
    // Bounded copy of the original `while(1) { if (c==c.last) break; c=c.next; }`
    cenum = cenum.first;
    steps = 0;
    while (steps < 10) begin
      if (cenum == cenum.last) break;   // X == X is X today -> never taken
      cenum = cenum.next;
      steps++;
    end
    `SVTEST_CHECK(steps < 10,
      $sformatf("IFACE_WALK: cenum never reached .last in 10 steps (hang shape)"))
    `SVTEST_CHECK(cenum.first === 6'd0,
      $sformatf("IFACE first=%0s (expect 0)", cenum.first))
    `SVTEST_CHECK(cenum.last === CC,
      $sformatf("IFACE last=%0s (expect 63)", cenum.last))
    `SVTEST_CHECK(cenum.num === 32'd3,
      $sformatf("IFACE num=%0s (expect 3)", cenum.num))
    `SVTEST_CHECK(cenum.name == "CC",
      $sformatf("IFACE name='%s' (expect CC)", cenum.name))
    // [control] the same calls WITH parens pass today:
    cenum = cenum.first();
    `SVTEST_CHECK(cenum === 6'd0 && cenum.last() === CC && cenum.num() === 32'd3,
      "IFACE [control] with-parens first/last/num")
  end
endinterface

// ----- PART 2 file-scope declarations (LRM §13.5.5 beyond enums) -----

// Free (module-scope) zero-arg functions.
function automatic int getv();  return 42; endfunction
function automatic int wrap();  return getv;  endfunction  // bare call in fn body
function automatic int wrapk(); return vp::pkf;   endfunction  // bare pkg fn in fn body
int vfun_runs = 0;
function automatic void vfun(); vfun_runs = vfun_runs + 1; endfunction

interface ifn_if;
  function int ifn(); return 3; endfunction
endinterface

class C;
  int cnt;
  function new(); cnt = 5; endfunction
  function int getcnt(); return cnt; endfunction
  function int wfree();  return getv; endfunction   // bare free fn in class method
  int cq[$];
  function void fill(); cq.push_back(3); cq.push_back(4); endfunction
  function int qsize();  return cq.size; endfunction       // [control] works
  function int qpopf();  return cq.pop_front; endfunction  // BROKEN
  function int qpopb();  return cq.pop_back; endfunction   // BROKEN
endclass

module tb_top;
  import vp::*;
  cov_if u_cov();

  clients_enum_t modvar;   // module-scope control variable

  // PART 2 module-scope declarations.
  int  r;
  string t;
  int  mq[$];          // module-scope queue      [control]
  int  md[];           // module-scope dyn array  [control]
  int  maa[int];       // module-scope assoc      [control]
  string ms = "hello"; // module-scope string
  C    c0 = new;       // module-scope class var
  ifn_if u_ifn();

  task automatic tk_wrap();      // bare free fn in TASK body
    r = getv;
    `SVTEST_CHECK(r === 42, $sformatf("TASK_BODY bare getv r=%0d (expect 42)", r))
  endtask

  // 1) Module initial-block local — every parenless method.
  initial begin
    clients_enum_t e;
    e = e.first;
    `SVTEST_CHECK(e === 6'd0, $sformatf("INIT first=%0s (expect 0)", e))
    e = e.last;
    `SVTEST_CHECK(e === CC, $sformatf("INIT last=%0s (expect 63)", e))
    e = C1;
    e = e.next;
    `SVTEST_CHECK(e === CC, $sformatf("INIT next=%0s (expect 63)", e))
    e = e.prev;
    `SVTEST_CHECK(e === C1, $sformatf("INIT prev=%0s (expect 1)", e))
    `SVTEST_CHECK(e.num === 32'd3, $sformatf("INIT num=%0s (expect 3)", e.num))
    e = CC;
    `SVTEST_CHECK(e.name == "CC", $sformatf("INIT name='%s' (expect CC)", e.name))
  end

  // 2) Always-block local.
  always begin
    clients_enum_t aloc;
    aloc = aloc.first;
    `SVTEST_CHECK(aloc === 6'd0, $sformatf("ALWAYS first=%0s (expect 0)", aloc))
    #10 $finish;   // one-shot: end the run after every t=0 process settled
  end

  // 3) Named block local.
  initial begin : nb
    clients_enum_t nloc;
    nloc = nloc.first;
    `SVTEST_CHECK(nloc === 6'd0, $sformatf("NAMEDBLK first=%0s (expect 0)", nloc))
  end

  // 4) Nested (unnamed) begin/end local.
  initial begin
    begin
      clients_enum_t dloc;
      dloc = dloc.first;
      `SVTEST_CHECK(dloc === 6'd0, $sformatf("NESTED first=%0s (expect 0)", dloc))
    end
  end

  // 5) For-loop-body local.
  initial begin
    for (int i = 0; i < 1; i++) begin
      clients_enum_t fi;
      fi = fi.first;
      `SVTEST_CHECK(fi === 6'd0, $sformatf("FORBODY first=%0s (expect 0)", fi))
    end
  end

  // 6) Fork-child local.
  initial begin
    fork
      begin
        clients_enum_t kloc;
        kloc = kloc.first;
        `SVTEST_CHECK(kloc === 6'd0, $sformatf("FORK first=%0s (expect 0)", kloc))
      end
    join
  end


  // 7) Expression contexts (if / case / $display argument), initial local.
  //    EXPR_IF keeps the original exit comparison (`cenum == cenum.last`);
  //    with the bug it evaluates to X (no branch taken), so the walk-shape
  //    detection above (IFACE_WALK) is what catches that form.
  initial begin
    clients_enum_t x;
    x = C0;
    if (x == x.last) begin      // broken: .last reads 0, 0==0 -> true
      failures++;
      $display("FAIL @%0t : EXPR_IF: x == x.last evaluated true (x=%0d)", $time, x);
    end
    `SVTEST_CHECK(x.next === C1, $sformatf("EXPR_CASE next=%0s (expect 1)", x.next))
    `SVTEST_CHECK(x.num === 32'd3, $sformatf("EXPR_DISP num=%0s (expect 3)", x.num))
  end

  // 8) Subroutine contexts — these fail SILENTLY (return 0, not X).
  function automatic void fn_local();
    clients_enum_t fl;
    fl = fl.last;
    `SVTEST_CHECK(fl === CC, $sformatf("FUNC_LOCAL last=%0s (expect 63)", fl))
  endfunction

  task automatic tk_local();
    clients_enum_t tl;
    tl = tl.last;
    `SVTEST_CHECK(tl === CC, $sformatf("TASK_LOCAL last=%0s (expect 63)", tl))
  endtask

  function automatic void fn_formal(clients_enum_t v);
    v = v.last;
    `SVTEST_CHECK(v === CC, $sformatf("FORMAL last=%0s (expect 63)", v))
  endfunction

  class Checker;
    function void mth();
      clients_enum_t cl;
      cl = cl.last;
      `SVTEST_CHECK(cl === CC, $sformatf("CLS_METHOD last=%0s (expect 63)", cl))
    endfunction
  endclass

  // 9) foreach loop KEY of an enum-keyed associative array.
  initial begin
    int aa[clients_enum_t];
    aa[C0] = 10;
    foreach (aa[k]) begin
      `SVTEST_CHECK(k.first === 6'd0, $sformatf("FOREACH_KEY first=%0s (expect 0)", k.first))
      `SVTEST_CHECK(k.last === CC, $sformatf("FOREACH_KEY last=%0s (expect 63)", k.last))
    end
  end

  // 10) Module-scope variable: .first/.last/.num work, but .name is broken
  //     even here (returns the empty string).
  initial begin
    modvar = modvar.first;
    `SVTEST_CHECK(modvar === 6'd0, $sformatf("MODSCOPE first=%0s (expect 0)", modvar))
    modvar = modvar.last;
    `SVTEST_CHECK(modvar === CC, $sformatf("MODSCOPE last=%0s (expect 63)", modvar))
    `SVTEST_CHECK(modvar.num === 32'd3, $sformatf("MODSCOPE num=%0s (expect 3)", modvar.num))
    modvar = CC;
    `SVTEST_CHECK(modvar.name == "CC", $sformatf("MODSCOPE name='%s' (expect CC)", modvar.name))
    // [control] with parens:
    `SVTEST_CHECK(modvar.name() == "CC", "MODSCOPE [control] with-parens name")
  end

  // 11) Anonymous-enum local in an initial block — [control]: works today
  //     via the enum_members[varname] fallback.
  initial begin
    enum { A0, A1, A2 } anon;
    anon = anon.first;
    `SVTEST_CHECK(anon === 0, $sformatf("ANON first=%0s (expect 0)", anon))
    `SVTEST_CHECK(anon.num === 32'd3, $sformatf("ANON num=%0s (expect 3)", anon.num))
  end

  // 12) PART 2 — LRM §13.5.5 beyond enum methods (see header): the same
  //     checks as the standalone mwe_parenless_1355.sv probe, in the same
  //     order (with-parens controls, then A-E, then known-good controls).
  initial begin
    // ---------------- [controls] with parens ----------------
    r = getv();   `SVTEST_CHECK(r === 42, $sformatf("CTRL getv() r=%0d", r))
    r = pkf();    `SVTEST_CHECK(r === 7,  $sformatf("CTRL pkf() r=%0d", r))
    r = ms.len(); `SVTEST_CHECK(r === 5,  $sformatf("CTRL ms.len() r=%0d", r))
    r = c0.getcnt(); `SVTEST_CHECK(r === 5, $sformatf("CTRL c0.getcnt() r=%0d", r))
    r = u_ifn.ifn(); `SVTEST_CHECK(r === 3, $sformatf("CTRL u_ifn.ifn() r=%0d", r))

    // ---------------- A. bare free function, parenless ----------------
    r = getv;     `SVTEST_CHECK(r === 42, $sformatf("INIT_BODY bare getv r=%0d (expect 42)", r))
    r = wrap();   `SVTEST_CHECK(r === 42, $sformatf("FN_BODY bare getv (via wrap) r=%0d (expect 42)", r))
    tk_wrap();
    r = c0.wfree(); `SVTEST_CHECK(r === 42, $sformatf("CLS_BODY bare getv r=%0d (expect 42)", r))

    // ---------------- B. package function, parenless ----------------
    r = pkf;      `SVTEST_CHECK(r === 7,  $sformatf("PKG_IMPORTED bare pkf r=%0d (expect 7)", r))
    r = vp::pkf;  `SVTEST_CHECK(r === 7,  $sformatf("PKG_QUALIFIED vp::pkf r=%0d (expect 7)", r))
    r = wrapk();  `SVTEST_CHECK(r === 7,  $sformatf("PKG_FN_BODY bare pkf (via wrapk) r=%0d (expect 7)", r))

    // ---------------- C. string methods, parenless ----------------
    r = ms.len;      `SVTEST_CHECK(r === 5,    $sformatf("STR_MOD len r=%0d (expect 5)", r))
    t = ms.toupper;  `SVTEST_CHECK(t == "HELLO", $sformatf("STR_MOD toupper t='%s' (expect HELLO)", t))
    begin
      string ls = "abcd";
      r = ls.len;    `SVTEST_CHECK(r === 4,    $sformatf("STR_LOC len r=%0d (expect 4)", r))
      t = ls.tolower; `SVTEST_CHECK(t == "abcd", $sformatf("STR_LOC tolower t='%s' (expect abcd)", t))
    end

    // ---------------- D. class-property queue methods, parenless ----------------
    c0.fill();
    r = c0.qsize();  `SVTEST_CHECK(r === 2,  $sformatf("CTRL cq.size r=%0d (expect 2)", r))
    r = c0.qpopf();  `SVTEST_CHECK(r === 3,  $sformatf("CLS_Q pop_front r=%0d (expect 3)", r))
    r = c0.qsize();  `SVTEST_CHECK(r === 1,  $sformatf("CLS_Q size_after_popf r=%0d (expect 1)", r))
    r = c0.qpopb();  `SVTEST_CHECK(r === 4,  $sformatf("CLS_Q pop_back r=%0d (expect 4)", r))
    r = c0.qsize();  `SVTEST_CHECK(r === 0,  $sformatf("CLS_Q size_after_popb r=%0d (expect 0)", r))

    // ---------------- E. interface function, parenless ----------------
    r = u_ifn.ifn;   `SVTEST_CHECK(r === 3,  $sformatf("IFACE_FN u_ifn.ifn r=%0d (expect 3)", r))

    // ---------------- [controls] known-good parenless forms ----------------
    vfun;                                    // void fn statement
    `SVTEST_CHECK(vfun_runs === 1, $sformatf("CTRL vfun; runs=%0d", vfun_runs))
    mq.push_back(11); mq.push_back(22);
    r = mq.pop_front; `SVTEST_CHECK(r === 11, $sformatf("CTRL mq.pop_front r=%0d", r))
    r = mq.sum;       `SVTEST_CHECK(r === 22, $sformatf("CTRL mq.sum r=%0d", r))
    begin
      int lq[$];
      lq.push_back(31); lq.push_back(32);
      r = lq.pop_front; `SVTEST_CHECK(r === 31, $sformatf("CTRL lq.pop_front r=%0d", r))
      r = lq.size;      `SVTEST_CHECK(r === 1,  $sformatf("CTRL lq.size r=%0d", r))
    end
    md = new[3];
    r = md.size;      `SVTEST_CHECK(r === 3,  $sformatf("CTRL md.size r=%0d", r))
    md.delete;
    r = md.size;      `SVTEST_CHECK(r === 0,  $sformatf("CTRL md.delete r=%0d", r))
    maa[5] = 1; maa[6] = 2; maa[7] = 3;
    r = maa.num;      `SVTEST_CHECK(r === 3,  $sformatf("CTRL maa.num r=%0d", r))
    begin
      int laa[string];
      laa["a"] = 1; laa["b"] = 2;
      r = laa.size;   `SVTEST_CHECK(r === 2,  $sformatf("CTRL laa.size r=%0d", r))
      laa.delete;
      r = laa.num;    `SVTEST_CHECK(r === 0,  $sformatf("CTRL laa.delete r=%0d", r))
    end
    r = c0.getcnt;    `SVTEST_CHECK(r === 5,  $sformatf("CTRL c0.getcnt r=%0d", r))
    begin
      C cl = new;
      r = cl.getcnt;  `SVTEST_CHECK(r === 5,  $sformatf("CTRL cl.getcnt r=%0d", r))
    end
    r = c0.cnt;       `SVTEST_CHECK(r === 5,  $sformatf("CTRL c0.cnt r=%0d", r))
  end

  // Driver: exercise the subroutine contexts.
  initial begin
    Checker c = new();
    fn_local();
    tk_local();
    fn_formal(C0);
    c.mth();
  end

  // Final-block local + verdict.  Finals run at $finish (declaration order),
  // so this is the last thing that prints.
  final begin
    clients_enum_t floc;
    floc = floc.last;
    `SVTEST_CHECK(floc === CC, $sformatf("FINAL_LOCAL last=%0s (expect 63)", floc))
    r = getv;
    `SVTEST_CHECK(r === 42, $sformatf("FINAL_BODY bare getv r=%0d (expect 42)", r))
    `SVTEST_PASSFAIL
  end
endmodule
