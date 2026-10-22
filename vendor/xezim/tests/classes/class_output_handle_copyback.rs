//! Class-handle copy-back after a same-named record formal.

use xezim::simulate;

fn value(sim: &xezim::compiler::Simulator, name: &str) -> u64 {
    sim.get_signal(name)
        .or_else(|| sim.get_signal(&format!("top.{name}")))
        .unwrap_or_else(|| panic!("signal not found: {name}"))
        .to_u64()
        .expect("signal contains x/z")
}

/// IEEE 1800-2017 §13.5.2: each formal has its declared type for that call.
#[test]
fn class_output_replaces_prior_record_metadata() {
    let source = r#"
typedef struct packed {
  int payload;
} encoded_t;

module top;
  class parcel;
    int payload;

    function int marker();
      return payload;
    endfunction
  endclass

  class adapter;
    extern static function void encode(input parcel source, output encoded_t target);
    extern static function void decode(input encoded_t source, output parcel target);
  endclass

  function void adapter::encode(input parcel source, output encoded_t target);
    target.payload = source.payload;
  endfunction

  function void adapter::decode(input encoded_t source, output parcel target);
    target = new();
    target.payload = source.payload;
  endfunction

  parcel source_item;
  parcel result_item;
  encoded_t encoded;
  int observed;

  initial begin
    source_item = new();
    source_item.payload = 37;
    adapter::encode(source_item, encoded);
    adapter::decode(encoded, result_item);
    if (result_item != null)
      observed = result_item.marker();
  end
endmodule
"#;

    let sim = simulate(source, 20).expect("simulation failed");
    assert_eq!(value(&sim, "observed"), 37);
}
