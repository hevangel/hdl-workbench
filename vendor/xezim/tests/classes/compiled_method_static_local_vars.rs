//! IEEE 1800-2023 §6.21: a `static` variable declared inside a class method
//! (or a package function) is one cell shared by every call. The cell must
//! keep counting when the method switches from the interpreter to bytecode
//! (the call-count tier), when a task has `output` formals next to it, and
//! when the subroutine re-enters itself.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn all_policies() -> bool {
    super::compiled_method_test_env::policies(&[("1", "0"), ("1", "1000"), ("0", "1000")])
}

#[test]
fn static_local_counts_across_the_tier_switch() {
    if !all_policies() {
        return;
    }
    let src = r#"
module top;
  class C;
    int seen;
    function int next();
      static int tick;
      tick++;
      return tick;
    endfunction
    task tcount();
      static int tt;
      tt += 2;
      seen = tt;
    endtask
  endclass
  class D extends C; endclass
  function automatic int pkg_next();
    static int k;
    k += 3;
    return k;
  endfunction
  C c, c2; D d;
  int s1, s2, s3;
  initial begin
    c = new(); c2 = new(); d = new();
    for (int i = 0; i < 1500; i++) begin s1 = c.next(); s1 = c2.next(); s1 = d.next(); end
    $display("T|next=%0d", s1);
    for (int i = 0; i < 1500; i++) s2 = d.next();
    $display("T|then_d next=%0d", s2);
    for (int i = 0; i < 5; i++) c.tcount();
    $display("T|tcount seen=%0d", c.seen);
    for (int i = 0; i < 1200; i++) s3 = pkg_next();
    $display("T|pkg_next=%0d", s3);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|next=4500",
            "T|then_d next=6000",
            "T|tcount seen=10",
            "T|pkg_next=3600"
        ]
    );
}

#[test]
fn static_local_next_to_output_formals() {
    if !all_policies() {
        return;
    }
    let src = r#"
module top;
  class C;
    task ta(output int o); static int tq; tq += 3; o = 7; endtask
    task tb(output int o); int loc; loc = 3; o = loc; endtask
    function void fa(output int o); static int fq; fq += 3; o = fq; endfunction
    task tc(output int o, input int k); static int tr; tr += k; o = tr; endtask
    task td(input int k, output int o); static int ts; ts += k; o = ts; endtask
  endclass
  C c; int a, b, f, x, y;
  initial begin
    c = new();
    repeat (2) begin c.ta(a); c.tb(b); c.fa(f); c.tc(x, 4); c.td(5, y); end
    $display("T|ta=%0d tb=%0d fa=%0d tc=%0d td=%0d", a, b, f, x, y);
    for (int i = 0; i < 1200; i++) begin c.fa(f); c.td(1, y); end
    $display("T|fa=%0d td=%0d", f, y);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|ta=7 tb=3 fa=6 tc=8 td=10", "T|fa=3606 td=1210"]
    );
}

#[test]
fn static_local_seen_by_a_recursive_call() {
    if !all_policies() {
        return;
    }
    let src = r#"
module top;
  class R;
    function int depth_count(int n);
      static int calls;
      calls++;
      if (n > 0) void'(depth_count(n - 1));
      return calls;
    endfunction
  endclass
  int r;
  initial begin
    R x = new;
    for (int i = 0; i < 1200; i++) r = x.depth_count(2);
    $display("T|calls=%0d", r);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|calls=3600"]);
}
