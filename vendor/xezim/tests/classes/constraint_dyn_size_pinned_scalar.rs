//! §18.5 / dimethyl — a rand DYNAMIC array whose `.size()` equals a
//! co-constrained rand SCALAR (`m_data.size == m_length` with `m_length ==
//! 10`, plus `m_streaming_width == m_length`, `m_byte_enable_length <=
//! m_length`/`.size` coupling) must be sized to the scalar's FINAL constrained
//! value — not the scalar pinned to the array's current (empty) size 0, which
//! never satisfies `m_length == 10` and would burn all 1000 trial retries,
//! making randomize() FAIL. This is the regression seen on the UVM generic
//! payload (obj1.randomize()). The source below is byte-for-byte validated
//! against the reference simulator (TAG_PASS).

use xezim::simulate;

fn tags(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("sim");
    sim.output
        .iter()
        .filter(|o| o.message.starts_with("TAG_"))
        .map(|o| o.message.clone())
        .collect()
}

/// 8 randomized objects must all satisfy the full coupled set; the dyn array
/// sizes equal their co-constrained scalars and the pinned scalar stays 10.
#[test]
fn dyn_array_size_coupled_to_pinned_scalar() {
    let src = "\
class gp; rand bit[63:0] m_address; rand byte unsigned m_data[];\n\
  rand int unsigned m_length; rand byte unsigned m_byte_enable[];\n\
  rand int unsigned m_byte_enable_length; rand int unsigned m_streaming_width;\n\
  constraint body {\n\
    m_address >= 0 && m_address < 256;\n\
    m_length == 10;\n\
    m_data.size == m_length;\n\
    m_streaming_width == m_length;\n\
    m_byte_enable_length <= m_length;\n\
    (m_byte_enable_length % 4) == 0;\n\
    m_byte_enable.size == m_byte_enable_length;\n\
    foreach (m_byte_enable[i]) m_byte_enable[i] inside { 0, 255 };\n\
  }\n\
endclass\n\
module top;\n\
  initial begin : b\n\
    automatic int failures = 0;\n\
    automatic gp g;\n\
    for (int n = 0; n < 8; n++) begin\n\
      g = new();\n\
      if (!g.randomize()) failures++;\n\
      if (g.m_length != 10) failures++;\n\
      if (g.m_data.size() != g.m_length) failures++;\n\
      if (g.m_streaming_width != g.m_length) failures++;\n\
      if (g.m_byte_enable_length > g.m_length) failures++;\n\
      if ((g.m_byte_enable_length % 4) != 0) failures++;\n\
      if (g.m_byte_enable.size() != g.m_byte_enable_length) failures++;\n\
    end\n\
    if (failures == 0) $display(\"TAG_PASS\"); else $display(\"TAG_FAIL %0d\", failures);\n\
    $finish;\n\
  end endmodule";
    let t = tags(src);
    assert_eq!(
        t,
        vec!["TAG_PASS"],
        "all draws must satisfy the coupled set"
    );
}

/// The SAME set reached through an INLINE `randomize() with {…}` (as the UVM
/// test drives it) must also solve: the dyn-array `.size()` must not collapse
/// the equality `m_data.size == m_length` into a spurious `m_length == 0` that
/// contradicts `m_length == 20`.
#[test]
fn dyn_array_size_coupled_through_inline_with() {
    let src = "\
class gp;  rand byte unsigned m_data[]; rand int unsigned m_length;
\
  rand int unsigned m_streaming; endclass\n\
module top; initial begin : b\n\
  automatic gp g = new();\n\
  if (!g.randomize() with { m_length == 20; m_data.size == m_length; m_streaming == m_length; })\n\
    $display(\"TAG_FAIL rand\");\n\
  else if (g.m_length != 20 || g.m_data.size() != 20 || g.m_streaming != 20)\n\
    $display(\"TAG_FAIL vals %0d %0d %0d\", g.m_length, g.m_data.size(), g.m_streaming);\n\
  else $display(\"TAG_PASS\");\n\
  $finish; end endmodule\n";
    assert_eq!(
        tags(src),
        vec!["TAG_PASS"],
        "inline with + .size must solve"
    );
}

/// §18.5.8.1 — `d.size() == n` sizes the array FROM n, so n is still drawn
/// over its whole range. Reading the current size back into n pinned it to
/// the first call's length forever. Cross-checked against the reference
/// simulator.
#[test]
fn size_tied_scalar_keeps_its_full_range() {
    let src = r#"
class dq;
  rand bit [7:0] d[];
  rand bit [3:0] n;
  constraint c1 { d.size() == n; n inside {[3:8]}; }
endclass
module tb;
  initial begin
    automatic dq q = new;
    automatic int sizes[int];
    automatic int bad = 0;
    for (int i = 0; i < 100; i++) begin
      void'(q.randomize());
      sizes[q.n] = 1;
      if (q.d.size() != q.n) bad++;
    end
    $display("TAG_SIZES=%0d BAD=%0d", sizes.num(), bad);
  end
endmodule
"#;
    assert_eq!(tags(src), vec!["TAG_SIZES=6 BAD=0"]);
}
