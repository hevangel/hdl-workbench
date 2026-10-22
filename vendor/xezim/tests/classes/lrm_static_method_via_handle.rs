//! IEEE 1800-2023 §8.25 / §8.10: a static method called through a handle of a specialized class runs in that specialization (`St#(2) c; c.depth()` reads N=2).
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

const SVH: &str = r#"
// top: t8_25b
class St #(int N = 4); static function int depth(); return N; endfunction function int d2(); return N; endfunction endclass
module t8_25b;
  St#(2) c;
  initial begin c = new; $display("T|a|handle.static=%0d handle.nonstatic=%0d scoped=%0d", c.depth(), c.d2(), St#(2)::depth()); end
endmodule
"#;

const SVH_FAMILY: &str = r#"
class St #(int N = 4);
  static int cnt = N;
  static function int depth(); return N; endfunction
  function int d2(); return N; endfunction
  static function int getcnt(); return cnt; endfunction
endclass
typedef St#(7) St7;
module t;
  St#(2) c; St d; St7 e; St#(3) f;
  initial begin c = new; d = new; e = new; f = new;
    $display("T|a|c.depth=%0d c.d2=%0d scoped=%0d d.depth=%0d e.depth=%0d f.depth=%0d", c.depth(), c.d2(), St#(2)::depth(), d.depth(), e.depth(), f.depth());
    $display("T|b|c.cnt=%0d d.cnt=%0d e.cnt=%0d St2cnt=%0d c.getcnt=%0d", c.cnt, d.cnt, e.cnt, St#(2)::cnt, c.getcnt());
    c.cnt = 11;
    $display("T|c|c.cnt=%0d St2cnt=%0d d.cnt=%0d f.cnt=%0d", c.cnt, St#(2)::cnt, d.cnt, f.cnt);
  end
endmodule
"#;

#[test]
fn audit_repro() {
    let want = ["T|a|handle.static=2 handle.nonstatic=2 scoped=2"];
    assert_eq!(t_lines(SVH), want);
}

#[test]
fn statics_through_handles() {
    let want = [
        "T|a|c.depth=2 c.d2=2 scoped=2 d.depth=4 e.depth=7 f.depth=3",
        "T|b|c.cnt=2 d.cnt=4 e.cnt=7 St2cnt=2 c.getcnt=2",
        "T|c|c.cnt=11 St2cnt=11 d.cnt=4 f.cnt=3",
    ];
    assert_eq!(t_lines(SVH_FAMILY), want);
}
