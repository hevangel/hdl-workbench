//! IEEE 1800-2023 §8.14 / §8.10 / §8.15: a property redeclared in a derived class is a separate variable; access through a handle resolves by the handle's STATIC type (`Base b = d; b.v` reads Base's copy), including writes, `this.v`, `super.v`, chains and formals.
//!
//! Expected lines come from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const SHADOW: &str = r#"
// top: t8_14
class Base; int v = 1; endclass
class Derived extends Base; int v = 2; endclass
module t8_14;
  Base b; Derived d;
  initial begin d = new; b = d; $display("T|a|d.v=%0d b.v=%0d", d.v, b.v); end
endmodule
"#;

const SHADOW_FAMILY: &str = r#"
class Base; int v = 1; string s = "bs";
  function int getv(); return v; endfunction
  function void setv(int x); v = x; endfunction
  function int getthis(); return this.v; endfunction
endclass
class Derived extends Base; int v = 2; string s = "ds";
  function int dget(); return v; endfunction
  function int sget(); return super.v; endfunction
  function int tget(); return this.v; endfunction
endclass
class Leaf extends Derived; int v = 3; endclass
class Holder; Base hb; Derived hd; endclass
module t;
  Base b; Derived d; Leaf l; Holder h; Derived d2;
  function automatic int fread(Base bb); return bb.v; endfunction
  initial begin
    d = new; b = d;
    $display("T|a|d.v=%0d b.v=%0d b.s=%s d.s=%s", d.v, b.v, b.s, d.s);
    b.v = 10;
    $display("T|b|d.v=%0d b.v=%0d getv=%0d dget=%0d sget=%0d tget=%0d gt=%0d", d.v, b.v, b.getv(), d.dget(), d.sget(), d.tget(), d.getthis());
    d.v = 20; b.setv(30);
    $display("T|c|d.v=%0d b.v=%0d", d.v, b.v);
    l = new; b = l; d2 = l;
    $display("T|d|l.v=%0d d2.v=%0d b.v=%0d", l.v, d2.v, b.v);
    d2.v = 22; b.v = 11;
    $display("T|e|l.v=%0d d2.v=%0d b.v=%0d sget=%0d", l.v, d2.v, b.v, d2.sget());
    h = new; h.hb = l; h.hd = l;
    $display("T|f|h.hb.v=%0d h.hd.v=%0d fread=%0d", h.hb.v, h.hd.v, fread(l));
    h.hb.v = 5;
    $display("T|g|h.hb.v=%0d b.v=%0d l.v=%0d", h.hb.v, b.v, l.v);
    b.v++;
    $display("T|h|b.v=%0d", b.v);
  end
endmodule
"#;

#[test]
fn base_handle_reads_base_copy() {
    let want = ["T|a|d.v=2 b.v=1"];
    assert_eq!(t_lines(SHADOW), want);
}

#[test]
fn shadowed_property_family() {
    let want = [
        "T|a|d.v=2 b.v=1 b.s=bs d.s=ds",
        "T|b|d.v=2 b.v=10 getv=10 dget=2 sget=10 tget=2 gt=10",
        "T|c|d.v=20 b.v=30",
        "T|d|l.v=3 d2.v=2 b.v=1",
        "T|e|l.v=3 d2.v=22 b.v=11 sget=11",
        "T|f|h.hb.v=11 h.hd.v=22 fread=11",
        "T|g|h.hb.v=5 b.v=5 l.v=3",
        "T|h|b.v=6",
    ];
    assert_eq!(t_lines(SHADOW_FAMILY), want);
}
