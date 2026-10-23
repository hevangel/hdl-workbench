//! A class that extends a parameterized class sees the base's type
//! parameters (§8.25): `class d extends drv #(item)` names `REQ` and the
//! defaulted `RSP = REQ` (§6.20.2) in its own methods. Such a class has no
//! specialization of its own and its objects carry no bindings, so `REQ`
//! and `RSP` resolved to nothing there: `RSP::type_id::create()` returned
//! null (the axi4 AVIP drivers' "Driver put a null response"), and
//! `RSP x = new` or `r = new` on a `RSP`-typed property built nothing
//! usable. They now resolve through the base specialization the method's
//! class binds.
//!
//! Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
class base_item; virtual function string nm(); return "base"; endfunction endclass
class wrapper #(type T = base_item); static function T create(); T t = new(); return t; endfunction endclass
class my_item extends base_item;
  typedef wrapper #(my_item) type_id;
  virtual function string nm(); return "my"; endfunction
endclass
class other_item extends base_item;
  typedef wrapper #(other_item) type_id;
  virtual function string nm(); return "other"; endfunction
endclass
class drv #(type REQ = base_item, type RSP = REQ);
  REQ q; RSP r;
endclass
class e1 extends drv #(my_item);
  function void run();
    REQ a; RSP b;
    a = new(); b = new();
    $display("T|e1 a=%s b=%s", a.nm(), b.nm());
    q = new(); r = new();
    $display("T|e1 q=%s r=%s", q.nm(), r.nm());
    q = REQ::type_id::create(); r = RSP::type_id::create();
    $display("T|e1 cq=%s cr=%s", q.nm(), r.nm());
  endfunction
  static function string sf(); RSP x; x = RSP::type_id::create(); return x.nm(); endfunction
endclass
class e2 extends drv #(my_item, other_item);
  function void run(); r = RSP::type_id::create(); q = REQ::type_id::create(); $display("T|e2 q=%s r=%s", q.nm(), r.nm()); endfunction
endclass
class mid extends drv #(other_item); endclass
class leaf extends mid;
  function void run(); RSP x; x = RSP::type_id::create(); r = new(); $display("T|leaf x=%s r=%s", x.nm(), r.nm()); endfunction
endclass
class pd #(type T = my_item) extends drv #(T);
  function void run(); RSP x; x = RSP::type_id::create(); q = REQ::type_id::create(); $display("T|pd x=%s q=%s", x.nm(), q.nm()); endfunction
endclass
class pc #(type REQ = base_item) extends drv #(my_item);
  function void run(); REQ x; x = REQ::type_id::create(); r = RSP::type_id::create(); $display("T|pc x=%s r=%s", x.nm(), r.nm()); endfunction
endclass
class tp extends drv #(my_item);
  function void run();
    r = RSP::type_id::create();
    $display("T|tp null=%0d", r == null);
  endfunction
endclass
module top; initial begin
  e1 a = new(); e2 b = new(); leaf c = new(); pd #(other_item) d = new(); pd e = new();
  pc #(other_item) f = new(); tp g = new();
  a.run(); $display("T|sf=%s", e1::sf()); b.run(); c.run(); d.run(); e.run(); f.run(); g.run();
end endmodule
"#;

#[test]
fn inherited_type_params_resolve_in_derived_class() {
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
            "T|e1 a=my b=my",
            "T|e1 q=my r=my",
            "T|e1 cq=my cr=my",
            "T|sf=my",
            "T|e2 q=my r=other",
            "T|leaf x=other r=other",
            "T|pd x=other q=other",
            "T|pd x=my q=my",
            "T|pc x=other r=my",
            "T|tp null=0",
        ]
    );
}
