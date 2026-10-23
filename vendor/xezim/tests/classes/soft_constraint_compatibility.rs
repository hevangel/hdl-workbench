use xezim::simulate;

fn check(source: &str) {
    let result = simulate(source, 100).expect("simulate");
    let output: Vec<_> = result
        .output
        .iter()
        .map(|entry| entry.message.as_str())
        .collect();
    assert_eq!(output, ["T|errors=0"]);
}

const SCALAR: &str = r#"
class preferred_values;
  rand int standalone, bounded, reversed, overlap;
  constraint choices {
    soft standalone == 5;
    bounded >= 0;
    soft bounded == 5;
    0 <= reversed;
    soft reversed == 5;
    soft overlap inside {[0:10]};
    soft overlap inside {[5:15]};
    overlap inside {[7:8]};
  }
endclass
module scalar_probe;
  initial begin
    preferred_values sample = new();
    int errors = 0;
    repeat (32) begin
      if (!sample.randomize()) errors++;
      if (sample.standalone != 5 || sample.bounded != 5 || sample.reversed != 5) errors++;
      if (!(sample.overlap inside {[7:8]})) errors++;
    end
    $display("T|errors=%0d", errors);
  end
endmodule
"#;

#[test]
fn compatible_hard_and_soft_domains_are_retained() {
    check(SCALAR);
}

const PRIORITY: &str = r#"
class inherited_preferences;
  rand int selected, retained, trailing;
  constraint defaults {
    soft selected == 11;
    soft retained == 47;
    soft trailing == 21;
    soft trailing == 22;
  }
endclass
class refined_preferences extends inherited_preferences;
  constraint refinement { soft selected == 13; selected >= 0; }
endclass
module priority_probe;
  initial begin
    refined_preferences sample = new();
    int errors = 0;
    repeat (16) begin
      if (!sample.randomize()) errors++;
      if (sample.selected != 13 || sample.retained != 47 || sample.trailing != 22) errors++;
      if (!sample.randomize() with { selected == 19; }) errors++;
      if (sample.selected != 19 || sample.retained != 47 || sample.trailing != 22) errors++;
      if (!sample.randomize() with { soft selected == 23; }) errors++;
      if (sample.selected != 23 || sample.retained != 47 || sample.trailing != 22) errors++;
    end
    $display("T|errors=%0d", errors);
  end
endmodule
"#;

#[test]
fn conflicting_preferences_do_not_drop_unrelated_defaults() {
    check(PRIORITY);
}

const FIXED: &str = r#"
class conditional_preferences;
  bit relaxed;
  rand int entries[3];
  constraint limits {
    if (!relaxed) { foreach (entries[index]) if (index != 1) entries[index] inside {[0:15]}; }
  }
  constraint defaults { foreach (entries[index]) soft entries[index] == -1; }
endclass
class contradictory_limits;
  bit enabled = 1;
  rand int entries[2];
  constraint impossible {
    if (enabled) { foreach (entries[index]) { entries[index] == 1; entries[index] == 2; } }
  }
endclass
module fixed_probe;
  initial begin
    conditional_preferences sample = new();
    contradictory_limits invalid = new();
    int errors = 0;
    repeat (16) begin
      sample.relaxed = 0;
      if (!sample.randomize()) errors++;
      if (!(sample.entries[0] inside {[0:15]}) || !(sample.entries[2] inside {[0:15]})) errors++;
      if (sample.entries[1] != -1) errors++;
      sample.relaxed = 1;
      if (!sample.randomize()) errors++;
      foreach (sample.entries[index]) if (sample.entries[index] != -1) errors++;
    end
    if (invalid.randomize() != 0) errors++;
    $display("T|errors=%0d", errors);
  end
endmodule
"#;

#[test]
fn conditional_foreach_hard_constraints_win_per_element() {
    check(FIXED);
}

const DYNAMIC: &str = r#"
class resized_preferences;
  bit active;
  rand int entries[];
  constraint shape { entries.size() == 3; }
  constraint limits { if (active) { foreach (entries[index]) if (index != 1) entries[index] inside {[0:15]}; } }
  constraint defaults { foreach (entries[index]) soft entries[index] == -1; }
endclass
module dynamic_probe;
  initial begin
    resized_preferences sample = new();
    int errors = 0;
    repeat (8) begin
      sample.active = 1;
      if (!sample.randomize()) errors++;
      if (sample.entries.size() != 3) errors++;
      if (!(sample.entries[0] inside {[0:15]}) || !(sample.entries[2] inside {[0:15]})) errors++;
      if (sample.entries[1] != -1) errors++;
      sample.active = 0;
      if (!sample.randomize()) errors++;
      foreach (sample.entries[index]) if (sample.entries[index] != -1) errors++;
    end
    $display("T|errors=%0d", errors);
  end
endmodule
"#;

#[test]
fn conditional_dynamic_array_preferences_preserve_hard_domains() {
    check(DYNAMIC);
}
