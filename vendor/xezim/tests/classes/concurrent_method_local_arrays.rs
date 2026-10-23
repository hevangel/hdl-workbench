//! §6.21 / §13.3 — a fixed-size array declared in a class method is
//! automatic: every invocation owns its storage. It was registered under its
//! bare name, so two sequences running `body()` concurrently shared one
//! `item arr[3]` — one saw the other's objects and the second declaration
//! re-seeded the first's elements. A pipelined UVM test read back the wrong
//! items. The expected lines are the reference simulator's output.

use xezim::simulate;

const SRC: &str = r#"
class item;
  int v;
endclass
class seq;
  int base;
  task body();
    item arr[3];
    int vals[3];
    for (int i = 0; i < 3; i++) begin
      arr[i] = new;
      arr[i].v = base + i;
      vals[i] = base + 10 + i;
      #1;
    end
    #5;
    for (int i = 0; i < 3; i++)
      $display("%0t base=%0d arr[%0d].v=%0d vals=%0d", $time, base, i, arr[i].v, vals[i]);
  endtask
endclass
module top;
  seq s1, s2;
  initial begin
    s1 = new; s1.base = 100;
    s2 = new; s2.base = 200;
    fork
      s1.body();
      s2.body();
    join
  end
endmodule
"#;

#[test]
fn concurrent_method_calls_get_their_own_local_arrays() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "8 base=100 arr[0].v=100 vals=110",
        "8 base=100 arr[1].v=101 vals=111",
        "8 base=100 arr[2].v=102 vals=112",
        "8 base=200 arr[0].v=200 vals=210",
        "8 base=200 arr[1].v=201 vals=211",
        "8 base=200 arr[2].v=202 vals=212",
    ];
    assert_eq!(out, want, "{out:?}");
}
