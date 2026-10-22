//! §8.10 / §8.7 — a property a derived class REDECLARES is a separate
//! variable per declaring class, and the base declaration's inline
//! initializer belongs to the base copy. The base copy is stored under its
//! own key, but construction ran every initializer into the bare (leaf)
//! slot: the base's `pool = new("pool")` landed in the derived's `pool`, the
//! base's own copy stayed null, and a base accessor returned null. This is
//! the shape of a UVM sequence item that declares its own `events` and fills
//! it from `get_event_pool()` — every `wait_trigger` then ran on a null
//! event and returned at once. The expected lines are the reference
//! simulator's output.

use xezim::simulate;

#[test]
fn base_initializer_fills_the_base_copy_of_a_shadowed_property() {
    let src = r#"
class box #(type T = int);
  T items[string];
  string name;
  function new(string name = ""); this.name = name; endfunction
  function T get(string key);
    if (!items.exists(key))
      items[key] = new(key);
    return items[key];
  endfunction
endclass
class leaf;
  string name;
  function new(string name = ""); this.name = name; endfunction
endclass
typedef box #(leaf) leaf_box;
class base;
  local leaf_box pool = new("pool");
  int tag = 11;
  function leaf_box get_pool(); return pool; endfunction
  function int get_tag(); return tag; endfunction
endclass
class derived extends base;
  leaf_box pool;
  int tag;
  function new();
    pool = get_pool();
    tag = 22;
  endfunction
endclass
module top;
  derived d;
  initial begin
    d = new;
    $display("pool null=%0d name=%s", d.pool == null, d.pool == null ? "-" : d.pool.name);
    $display("leaf=%s", d.pool.get("x").name);
    $display("base tag=%0d derived tag=%0d", d.get_tag(), d.tag);
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        vec![
            "pool null=0 name=pool",
            "leaf=x",
            "base tag=11 derived tag=22"
        ]
    );
}
