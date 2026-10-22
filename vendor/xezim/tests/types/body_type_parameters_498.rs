//! IEEE 1800 §6.20.3 / §8.3: body type parameters bind their declared types.
use xezim::simulate;
#[test]
fn original_body_type_parameter_scopes() {
    let result = simulate(include_str!("../issue_cases/issue498_0.sv"), 100).unwrap();
    for line in [
        "VALUE_PORTLIST_TYPEBITS=4",
        "VALUE_MOD_BITS=8",
        "VALUE_MOD_TYPEBITS=8",
        "VALUE_PKG_BITS=4",
        "VALUE_CLS_BITS=6",
    ] {
        assert!(
            result.output.iter().any(|o| o.message == line),
            "missing {line}: {:?}",
            result.output
        );
    }
}

fn check(source: &str, expected: &[&str]) {
    let result = simulate(source, 100).unwrap();
    let lines: Vec<_> = result
        .output
        .iter()
        .map(|o| o.message.as_str())
        .filter(|s| s.starts_with("T|"))
        .collect();
    assert_eq!(lines, expected);
}

#[test]
fn top_body_type_and_alias_chain() {
    check(
        r#"module top;
 parameter int N=9;
 parameter type T=logic[N-1:0];
 localparam type U=T;
 T v; U u;
 localparam int K=$bits(T);
 initial $display("T|top %0d %0d %0d %0d",K,$bits(U),$bits(v),$bits(u));
 endmodule"#,
        &["T|top 9 9 9 9"],
    );
}
#[test]
fn body_type_overrides_are_per_instance() {
    check(
        r#"module m;
 parameter type T=logic[7:0]; T v;
 localparam int K=$bits(T);
 initial $display("T|child %0d %0d",K,$bits(v));
 endmodule
 module top; m #(.T(logic[11:0])) a(); m b(); endmodule"#,
        &["T|child 12 12", "T|child 8 8"],
    );
}
#[test]
fn package_body_types_remain_scoped() {
    check(
        r#"package p; parameter type T=logic[3:0]; T v; endpackage
 package q; parameter type T=logic[5:0]; T v; endpackage
 module top;
 p::T a; q::T b;
 initial $display("T|package %0d %0d %0d %0d",$bits(a),$bits(b),$bits(p::v),$bits(q::v));
 endmodule"#,
        &["T|package 4 6 4 6"],
    );
}
#[test]
fn class_body_type_names_do_not_leak() {
    check(
        r#"class a;
 parameter type T=logic[3:0]; T v;
 function int width();return $bits(T);endfunction
 endclass
 class b;
 parameter type T=logic[5:0]; T v;
 function int width();return $bits(T);endfunction
 endclass
 module top; initial begin a x=new; b y=new;
 $display("T|classes %0d %0d %0d %0d",$bits(x.v),$bits(y.v),x.width(),y.width());
 end endmodule"#,
        &["T|classes 4 6 4 6"],
    );
}
#[test]
fn body_type_keeps_struct_fields_and_signing() {
    check(
        r#"package p;
 parameter type T=struct packed {logic [2:0] a; logic [4:0] b;};
 T v;
 endpackage
 class c; parameter type T=logic signed[4:0]; T v;
 function new();v=-1;endfunction
 endclass
 module top;initial begin c x=new; p::v='{3'd5,5'd17};
 $display("T|types %0d %0d %0d %0d",$bits(p::v),p::v.a,p::v.b,x.v);
 end endmodule"#,
        &["T|types 8 5 17 -1"],
    );
}

#[test]
fn class_body_alias_tracks_header_specialization() {
    check(
        r#"
class c #(type H=logic[3:0]);
 parameter type T=H;T v;
 function int width();return $bits(T);endfunction
endclass
module top;initial begin c#(logic[7:0]) a=new;c b=new;
 $display("T|specialized %0d %0d %0d %0d",$bits(a.v),$bits(b.v),a.width(),b.width());
end endmodule
"#,
        &["T|specialized 8 4 8 4"],
    );
}
