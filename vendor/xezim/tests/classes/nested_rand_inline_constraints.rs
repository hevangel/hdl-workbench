//! §18.5.9 / §18.7 — inline constraints on members of NESTED rand objects
//! (`rb.randomize() with { rb.ctrl.len.value inside {…}; ctrl.go.value ==
//! ie; }`, the shape a UVM register-model sequence uses on its block). The
//! enclosing solve only re-pinned a nested member for a one-level `==`; an
//! `inside`, a relational item, a two-level path or the receiver-prefixed
//! spelling was left to chance, every trial failed, and randomize() returned
//! 0 with the fields untouched — an SPI test then never selected a slave and
//! waited forever. The items that constrain only one sub-object now join
//! that object's own solve. Values are random, so the test checks that every
//! constraint holds; the reference simulator prints the same lines.

use xezim::simulate;

const SRC: &str = r#"
class fld;
  rand bit [63:0] value;
  int unsigned m_size;
  constraint valid { if (64 > m_size) { value < (64'h1 << m_size); } }
  function new(int unsigned sz); m_size = sz; endfunction
endclass
class reg_c;
  rand fld go;
  rand fld len;
  function new(); go = new(1); len = new(7); endfunction
endclass
class blk_c;
  rand reg_c ctrl;
  rand fld div;
  function new(); ctrl = new; div = new(16); endfunction
endclass
module top;
  blk_c rb;
  bit ie = 1;
  int ok;
  initial begin
    rb = new;
    repeat (20) begin
      ok = rb.randomize() with { rb.ctrl.go.value == ie;
                                 rb.ctrl.len.value inside {0, 1, [31:33], [63:65], 126, 127};
                                 rb.div.value inside {16'h0, 16'h1, 16'h2, 16'h4, 16'h80};
                                 ctrl.len.value != 32; };
      if (!ok || rb.ctrl.go.value != 1 || rb.ctrl.len.value == 32
          || !(rb.ctrl.len.value inside {0, 1, [31:33], [63:65], 126, 127})
          || !(rb.div.value inside {0, 1, 2, 4, 128}))
        $display("bad ok=%0d go=%0d len=%0d div=%0d", ok, rb.ctrl.go.value, rb.ctrl.len.value, rb.div.value);
    end
    ok = rb.randomize() with { ctrl.len.value > 120; div.value < 3; };
    $display("rel ok=%0d len_ok=%0d div_ok=%0d", ok, rb.ctrl.len.value > 120, rb.div.value < 3);
    $display("done");
  end
endmodule
"#;

#[test]
fn inline_constraints_reach_nested_rand_objects() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, vec!["rel ok=1 len_ok=1 div_ok=1", "done"], "{out:?}");
}
