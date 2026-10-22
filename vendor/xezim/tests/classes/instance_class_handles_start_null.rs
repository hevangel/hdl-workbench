//! §8.4 / §6.20.3 — an uninitialized class handle is `null` wherever it is
//! declared. A handle declared in an INTERFACE or a child MODULE (an
//! interrupt BFM's `proxy` handle is the usual case) read X: the null default
//! was applied to the root module's signals before the child instances were
//! inlined, so every `h == null` guard inside an instance evaluated to x and
//! took neither branch. The expected lines are the reference simulator's
//! output.

use xezim::simulate;

const SRC: &str = r#"
package ip;
  class util;
    int id;
  endclass
endpackage
class lutil;
  int id;
endclass
interface h_if;
  ip::util p1;
  lutil p2;
  initial #1 $display("%m: p1_null=%0d p2_null=%0d", p1 == null, p2 == null);
endinterface
module sub;
  ip::util p1;
  lutil p2;
  initial #1 $display("%m: p1_null=%0d p2_null=%0d", p1 == null, p2 == null);
endmodule
module top;
  h_if I();
  h_if IA [2]();
  sub S();
  lutil t;
  initial #2 $display("top: t_null=%0d", t == null);
endmodule
"#;

#[test]
fn class_handles_in_instances_default_to_null() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        vec![
            "top.I: p1_null=1 p2_null=1".to_string(),
            "top.IA[0]: p1_null=1 p2_null=1".to_string(),
            "top.IA[1]: p1_null=1 p2_null=1".to_string(),
            "top.S: p1_null=1 p2_null=1".to_string(),
            "top: t_null=1".to_string(),
        ],
        "{out:?}"
    );
}
