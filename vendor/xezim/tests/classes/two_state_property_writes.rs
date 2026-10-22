//! §6.11.1 / §10.7: a 2-state variable drops X/Z bits on assignment. A class
//! property declared `bit`/`int` (directly or through a typedef) and a
//! 2-state member of an unpacked struct kept the X bits of a 4-state
//! right-hand side; UVM register data (`uvm_reg_data_t`) read from a bus
//! with an undriven bit then carried X, and a status poll on it never saw
//! its flag. The expected lines were cross-checked against the reference
//! simulator.

use xezim::simulate;

const SRC: &str = r#"
typedef bit [63:0] data_t;
typedef struct { data_t data; bit [63:0] d2; int i; } bus_op;
class regc;
  data_t value;
  bit [63:0] v2;
  int i3;
  function void pred(data_t v); value = v; endfunction
  function void pred2(bit [63:0] v); v2 = v; endfunction
  function void pred3(logic [63:0] v); value = v; endfunction
endclass
module top;
  regc r;
  bus_op op;
  logic [31:0] x;
  initial begin
    r = new;
    x = 32'h0000_00x0 | 32'h20;
    r.pred(x);   $display("1 %h", r.value);
    r.pred2(x);  $display("2 %h", r.v2);
    r.pred3(x);  $display("3 %h", r.value);
    r.value = x; $display("4 %h", r.value);
    r.v2 = x;    $display("5 %h", r.v2);
    r.i3 = x;    $display("6 %h", r.i3);
    op.data = x; $display("7 %h", op.data);
    op.d2 = x;   $display("8 %h", op.d2);
    op.i = x;    $display("9 %h", op.i);
  end
endmodule
"#;

#[test]
fn two_state_properties_and_struct_members_drop_xz() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "1 0000000000000020",
        "2 0000000000000020",
        "3 0000000000000020",
        "4 0000000000000020",
        "5 0000000000000020",
        "6 00000020",
        "7 0000000000000020",
        "8 0000000000000020",
        "9 00000020",
    ];
    assert_eq!(out, want, "{out:?}");
}
