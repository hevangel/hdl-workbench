//! Language constructs whose output was cross-checked line by line against
//! the reference simulator. The passing tests pin behaviour that already
//! matches; every case remains active to pin the expected behavior.

use xezim::simulate;

fn out(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn has(o: &[String], want: &str) -> bool {
    o.iter().any(|l| l.contains(want))
}

/// `req |-> ##[1:3] gnt` under `disable iff`, plus `$rose/$stable` and two
/// cover properties: the property fails on exactly the two cycles the
/// reference reports.
#[test]
fn sva_ranged_implication_fails_on_the_reference_cycles() {
    let o = out(r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic req = 0, gnt = 0, rst = 1; int cyc = 0;
  always @(posedge clk) cyc <= cyc + 1;
  property p_req_gnt; @(posedge clk) disable iff (rst) req |-> ##[1:3] gnt; endproperty
  a_rg: assert property (p_req_gnt) else $error("a_rg failed at cyc %0d", cyc);
  c_rg: cover property (@(posedge clk) req ##1 !req ##1 gnt);
  a_stable: assert property (@(posedge clk) disable iff (rst) $rose(gnt) |=> $stable(req) or !req);
  sequence s2; req ##1 req; endsequence
  c_s2: cover property (@(posedge clk) s2);
  initial begin
    #12 rst = 0;
    repeat (3) begin @(negedge clk) req = 1; @(negedge clk) req = 0; @(negedge clk) gnt = 1; @(negedge clk) gnt = 0; end
    @(negedge clk) req = 1; @(negedge clk) req = 1; @(negedge clk) req = 0;
    repeat (5) @(negedge clk);
    $display("DONE cyc=%0d", cyc);
    $finish;
  end
endmodule
"#);
    assert!(
        has(&o, "a_rg failed at cyc 18") && has(&o, "a_rg failed at cyc 19"),
        "{o:?}"
    );
    assert_eq!(
        o.iter().filter(|l| l.contains("a_rg failed")).count(),
        2,
        "{o:?}"
    );
    assert!(has(&o, "DONE cyc=21"), "{o:?}");
}

/// Interface class, `let`, `randsequence`, `process::kill`/`status` and a
/// program block, all matching the reference's transcript.
#[test]
fn interface_class_let_randsequence_process_and_program() {
    let o = out(r#"
interface class shape; pure virtual function int area(); endclass
class sq implements shape; int s = 3; virtual function int area(); return s*s; endfunction endclass
program automatic prog(input logic clk, output logic [3:0] v);
  initial begin
    repeat (2) @(posedge clk);
    v = 4'b0100;
    @(posedge clk);
    $display("PROG done v=%b t=%0t", v, $time);
  end
endprogram
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [3:0] v;
  int acc = 0;
  prog p(clk, v);
  let max(a, b) = (a > b) ? a : b;
  initial begin
    automatic sq s = new; automatic shape sh = s; automatic process pr;
    $display("IFC area=%0d let=%0d", sh.area(), max(3, 9));
    randsequence (main)
      main : first second third;
      first : { acc += 1; };
      second : { acc += 10; };
      third : { acc += 100; };
    endsequence
    $display("RSEQ acc=%0d", acc);
    fork begin pr = process::self(); #100 $display("SHOULD NOT PRINT"); end join_none
    #1 pr.kill(); #1 $display("PROC status=%s", pr.status().name());
    #50 $finish;
  end
endmodule
"#);
    for want in [
        "IFC area=9 let=9",
        "RSEQ acc=111",
        "PROC status=KILLED",
        "PROG done v=0100 t=25",
    ] {
        assert!(has(&o, want), "missing `{want}`: {o:?}");
    }
    assert!(!has(&o, "SHOULD NOT PRINT"), "{o:?}");
}

/// §17 — a checker whose event formal takes `posedge clk` as its actual
/// (the actual used to be a parse error). The reference flags the one
/// non-one-hot value, sampled at the 25 ns rising edge.
#[test]
fn checker_with_event_expression_actual() {
    let o = out(r#"
checker chk_onehot(logic [3:0] v, event clk);
  a_oh: assert property (@clk $onehot0(v)) else $error("onehot0 violated v=%b", v);
endchecker
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [3:0] v = 0;
  chk_onehot u_chk(v, posedge clk);
  initial begin
    #12 v = 4'b0010; #10 v = 4'b0110; #10 v = 4'b1000; #10 $display("CHK done"); $finish;
  end
endmodule
"#);
    assert!(
        has(&o, "onehot0 violated v=0110") && has(&o, "CHK done"),
        "{o:?}"
    );
    assert_eq!(
        o.iter().filter(|l| l.contains("onehot0 violated")).count(),
        1,
        "{o:?}"
    );
}

/// §9.3.1 — block item declarations precede the statements of a
/// `begin`-`end` block. The reference rejects this source ("Illegal
/// declaration after the statement"); so does the strict check now
/// (`--no-strict` accepts it).
#[test]
fn declaration_after_statement_is_rejected() {
    let src = "module tb;\n  initial begin\n    int a;\n    a = 1;\n    int b;\n    b = a;\n    $display(\"B=%0d\", b);\n  end\nendmodule\n";
    assert!(
        simulate(src, 100).is_err(),
        "a declaration after a statement must be a compile error"
    );
}

/// §9.4.2 / §16.5 — a clocking event without an edge (`@(clk)`, or `@clk`
/// on a checker event formal bound to `clk`) samples on ANY change of the
/// clock. The reference reports the violation at both 25 ns (rise) and
/// 30 ns (fall); these used to be clocked on the rising edge only.
#[test]
fn edgeless_clocking_event_samples_on_both_edges() {
    let module_level = out(r#"
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [3:0] v = 0;
  a_oh: assert property (@(clk) $onehot0(v)) else $error("M onehot0 violated v=%b", v);
  initial begin #12 v = 4'b0010; #10 v = 4'b0110; #10 v = 4'b1000; #10 $finish; end
endmodule
"#);
    assert_eq!(
        module_level
            .iter()
            .filter(|l| l.contains("onehot0 violated"))
            .count(),
        2,
        "{module_level:?}"
    );
    let checker = out(r#"
checker chk_onehot(logic [3:0] v, event clk);
  a_oh: assert property (@clk $onehot0(v)) else $error("C onehot0 violated v=%b", v);
endchecker
module tb;
  logic clk = 0; always #5 clk = ~clk;
  logic [3:0] v = 0;
  chk_onehot u_chk(v, clk);
  initial begin #12 v = 4'b0010; #10 v = 4'b0110; #10 v = 4'b1000; #10 $finish; end
endmodule
"#);
    assert_eq!(
        checker
            .iter()
            .filter(|l| l.contains("onehot0 violated"))
            .count(),
        2,
        "{checker:?}"
    );
}
