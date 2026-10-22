//! A dynamic-array class property initialized with its constructor
//! (`int da[] = new[4];`) is sized when the object is constructed, as if
//! the assignment were the first statement of `new()` (§8.7, §7.5.1). The
//! property-initializer pass deferred it as a call-bearing initializer and
//! evaluated it as a scalar, so the array stayed empty.
//!
//! Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
class p4; int da[] = new[4]; int db[]; function new(); db = new[3]; endfunction endclass
class pn #(int N = 5); int d[] = new[N]; byte e[] = new[3]('{7, 8, 9}); endclass
class pd extends p4; string ds[] = new[2]; logic [3:0] dl[] = new[2]; endclass
class pq; int q[$]; int dd[] = new[2]; function new(); dd[1] = 6; endfunction endclass
module top; initial begin
  p4 o = new(); p4 o2 = new(); pn #(6) n = new(); pn n5 = new(); pd d = new(); pq q = new();
  o.da[0] = 11;
  $display("T|da=%0d db=%0d o2da0=%0d oda0=%0d", o.da.size(), o.db.size(), o2.da[0], o.da[0]);
  $display("T|n=%0d n5=%0d e=%0d e2=%0d", n.d.size(), n5.d.size(), n.e.size(), n.e[2]);
  $display("T|ds=%0d dl=%0d dl0=%b da=%0d", d.ds.size(), d.dl.size(), d.dl[0], d.da.size());
  $display("T|dd=%0d %0d %0d", q.dd.size(), q.dd[0], q.dd[1]);
end endmodule
"#;

#[test]
fn dynamic_array_property_initializer_sizes_the_array() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        out,
        [
            "T|da=4 db=3 o2da0=0 oda0=11",
            "T|n=6 n5=5 e=3 e2=9",
            "T|ds=2 dl=2 dl0=xxxx da=4",
            "T|dd=2 0 6",
        ]
    );
}
