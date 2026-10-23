//! Associative-array element of a TYPEDEF'd class type: construction and
//! method dispatch.
//!
//! Guards `exec_statement`'s `collection[key] = new` path. The element type
//! is resolved from the collection's declared element type, but that type is
//! frequently a TYPEDEF alias (e.g. UVM's
//! `typedef uvm_pool#(uvm_severity,uvm_severity) uvm_sev_override_array;`
//! used as `uvm_sev_override_array sev_id_overrides [string];`). The maps
//! record the typedef NAME, which is not a key in `module.classes`, so the
//! old `contains_key` filter rejected it: `arr[k] = new` stored an object
//! with no resolvable class, and every method call on it (`add`/`get`/
//! `exists`/`num`) silently no-op'd. For UVM this meant severity-id overrides
//! were never stored, so `set_report_severity_id_override` had no effect.
//!
//! Both a simple typedef (`typedef C alias_t;`) and a parameterized typedef
//! (`typedef C#(T) alias_t;`) are exercised.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
}

/// Simple (non-parameterized) typedef element class.
#[test]
fn assoc_array_of_simple_typedef_class() {
    const SRC: &str = "class box;
  int n;
  function new(); n = 0; endfunction
  function void add(int v); n = v; endfunction
  function int get(); return n; endfunction
endclass

typedef box box_t;

class holder;
  box_t arr [string];
  function void put(string id, int v);
    if (!arr.exists(id)) arr[id] = new;
    arr[id].add(v);
  endfunction
  function int get(string id);
    if (!arr.exists(id)) return -1;
    return arr[id].get();
  endfunction
endclass

module top;
  int r1, r2;
  initial begin
    holder h = new;
    h.put(\"k1\", 11);
    h.put(\"k2\", 22);
    r1 = h.get(\"k1\");
    r2 = h.get(\"k2\");
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "r1"), 11, "typedef'd element add/get persists (k1)");
    assert_eq!(u(&sim, "r2"), 22, "typedef'd element add/get persists (k2)");
}

/// Parameterized typedef element class: the element's type-param members
/// must resolve against the typedef's specialization during construction.
#[test]
fn assoc_array_of_parameterized_typedef_class() {
    const SRC: &str = "class reg_cell #(parameter int W = 8);
  bit [W-1:0] val;
  function new(); val = '0; endfunction
  function void set(bit [W-1:0] v); val = v; endfunction
  function bit [W-1:0] get(); return val; endfunction
endclass

// Element type is a PARAMETERIZED typedef alias.
typedef reg_cell#(16) cell16_t;

class tbl;
  cell16_t cells [int];
  function void put(int idx, bit [15:0] v);
    if (!cells.exists(idx)) cells[idx] = new;
    cells[idx].set(v);
  endfunction
  function bit [15:0] get(int idx);
    return cells[idx].get();
  endfunction
endclass

module top;
  bit [15:0] a, b;
  initial begin
    tbl t = new;
    t.put(0, 16'h1234);
    t.put(1, 16'hABCD);
    a = t.get(0);
    b = t.get(1);
  end
endmodule
";
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(u(&sim, "a"), 0x1234, "param-typedef element set/get (0)");
    assert_eq!(u(&sim, "b"), 0xABCD, "param-typedef element set/get (1)");
}
