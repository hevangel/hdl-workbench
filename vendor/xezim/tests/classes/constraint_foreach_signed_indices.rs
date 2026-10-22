//! Ordinary constraint foreach indices are signed ints (IEEE 1800 §12.7.3).

use xezim::simulate;

fn check(src: &str, expected: &str) {
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(out, [expected]);
}

#[test]
fn joint_solver_preserves_negative_indices_and_guards() {
    check(
        r#"
class index_record;
  rand int entries[-2:1];
  constraint limits {
    foreach (entries[i]) {
      soft entries[i] inside {[-20:20]};
      entries[i] == ((i < 0) ? i - 4 : i + 4);
    }
  }
endclass
module index_guard_tb;
  initial begin
    index_record sample = new();
    int accepted = sample.randomize();
    $display("T|joint %0d %0d %0d %0d %0d", accepted,
      sample.entries[-2], sample.entries[-1], sample.entries[0], sample.entries[1]);
    $finish;
  end
endmodule
"#,
        "T|joint 1 -6 -5 4 5",
    );
}

#[test]
fn object_array_constraints_keep_signed_indices() {
    check(
        r#"
class entry_record;
  rand int payload;
endclass
class object_matrix_record;
  rand entry_record entries[-1:0][1:2];
  function new(); foreach (entries[i,j]) entries[i][j] = new(); endfunction
  constraint limits { foreach (entries[i,j]) entries[i][j].payload == i * 10 + j; }
endclass
module object_index_tb;
  initial begin
    object_matrix_record sample = new();
    int accepted = sample.randomize();
    $display("T|objects %0d %0d %0d %0d %0d", accepted,
      sample.entries[-1][1].payload, sample.entries[-1][2].payload,
      sample.entries[0][1].payload, sample.entries[0][2].payload);
    $finish;
  end
endmodule
"#,
        "T|objects 1 -9 -8 1 2",
    );
}

#[test]
fn dynamic_row_sizes_and_elements_use_signed_index_arithmetic() {
    check(
        r#"
class dynamic_index_record;
  rand int entries[][];
  rand int queue_entries[$];
  constraint limits {
    entries.size() == 2;
    foreach (entries[i])
      if (i > -1) entries[i].size() == 2;
      else entries[i].size() == 1;
    foreach (entries[i,j]) entries[i][j] == ((j - 1 < 0) ? i + 10 : i + 20);
    queue_entries.size() == 2;
    foreach (queue_entries[i]) queue_entries[i] == ((i > -1) ? i + 30 : -1);
  }
endclass
module dynamic_index_tb;
  initial begin
    dynamic_index_record sample = new();
    int accepted = sample.randomize();
    $display("T|dynamic %0d %0d %0d %0d %0d %0d %0d %0d %0d", accepted,
      sample.entries[0].size(), sample.entries[1].size(),
      sample.entries[0][0], sample.entries[0][1],
      sample.entries[1][0], sample.entries[1][1],
      sample.queue_entries[0], sample.queue_entries[1]);
    $finish;
  end
endmodule
"#,
        "T|dynamic 1 2 2 10 20 11 21 30 31",
    );
}

#[test]
fn scope_randomize_preserves_negative_foreach_indices() {
    check(
        r#"
module scope_index_tb;
  int entries[-2:1];
  initial begin
    int accepted;
    accepted = std::randomize(entries) with {
      foreach (entries[i]) entries[i] == i - 3;
    };
    $display("T|scope %0d %0d %0d %0d %0d", accepted,
      entries[-2], entries[-1], entries[0], entries[1]);
    $finish;
  end
endmodule
"#,
        "T|scope 1 -5 -4 -3 -2",
    );
}

#[test]
fn packed_foreach_guards_use_signed_indices() {
    check(
        r#"
class packed_index_record;
  rand bit [3:0] entries;
  constraint limits {
    foreach (entries[i])
      if (i > -1) entries[i] == 1;
      else entries[i] == 0;
  }
endclass
module packed_index_tb;
  bit [3:0] entries;
  initial begin
    packed_index_record sample;
    int class_ok;
    int scope_ok;
    sample = new();
    class_ok = sample.randomize();
    scope_ok = std::randomize(entries) with {
      foreach (entries[i])
        if (i > -1) entries[i] == 1;
        else entries[i] == 0;
    };
    $display("T|packed %0d %0h %0d %0h", class_ok, sample.entries, scope_ok, entries);
    $finish;
  end
endmodule
"#,
        "T|packed 1 f 1 f",
    );
}
