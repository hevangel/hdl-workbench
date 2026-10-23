use xezim::simulate;

fn outputs(src: &str, max_time: u64) -> Vec<String> {
    simulate(src, max_time)
        .expect("simulate")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn nonzero_ascending_class_labels_and_indexed_constraints() {
    let msgs = outputs(
        r#"
class label_probe;
  rand bit [8:23] word;
  constraint sections {
    word[8 +: 8] == 8'h31;
    word[23 -: 8] == 8'h27;
  }
endclass
module label_check;
  initial begin
    label_probe obj = new();
    int errors;
    obj.word = '0;
    obj.word[8 +: 8] = 8'h31;
    obj.word[23 -: 8] = 8'h27;
    if (obj.word !== 16'h3127 || obj.word[8] !== 1'b0 ||
        obj.word[10] !== 1'b1 || obj.word[23] !== 1'b1) errors++;
    repeat (16) begin
      if (!obj.randomize()) errors++;
      else if (obj.word !== 16'h3127) errors++;
    end
    $display("T labels errors=%0d", errors);
  end
endmodule
"#,
        100_000,
    );
    assert!(msgs.iter().any(|m| m == "T labels errors=0"), "{msgs:#?}");
}

#[test]
fn rand_scope_joint_selects_and_sized_foreach() {
    let msgs = outputs(
        r#"
module scope_probe;
  bit [7:0] word;
  bit [3:0] nibble;
  int entries[];
  int failures;
  initial begin
    repeat (16) begin
      if (!std::randomize(word, nibble) with {
        word[7:4] == 3;
        word[3:0] == nibble;
        nibble inside {[2:5]};
      }) failures++;
      else if (word[7:4] != 3 || word[3:0] != nibble ||
               nibble < 2 || nibble > 5) failures++;
    end
    if (!std::randomize(entries) with {
      entries.size() == 3;
      foreach (entries[k]) entries[k] == k + 10;
    }) failures++;
    if (entries.size() != 3) failures++;
    else foreach (entries[k]) if (entries[k] != k + 10) failures++;
    $display("T scope failures=%0d", failures);
  end
endmodule
"#,
        100_000,
    );
    assert!(msgs.iter().any(|m| m == "T scope failures=0"), "{msgs:#?}");
}

#[test]
fn rand_class_ascending_and_masked_selects() {
    let msgs = outputs(
        r#"
class select_probe;
  rand bit [0:15] word;
  rand bit [7:0] mask_value;
  constraint selection {
    word[0:7] == 8'h31;
    word[8:15] == 8'h27;
    mask_value == 8'h20;
    (word[8:15] & 8'hf0) == mask_value;
  }
endclass
module probe;
  initial begin
    select_probe obj = new();
    int failures;
    obj.word = '0;
    obj.word[0:7] = 8'h31;
    obj.word[8 +: 8] = 8'h27;
    if (obj.word !== 16'h3127 || obj.word[0] !== 1'b0 ||
        obj.word[2] !== 1'b1 || obj.word[15] !== 1'b1) failures++;
    obj.word[0] = 1'b1;
    if (obj.word !== 16'hb127) failures++;
    obj.word[0] = 1'b0;
    obj.word[15 -: 8] = 8'h27;
    if (obj.word !== 16'h3127) failures++;
    repeat (16) begin
      if (!obj.randomize()) failures++;
      else if (obj.word !== 16'h3127 || obj.mask_value !== 8'h20) failures++;
    end
    $display("T selects failures=%0d", failures);
  end
endmodule
"#,
        100_000,
    );
    assert!(
        msgs.iter().any(|m| m == "T selects failures=0"),
        "{msgs:#?}"
    );
}
