//! §21.2.1.3: `%s` of a `string` member reached THROUGH an element select
//! (`q[i].name`, `arr[0].name`, `oq[0].in.name`) printed the text padded with
//! leading spaces to the member's 1024-bit storage width.
//!
//! Such a reference parses as `MemberAccess { Index { .. }, name }`; the
//! string classification of `%s` operands only followed plain dotted paths
//! and gave up at the first select, so the member was formatted as a packed
//! vector (every leading NUL byte printing as a space). The member's declared
//! type is now followed through element selects, struct members and class
//! properties. Every expectation below was cross-checked against the
//! reference simulator.

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
fn module_scope_collection_elements() {
    let o = out(r#"
typedef struct { int level; string name; } row_info;
module top;
  row_info q[$];
  row_info arr[2];
  row_info d[];
  initial begin
    row_info r;
    r.level = 5; r.name = "mod";
    q.push_back(r);
    arr[0] = r;
    d = new[1];
    d[0] = r;
    $display("Q: level=%0d name='%s' len=%0d", q[0].level, q[0].name, q[0].name.len());
    $display("A: level=%0d name='%s' len=%0d", arr[0].level, arr[0].name, arr[0].name.len());
    $display("D: level=%0d name='%s' len=%0d", d[0].level, d[0].name, d[0].name.len());
  end
endmodule
"#);
    expect_all(
        &o,
        &[
            "Q: level=5 name='mod' len=3",
            "A: level=5 name='mod' len=3",
            "D: level=5 name='mod' len=3",
        ],
    );
}

#[test]
fn class_property_collection_elements() {
    let o = out(r#"
typedef struct { int level; string name; } row_t;
typedef struct { int a; row_t in; string tag; } outer_t;
class C;
  row_t q[$];
  row_t fa[2];
  outer_t oq[$];
  function void run();
    row_t r;
    outer_t o;
    r.level = 1; r.name = "alpha";
    q.push_back(r);
    fa[1] = r;
    $display("size=%0d direct[last]: level=%0d name='%s'", q.size(), q[q.size()-1].level,
             q[q.size()-1].name);
    $display("F '%s'", fa[1].name);
    $display("CAT %s", {"<", q[0].name, ">"});
    o.a = 7; o.in.level = 8; o.in.name = "nest"; o.tag = "tg";
    oq.push_back(o);
    $display("N2 %0d '%s' '%s'", oq[0].in.level, oq[0].in.name, oq[0].tag);
  endfunction
endclass
module top;
  initial begin
    C c = new;
    c.run();
    $display("X2 '%s' %0d", c.q[0].name, c.q[0].level);
  end
endmodule
"#);
    expect_all(
        &o,
        &[
            "size=1 direct[last]: level=1 name='alpha'",
            "F 'alpha'",
            "CAT <alpha>",
            "N2 8 'nest' 'tg'",
            "X2 'alpha' 1",
        ],
    );
}
