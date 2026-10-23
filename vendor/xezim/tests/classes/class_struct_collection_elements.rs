//! §7.2/§7.4: a WHOLE element of a class-property collection of unpacked
//! structs (`r = q[i]`, `row_t row = m_rows[i];`, `return q[i];`, `q[j] ==
//! q[i]`) read back as zero.
//!
//! The element's leaves live under the instance — `<h>#q[i].<member>` for a
//! queue / dynamic / associative property, the object's property map for a
//! fixed array — while the whole-struct copy looked for them under the flat
//! name `q[i].<member>`, found nothing, and fell back to evaluating the
//! element as one packed value that does not exist. Member reads (`q[i].m`)
//! always worked, so the copy now goes member by member through them.
//!
//! This is what left every UVM table blank: `uvm_table_printer::emit` walks
//! `uvm_printer_row_info row = m_rows[i];` and every row came out empty.
//! Every expectation below was cross-checked against the reference simulator.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

fn expect_all(got: &[String], want: &[&str]) {
    for w in want {
        assert!(got.iter().any(|l| l == w), "expected {w:?} in:\n{got:#?}");
    }
}

#[test]
fn element_copies_from_queue_fixed_and_dynamic_properties() {
    let o = out(r#"
typedef struct { int level; string name; } row_t;
typedef struct { int a; row_t in; string tag; } outer_t;
class C;
  row_t q[$];
  row_t fa[2];
  row_t da[];
  outer_t oq[$];
  function string show(row_t r);
    return $sformatf("%0d/%s", r.level, r.name);
  endfunction
  function row_t mk(int l, string n);
    row_t r;
    r.level = l; r.name = n;
    return r;
  endfunction
  function void run();
    row_t r, r2;
    outer_t o, o2;
    r.level = 1; r.name = "one";
    q.push_back(r);
    r.level = 2; r.name = "two";
    q.push_back(r);
    fa[0] = r;
    fa[1] = q[0];
    da = new[2];
    da[0] = q[1];
    da[1] = mk(3, "three");
    r2 = q[0];
    $display("Q0 %0d '%s'", r2.level, r2.name);
    r2 = fa[1];
    $display("F1 %0d '%s'", r2.level, r2.name);
    r2 = da[1];
    $display("D1 %0d '%s'", r2.level, r2.name);
    foreach (q[i]) begin
      row_t row = q[i];
      $display("FE %0d %0d '%s' len=%0d", i, row.level, row.name, row.name.len());
    end
    $display("FN %s %s %s", show(q[1]), show(fa[0]), show(da[1]));
    $display("P %p", q[0]);
    q[0] = q[1];
    $display("CP %0d '%s'", q[0].level, q[0].name);
    o.a = 7; o.in.level = 8; o.in.name = "nest"; o.tag = "tg";
    oq.push_back(o);
    o2 = oq[0];
    $display("N1 %0d %0d '%s' '%s'", o2.a, o2.in.level, o2.in.name, o2.tag);
    r2 = oq[0].in;
    $display("N3 %0d '%s'", r2.level, r2.name);
  endfunction
endclass
module top;
  initial begin
    C c = new;
    row_t r;
    c.run();
    r = c.q[1];
    $display("X1 %0d '%s'", r.level, r.name);
  end
endmodule
"#);
    expect_all(
        &o,
        &[
            "Q0 1 'one'",
            "F1 1 'one'",
            "D1 3 'three'",
            "FE 0 1 'one' len=3",
            "FE 1 2 'two' len=3",
            "FN 2/two 2/two 3/three",
            "P '{level:1, name:\"one\"}",
            "CP 2 'two'",
            "N1 7 8 'nest' 'tg'",
            "N3 8 'nest'",
            "X1 2 'two'",
        ],
    );
}

#[test]
fn element_return_and_equality() {
    let o = out(r#"
typedef struct { int level; string name; } row_t;
class C;
  row_t q[$];
  row_t fa[3];
  function row_t get(int i);
    return q[i];
  endfunction
  function void run();
    row_t r;
    r.level = 4; r.name = "four";
    q.push_back(r);
    r.level = 5; r.name = "five";
    q.push_back(r);
    r = get(1);
    $display("G %0d '%s'", r.level, r.name);
    fa[2] = q[0];
    r = fa[2];
    $display("H %0d '%s'", r.level, r.name);
    if (q[0] == q[0]) $display("EQ same");
    if (q[0] != q[1]) $display("NE diff");
  endfunction
endclass
module top;
  initial begin
    C c = new;
    c.run();
  end
endmodule
"#);
    expect_all(&o, &["G 5 'five'", "H 4 'four'", "EQ same", "NE diff"]);
}

/// The shape `uvm_table_printer` uses: rows pushed by one method, copied out
/// of the protected queue by another.
#[test]
fn printer_rows_survive_the_decl_init_copy() {
    let o = out(r#"
typedef struct {
  int    level;
  string name;
  string type_name;
  string size;
  string val;
} row_info;
class printer;
  protected row_info m_rows[$];
  function void add(string n, string t, int lvl);
    row_info r;
    r.level = lvl;
    r.name = n;
    r.type_name = t;
    r.size = $sformatf("%0d", lvl * 8);
    r.val = "v";
    m_rows.push_back(r);
  endfunction
  function void dump();
    foreach (m_rows[i]) begin
      row_info row = m_rows[i];
      $display("row %0d: level=%0d name='%s' type='%s' size='%s' val='%s' len=%0d", i,
               row.level, row.name, row.type_name, row.size, row.val, row.name.len());
    end
  endfunction
endclass
module top;
  initial begin
    printer p = new;
    p.add("alpha", "int", 1);
    p.add("beta", "string", 2);
    p.dump();
  end
endmodule
"#);
    expect_all(
        &o,
        &[
            "row 0: level=1 name='alpha' type='int' size='8' val='v' len=5",
            "row 1: level=2 name='beta' type='string' size='16' val='v' len=4",
        ],
    );
}
