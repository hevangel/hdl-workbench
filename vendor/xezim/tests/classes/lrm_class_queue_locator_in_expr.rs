//! IEEE 1800-2023 §7.12: locator and reduction methods of a class queue property inside expressions (`ar.a.min() == ar.a.max()` used to panic).
//!
//! Expected lines come from the reference simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const LOC: &str = r#"
// top: t7_12
class Arr; int a[$]; endclass
module t7_12;
  Arr ar;
  initial begin
    ar = new; ar.a = '{3, 1, 2};
    $display("T|b|cmp=%0d", ar.a.min() == ar.a.max());
  end
endmodule
"#;

const LOC_FAMILY: &str = r#"
class Arr; int a[$]; int d[]; int f[3]; int aa[int]; byte b[$]; endclass
module t;
  Arr ar; int r; int qq[$];
  initial begin
    ar = new; ar.a = '{3, 1, 2}; ar.d = '{5, 6, 4}; ar.f = '{9, 8, 7}; ar.aa[1] = 4; ar.aa[2] = 8; ar.b = '{-1, 2};
    $display("T|b|cmp=%0d", ar.a.min() == ar.a.max());
    $display("T|c|min=%p max=%p", ar.a.min(), ar.a.max());
    $display("T|d|sum=%0d prod=%0d and=%0d or=%0d xor=%0d", ar.a.sum() + 1, ar.a.product() * 2, ar.a.and(), ar.a.or() | 8, ar.a.xor());
    qq = ar.a.find_index(x) with (x == 2); $display("T|f|fi=%p fe=%p", qq, ar.a.find_first(x) with (x < 3));
    r = ar.a.max()[0] + ar.d.min()[0]; $display("T|g|r=%0d", r);
    $display("T|h|cmp2=%0d s=%0d c=%0d", ar.a.sum() == ar.d.sum() - 9, ar.d.sum() with (item * 2), ar.a.sum() with (item > 1));
    if (ar.a.min() == '{1}) $display("T|j|yes");
    r = (ar.a.min() == ar.a.max()) ? 1 : 2; $display("T|k|r=%0d", r);
  end
endmodule
"#;

#[test]
fn audit_repro() {
    let want = ["T|b|cmp=0"];
    assert_eq!(t_lines(LOC), want);
}

#[test]
fn locators_and_reductions() {
    let want = [
        "T|b|cmp=0",
        "T|c|min='{1} max='{3}",
        "T|d|sum=7 prod=12 and=0 or=11 xor=0",
        "T|f|fi='{2} fe='{1}",
        "T|g|r=7",
        "T|h|cmp2=1 s=30 c=0",
        "T|j|yes",
        "T|k|r=2",
    ];
    assert_eq!(t_lines(LOC_FAMILY), want);
}
