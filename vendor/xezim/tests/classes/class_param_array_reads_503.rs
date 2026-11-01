//! IEEE 1800 §6.20.2, §8.25 and §10.9.1: class constant arrays retain
//! lexical scope, specialization, declared element types and pattern layout.

fn lines(src: &str, top: &str) -> Vec<String> {
    xezim::simulate_multi(
        &[src.to_string()],
        1000,
        Some(top),
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("sim")
    .output
    .iter()
    .map(|o| o.message.clone())
    .collect()
}

#[test]
fn class_array_narrow() {
    let source = r#"class C;
localparam logic [3:0] A[2]='{8'hf6,8'h85};
endclass
module top;initial begin C c=new;
$display("T|narrow %0d %0d",C::A[0],C::A[1]);
$display("T|instance %0d %0d",c.A[0],c.A[1]);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|narrow 6 5", "T|instance 6 5"]);
}

#[test]
fn class_array_signing() {
    let source = r#"class C;
localparam int unsigned A[1]='{-1};
localparam int signed B[1]='{32'hfffffff6};
endclass
module top;initial begin
$display("T|sign %0d %0d %0d %0d",C::A[0],C::B[0],C::A[0]<0,C::B[0]<0);
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|sign 4294967295 -10 0 1"]);
}

#[test]
fn class_array_descending() {
    let source = r#"class C;
localparam int A[2:1]='{10,20};
static int S[2:1]='{30,40};
endclass
module top;initial begin
$display("T|desc %0d %0d %0d %0d",C::A[2],C::A[1],C::S[2],C::S[1]);
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|desc 10 20 30 40"]);
}

#[test]
fn class_array_keyed() {
    let source = r#"class C;
localparam int A[3]='{default:9};
static int S[3]='{default:7};
localparam int B[3]='{2:30,0:10,1:20};
static int D[3]='{2:60,0:40,1:50};
endclass
module top;initial begin
$display("T|default %0d %0d %0d %0d %0d %0d",C::A[0],C::A[1],C::A[2],C::S[0],C::S[1],C::S[2]);
$display("T|keyed %0d %0d %0d %0d %0d %0d",C::B[0],C::B[1],C::B[2],C::D[0],C::D[1],C::D[2]);
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|default 9 9 9 7 7 7", "T|keyed 10 20 30 40 50 60"]);
}

#[test]
fn class_array_shadow_local() {
    let source = r#"class C;
localparam int A[2]='{1,2};
static function int f();
int A[2]='{7,8};return A[0];
endfunction
endclass
module top;initial $display("T|local %0d",C::f());endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|local 7"]);
}

#[test]
fn class_array_shadow_formal() {
    let source = r#"class C;
localparam int A[2]='{1,2};
static function int f(input int A[2]);return A[0];endfunction
endclass
module top;int B[2]='{7,8};initial $display("T|formal %0d",C::f(B));endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|formal 7"]);
}

#[test]
fn class_array_alias() {
    let source = r#"class C#(int P=5);
localparam int A[2]='{P,P+1};
endclass
typedef C#(9) T;
module top;initial $display("T|alias %0d %0d",T::A[0],C#(9)::A[0]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|alias 9 9"]);
}

#[test]
fn class_array_inherit() {
    let source = r#"class C#(int P=5);
localparam int A[2]='{P,P+1};
static function int f();return A[1];endfunction
endclass
class D extends C#(8);endclass
module top;initial $display("T|inherit %0d %0d",D::A[0],D::f());endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|inherit 8 9"]);
}

#[test]
fn class_array_context_width() {
    let source = r#"class C;
 localparam int A[2]='{1'b1+1'b1,8'sh80};
 static int S[2]='{1'b1+1'b1,8'sh80};
endclass
module top;initial $display("T|context %0d %0d %0d %0d",C::A[0],C::A[1],C::S[0],C::S[1]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|context 2 -128 2 -128"]);
}

#[test]
fn class_array_index_defaults() {
    let source = r#"class C;
 localparam int A[1]='{'x};
 localparam logic [3:0] B[1]='{'x};
 static bit [3:0] S[1]='{'z};
endclass
module top;initial $display("T|domain %0d %0d %0d %0d %0d %0d",C::A[0],C::A[1],C::A['x],$isunknown(C::B[0]),$isunknown(C::B[1]),C::S[0]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|domain 0 0 0 1 1 0"]);
}

#[test]
fn class_array_specialized_bounds() {
    let source = r#"class C#(int N=2,int W=4,int A[N]='{default:1});
 localparam logic[W-1:0] B[N]='{N{'1}};
 static function int at(int i);return A[i];endfunction
endclass
typedef C#(3,5) Alias;
module top;initial begin
 $display("T|bounds %0d %0d %0d %0d %0d",C#(4,6)::A[3],C#(4,6)::B[3],C#()::B[1],Alias::B[2],C#(4,6)::at(3));
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|bounds 1 63 15 31 1"]);
}

#[test]
fn class_array_named_override() {
    let source = r#"class C#(int N=2,int A[N]='{default:1});endclass
module top;initial begin
 $display("T|named %0d %0d %0d",C#(.N(3),.A('{2:9,default:6}))::A[0],C#(.N(3),.A('{2:9,default:6}))::A[2],C#()::A[1]);
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|named 6 9 1"]);
}

#[test]
fn class_array_element_type_parameter() {
    let source = r#"class C#(type T=int);
 localparam T A[2]='{-1,'x};
endclass
module top;initial begin
 $display("T|types %0d %0d %0d %0d",C#(bit[3:0])::A[0],C#(bit[3:0])::A[1],C#(integer)::A[0],$isunknown(C#(integer)::A[1]));
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|types 15 0 -1 1"]);
}

#[test]
fn class_array_pattern_types() {
    let source = r#"class C;
 localparam int A[3]='{int:5};
 static int S[3]='{int:7};
endclass
module top;initial $display("T|typekeys %0d %0d %0d %0d %0d %0d",C::A[0],C::A[1],C::A[2],C::S[0],C::S[1],C::S[2]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|typekeys 5 5 5 7 7 7"]);
}

#[test]
fn class_array_negative_bounds() {
    let source = r#"class C;
 localparam int A[-1:-3]='{10,20,30};
 static int S[-1:-3]='{40,50,60};
endclass
module top;initial $display("T|negative %0d %0d %0d %0d %0d %0d",C::A[-1],C::A[-2],C::A[-3],C::S[-1],C::S[-2],C::S[-3]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|negative 10 20 30 40 50 60"]);
}

#[test]
fn class_array_explicit_shadow() {
    let source = r#"class C;
 localparam int A[2]='{1,2};
 static function int f();int A[2]='{7,8};return A[0]+C::A[0];endfunction
endclass
module top;initial $display("T|qualified %0d",C::f());endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|qualified 8"]);
}

#[test]
fn class_array_instance_header_override() {
    let source = r#"class C#(int A[2]='{1,2});
 function int at(int i);return A[i];endfunction
endclass
module top;initial begin C#('{7,8}) c=new;
 $display("T|obj %0d %0d %0d %0d",c.A[0],c.A[1],c.at(0),C#('{7,8})::A[1]);
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|obj 7 8 7 8"]);
}

#[test]
fn class_array_caller_local_parameter() {
    let source = r#"class C#(int N=5);
 localparam int A[2]='{N,N+1};
 static function int f();int N=99;return C#(7)::A[0];endfunction
endclass
module top;initial $display("T|caller %0d",C#()::f());endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|caller 7"]);
}

#[test]
fn class_array_const_instance_property() {
    let source = r#"class C;
 int x=7;
 const int A[1]='{x};
 function int read();return A[0];endfunction
endclass
module top;initial begin C c=new;C d=new;c.x=9;
 $display("T|constprop %0d %0d %0d %0d",c.A[0],c.read(),d.A[0],d.read());
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|constprop 7 7 7 7"]);
}

#[test]
fn class_array_instance_literal_types() {
    let source = r#"class C#(logic signed[7:0] A[3]='{default:0});
 function int read(int i);return A[i];endfunction
endclass
module top;initial begin C#('{8'hf6,-1,'1}) c=new;
 $display("T|literals %0d %0d %0d %0d",c.A[0],c.A[1],c.A[2],c.read(0));
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|literals -10 -1 -1 -10"]);
}

#[test]
fn class_array_instance_bounds() {
    let source = r#"class C#(int N=2,int W=4,int A[N]='{default:1});
 localparam logic[W-1:0] B[N]='{N{'1}};
endclass
module top;initial begin C#(4,6) c=new;
 $display("T|objbounds %0d %0d",c.A[3],c.B[3]);
end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|objbounds 1 63"]);
}

#[test]
fn class_array_module_name_collision() {
    let source = r#"class C;localparam int A[2]='{1,2};endclass
module top;int A[2]='{9,9};initial $display("T|collision %0d %0d",C::A[0],A[0]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|collision 1 9"]);
}

#[test]
fn class_array_method_module_collision() {
    let source = r#"class C;localparam int A[2]='{1,2};static function int read();return A[0];endfunction endclass
module top;int A[2]='{9,9};initial $display("T|methodcollision %0d %0d",C::read(),A[0]);endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|methodcollision 1 9"]);
}

#[test]
fn class_array_derived_field_shadow() {
    let source = r#"class B;localparam int A[2]='{1,2};endclass
class C extends B;int A[2]='{7,8};function int read();return A[0];endfunction endclass
module top;initial begin C c=new;$display("T|derivedshadow %0d %0d",c.read(),B::A[0]);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|derivedshadow 7 1"]);
}

#[test]
fn class_array_package_cache_and_artifact() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!(
        "xezim-class-array-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(dir.clone());
    std::fs::write(
        dir.join("input.sv"),
        r#"
package p;
  class C #(int N=3);
    localparam int A[2]='{N,N+1};
  endclass
endpackage
module top;
  initial $display("T|array %0d %0d",p::C#(7)::A[1],p::C#()::A[0]);
endmodule
"#,
    )
    .unwrap();
    let run = |args: &[&str], expected: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_xezim"))
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8_lossy(&out.stdout);
        let actual = text
            .lines()
            .filter(|line| line.starts_with("T|"))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{text}");
    };
    run(&["--no-cache", "-s", "top", "input.sv"], &["T|array 8 3"]);
    for _ in 0..2 {
        run(
            &["--cache-dir", "cache", "-s", "top", "input.sv"],
            &["T|array 8 3"],
        );
    }
    run(
        &[
            "--compile",
            "--no-cache",
            "-s",
            "top",
            "-o",
            "input.xez",
            "input.sv",
        ],
        &[],
    );
    run(&["--simulate", "input.xez"], &["T|array 8 3"]);
}
