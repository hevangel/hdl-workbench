//! §13.5 / §6.16: a task's `string` formal is a string only in that task's
//! own frame. While the task is suspended, after `disable fork` kills it, and
//! while another task runs interleaved with it, a same-named packed variable
//! or formal elsewhere keeps its bit writes. Expected values come from the
//! reference simulator.

use xezim::simulate;

#[test]
fn task_string_formal_does_not_leak() {
    let src = r#"
class K;
  bit [7:0] s;
  function void w(); s = 0; s[5] = 1; endfunction
endclass
module top;
  logic [7:0] s;
  task automatic t(ref string s); #5; s[0] = "Z"; endtask
  task automatic t2(ref string s); #2; s[1] = "Y"; #6; s[2] = "X"; endtask
  task automatic tw(output logic [7:0] s); s = 0; #4; s[3] = 1; endtask
  task automatic tb(output logic [7:0] s); s = 0; s[4] = 1; endtask
  function automatic logic [7:0] fb(input logic [7:0] s); s[1] = 1; return s; endfunction
  function automatic void fs(); string s; s = "abc"; s[0] = "A"; endfunction
  string a = "abc", b = "defghi"; logic [7:0] r1, r2, w; K k;
  initial begin
    fork t(a); join_none
    #1 tb(r1); r2 = fb(8'h00); k = new(); k.w();
    s = 0; s[2] = 1;
    $display("T|during tb %b fb %b k %b s %b", r1, r2, k.s, s);
    #10 tb(r1); $display("T|after tb %b a '%s'", r1, a);
    fork t(a); join_none
    #1 disable fork;
    #1 s[6] = 1; tb(r1); r2 = fb(8'h00);
    $display("T|killed tb %b fb %b s %b a '%s'", r1, r2, s, a);
    fork t2(b); tw(w); join
    $display("T|interleaved b '%s' w %b", b, w);
    fs(); tb(r1); r2 = fb(8'h00);
    $display("T|string local tb %b fb %b", r1, r2);
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let got: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect();
    assert_eq!(
        got,
        [
            "T|during tb 00010000 fb 00000010 k 00100000 s 00000100",
            "T|after tb 00010000 a 'Zbc'",
            "T|killed tb 00010000 fb 00000010 s 01000100 a 'Zbc'",
            "T|interleaved b 'dYXghi' w 00001000",
            "T|string local tb 00010000 fb 00000010",
        ],
        "T| lines differ"
    );
}
