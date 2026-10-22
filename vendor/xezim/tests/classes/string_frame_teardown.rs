//! Caller string classification survives nested integral declarations (§6.21).

use xezim::simulate;

const SOURCE: &str = r#"
class string_frame_c;
  function void inner(int text);
    begin
      logic [7:0] text;
      text = 0;
    end
  endfunction
  function void inner_signed(int text);
    begin
      byte text;
      text = -1;
    end
  endfunction
  function void inner_repeat(int text);
    begin
      logic [15:0] text;
      text = 0;
      repeat (2) text = text + 1;
    end
  endfunction
  function void inner_early(int text);
    begin
      logic [7:0] text;
      text = 1;
      return;
    end
  endfunction
  function void inner_recursive(int text, int depth);
    begin
      logic [7:0] text;
      text = 0;
      if (depth > 0) inner_recursive(text, depth - 1);
    end
  endfunction
  function void inner_vector(logic [31:0] text);
    begin
      logic [7:0] text;
      text = 0;
    end
  endfunction
  function int outer(string text, int variant);
    case (variant)
      0: inner(0);
      1: inner_signed(0);
      2: inner_repeat(0);
      3: inner_early(0);
      4: inner_recursive(0, 2);
      5: inner_vector(0);
    endcase
    return text[0] * 256 + text[1];
  endfunction
  function int numeric(int text);
    return text[0];
  endfunction
endclass
module top;
  int text = 0;
  int first, signed_local, repeated, early, recursive, vector_formal;
  int scalar_control, again;
  string_frame_c c;
  initial begin
    c = new;
    first = c.outer("AB", 0);
    signed_local = c.outer("CD", 1);
    repeated = c.outer("EF", 2);
    early = c.outer("GH", 3);
    recursive = c.outer("IJ", 4);
    vector_formal = c.outer("KL", 5);
    scalar_control = c.numeric(2);
    again = c.outer("MN", 0);
    $display("T|frames %0d %0d %0d %0d %0d %0d %0d %0d", first,
      signed_local, repeated, early, recursive, vector_formal, scalar_control, again);
  end
endmodule
"#;

#[test]
fn caller_string_flags_outlive_integral_declaration_snapshots() {
    if !super::compiled_method_test_env::policies(&[("0", "1000"), ("1", "0"), ("1", "1000")]) {
        return;
    }
    let sim = simulate(SOURCE, 1).expect("string frame regression");
    // Reference-validated byte pairs, including re-entry after scalar use.
    for (name, expected) in [
        ("first", 0x4142),
        ("signed_local", 0x4344),
        ("repeated", 0x4546),
        ("early", 0x4748),
        ("recursive", 0x494a),
        ("vector_formal", 0x4b4c),
        ("scalar_control", 0),
        ("again", 0x4d4e),
    ] {
        let actual = sim
            .get_signal(name)
            .or_else(|| sim.get_signal(&format!("top.{name}")))
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(actual.to_u64(), Some(expected), "{name}");
    }
}
