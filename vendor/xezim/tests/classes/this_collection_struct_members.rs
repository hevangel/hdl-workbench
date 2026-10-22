//! §8.11 — `this.q[i].m`: a member of a struct ELEMENT of a class collection
//! reached through an explicit `this`. The flattened-name builder had no
//! `this` receiver case, so every such read returned 0 and every write was
//! dropped, while the same access without `this.` worked.
//!
//! The expected lines were cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
typedef struct { int m; int n; } item_t;
class C;
  item_t q[$];
  item_t fa[2];
  item_t da[];
  function void fill();
    item_t t;
    t.m = 5; t.n = 6; q.push_back(t);
    t.m = 7; t.n = 8; q.push_back(t);
    fa[1].m = 9;
    da = new[2];
    da[1].m = 11;
  endfunction
  function void show();
    $display("this.q[0].m=%0d this.q[1].n=%0d q[1].m=%0d", this.q[0].m, this.q[1].n, q[1].m);
    $display("this.fa[1].m=%0d this.da[1].m=%0d", this.fa[1].m, this.da[1].m);
    for (int i = 0; i < 2; i++) $display("loop this.q[%0d].m=%0d", i, this.q[i].m);
    this.q[0].m = 42;
    this.da[0].n = 13;
    $display("after write q[0].m=%0d this.q[0].m=%0d da[0].n=%0d", q[0].m, this.q[0].m, da[0].n);
  endfunction
endclass
module top;
  initial begin
    C c = new;
    c.fill();
    c.show();
    $display("c.q[0].m=%0d", c.q[0].m);
  end
endmodule
"#;

#[test]
fn this_qualified_collection_struct_members() {
    let sim = simulate(SRC, 1000).expect("simulate failed");
    let got: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        got,
        vec![
            "this.q[0].m=5 this.q[1].n=8 q[1].m=7",
            "this.fa[1].m=9 this.da[1].m=11",
            "loop this.q[0].m=5",
            "loop this.q[1].m=7",
            "after write q[0].m=42 this.q[0].m=42 da[0].n=13",
            "c.q[0].m=42",
        ]
    );
}
