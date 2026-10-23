//! IEEE 1800-2017 §6.20.2: a class-scope `localparam` / `parameter` carrying
//! an UNPACKED dimension declares a constant ARRAY.
//!
//! Class-body parameters parse as `ClassItem::Parameter`, and elaboration
//! stored each one as a single scalar `Signal` sized by `resolve_type_width`
//! — the declarator's unpacked dimension was dropped on the floor. The whole
//! array therefore collapsed into one element-width scalar, which broke four
//! things at once and all of them silently:
//!
//!   * `Arr[i]` became a BIT select of that scalar, so every element read 0
//!   * `$size(Arr)` returned the BIT width (32 for `int`, 96 for a 3-deep one)
//!   * the synchronous `foreach` ran exactly ONE iteration (i=0)
//!   * the async (blocking-body) `foreach` ran NONE at all
//!
//! A `const` array property — same shape, different keyword — always worked,
//! which is what makes the parameter form's failure a pure elaboration gap
//! rather than a missing runtime capability.

use xezim::simulate;

fn notes(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect()
}

const READ_AND_SIZE: &str = r#"
module top;
  class C;
    localparam int Arr[3] = '{10, 20, 30};
    function void show();
      for (int i = 0; i < 3; i++)
        $display("NOTE: read[%0d]=%0d", i, Arr[i]);
      $display("NOTE: size=%0d", $size(Arr));
    endfunction
  endclass
  initial begin
    C c; c = new(); c.show(); $finish;
  end
endmodule
"#;

/// §6.20.2 — element reads and `$size` see the declared array, not a scalar.
#[test]
fn class_localparam_array_elements_and_size() {
    assert_eq!(
        notes(READ_AND_SIZE),
        vec![
            "NOTE: read[0]=10",
            "NOTE: read[1]=20",
            "NOTE: read[2]=30",
            "NOTE: size=3",
        ],
        "a class localparam array must read per-ELEMENT; collapsing it to a \
         scalar makes Arr[i] a bit select (all 0) and $size report 32"
    );
}

const FOREACH_BOTH_PATHS: &str = r#"
`timescale 1ns/1ps
module top;
  logic clk = 0;
  always #5 clk = ~clk;
  class C;
    localparam int Arr[3] = '{10, 20, 30};
    // non-blocking body -> synchronous foreach path
    function void sync_walk();
      foreach (Arr[i]) $display("NOTE: sync[%0d]=%0d", i, Arr[i]);
    endfunction
    // blocking body -> async ForeachTail continuation path
    task blocking_walk();
      foreach (Arr[i]) begin
        @(posedge clk);
        $display("NOTE: async[%0d]=%0d", i, Arr[i]);
      end
    endtask
  endclass
  initial begin
    C c; c = new();
    c.sync_walk();
    c.blocking_walk();
    $finish;
  end
endmodule
"#;

/// §12.7.3 — `foreach` iterates the full declared range on BOTH the
/// synchronous path and the blocking-body (`ForeachTail`) path. These are
/// separate shape lookups in the simulator, so both need pinning.
#[test]
fn class_localparam_array_foreach_sync_and_async() {
    assert_eq!(
        notes(FOREACH_BOTH_PATHS),
        vec![
            "NOTE: sync[0]=10",
            "NOTE: sync[1]=20",
            "NOTE: sync[2]=30",
            "NOTE: async[0]=10",
            "NOTE: async[1]=20",
            "NOTE: async[2]=30",
        ],
        "with no recorded shape the sync path ran one iteration and the async \
         path ran zero — both without any diagnostic"
    );
}

const SIZED_BY_PARAM: &str = r#"
module top;
  class C;
    localparam int N = 4;
    localparam int Arr[N] = '{1, 2, 3, 4};
    function void show();
      $display("NOTE: size=%0d last=%0d", $size(Arr), Arr[N-1]);
    endfunction
  endclass
  initial begin
    C c; c = new(); c.show(); $finish;
  end
endmodule
"#;

/// §6.20.2 — the dimension may itself be an earlier class-body localparam;
/// it const-evaluates against the class constant scope.
#[test]
fn class_localparam_array_sized_by_earlier_localparam() {
    assert_eq!(notes(SIZED_BY_PARAM), vec!["NOTE: size=4 last=4"]);
}
