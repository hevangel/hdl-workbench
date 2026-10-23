//! §13.3/§13.4 — a task/function FORMAL (or a function return type) whose width
//! uses a MODULE PARAMETER (`input [word_width-1:0] d`) must resolve that
//! parameter. When the module is INSTANTIATED (inlined), its parameters are not
//! in the flat runtime param map at call time, so the inline path must bake the
//! resolved value into the formal's data type — otherwise the formal collapsed
//! to 1 bit and only bit 0 of the argument survived (surfaced as an RF2-style
//! memory model writing/reading garbage). Works when the module is top; broke
//! only when inlined.

use xezim::simulate;

fn hexline(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("sim")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn task_formal_param_width_when_inlined() {
    // `Write` has `input [word_width-1:0] data`; module is instantiated.
    let src = r#"
module m;
  parameter word_width = 20;
  parameter WORDS = 128;
  reg [word_width-1:0] mem [0:WORDS-1];
  integer row_address;
  task Write; input [word_width-1:0] data; begin mem[row_address] = data; end endtask
  task WRITE_ROW; begin row_address = 5; Write(20'hABCDE); end endtask
  initial WRITE_ROW;
endmodule
module tb;
  m u();
  initial begin #1; $display("R=%h", u.mem[5]); end
endmodule
"#;
    let out = hexline(src);
    assert!(
        out.iter().any(|l| l == "R=abcde"),
        "task formal param-width collapsed when inlined; got {:?}",
        out
    );
}

#[test]
fn function_formal_and_return_param_width_when_inlined() {
    // Param-sized formal AND param-sized return type, module instantiated.
    let src = r#"
module m;
  parameter word_width = 20;
  function [word_width-1:0] widen;
    input [word_width-1:0] d;
    widen = d;
  endfunction
  reg [word_width-1:0] r;
  initial r = widen(20'hABCDE);
endmodule
module tb;
  m u();
  initial begin #1; $display("R=%h", u.r); end
endmodule
"#;
    let out = hexline(src);
    assert!(
        out.iter().any(|l| l == "R=abcde"),
        "function formal/return param-width collapsed when inlined; got {:?}",
        out
    );
}
