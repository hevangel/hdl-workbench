//! §18.7 — an inline `obj.randomize() with { obj.member == … }` constraint that
//! refers to the randomized object's OWN rand fields through a PREFIXED
//! receiver (`obj.size`, `obj.error_pos`, `obj.read_write`) must be forced like
//! the bare-name form. Before the fix, the solver keyed targets by the bare
//! property name only, so a prefixed reference was never a forcing target — the
//! field stayed at its raw draw — and stacked with a dyn-array-`.size() ==
//! scalar` coupling it made randomize() burn all 1000 trial retries and return
//! 0. Sources below are validated byte-for-byte against the reference
//! simulator (TAG_PASS).

use xezim::simulate;

fn tags(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("sim");
    sim.output
        .iter()
        .filter(|o| o.message.starts_with("TAG_"))
        .map(|o| o.message.clone())
        .collect()
}

/// Prefixed inline members that equal literals, over a rand scalar coupled to a
/// dynamic array's `.size()`.
#[test]
fn prefixed_inline_member_equals_forced() {
    let src = "\
typedef enum { WRITE, READ } xbus_rw;\n\
class xbus_transfer;\n\
  rand bit [15:0] addr; rand xbus_rw read_write;\n\
  rand int unsigned size; rand byte unsigned data[];\n\
  rand bit [3:0] wait_state[]; rand int unsigned error_pos;\n\
  rand int unsigned transmit_delay;\n\
  constraint c_read_write { read_write inside { WRITE, READ }; }\n\
  constraint c_size { size inside {1,2,4,8}; }\n\
  constraint c_data_wait_size { data.size() == size; wait_state.size() == size; }\n\
  constraint c_transmit { transmit_delay <= 10; }\n\
endclass\n\
module top;\n\
  initial begin : body\n\
    automatic int fails = 0;\n\
    for (int i = 0; i < 8; i++) begin : it\n\
      automatic xbus_transfer m = new();\n\
      automatic int r = m.randomize() with {\n\
        m.size == 1; m.error_pos == 1000; m.read_write == READ;\n\
      };\n\
      if (r != 1) fails++;\n\
      else if (m.size != 1 || m.data.size() != 1 || m.wait_state.size() != 1\n\
               || m.error_pos != 1000) fails++;\n\
    end\n\
    if (fails == 0) $display(\"TAG_PASS\"); else $display(\"TAG_FAIL %0d\", fails);\n\
    $finish;\n\
  end endmodule\n";
    assert_eq!(
        tags(src),
        vec!["TAG_PASS"],
        "prefixed inline members must be forced"
    );
}

/// The prefixed member is pinned to a CONSTANT that is NOT 1 (receiver `t`,
/// scalar only), proving the prefix is genuinely honoured rather than matched
/// by luck in a small domain.
#[test]
fn prefixed_inline_scalar_distinct_pinned() {
    let src = "\
class t;\n\
  rand int unsigned size;\n\
  rand bit[15:0] zz;\n\
  constraint c { size inside {1,2,4,8}; }\n\
endclass\n\
module top; initial begin : body\n\
  automatic t o = new();\n\
  automatic int r = o.randomize() with { o.size == 2; };\n\
  if (r != 1 || o.size != 2) $display(\"TAG_FAIL %0d %0d\", r, o.size);\n\
  else $display(\"TAG_PASS\");\n\
  $finish; end endmodule\n";
    assert_eq!(
        tags(src),
        vec!["TAG_PASS"],
        "prefixed scalar must be pinned exactly"
    );
}

/// A prefixed rand member that couples a dynamic array's size (`m_data.size()
/// == m_length` with `m.length == K`) must size the array to the scalar.
#[test]
fn prefixed_member_coupled_to_dyn_array_size() {
    let src = "\
class gp;\n  rand int unsigned m_length;\n  rand byte unsigned m_data[];\n\
  constraint c { m_length inside {1,2,4,8}; m_data.size() == m_length; }\n\
endclass\n\
module top; initial begin : body\n\
  automatic gp q = new();\n\
  automatic int r = q.randomize() with { q.m_length == 4; };\n\
  if (r != 1 || q.m_length != 4 || q.m_data.size() != 4)\n\
    $display(\"TAG_FAIL %0d %0d %0d\", r, q.m_length, q.m_data.size());\n\
  else $display(\"TAG_PASS\");\n\
  $finish; end endmodule\n";
    assert_eq!(
        tags(src),
        vec!["TAG_PASS"],
        "prefixed scalar must drive the dyn-array size"
    );
}
