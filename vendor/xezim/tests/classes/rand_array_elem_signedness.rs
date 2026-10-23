//! Signed collection draws preserve their declared element types.

use xezim::simulate;

#[test]
fn inherited_type_parameters_keep_retry_signedness() {
    let src = r#"
class typed_retry_base #(type ELEMENT = byte);
  rand ELEMENT entries[2:3][5:8];
  constraint limits {
    foreach (entries[i,j]) {
      entries[i][j] inside {[-3:-1]};
      entries[i][j] < 0;
      entries[i][j] % 2 == 0;
    }
  }
endclass
class byte_retry_record extends typed_retry_base #(byte); endclass
class short_retry_record extends typed_retry_base #(shortint); endclass
class wide_retry_record extends typed_retry_base #(longint); endclass
class packed_retry_record extends typed_retry_base #(logic signed [4:0]); endclass
module typed_retry_tb;
  initial begin
    byte_retry_record byte_item = new();
    short_retry_record short_item = new();
    wide_retry_record wide_item = new();
    packed_retry_record packed_item = new();
    int errors = 0;
    for (int trial = 1; trial <= 3; trial++) begin
      byte_item.srandom(trial);
      short_item.srandom(trial);
      wide_item.srandom(trial);
      packed_item.srandom(trial);
      if (!byte_item.randomize()) errors++;
      if (!short_item.randomize()) errors++;
      if (!wide_item.randomize()) errors++;
      if (!packed_item.randomize()) errors++;
      foreach (byte_item.entries[i,j]) if (byte_item.entries[i][j] !== -2) errors++;
      foreach (short_item.entries[i,j]) if (short_item.entries[i][j] !== -2) errors++;
      foreach (wide_item.entries[i,j]) if (wide_item.entries[i][j] !== -2) errors++;
      foreach (packed_item.entries[i,j]) if (packed_item.entries[i][j] !== -2) errors++;
    end
    $display("T|typed errors=%0d byte=%0d short=%0d wide=%0d packed=%0d", errors,
      byte_item.entries[2][5], short_item.entries[3][8],
      wide_item.entries[2][8], packed_item.entries[3][5]);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(out, ["T|typed errors=0 byte=-2 short=-2 wide=-2 packed=-2"]);
}

#[test]
fn unsigned_retry_candidates_keep_high_bit_values() {
    let src = r#"
class unsigned_retry_record;
  rand bit [7:0] narrow_entries[3:4][5:8];
  rand int unsigned wide_entries[2][4];
  constraint limits {
    foreach (narrow_entries[i,j]) {
      narrow_entries[i][j] inside {[253:255]};
      narrow_entries[i][j] % 2 == 0;
    }
    foreach (wide_entries[i,j]) {
      wide_entries[i][j] inside {[32'hffff_fffd:32'hffff_ffff]};
      wide_entries[i][j] % 2 == 0;
    }
  }
endclass
module unsigned_retry_tb;
  initial begin
    unsigned_retry_record sample = new();
    int errors = 0;
    for (int trial = 1; trial <= 3; trial++) begin
      sample.srandom(trial);
      if (!sample.randomize()) begin errors++; $display("T|randomize failed trial=%0d", trial); end
      foreach (sample.narrow_entries[i,j])
        if (sample.narrow_entries[i][j] !== 254 || sample.narrow_entries[i][j] < 0) begin errors++; $display("T|narrow trial=%0d index=%0d,%0d value=%0d", trial,i,j,sample.narrow_entries[i][j]); end
      foreach (sample.wide_entries[i,j])
        if (sample.wide_entries[i][j] !== 32'hffff_fffe || sample.wide_entries[i][j] < 0) begin errors++; $display("T|wide trial=%0d index=%0d,%0d value=%0d", trial,i,j,sample.wide_entries[i][j]); end
    end
    $display("T|unsigned errors=%0d narrow=%0d wide=%0d", errors,
      sample.narrow_entries[3][5], sample.wide_entries[1][3]);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(out, ["T|unsigned errors=0 narrow=254 wide=4294967294"]);
}

#[test]
fn negative_bounds_support_unsigned_retry_candidates() {
    let src = r#"
class unsigned_retry_record;
  rand bit [7:0] narrow_entries[3:4][-2:1];
  rand int unsigned wide_entries[2][4];
  constraint limits {
    foreach (narrow_entries[i,j]) {
      narrow_entries[i][j] inside {[253:255]};
      narrow_entries[i][j] % 2 == 0;
    }
    foreach (wide_entries[i,j]) {
      wide_entries[i][j] inside {[32'hffff_fffd:32'hffff_ffff]};
      wide_entries[i][j] % 2 == 0;
    }
  }
endclass
module unsigned_retry_tb;
  initial begin
    unsigned_retry_record sample = new();
    int errors = 0;
    for (int trial = 1; trial <= 3; trial++) begin
      sample.srandom(trial);
      if (!sample.randomize()) begin errors++; $display("T|randomize failed trial=%0d", trial); end
      foreach (sample.narrow_entries[i,j])
        if (sample.narrow_entries[i][j] !== 254 || sample.narrow_entries[i][j] < 0) begin errors++; $display("T|narrow trial=%0d index=%0d,%0d value=%0d", trial,i,j,sample.narrow_entries[i][j]); end
      foreach (sample.wide_entries[i,j])
        if (sample.wide_entries[i][j] !== 32'hffff_fffe || sample.wide_entries[i][j] < 0) begin errors++; $display("T|wide trial=%0d index=%0d,%0d value=%0d", trial,i,j,sample.wide_entries[i][j]); end
    end
    $display("T|unsigned errors=%0d narrow=%0d wide=%0d", errors,
      sample.narrow_entries[3][-2], sample.wide_entries[1][3]);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(out, ["T|unsigned errors=0 narrow=254 wide=4294967294"]);
}

#[test]
fn negative_matrix_candidates_satisfy_parity() {
    let src = r#"
class retry_record;
  rand int slots[8][8];
  constraint limits {
    foreach (slots[i,j]) {
      slots[i][j] inside {[-3:-1]};
      slots[i][j] < 0;
      slots[i][j] % 2 == 0;
    }
  }
endclass
module retry_probe_tb;
  initial begin
    retry_record sample = new();
    int errors = 0;
    int accepted = sample.randomize();
    foreach (sample.slots[i,j]) if (sample.slots[i][j] != -2) errors++;
    $display("T|accepted=%0d errors=%0d", accepted, errors);
    $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulation failed");
    let out: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    assert_eq!(out, ["T|accepted=1 errors=0"]);
}

const SRC: &str = r#"
class fixed_record; rand int slots[2]; constraint limits { foreach (slots[i]) slots[i] inside {[-3:-1]}; } endclass
class matrix_record; rand int slots[2][2]; constraint limits { foreach (slots[i,j]) slots[i][j] inside {[-3:-1]}; } endclass
class narrow_record; rand byte slots[2]; constraint limits { foreach (slots[i]) slots[i] inside {[-3:-1]}; } endclass
class dynamic_record; rand int slots[]; constraint limits { slots.size() == 2; foreach (slots[i]) slots[i] inside {[-3:-1]}; } endclass
class queue_record; rand int slots[$]; constraint limits { slots.size() == 2; foreach (slots[i]) slots[i] inside {[-3:-1]}; } endclass
class parity_record; rand int slots[2]; constraint limits { foreach (slots[i]) { slots[i] < 0; slots[i] > -100; slots[i] % 2 == 0; } } endclass
class ordered_record; rand int slots[2]; constraint limits { foreach (slots[i]) slots[i] inside {[-10:10]}; slots[0] < slots[1]; slots[0] < 0; } endclass
class unsigned_record; rand bit [7:0] slots[2]; constraint limits { foreach (slots[i]) slots[i] inside {[250:255]}; } endclass
module review_array_tb;
  int errors;
  initial begin
    fixed_record fixed_item = new(); matrix_record matrix_item = new();
    narrow_record narrow_item = new(); dynamic_record dynamic_item = new();
    queue_record queue_item = new(); parity_record parity_item = new();
    ordered_record ordered_item = new(); unsigned_record unsigned_item = new();
    for (int trial = 0; trial < 20; trial++) begin
      if (!fixed_item.randomize()) errors++;
      foreach (fixed_item.slots[i]) if (!(fixed_item.slots[i] < 0 && fixed_item.slots[i] >= -3)) errors++;
      if (!matrix_item.randomize()) errors++;
      foreach (matrix_item.slots[i,j]) if (!(matrix_item.slots[i][j] < 0 && matrix_item.slots[i][j] >= -3)) errors++;
      if (!narrow_item.randomize()) errors++;
      foreach (narrow_item.slots[i]) if (!(narrow_item.slots[i] < 0 && narrow_item.slots[i] >= -3)) errors++;
      if (!dynamic_item.randomize()) errors++;
      foreach (dynamic_item.slots[i]) if (!(dynamic_item.slots[i] < 0 && dynamic_item.slots[i] >= -3)) errors++;
      if (!queue_item.randomize()) errors++;
      foreach (queue_item.slots[i]) if (!(queue_item.slots[i] < 0 && queue_item.slots[i] >= -3)) errors++;
      if (!parity_item.randomize()) errors++;
      foreach (parity_item.slots[i]) if (!(parity_item.slots[i] < 0 && parity_item.slots[i] > -100 && parity_item.slots[i] % 2 == 0)) errors++;
      if (!ordered_item.randomize() || !(ordered_item.slots[0] < 0 && ordered_item.slots[0] < ordered_item.slots[1] && ordered_item.slots[0] >= -10 && ordered_item.slots[1] <= 10)) errors++;
      if (!unsigned_item.randomize()) errors++;
      foreach (unsigned_item.slots[i]) if (!(unsigned_item.slots[i] >= 250)) errors++;
    end
    $display("T|errors=%0d fixed=%0d dynamic=%0d unsigned=%0d", errors, fixed_item.slots[0] < 0, dynamic_item.slots[0] < 0, unsigned_item.slots[0] > 0);
    $finish;
  end
endmodule
"#;

#[test]
fn rand_signed_array_elements_keep_their_sign() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, ["T|errors=0 fixed=1 dynamic=1 unsigned=1"], "{out:?}");
}
