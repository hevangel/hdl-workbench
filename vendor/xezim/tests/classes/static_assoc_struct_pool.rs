//! Issue #113 root causes — the UVM resource-pool storage shapes, all
//! reference-validated as minimal repros.
//!
//! 1. §8.9/§7.8: a STATIC class-property assoc keyed by a class handle with
//!    STRUCT elements written member-wise (`ri_tab[rsrc].scope = s`) — the
//!    write went down the struct-leaf path into the instance property map,
//!    invisible to exists()/num()/foreach, so UVM's resource pool lost
//!    every scope and every uvm_config_db get silently missed.
//! 2. §7.8/§8.25: a method-local assoc whose ELEMENT type is a
//!    parameterized-class typedef (`box_t all[int]; all[k] = new;`) never
//!    resolved the element class — the element read back X
//!    (uvm_resource_pool::sort_by_precedence's `all[prec]`).
//! 3. §13.4: a method call on an INDEXED receiver whose element is a live
//!    class handle (`all[prec].size()`) was misrouted to the
//!    nested-collection fallback and answered 0.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> String {
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reference: a=[top.env.in] 7 / b=[top.env.out] 7; same-method
/// exists=1 num=1 with clean readback.
#[test]
fn static_handle_keyed_assoc_struct_members() {
    let src = r#"
class W;
  int id;
  function new(int i); id = i; endfunction
endclass
class Pool;
  typedef struct { string scope; int unsigned precedence; } info_t;
  static info_t ri_tab [W];
  function void set_scope(W r, string s);
    ri_tab[r].scope = s;
    ri_tab[r].precedence = 7;
  endfunction
  function string get_scope(W r);
    if (ri_tab.exists(r)) return ri_tab[r].scope;
    return "<none>";
  endfunction
  function int get_prec(W r);
    if (ri_tab.exists(r)) return ri_tab[r].precedence;
    return -1;
  endfunction
endclass
module top;
  W a, b;
  Pool p;
  initial begin
    a = new(1); b = new(2);
    p = new;
    p.set_scope(a, "top.env.in");
    p.set_scope(b, "top.env.out");
    $display("T|a=[%s] %0d", p.get_scope(a), p.get_prec(a));
    $display("T|b=[%s] %0d", p.get_scope(b), p.get_prec(b));
  end
endmodule
"#;
    let out = outs(&simulate(src, 10).expect("sim"));
    assert!(out.contains("T|a=[top.env.in] 7"), "a scope/prec:\n{out}");
    assert!(out.contains("T|b=[top.env.out] 7"), "b scope/prec:\n{out}");
}

/// Reference: elem constructs (null=0), push_front lands (size=1), the
/// element method dispatches (sz=1), and the item round-trips (id=7).
#[test]
fn local_assoc_of_parameterized_typedef_elements() {
    let src = r#"
class Item;
  int id;
  function new(int i); id = i; endfunction
endclass
class Box #(type T = int);
  T things[$];
  function new(string name = "");
  endfunction
  function void push_front(T t); things.push_front(t); endfunction
  function int size(); return things.size(); endfunction
  function T get(int i);
    if (i >= things.size()) return null;
    return things[i];
  endfunction
endclass
typedef Box#(Item) box_t;
class S;
  static function Item pick();
    box_t all[int];
    Item t;
    int prec;
    t = new(7);
    prec = 998;
    if (!all.exists(prec))
      all[prec] = new;
    $display("T|elem null=%0d", all[prec] == null);
    all[prec].push_front(t);
    $display("T|size=%0d", all[prec] == null ? -1 : all[prec].size());
    foreach (all[i]) begin
      return all[i].get(0);
    end
    return null;
  endfunction
endclass
module top;
  Item r;
  initial begin
    r = S::pick();
    $display("T|out null=%0d id=%0d", r == null, r == null ? -1 : r.id);
  end
endmodule
"#;
    let out = outs(&simulate(src, 10).expect("sim"));
    assert!(out.contains("T|elem null=0"), "element constructs:\n{out}");
    assert!(
        out.contains("T|size=1"),
        "push_front + class-method dispatch:\n{out}"
    );
    assert!(out.contains("T|out null=0 id=7"), "round-trip:\n{out}");
}
