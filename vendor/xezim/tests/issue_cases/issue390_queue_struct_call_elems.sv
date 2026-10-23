`timescale 1ns/1ps

// ============================================================================
// mwe_i390 — xezim issue #390 (+ cousins: foreach formal, queue copy,
//                            multi-element, mixed sourcing)
//
// SYMPTOM (issue #390): a queue of structs whose elements are pushed as a
// FUNCTION'S STRUCT RETURN VALUE (`q.push_back(make(...))`) loses every
// string member when the queue is passed to a function. The caller still
// reads the element correctly; the callee sees the right queue size but
// every string member comes back "".
//
// ROOT CAUSE (xezim src/compiler/simulator.rs):
//   - `queue_store_elem`: a function-call argument matches none of the
//     member-wise arms (assignment pattern / class property /
//     `flat_member_name`), so it falls through to `queue_eval_arg` and
//     stores ONE packed Value at `from_func[0]`. No per-member leaf signals
//     (`from_func[0].c`) are created.
//   - `bind_queue_param`: for a struct-element queue formal the packed
//     `vals` are deliberately emptied and elements are copied via
//     `copy_unpacked_struct("recs[j]", "from_func[j]", su)` — which reads
//     the LEAF signals. For call-sourced elements those leaves don't exist,
//     so the callee reads "".
//   - The caller's read path prefers the leaf, then falls back to slicing
//     the packed element value — which is why the caller still sees "hello".
//   - Cousins in the same family (all leaf-based consumers):
//       P3: foreach over an input formal inside the callee
//       P4: queue-to-queue assignment (`copy_q = from_func` ->
//           `assign_whole_coll` -> `copy_whole_queue` -> `queue_copy_elem`
//           -> `copy_unpacked_struct`)
//     both read the same nonexistent leaves.
//
// FIX: in `queue_store_elem`, a value-source actual (a call result) is
// evaluated ONCE and scattered into the element's member leaves via
// `spread_into_unpacked_struct` (the same thing `r = make(...)` does), while
// keeping the packed value for whole-element reads.
//
// Staged checks (controls are shapes that work today and must keep working):
//   A  control : caller reads from_func[0].c
//   B  PRIMARY : callee reads via input formal
//   C  control : callee reads via ref formal
//   D  cousin  : foreach over an input formal in the callee
//   E  cousin  : queue-to-queue copy, then caller + callee reads
//   F  control : struct variable assigned from a call result
//   G  cousin  : multi-element queue of call-sourced elements (callee reads)
//   H  control : variable-pushed element survives the same call
//   I  cousin  : mixed queue (call-sourced [0] + var-sourced [1])
// ============================================================================

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

class util_c;
  typedef struct {
    string a;
    string b;
    string c;
  } rec_s;

  static function rec_s make(string a = " ", string b = " ", string c = " ");
    rec_s r;
    r.a = a;
    r.b = b;
    r.c = c;
    return r;
  endfunction

  // B / E / H: input formal — the primary failing shape.
  static function string first_c(rec_s recs[$]);
    return recs[0].c;
  endfunction

  // C: ref formal — control.
  static function string first_c_ref(ref rec_s recs[$]);
    return recs[0].c;
  endfunction

  // D / I: foreach over an input formal.
  static function string last_c_fe(rec_s recs[$]);
    string s;
    foreach (recs[i]) s = recs[i].c;
    return s;
  endfunction

  // G / I: indexed element read inside a callee.
  static function string get_c(rec_s recs[$], int i);
    return recs[i].c;
  endfunction

  // G / I: every element's string must be intact (length check).
  static function int sum_lens(rec_s recs[$]);
    int total = 0;
    foreach (recs[i]) total += recs[i].c.len();
    return total;
  endfunction
endclass

module top;
  initial begin
    `SVTEST_INIT
    util_c::rec_s from_var[$];
    util_c::rec_s from_func[$];
    util_c::rec_s copy_q[$];
    util_c::rec_s multi[$];
    util_c::rec_s r;
    util_c::rec_s r2;
    string got;

    r.c = "hello";
    from_var.push_back(r);                            // var-sourced element
    from_func.push_back(util_c::make(.c("hello")));   // call-sourced element

    // A (control): caller reads the call-sourced element.
    `SVTEST_CHECK(from_func[0].c == "hello", "A caller from_func[0].c")

    // B (PRIMARY): callee reads via input formal.
    got = util_c::first_c(from_func);
    $display("B callee input formal: '%s' (expect hello)", got);
    `SVTEST_CHECK(got == "hello", "B callee input formal")

    // C (control): callee reads via ref formal.
    got = util_c::first_c_ref(from_func);
    `SVTEST_CHECK(got == "hello", "C callee ref formal")

    // D (cousin): foreach over an input formal in the callee.
    got = util_c::last_c_fe(from_func);
    `SVTEST_CHECK(got == "hello", "D foreach over input formal")

    // E (cousin): queue-to-queue copy, then caller + callee reads.
    copy_q = from_func;
    `SVTEST_CHECK(copy_q[0].c == "hello", "E caller copy_q[0].c")
    got = util_c::first_c(copy_q);
    `SVTEST_CHECK(got == "hello", "E callee after queue copy")

    // F (control): struct variable assigned from a call result.
    r2 = util_c::make(.c("world"));
    `SVTEST_CHECK(r2.c == "world", "F struct var from call result")

    // G (cousin): multi-element queue of call-sourced elements.
    multi.push_back(util_c::make(.a("x"), .c("one")));
    multi.push_back(util_c::make(.c("two")));
    multi.push_back(util_c::make(.a("y"), .b("z"), .c("three")));
    `SVTEST_CHECK(multi[2].a == "y", "G caller multi[2].a")
    `SVTEST_CHECK(util_c::get_c(multi, 0) == "one", "G callee get_c(multi,0)")
    `SVTEST_CHECK(util_c::get_c(multi, 1) == "two", "G callee get_c(multi,1)")
    `SVTEST_CHECK(util_c::get_c(multi, 2) == "three", "G callee get_c(multi,2)")
    `SVTEST_CHECK(util_c::sum_lens(multi) == 11, "G callee sum_lens(multi)")

    // H (control): variable-pushed element survives the same call.
    `SVTEST_CHECK(util_c::first_c(from_var) == "hello", "H var-sourced elem")

    // I (cousin): mixed queue — call-sourced [0] + var-sourced [1].
    begin
      util_c::rec_s r3;
      r3.c = "varsrc";
      from_func.push_back(r3);
      `SVTEST_CHECK(util_c::get_c(from_func, 1) == "varsrc", "I mixed get_c(from_func,1)")
      `SVTEST_CHECK(util_c::last_c_fe(from_func) == "varsrc", "I mixed last_c_fe")
      `SVTEST_CHECK(util_c::sum_lens(from_func) == 11, "I mixed sum_lens(from_func)")
    end

    `SVTEST_PASSFAIL
    $finish;
  end
endmodule
