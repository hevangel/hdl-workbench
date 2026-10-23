//! §8.3: a class property that is an unpacked array, selected inside a
//! compiled procedural block (`bk.arr[i]` in an `always` block).
//!
//! `bk.arr` resolves to no signal, so the bytecode compiler asked the AST
//! interpreter for its value and got ONE number back — an array does not
//! reduce to a scalar — then took a bit of that. The accumulator stayed 0
//! while `$display("%0d", bk.arr[i])` printed the right element, because the
//! display goes through the interpreter, which walks the whole select. The
//! compiler now hands the interpreter the entire select whenever the base is
//! an identifier only the interpreter can evaluate.
//!
//! Expected values from the reference simulator.
use xezim::simulate;

fn messages(src: &str) -> Vec<String> {
    let sim = simulate(src, 100_000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// The clocked block accumulates the LAST element the unrolled loop assigns
/// (all eight are non-blocking writes to the same target), so `acc2` grows by
/// `arr[7]` = 22 each of the twenty cycles.
#[test]
fn array_property_element_in_clocked_block() {
    let msgs = messages(
        "module tb;
  logic clk = 0; always #5 clk = ~clk;
  class bank;
    int prop = 7;
    int arr[8];
    function new(); foreach (arr[k]) arr[k] = k * 3 + 1; endfunction
  endclass
  bank bk = new();
  int acc = 0, acc2 = 0;
  int i;
  always @(posedge clk) begin
    for (i = 0; i < 8; i++) begin
      acc  <= acc + bk.prop;
      acc2 <= acc2 + bk.arr[i];
    end
  end
  initial begin
    repeat (20) @(posedge clk);
    #1 $display(\"acc=%0d acc2=%0d\", acc, acc2);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter().any(|m| m.contains("acc=140 acc2=440")),
        "class array property read as a scalar: {msgs:?}"
    );
}

/// A BIT of an element, and a part-select of one: the same base reaches the
/// bit-select and indexed-part-select paths, which each compiled their base
/// on its own.
#[test]
fn bit_and_part_select_of_array_property_element() {
    let msgs = messages(
        "module tb;
  logic clk = 0; always #5 clk = ~clk;
  class bank;
    int arr[4];
    function new(); foreach (arr[k]) arr[k] = 8 + k; endfunction
  endclass
  bank bk = new();
  int sel = 2;
  logic b = 0;
  logic [3:0] nib = 0, up = 0;
  always @(posedge clk) begin
    b   <= bk.arr[3][0];
    nib <= bk.arr[sel][3:0];
    up  <= bk.arr[1][0 +: 4];
  end
  initial begin
    repeat (2) @(posedge clk);
    #1 $display(\"b=%0b nib=%0d up=%0d\", b, nib, up);
    $finish;
  end
endmodule",
    );
    assert!(
        msgs.iter().any(|m| m.contains("b=1 nib=10 up=9")),
        "selects of a class array element: {msgs:?}"
    );
}
