//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    xezim::simulate(src, 1_000_000)
        .expect("simulation failed")
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn task_local_array_shadows_instance_array() {
    let got = output(
        r###"module child;
 int mem[7];
 task automatic run();
  int mem[16];
  logic [$clog2($size(mem))-1:0] v;
  v=15;
  $display("T|local size %0d width %0d value %0d",$size(mem),$bits(v),v);
 endtask
 initial begin run(); $finish; end
endmodule
module top; child u(); endmodule
"###,
    );
    assert!(got.contains("T|local size 16 width 4 value 15"), "{got}");
}

#[test]
fn generated_array_query_scope() {
    let got = output(
        r###"module child;
 for(genvar g=0;g<2;g++) begin: bank
  int mem[7+g];
  initial begin
   logic [$clog2($size(mem))-1:0] v;
   v=6;
   #1; $display("T|g %0d size %0d width %0d value %0d",g,$size(mem),$bits(v),v);
  end
 end
endmodule
module top; child u(); initial #2 $finish; endmodule
"###,
    );
    assert!(got.contains("T|g 0 size 7 width 3 value 6"), "{got}");
    assert!(got.contains("T|g 1 size 8 width 3 value 6"), "{got}");
}

#[test]
fn static_task_local_array_shadows_instance_array() {
    let got = output(
        r###"module child;
 int mem[7];
 task run();
  int mem[16];
  logic [$clog2($size(mem))-1:0] v;
  v=15;
  $display("T|local size %0d width %0d value %0d",$size(mem),$bits(v),v);
 endtask
 initial begin run(); $finish; end
endmodule
module top; child u(); endmodule
"###,
    );
    assert!(got.contains("T|local size 16 width 4 value 15"), "{got}");
}

#[test]
fn function_local_array_shadows_instance_array() {
    let got = output(
        r###"module child;
 int mem[7];
 function automatic void run();
  int mem[16];
  logic [$clog2($size(mem))-1:0] v;
  v=15;
  $display("T|local size %0d width %0d value %0d",$size(mem),$bits(v),v);
 endfunction
 initial begin run(); $finish; end
endmodule
module top; child u(); endmodule
"###,
    );
    assert!(got.contains("T|local size 16 width 4 value 15"), "{got}");
}

#[test]
fn static_query_ranges_follow_local_renaming() {
    let template = r#"module child;
 int mem[7];
 task run();
  int mem[16];
  begin
   logic [$clog2($size(mem))-1:0] v;
   int copies[$size(mem)];
   v=15;
   $display("T|nested width %0d value %0d copies %0d",$bits(v),v,$size(copies));
  end
 endtask
 initial begin run(); $finish; end
endmodule
module top; child u(); endmodule"#;
    for source in [
        template.to_string(),
        template.replace("task run();", "task static run();"),
        template
            .replace("task run();", "function void run();")
            .replace("endtask", "endfunction"),
    ] {
        let got = output(&source);
        assert!(got.contains("T|nested width 4 value 15 copies 16"), "{got}");
    }
}
