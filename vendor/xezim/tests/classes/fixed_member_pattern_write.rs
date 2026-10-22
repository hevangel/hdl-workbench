use xezim::simulate;

fn u(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("member_write_probe.{}", name)))
        .unwrap_or_else(|| panic!("signal not found: {}", name))
        .to_u64()
        .unwrap_or_else(|| panic!("{} contains an unknown bit", name))
}

#[test]
fn fixed_member_pattern_and_selected_write() {
    let src = r#"
class word_store;
  logic [15:0] words [2];
endclass

module member_write_probe;
  word_store store;
  logic [15:0] observed_lo, observed_hi;
  initial begin
    store = new();
    store.words = '{16'h1234, 16'habcd};
    store.words[1][11:4] = 8'h5a;
    observed_lo = store.words[0];
    observed_hi = store.words[1];
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    assert_eq!(u(&sim, "observed_lo"), 0x1234);
    assert_eq!(u(&sim, "observed_hi"), 0xa5ad);
}
