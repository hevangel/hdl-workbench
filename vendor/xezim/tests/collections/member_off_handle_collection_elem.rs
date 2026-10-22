//! class-perf: member READ rooted at an element of a class-handle member
//! collection — `m[k].data`, `this.aa[k].field`, `obj.q[i].m`. Previously
//! this shape fell off the bytecode path (`compile-fail Expr_MemberAccess`
//! in a method body), sending the whole method to the AST interpreter. Now
//! methods like a `uvm_regex_cache#(string,chandle)` `get` body compile:
//! the element read lowers to a `LoadCollElem` yielding the stored class
//! HANDLE, then `LoadClassMember` reads the field off it.
//!
//! Coverage (cross-checked against the reference simulator):
//! * stored handle field round-tripped (v, string member);
//! * a QUEUE of class handles (`q[0].val`) follows the same element path.

use xezim::simulate;

#[test]
fn member_read_off_handle_collection_element() {
    // Compiled from the first call (tier 0) in a child process, then again
    // here at the default tier.
    let _ = super::compiled_method_test_env::eager();
    let src = r#"
class node;
  string name;
  int val;
  function new(string n, int v); name = n; val = v; endfunction
endclass
class cache;
  node m[string];
  node q[$];
  function void put(string k, node n); m[k] = n; endfunction
  function void push(node n); q.push_back(n); endfunction
  function int get_vals_dotted(string k); return m[k].val; endfunction
  function string get_name_bare(string k); return this.m[k].name; endfunction
  function int first_q(); return q[0].val + q[1].val; endfunction
  function int safe_vals(string k); return m[k].val; endfunction
  function string safe_names(string k); return m[k].name; endfunction
endclass
module top;
  initial begin
    automatic cache c = new;
    automatic node a = new("alpha", 10), b = new("beta", 20);
    c.put("a", a); c.put("b", b); c.push(a); c.push(b);
    $display("V1 %0d", c.get_vals_dotted("a"));
    $display("V2 %s", c.get_name_bare("b"));
    $display("V3 %0d", c.first_q());
    $display("N2 [%0d][%s]", c.safe_vals("a"), c.safe_names("a"));
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    for want in ["V1 10", "V2 beta", "V3 30", "N2 [10][alpha]"] {
        assert!(
            o.iter().any(|l| l.trim() == want),
            "missing {want:?} in {o:?}"
        );
    }
}
