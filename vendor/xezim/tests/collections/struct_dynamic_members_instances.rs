//! §7.2.1: a dynamic member of an unpacked-struct variable in a module
//! instance, or in a second top module, is its own collection (#312).
//! Expected values are the reference simulator's.

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

/// Associative and queue members in a submodule.
#[test]
fn struct_dynamic_members_in_instance() {
    const SRC: &str = r#"typedef struct { int x; int q [string]; int d [$]; } h_t;
module sub; h_t h;
  initial begin h.q["a"] = 1; h.q["b"] = 2; h.d.push_back(5); #1 $display("T|sub hq=%0d hd=%0d", h.q.num(), h.d.size()); end
endmodule
module tb; sub u(); endmodule
"#;
    assert_eq!(t_lines(SRC), ["T|sub hq=2 hd=1",]);
}

/// The same declarations in two top modules stay separate.
#[test]
fn struct_dynamic_members_two_tops() {
    const SRC: &str = r#"typedef struct { int x; int q [string]; } h_t;
module ta; int v; h_t h; int aa [string];
  initial begin v = 1; h.x = 10; h.q["a"] = 1; aa["k1"] = 1; #1 $display("T|ta v=%0d hx=%0d hq=%0d aa=%0d", v, h.x, h.q.num(), aa.num()); end
endmodule
module tb; int v; h_t h; int aa [string];
  initial begin v = 2; h.x = 20; h.q["b"] = 1; h.q["c"] = 1; aa["k2"] = 1; aa["k3"] = 1; #1 $display("T|tb v=%0d hx=%0d hq=%0d aa=%0d", v, h.x, h.q.num(), aa.num()); end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        ["T|ta v=1 hx=10 hq=1 aa=1", "T|tb v=2 hx=20 hq=2 aa=2",]
    );
}
