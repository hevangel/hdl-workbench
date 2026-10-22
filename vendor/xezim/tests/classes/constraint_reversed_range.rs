//! §11.4.13 — a range whose left bound is above its right one (`[20:10]`)
//! is empty. Constraint ranges were normalized to `[10:20]`, so
//! `x inside {[20:10]}` held for 10..20 and randomize() succeeded where it
//! must fail; a `dist` item or a negated `inside` over such a range follows
//! from the same reading. The `inside` operator in an ordinary expression
//! already treated it as empty. Every line was cross-checked against the
//! reference simulator.

use xezim::simulate;

const SRC: &str = r#"
class C;
  rand bit [7:0] x;
  constraint c { x inside {[20:10]}; }
endclass
class D;
  rand bit [7:0] x;
endclass
class E;
  rand bit [7:0] x;
  constraint c { x inside {[20:10], 30}; }
endclass
class F;
  rand bit [7:0] x;
  constraint c { !(x inside {[20:10]}); x < 5; }
endclass
class G;
  rand bit [7:0] x;
  constraint c { x dist {[20:10] := 1, 40 := 1}; }
endclass
module top;
  initial begin
    automatic C c = new; automatic D d = new; automatic E e = new; automatic F f = new; automatic G g = new;
    automatic int r, n30, n40, nrange;
    automatic bit [7:0] v = 15;
    r = c.randomize();
    $display("R class reversed: r=%0d", r);
    r = d.randomize() with { x inside {[20:10]}; };
    $display("R inline reversed: r=%0d", r);
    r = d.randomize() with { x inside {[10:20]}; };
    $display("R inline normal: r=%0d ok=%0d", r, d.x >= 10 && d.x <= 20);
    for (int i = 0; i < 100; i++) begin r = e.randomize(); if (e.x == 30) n30++; end
    $display("R class mixed: r=%0d all30=%0d", r, n30);
    r = f.randomize();
    $display("R negated reversed: r=%0d ok=%0d", r, f.x < 5);
    for (int i = 0; i < 100; i++) begin r = g.randomize(); if (g.x == 40) n40++; else if (g.x >= 10 && g.x <= 20) nrange++; end
    $display("R dist reversed: r=%0d x40=%0d inrange=%0d", r, n40, nrange);
    $display("R operator: %0d %0d", v inside {[20:10]}, v inside {[10:20]});
    r = std::randomize(v) with { v inside {[20:10]}; };
    $display("R std::randomize reversed: r=%0d", r);
  end
endmodule
"#;

#[test]
fn reversed_range_is_empty() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|m| m.starts_with("R "))
        .collect();
    assert_eq!(
        got,
        [
            "R class reversed: r=0",
            "R inline reversed: r=0",
            "R inline normal: r=1 ok=1",
            "R class mixed: r=1 all30=100",
            "R negated reversed: r=1 ok=1",
            "R dist reversed: r=1 x40=100 inrange=0",
            "R operator: 0 1",
            "R std::randomize reversed: r=0",
        ]
    );
}
