//! Logical disjunction propagation for class constraints.

use xezim::simulate;

fn value(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("top.{name}")))
        .unwrap_or_else(|| panic!("signal not found: {name}"))
        .to_u64()
        .expect("signal contains x/z")
}

/// IEEE 1800-2017 §18.5.12: a solver may satisfy either side of a logical OR.
#[test]
fn disjunction_can_select_an_equality_branch() {
    let source = r#"
module top;
  class selector;
    rand int unsigned choice;

    function int option_count();
      return 0;
    endfunction

    constraint legal_choice {
      (choice < option_count()) || (choice == 0);
    }
  endclass

  selector item;
  int solve_ok;
  int picked;

  initial begin
    item = new();
    solve_ok = item.randomize();
    picked = item.choice;
  end
endmodule
"#;

    let sim = simulate(source, 20).expect("simulation failed");
    assert_eq!(value(&sim, "solve_ok"), 1);
    assert_eq!(value(&sim, "picked"), 0);
}
