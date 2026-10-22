//! §25.3 / §8.7: `B[0].proxy = new;` constructs into a class handle declared
//! in one element of an interface instance array. The element member's type
//! was not found through the indexed receiver, so the constructor never ran
//! and the handle read back x (`new()` with parentheses read back 0). The
//! expected lines were cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class proxy_c;
  static int count;
  int id;
  function new(); id = ++count; endfunction
endclass
interface bus_if;
  logic clk;
  proxy_c proxy;
endinterface
module sub;
  bus_if L[2]();
  initial begin
    #2;
    L[0].proxy = new;
    L[1].proxy = new;
    $display("sub L0=%0d L1=%0d", L[0].proxy.id, L[1].proxy.id);
  end
endmodule
module top;
  bus_if B[3]();
  sub u();
  initial begin
    B[0].proxy = new;
    B[2].proxy = new();
    #1;
    $display("B0 null=%0d id=%0d", B[0].proxy == null, B[0].proxy.id);
    $display("B1 null=%0d", B[1].proxy == null);
    $display("B2 id=%0d", B[2].proxy.id);
    B[1].proxy = B[0].proxy;
    B[1].proxy.id = 40;
    $display("B0 id=%0d", B[0].proxy.id);
  end
endmodule
"#;

#[test]
fn new_into_interface_array_element_handle() {
    let out: Vec<String> = simulate(SRC, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let want = [
        "B0 null=0 id=1",
        "B1 null=1",
        "B2 id=2",
        "B0 id=40",
        "sub L0=3 L1=4",
    ];
    assert_eq!(out, want, "{out:?}");
}
