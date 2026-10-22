//! class-perf: compiled methods may read members typed by a TYPE PARAMETER.
//!
//! `class_nonloadable_member_names` refused every property whose declared
//! type is a TypeReference that plan time cannot resolve — a bare TYPE
//! PARAMETER (`DATA_T data;` in a parameterized node class) or a
//! parameterized-class alias (`typedef uvm_lru_cache_node#(KEY_T,DATA_T)
//! node_type;`). The UVM LRU cache's `get` reads `node.data` off exactly
//! such a member, so the whole method declined with `method_receiver_field`
//! and ran interpreted (the report-message element-table regression's
//! `uvm_is_match` hot loop).
//!
//! A TypeReference-typed property is a plain heap slot: the interpreter's
//! dotted read fetches `properties[name]` verbatim and never resizes a
//! non-integral member, and `LoadClassMember` does the same — so admitting
//! it is faithful for every specialization (class handles, chandle,
//! typedef'd scalars). Specializations bound to a COLLECTION shape are not
//! affected: collections are keyed in their own tables and were already
//! excluded before this change.
//!
//! The parameterized-class-alias half is `typeref_class_name` resolving a
//! hoisted `Node#(K,D)`-shaped target by stripping the specialization
//! fragment to the base class — member admission sets are per class name
//! and inheritance-safe.

use xezim::simulate;

fn gate_on() -> bool {
    super::compiled_method_test_env::eager()
}

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("top.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

/// A member typed by a bare type parameter, read through a handle whose
/// type is a class-local typedef of a parameterized class — the
/// `uvm_lru_cache::get` shape. The member is read directly and through a
/// handle-returning method; a specialization binding the param to a class
/// handle must yield that handle (a garbage/zeroed read would break every
/// chained dispatch).
///
/// NOTE: an assoc array whose ELEMENT type is a bare type parameter loses
/// stores at the interpreter level (pre-existing, reproducible on the
/// committed build with `DATA_T m_hash[KEY_T]`); element reads through a
/// class-local typedef (`node_type m_hash[KEY_T]`) are fine and are what
/// the UVM cache uses, so that is the shape tested here.
const TYPE_PARAM_MEMBER: &str = r#"
class payload;
  int id;
  function new(int i);
    id = i;
  endfunction
endclass

class node_c #(type K = string, type D = int);
  K key;
  D data;
  node_c#(K,D) prev;
  node_c#(K,D) next;
  function new(K k, D d);
    key = k; data = d;
  endfunction
endclass

class cache_c #(type KEY_T = string, type DATA_T = int);
  typedef node_c#(KEY_T,DATA_T) node_type;
  node_type m_hash[KEY_T];
  node_type m_begin;
  function node_type get(KEY_T key);
    node_type node;
    if (m_hash.exists(key)) begin
      node = m_hash[key];
      if (m_begin != node) begin
        node.next = m_begin;
        m_begin.prev = node;
        m_begin = node;
      end
      return node;
    end
    return null;
  endfunction
  function void put(KEY_T key, DATA_T data);
    node_type node;
    node = new(key, data);
    node.next = m_begin;
    m_begin = node;
    m_hash[key] = node;
  endfunction
endclass

module top;
  int r0, r1, r2, r3;
  initial begin
    cache_c#(string, payload) c;
    payload p0, p1, pa, pb;
    pa = new(7);
    pb = new(11);
    c = new();
    c.put("a", pa);
    c.put("b", pb);
    p0 = c.get("a").data;
    p1 = c.get("b").data;
    r0 = p0.id;
    r1 = p1.id;
    r2 = c.get("a").data.id;
    r3 = c.get("zz") == null;
  end
endmodule
"#;

#[test]
fn type_param_member_read_compiles_and_matches() {
    if !gate_on() {
        return;
    }
    let sim = simulate(TYPE_PARAM_MEMBER, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 7, "hit through typed member");
    assert_eq!(u(&sim, "r1"), 11, "second element not aliased");
    assert_eq!(u(&sim, "r2"), 7, "chained read through returned handle");
    assert_eq!(u(&sim, "r3"), 1, "miss returns null");
}

/// A member typed by a bare type parameter holding a CLASS HANDLE: the
/// read must yield the handle and a chained read through it must dispatch
/// on the right object (a garbage/zeroed read would read the wrong `id`).
///
/// NOTE: an inline `new(...)` ACTUAL to a type-parameter-typed formal
/// arrives null at the interpreter level (pre-existing, reproduces on the
/// committed build), so the handles here are constructed into locals
/// first — same as any real UVM test body.
const TYPE_PARAM_HANDLE_MEMBER: &str = r#"
class payload;
  int id;
  function new(int i);
    id = i;
  endfunction
endclass

class node_c #(type K = string, type D = int);
  K key;
  D data;
  node_c#(K,D) prev;
  node_c#(K,D) next;
  function new(K k, D d);
    key = k; data = d;
  endfunction
endclass

class cache_c #(type KEY_T = string, type DATA_T = int);
  typedef node_c#(KEY_T,DATA_T) node_type;
  node_type m_hash[KEY_T];
  function node_type get(KEY_T key);
    node_type node;
    if (m_hash.exists(key)) begin
      node = m_hash[key];
      return node;
    end
    return null;
  endfunction
  function void put(KEY_T key, DATA_T d);
    node_type node;
    node = new(key, d);
    m_hash[key] = node;
  endfunction
endclass

module top;
  int r0, r1;
  initial begin
    cache_c#(string, payload) c;
    payload p0, p1, pa, pb;
    pa = new(7);
    pb = new(11);
    c = new();
    c.put("a", pa);
    c.put("b", pb);
    p0 = c.get("a").data;
    p1 = c.get("b").data;
    r0 = p0.id;
    r1 = p1.id;
  end
endmodule
"#;

#[test]
fn type_param_handle_member_chain_compiles_and_matches() {
    if !gate_on() {
        return;
    }
    let sim = simulate(TYPE_PARAM_HANDLE_MEMBER, 50).expect("simulate failed");
    assert_eq!(u(&sim, "r0"), 7, "handle member through typed typedef");
    assert_eq!(u(&sim, "r1"), 11, "second element not aliased");
}
