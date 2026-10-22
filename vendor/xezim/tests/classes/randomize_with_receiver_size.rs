//! §18.7 — `req.randomize() with { req.data.size() == N; … }`: a
//! receiver-qualified `size()` sizes the object's own array, like the bare
//! `data.size()`. It was ignored, and with element constraints next to it
//! `randomize()` returned 1 with the size violated. A size that conflicts
//! with the class constraints must make `randomize()` return 0. Expected
//! counts from the reference simulator.
use xezim::simulate;

fn tagged(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000).expect("simulate failed");
    sim.output
        .iter()
        .filter_map(|o| o.message.strip_prefix("T|").map(str::to_string))
        .collect()
}

const SIZE_ELEM: &str = r#"
module top;
  class T;
    rand bit [7:0] data[];
    constraint c { data.size() inside {[2:5]}; }
  endclass
  class S;
    T req;
    task body();
      int bad1 = 0, bad2 = 0, bad3 = 0, f1 = 0, f2 = 0, f3 = 0;
      req = new;
      repeat (20) begin
        if (!req.randomize() with { req.data.size() == 4; }) f1++;
        else if (req.data.size() != 4) bad1++;
        if (!req.randomize() with { req.data.size() == 3; foreach (req.data[i]) req.data[i] == i; }) f2++;
        else if (req.data.size() != 3 || req.data[2] != 2) bad2++;
        if (!req.randomize() with { req.data.size() == 3; req.data[0] == 8'h7; }) f3++;
        else if (req.data.size() != 3 || req.data[0] != 7) bad3++;
      end
      $display("T|size_only fail=%0d bad=%0d", f1, bad1);
      $display("T|size_foreach fail=%0d bad=%0d", f2, bad2);
      $display("T|size_elem fail=%0d bad=%0d", f3, bad3);
    endtask
  endclass
  initial begin S s = new; s.body(); end
endmodule
"#;

const SIZE_SHAPES: &str = r#"
module top;
  class T;
    rand bit [7:0] data[];
    rand bit [7:0] q[$];
    rand int unsigned n;
    constraint c { data.size() inside {[2:5]}; q.size() < 4; }
  endclass
  class S;
    T req;
    task body();
      int f1 = 0, b1 = 0, f2 = 0, b2 = 0, f3 = 0, b3 = 0, f4 = 0, b4 = 0, f5 = 0, b5 = 0;
      req = new;
      repeat (3) begin
        // conflicts with the class range: must fail, never return 1
        if (!req.randomize() with { req.data.size() == 7; }) f1++;
        else if (req.data.size() != 7) b1++;
        // relational + queue
        if (!req.randomize() with { req.data.size() > 3; req.q.size() == 2; }) f2++;
        else if (req.data.size() <= 3 || req.q.size() != 2) b2++;
        // coupling to a rand scalar
        if (!req.randomize() with { n == req.data.size(); n < 4; }) f3++;
        else if (req.n != req.data.size() || req.n >= 4) b3++;
        // inside on the prefixed size
        if (!req.randomize() with { req.data.size() inside {2, 5}; }) f4++;
        else if (!(req.data.size() inside {2, 5})) b4++;
        // plain spelling of the conflict, for comparison
        if (!req.randomize() with { data.size() == 7; }) f5++;
        else if (req.data.size() != 7) b5++;
      end
      $display("T|conflict fail=%0d bad=%0d", f1, b1);
      $display("T|rel_q fail=%0d bad=%0d", f2, b2);
      $display("T|couple fail=%0d bad=%0d", f3, b3);
      $display("T|inside fail=%0d bad=%0d", f4, b4);
      $display("T|plain_conflict fail=%0d bad=%0d", f5, b5);
    endtask
  endclass
  initial begin S s = new; s.body(); end
endmodule
"#;

#[test]
fn receiver_qualified_size_is_honoured() {
    assert_eq!(
        tagged(SIZE_ELEM),
        [
            "size_only fail=0 bad=0",
            "size_foreach fail=0 bad=0",
            "size_elem fail=0 bad=0"
        ]
    );
}

#[test]
fn receiver_qualified_size_shapes() {
    assert_eq!(
        tagged(SIZE_SHAPES),
        [
            "conflict fail=3 bad=0",
            "rel_q fail=0 bad=0",
            "couple fail=0 bad=0",
            "inside fail=0 bad=0",
            "plain_conflict fail=3 bad=0"
        ]
    );
}
