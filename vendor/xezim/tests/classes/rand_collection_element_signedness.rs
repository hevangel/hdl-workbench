use xezim::simulate;

#[test]
fn negative_collection_draws_keep_declared_signedness() {
    let src = r#"
class sample_record;
  rand int slots[2];
  rand int scalar;
  constraint bounds { foreach (slots[i]) slots[i] inside {[-3:-1]}; scalar inside {-3}; }
endclass
class shape_record;
  rand shortint dynamic_slots[];
  rand byte queue_slots[$];
  rand int keyed_slots[int];
  rand logic signed [4:0] matrix[2][2];
  rand int unsigned positive_slots[2];
  constraint bounds {
    dynamic_slots.size() == 2;
    queue_slots.size() == 2;
    foreach (dynamic_slots[i]) dynamic_slots[i] inside {[-7:-4]};
    foreach (queue_slots[i]) queue_slots[i] inside {[-8:-6]};
    foreach (keyed_slots[i]) keyed_slots[i] inside {[-3:-1]};
    foreach (matrix[i,j]) matrix[i][j] inside {[-5:-2]};
    foreach (positive_slots[i]) positive_slots[i] inside {[32'hffff_fffd:32'hffff_ffff]};
  }
  function new(); keyed_slots[-1] = 0; keyed_slots[4] = 0; endfunction
endclass
module top;
  initial begin
    sample_record sample = new();
    shape_record shaped = new();
    int errors = 0;
    for (int trial = 0; trial < 8; trial++) begin
      if (!sample.randomize()) errors++;
      foreach (sample.slots[i]) begin
        if (!(sample.slots[i] < 0)) errors++;
        if (!(sample.slots[i] inside {[-3:-1]})) errors++;
      end
      if (!shaped.randomize()) errors++;
      foreach (shaped.dynamic_slots[i]) if (!(shaped.dynamic_slots[i] < 0)) errors++;
      foreach (shaped.queue_slots[i]) if (!(shaped.queue_slots[i] < 0)) errors++;
      foreach (shaped.keyed_slots[i]) if (!(shaped.keyed_slots[i] < 0)) errors++;
      foreach (shaped.matrix[i,j]) if (!(shaped.matrix[i][j] < 0)) errors++;
      foreach (shaped.positive_slots[i]) if (shaped.positive_slots[i] < 0) errors++;
    end
    $display("errors=%0d scalar=%0d", errors, sample.scalar);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(out, ["errors=0 scalar=-3"]);
}

#[test]
fn collection_draws_format_signed_and_unsigned_elements() {
    let src = r#"
class number_record;
  rand int slots[2];
  rand byte narrow_slots[2];
  rand int unsigned positive_slots[2];
  constraint bounds {
    foreach (slots[i]) slots[i] inside {-3};
    foreach (narrow_slots[i]) narrow_slots[i] inside {-2};
    foreach (positive_slots[i]) positive_slots[i] == -1;
  }
endclass
module top;
  initial begin
    number_record numbers = new();
    int ok = numbers.randomize();
    $display("ok=%0d signed=%0d negative=%0d narrow=%0d unsigned=%0d nonnegative=%0d", ok,
      numbers.slots[0], numbers.slots[0] < 0, numbers.narrow_slots[0],
      numbers.positive_slots[0], numbers.positive_slots[0] >= 0);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(
        out,
        ["ok=1 signed=-3 negative=1 narrow=-2 unsigned=4294967295 nonnegative=1"]
    );
}
