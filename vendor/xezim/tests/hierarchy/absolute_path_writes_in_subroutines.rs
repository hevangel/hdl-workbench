//! §23.6: an absolute hierarchical path written inside a task, a function
//! or a class method (`tb.s.lv = v`, `tb.s.lv[3:0] = v`, `tb.s.mem[2] = v`,
//! `tb.s.iv <= v`) lands on the named variable. Inside a subroutine body the
//! path stays a member-access chain, and the lvalue arms looked the joined
//! name up in a signal table that keys the top module's contents without its
//! name, so every such write was silently dropped while the relative form
//! `s.lv` and the same write from an `initial` block worked. Expected output
//! cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
module sub;
  logic [7:0] lv;
  int iv;
  logic [7:0] mem [0:3];
endmodule
module tb;
  sub s();
  logic [7:0] top_v;
  class W;
    function void w();
      tb.s.iv = 42;
    endfunction
  endclass
  task t();
    tb.s.lv = 8'h34;
    tb.s.iv = -6;
    tb.top_v = 8'h11;
    $display("t: lv=%h iv=%0d top_v=%h", tb.s.lv, tb.s.iv, tb.top_v);
  endtask
  function void f();
    tb.s.lv = 8'h35;
    tb.s.iv = 7;
  endfunction
  task automatic ta();
    tb.s.lv[3:0] = 4'h9;
    tb.s.mem[2] = 8'hab;
    #1;
    tb.s.lv[7:4] = 4'hc;
    tb.s.iv <= 99;
    #1;
  endtask
  initial begin
    W wo;
    #1;
    t();
    $display("A lv=%h iv=%0d top_v=%h", s.lv, s.iv, top_v);
    f();
    $display("B lv=%h iv=%0d", s.lv, s.iv);
    ta();
    $display("C lv=%h mem2=%h iv=%0d", s.lv, s.mem[2], s.iv);
    wo = new;
    wo.w();
    $display("D iv=%0d", s.iv);
  end
endmodule
"#;

#[test]
fn absolute_path_writes_from_subroutines() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        msgs,
        [
            "t: lv=34 iv=-6 top_v=11",
            "A lv=34 iv=-6 top_v=11",
            "B lv=35 iv=7",
            "C lv=c9 mem2=ab iv=99",
            "D iv=42",
        ],
    );
}
