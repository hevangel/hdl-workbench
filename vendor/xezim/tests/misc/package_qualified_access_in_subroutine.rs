//! Package-qualified variables read and written inside tasks and functions.
//! A write like `store_pkg::scalar = d;` under a task/function frame went
//! through the dotted-name path as `store_pkg.scalar` and vanished, and a
//! read of `pkg::arr[i]` resolved its base only for a plain identifier and
//! returned 0 — both worked from an initial block, which is why they were
//! easy to miss. Package variables live in the table under their bare name;
//! both sides now strip the prefix and re-enter.
use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

#[test]
fn package_writes_inside_task_and_function_land() {
    let src = r#"
package store_pkg;
  int scalar = 0;
  int arr[4] = '{0,0,0,0};
  logic [7:0] vec = 0;
  int counter = 0;
endpackage
module top;
  import store_pkg::counter;
  task automatic write_all(input int d);
    store_pkg::scalar = d;
    store_pkg::arr[2] = d + 1;
    store_pkg::vec[3:0] = 4'hA;
    counter = d + 2;
    $display("IN_TASK scalar=%0d arr2=%0d vec=%h counter=%0d", store_pkg::scalar, store_pkg::arr[2], store_pkg::vec, counter);
  endtask
  function automatic int write_fn(input int d);
    store_pkg::scalar = d * 10;
    return store_pkg::scalar;
  endfunction
  initial begin
    write_all(5);
    $display("AFTER_TASK scalar=%0d arr2=%0d vec=%h counter=%0d", store_pkg::scalar, store_pkg::arr[2], store_pkg::vec, store_pkg::counter);
    $display("FN_RET=%0d AFTER_FN scalar=%0d", write_fn(7), store_pkg::scalar);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let msgs = outs(&sim);
    for want in [
        "IN_TASK scalar=5 arr2=6 vec=0a counter=7",
        "AFTER_TASK scalar=5 arr2=6 vec=0a counter=7",
        "FN_RET=70 AFTER_FN scalar=70",
    ] {
        assert!(
            msgs.iter().any(|m| m == want),
            "missing {want}; got {msgs:?}"
        );
    }
}

#[test]
fn package_array_reads_inside_subroutines() {
    let src = r#"
package rd_pkg;
  int arr[4] = '{10,11,12,13};
  int scalar = 42;
endpackage
module top;
  int k = 2;
  task automatic rd();
    int local_copy;
    local_copy = rd_pkg::arr[2];
    $display("TASK arr2=%0d arr_k=%0d scalar=%0d copy=%0d", rd_pkg::arr[2], rd_pkg::arr[k], rd_pkg::scalar, local_copy);
  endtask
  function automatic int rdf();
    return rd_pkg::arr[3];
  endfunction
  initial begin
    rd();
    $display("FN=%0d", rdf());
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let msgs = outs(&sim);
    for want in ["TASK arr2=12 arr_k=12 scalar=42 copy=12", "FN=13"] {
        assert!(
            msgs.iter().any(|m| m == want),
            "missing {want}; got {msgs:?}"
        );
    }
}

#[test]
fn block_local_integer_shadows_instance_variable() {
    // A block-local `integer k` inside an inlined instance must not clobber
    // the instance's own module-level `k`, and the loop must iterate.
    let src = r#"
module leaf(input logic [7:0] in, output logic [7:0] out);
  integer k = 5;
  always_comb begin : b
    integer k;
    integer sum;
    sum = 0;
    for (k = 0; k < 4; k = k + 1) sum = sum + in[k];
    out = sum[7:0];
  end
endmodule
module top;
  logic [7:0] in = 8'b0000_1011, out;
  leaf u(.in(in), .out(out));
  initial begin
    #1 $display("out=%0d u.k=%0d", out, u.k);
    in = 8'b0000_0111;
    #1 $display("out=%0d u.k=%0d", out, u.k);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let msgs = outs(&sim);
    assert!(msgs.iter().any(|m| m == "out=3 u.k=5"), "got {msgs:?}");
    assert_eq!(
        msgs.iter().filter(|m| m.as_str() == "out=3 u.k=5").count(),
        2,
        "got {msgs:?}"
    );
}
