//! §7.9.1: `aa.size` written without parentheses on an associative array
//! returns the number of entries (sv-tests `associative-arrays-size`,
//! `-delete`, `-allocating-elements`, `-access-nonexistent`). Since the
//! associative array gained a `.size` comb-dependency proxy signal — which
//! always holds 0 — the dotted-name lookups read that proxy first, so the
//! paren-less `size` was 0 in every context while `size()` and `num` were
//! right. Cross-checked against the reference simulator.

fn lines(src: &str) -> Vec<String> {
    xezim::simulate(src, 10)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn noparen_size_counts_entries() {
    let o = lines(
        r#"
module top;
  int a1 [ int ];
  bit [7:0] a2 [ string ];
  int a3 [*];
  int x;
  function int f(); return a1.size; endfunction
  initial begin
    $display("S|%0d", a1.size);
    a1[10] = 10; a1[16'hffff] = 2; a2["x"] = 1; a3[5] = 1;
    x = a1.size;
    $display("S|%0d %0d %0d %0d %0d", x, f(), a1.size, a2.size, a3.size);
    a1.delete(10);
    $display("S|%0d %0d", a1.size, a1.num);
    if (a2.size == 1) $display("S|cmp");
  end
endmodule
"#,
    );
    let s: Vec<&String> = o.iter().filter(|l| l.starts_with("S|")).collect();
    assert_eq!(s, vec!["S|0", "S|2 2 2 1 1", "S|1 1", "S|cmp"], "{o:?}");
}

#[test]
fn noparen_size_of_queue_and_dynamic_array_unchanged() {
    let o = lines(
        r#"
module top;
  int q[$];
  int d[];
  initial begin
    q.push_back(3); q.push_back(4); d = new[5];
    $display("Q|%0d %0d", q.size, d.size);
  end
endmodule
"#,
    );
    assert!(o.iter().any(|l| l == "Q|2 5"), "{o:?}");
}
