//! §8.10: inside a method a property name names the object's member even when
//! an unrelated package declares a variable of the same name (#312). Expected
//! values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    let mut v: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    v.sort();
    v
}

/// Element member reads of an associative property of structs.
#[test]
fn class_assoc_member_shadows_package_var() {
    const SRC: &str = r#"package other;
  int m_td [int][string];
  int m_q [$];
endpackage
typedef struct { int x; } el_t;
class bag;
  el_t m_td [string];
  int m_q [$];
  function void fill(); el_t e; e.x = 11; m_td["a"] = e; e.x = 22; m_td["b"] = e; m_q.push_back(4); endfunction
  function int sum(); int s = 0; foreach (m_td[k]) s += m_td[k].x; return s; endfunction
  function int rd(); return m_td["b"].x; endfunction
  function int rd2(); el_t e; e = m_td["b"]; return e.x; endfunction
  function int rd3(); string k = "b"; return m_td[k].x; endfunction
  function int qs(); return m_q.size() * 10 + m_q[0]; endfunction
endclass
module tb;
  initial begin
    bag b = new();
    b.fill();
    $display("T|sum=%0d rd=%0d q=%0d", b.sum(), b.rd(), b.qs());
    $display("T|rd2=%0d rd3=%0d", b.rd2(), b.rd3());
  end
endmodule
"#;
    assert_eq!(t_lines(SRC), ["T|rd2=22 rd3=22", "T|sum=33 rd=22 q=14",]);
}
