//! §26.3: `P::v` inside a module task or function names the package variable
//! as it does in a process: whole-array assignment, `new[n]`, element member
//! writes and locator methods (#312). Expected values are the reference
//! simulator's.

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

/// Associative array of structs, dynamic array and queue in a task.
#[test]
fn package_collections_in_task() {
    const SRC: &str = r#"package p;
  typedef struct { int x; } elem_t;
  elem_t aa [string];
  int d [];
  int q [$];
  int ia [string];
endpackage
module tb;
  task automatic t();
    p::elem_t e; int s = 0;
    e.x = 10; p::aa["a"] = e; e.x = 20; p::aa["b"] = e;
    $display("T|task read b.x=%0d num=%0d", p::aa["b"].x, p::aa.num());
    foreach (p::aa[k]) s += p::aa[k].x;
    $display("T|task sum=%0d", s);
    p::d = new[3]; foreach (p::d[i]) p::d[i] = i + 1;
    $display("T|task dyn size=%0d d2=%0d", p::d.size(), p::d[2]);
    p::q.push_back(5); p::q.push_back(2);
    $display("T|task q max=%0d size=%0d", p::q.max()[0], p::q.size());
    p::ia["z"] = 4; $display("T|task ia size=%0d first ok", $size(p::ia));
  endtask
  initial begin
    p::elem_t e; int s = 0;
    t();
    e.x = 1; p::aa["c"] = e;
    $display("T|init read c.x=%0d b.x=%0d", p::aa["c"].x, p::aa["b"].x);
    foreach (p::aa[k]) s += p::aa[k].x;
    $display("T|init sum=%0d", s);
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|init read c.x=1 b.x=20",
            "T|init sum=31",
            "T|task dyn size=3 d2=3",
            "T|task ia size=1 first ok",
            "T|task q max=5 size=2",
            "T|task read b.x=20 num=2",
            "T|task sum=30",
        ]
    );
}

/// `new[n]` from automatic and static tasks, a function and a block; `P::q.max()`.
#[test]
fn package_dyn_new_in_subroutines() {
    const SRC: &str = r#"package p;
  int d1 []; int d2 []; int d3 []; int d4 [];
  int q1 [$]; int q2 [$];
endpackage
module tb;
  task t_auto(); p::d1 = new[3]; p::q1.push_back(5); p::q1.push_back(2); endtask
  task static t_stat(); p::d2 = new[3]; endtask
  function automatic void f(); p::d3 = new[3]; endfunction
  initial begin
    t_auto(); t_stat(); f();
    begin : blk p::d4 = new[3]; end
    $display("T|d1=%0d d2=%0d d3=%0d d4=%0d", p::d1.size(), p::d2.size(), p::d3.size(), p::d4.size());
    $display("T|q1 max=%0d size=%0d", p::q1.max()[0], p::q1.size());
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        ["T|d1=3 d2=3 d3=3 d4=3", "T|q1 max=5 size=2",]
    );
}
