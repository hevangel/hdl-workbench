//! §10.6.2: `force <target> = <expr>` acts as a continuous assignment —
//! it re-evaluates when its operands change until release. One-shot
//! snapshots held the arm-time value forever. Reference-validated.

use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(n)
        .or_else(|| sim.get_signal(&format!("tb.{}", n)))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} is x/z", n))
}

const SRC: &str = r#"
module tb;
  reg  [7:0] src;
  wire [7:0] n;
  reg  [7:0] v;
  reg  [7:0] n_pre, v_pre, n_post, v_post, v_rel;
  assign n = src;
  initial begin
    src = 8'h10;
    v = 8'h00;
    force n = src + 8'h1;
    force v = src + 8'h2;
    #1 n_pre = n; v_pre = v;
    src = 8'h20;
    #1 n_post = n; v_post = v;
    release v;
    src = 8'h30;
    #1 v_rel = v;   // variable keeps last forced value after release
  end
endmodule
"#;

#[test]
fn force_expression_tracks_operands() {
    let sim = simulate(SRC, 20).expect("simulate failed");
    assert_eq!(u(&sim, "n_pre"), 0x11);
    assert_eq!(u(&sim, "v_pre"), 0x12);
    assert_eq!(u(&sim, "n_post"), 0x21, "forced net tracks src change");
    assert_eq!(u(&sim, "v_post"), 0x22, "forced variable tracks src change");
    assert_eq!(u(&sim, "v_rel"), 0x22, "released variable holds last value");
}

const DEPENDENCY_FILTER_SRC: &str = r#"
module dependency_filter_case;
  logic [7:0] stimulus = 8'h03;
  logic [7:0] observed;
  logic [7:0] captured;
  integer unrelated = 0;
  initial begin
    force observed = stimulus + 8'h1;
    repeat (200)
      unrelated = unrelated + 1;
    stimulus = 8'h20;
    #1 captured = observed;
  end
endmodule
"#;

#[test]
fn active_force_skips_unrelated_procedural_writes() {
    let sim = simulate(DEPENDENCY_FILTER_SRC, 10).expect("simulate failed");
    assert_eq!(u(&sim, "captured"), 0x21);
    let (evaluations, skips) = sim.active_force_work_counters();
    assert!(
        evaluations <= 2,
        "clean writes caused too many RHS evaluations: {evaluations}"
    );
    assert!(
        skips >= 100,
        "dependency filter did not skip enough clean refreshes: {skips}"
    );
}

const REVERSE_CHAIN_SRC: &str = r#"
module reverse_chain_case;
  logic [7:0] origin = 8'h01;
  logic [7:0] middle;
  logic [7:0] endpoint;
  logic [7:0] captured;
  initial begin
    // Arm the downstream expression first to exercise reverse entry order.
    force endpoint = middle + 8'h1;
    force middle = origin + 8'h1;
    origin = 8'h10;
    #1 captured = endpoint;
  end
endmodule
"#;

#[test]
fn active_force_dependency_chain_reaches_fixpoint() {
    let sim = simulate(REVERSE_CHAIN_SRC, 10).expect("simulate failed");
    assert_eq!(u(&sim, "captured"), 0x12);
}
