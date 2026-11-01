//! Behavioral regressions for reviewed PR follow-ups (IEEE 1800 §6.21, §11.6, §16.5, §18.12, §20.7).
fn output(src: &str) -> String {
    let result = xezim::simulate(src, 1_000_000).expect("simulation failed");
    assert_eq!(result.error_count, 0, "unexpected simulation error");
    result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn constant_array_directions_and_copy() {
    let got = output(
        r###"package p;
 class C;
 localparam int ASC[4:5]='{11,22};
 localparam int DSC[5:4]='{11,22};
 endclass
endpackage
module top;
 int a[0:1]; int b[0:1];
 initial begin
  a=p::C::ASC; b=p::C::DSC;
  $display("T|bounds asc %0d %0d %0d desc %0d %0d %0d",$left(p::C::ASC),$right(p::C::ASC),$increment(p::C::ASC),$left(p::C::DSC),$right(p::C::DSC),$increment(p::C::DSC));
  $display("T|copy asc %0d %0d desc %0d %0d",a[0],a[1],b[0],b[1]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|bounds asc 4 5 -1 desc 5 4 1"), "{got}");
    assert!(got.contains("T|copy asc 11 22 desc 11 22"), "{got}");
}

#[test]
fn constant_array_copy_descending_destination() {
    let got = output(
        r###"package p;
 class C;
 localparam int ASC[4:5]='{11,22};
 localparam int DSC[5:4]='{11,22};
 endclass
endpackage
module top;
 int a[1:0]; int b[1:0];
 initial begin
  a=p::C::ASC; b=p::C::DSC;
  $display("T|bounds asc %0d %0d %0d desc %0d %0d %0d",$left(p::C::ASC),$right(p::C::ASC),$increment(p::C::ASC),$left(p::C::DSC),$right(p::C::DSC),$increment(p::C::DSC));
  $display("T|copy asc %0d %0d desc %0d %0d",a[1],a[0],b[1],b[0]);
  $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|copy asc 11 22 desc 11 22"), "{got}");
}

#[test]
fn specialized_array_bounds() {
    let got = output(
        r###"class C #(parameter int N=2);
 localparam int T[N]='{default:3};
endclass
module top;
 initial begin
 $display("T|N2 %0d %0d %0d N5 %0d %0d %0d",$size(C#(2)::T),$left(C#(2)::T),$right(C#(2)::T),$size(C#(5)::T),$left(C#(5)::T),$right(C#(5)::T));
 $finish;
 end
endmodule
"###,
    );
    assert!(got.contains("T|N2 2 0 1 N5 5 0 4"), "{got}");
}

#[test]
fn automatic_descending_destination_preserves_declared_order() {
    let src = r#"class C;
 localparam int T[5:4]='{11,22};
 function void run(); int dst[7:6]; dst=T;
 $display("T|local copy %0d %0d",dst[7],dst[6]); endfunction
endclass
module top; C c; initial begin c=new; c.run(); $finish; end endmodule"#;
    for src in [src.to_string(), src.replace("dst=T;", "dst=C::T;")] {
        let got = output(&src);
        assert!(got.contains("T|local copy 11 22"), "{got}");
    }
}

#[test]
fn copying_typed_byte_constants_preserves_values() {
    let got = output(
        r#"class C; localparam byte unsigned T[2]='{9'h1ff,-1}; endclass
module top; byte unsigned dst[2]; initial begin dst=C::T;
 $display("T|narrow %0d %0d",dst[0],dst[1]); $finish; end endmodule"#,
    );
    assert!(got.contains("T|narrow 255 255"), "{got}");
}

#[test]
fn specialized_descending_constant_bounds_follow_specialization() {
    let got = output(
        r#"class C #(parameter int N=2);
 localparam int T[N-1:0]='{default:N}; endclass
module top; initial begin
 $display("T|descending %0d %0d %0d %0d",$size(C#(5)::T),$left(C#(5)::T),$right(C#(5)::T),$increment(C#(5)::T));
 $finish; end endmodule"#,
    );
    assert!(got.contains("T|descending 5 4 0 1"), "{got}");
}
