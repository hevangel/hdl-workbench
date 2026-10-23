//! §6.19.6/§12.7.3: enum-typed associative-array KEYS on CLASS PROPERTIES —
//! reference-validated (open_issues §3b; the UVM severity-summary bug).
//!
//! Two stacked fixes:
//! 1. The class-property declaration path now records the key's NAMED type
//!    (`ElaboratedClass::assoc_key_types`), and the `foreach` binding falls
//!    back to the `this` instance's class hierarchy when the module map
//!    misses.
//! 2. The binding is TRUSTED for the loop's duration: the foreach index is
//!    a fresh declaration (§12.7.3), so its type shadows even a same-named
//!    property of an ancestor class — UVM's summary got the type of an
//!    unrelated ancestor property and printed members of the wrong enum.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// Reference: SEV_INFO=5 and SEV_ERR=9 (member names of the KEY enum).
#[test]
fn class_property_assoc_enum_key_names() {
    let src = r#"
module tb;
  typedef enum bit [1:0] { SEV_INFO, SEV_WARN, SEV_ERR } sev_e;

  class counter_t;
    int cnt [sev_e];
    function void bump();
      cnt[SEV_INFO] = 5;
      cnt[SEV_ERR]  = 9;
    endfunction
    function void show();
      foreach (cnt[s])
        $display("T|key=%s val=%0d", s.name(), cnt[s]);
    endfunction
  endclass

  counter_t c;
  initial begin
    c = new;
    c.bump();
    c.show();
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"T|key=SEV_INFO val=5".to_string()), "{o:?}");
    assert!(o.contains(&"T|key=SEV_ERR val=9".to_string()), "{o:?}");
}

/// The shadowing half: an ANCESTOR property named like the loop var must
/// not hijack the index type (that was the UVM failure shape exactly).
#[test]
fn loop_var_shadows_same_named_ancestor_property() {
    let src = r#"
module tb;
  typedef enum bit [1:0] { K_A, K_B, K_C } key_e;
  typedef enum int { OTHER_X = 0, OTHER_Y = 1, OTHER_Z = 2, OTHER_W = 3 } other_e;

  class base_t;
    other_e s;   // same NAME as the foreach index below
  endclass

  class derived_t extends base_t;
    int cnt [key_e];
    function void go();
      cnt[K_B] = 2;
      foreach (cnt[s])
        $display("T|key=%s", s.name());
    endfunction
  endclass

  derived_t d;
  initial begin
    d = new;
    d.go();
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(
        o.contains(&"T|key=K_B".to_string()),
        "the loop var's KEY enum wins over the ancestor property's type: {o:?}"
    );
}
