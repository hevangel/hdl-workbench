//! §6.16: `s[i]` on a string local is a byte, so `s[0] != "["` is an
//! integral compare. The compiled-method path took any select of a string
//! local for a string and ran a textual compare, so the test was always
//! false once the body compiled. UVM 1.2's `uvm_scope_stack::up()` then
//! popped the whole scope stack and `print_topology()` printed an empty
//! table. Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
class scope_stack;
  local string m_stack[$];
  function int depth(); return m_stack.size(); endfunction
  function void down(string s); m_stack.push_back(s); endfunction
  function void up(byte separator = ".");
    bit found;
    string s;
    while (m_stack.size() && !found) begin
      s = m_stack.pop_back();
      if (separator == ".") begin
        if (s == "" || (s[0] != "[" && s[0] != "(" && s[0] != "{"))
          found = 1;
      end
      else begin
        if (s != "" && s[0] == separator)
          found = 1;
      end
    end
  endfunction
endclass
class C;
  function int probe(string a);
    string s;
    s = a;
    return (s[0] != "[") + 2 * (s[0] == "e") + 4 * (s == "e") + 16 * (s[1] == "x");
  endfunction
endclass
module top;
  initial begin
    scope_stack st;
    C c;
    int d1, d2, d3, n;
    st = new;
    c = new;
    st.down("top"); st.down("e"); st.down("[0]"); st.down("a");
    st.up(".");
    d1 = st.depth();
    st.up(".");
    d2 = st.depth();
    st.down("[1]");
    st.up("[");
    d3 = st.depth();
    $display("T|depth %0d %0d %0d", d1, d2, d3);
    $display("T|probe %0d %0d %0d", c.probe("ex"), c.probe("[0]"), c.probe("e"));
    for (int i = 0; i < 3000; i++) n += c.probe("ex");
    $display("T|hot %0d", n);
  end
endmodule
"#;

fn check() {
    let sim = simulate(SRC, 10).expect("simulate failed");
    let o: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    for want in ["T|depth 3 1 1", "T|probe 19 0 7", "T|hot 57000"] {
        assert!(o.iter().any(|l| l == want), "missing {want:?} in {o:?}");
    }
}

#[test]
fn compiled_string_local_byte_compare() {
    check();
}

#[test]
fn compiled_string_local_byte_compare_eager() {
    if super::compiled_method_test_env::eager() {
        check();
    }
}
