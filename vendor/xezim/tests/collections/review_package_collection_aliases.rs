//! §6.18 / §26.3: qualified collection typedefs retain member dimensions.
fn lines(source: &str) -> Vec<String> {
    let sim = xezim::simulate(source, 100).expect("simulate package collection aliases");
    assert_eq!(sim.error_count, 0);
    sim.output
        .iter()
        .filter(|o| o.message.starts_with("T|"))
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn package_queue() {
    let source = r#"package p; typedef int q_t[$]; typedef struct { q_t q; } record_t[string];
 record_t records='{"k":'{q:{1,2,3}}}; endpackage
module top; import p::*; initial begin
 $display("T|queue %0d %0d",records["k"].q.size(),records["k"].q[1]); $finish; end endmodule"#;
    assert_eq!(lines(source), ["T|queue 3 2"]);
}

#[test]
fn qualified_member() {
    let source = r#"package p; typedef int q_t[$]; endpackage
package q; typedef bit [3:0] q_t[2]; typedef struct { p::q_t q; } record_t[string];
 record_t records='{"k":'{q:{20,21,22}}}; endpackage
module top; import q::*; initial begin
 $display("T|owner %0d %0d",records["k"].q.size(),records["k"].q[2]); $finish; end endmodule"#;
    assert_eq!(lines(source), ["T|owner 3 22"]);
}

#[test]
fn dynamic_and_associative() {
    let source = r#"package p; typedef int d_t[]; typedef int a_t[string];
 typedef struct { d_t d; a_t a; } record_t[string]; record_t records; endpackage
module top; import p::*; initial begin records["k"].d=new[3]; records["k"].d[2]=7; records["k"].a["v"]=9;
 $display("T|mixed %0d %0d %0d %0d",records["k"].d.size(),records["k"].d[2],records["k"].a.num(),records["k"].a["v"]); $finish; end endmodule"#;
    assert_eq!(lines(source), ["T|mixed 3 7 1 9"]);
}

#[test]
fn queue_alias_chain() {
    let source = r#"package p; typedef int base_t[$]; typedef base_t alias_t;
 typedef struct { alias_t q; } record_t[string]; record_t records='{"k":'{q:{8,9}}}; endpackage
module top; import p::*; initial begin records["k"].q.push_back(10);
 $display("T|alias %0d %0d %0d",records["k"].q.size(),records["k"].q[0],records["k"].q[2]); $finish; end endmodule"#;
    assert_eq!(lines(source), ["T|alias 3 8 10"]);
}

#[test]
fn module_type_shadow() {
    let source = r#"package p; typedef int q_t[$]; typedef struct { q_t q; } record_t[string];
 record_t records='{"k":'{q:{11,12,13}}}; endpackage
module top; typedef bit q_t[2]; q_t local_q; initial begin
 $display("T|shadow %0d %0d %0d",p::records["k"].q.size(),p::records["k"].q[1],$size(local_q)); $finish; end endmodule"#;
    assert_eq!(lines(source), ["T|shadow 3 12 2"]);
}

#[test]
fn package_matrix() {
    let source = r#"package p; typedef int matrix_t[int][string]; matrix_t values; endpackage
module top; int n,sum;
 initial begin p::values[7]["a"]=1; p::values[7]["b"]=2; p::values[9]["c"]=3;
 foreach(p::values[i,j]) begin n++; sum+=p::values[i][j]; end
 $display("T|matrix %0d %0d",n,sum); $finish; end
endmodule"#;
    assert_eq!(lines(source), ["T|matrix 3 6"]);
}

#[test]
fn scalar_homonyms() {
    let source = r#"package p; typedef int q_t[$]; typedef logic [1:0][7:0] lane_t; endpackage
package q; typedef bit q_t; typedef bit lane_t; endpackage
module top; import q::*; q_t scalar=1; lane_t flag=1;
 initial begin $display("T|scalars %0d %0d bits %0d %0d",scalar,flag,$bits(scalar),$bits(flag)); $finish; end
endmodule"#;
    assert_eq!(lines(source), ["T|scalars 1 1 bits 1 1"]);
}

#[test]
fn separate_module_imports() {
    let source = r#"package p; typedef int q_t[$]; endpackage
package q; typedef bit q_t; endpackage
module left_side; import p::*; q_t values; initial values={3,4,5}; endmodule
module right_side; import q::*; q_t value=1; endmodule
module top; left_side a(); right_side b(); initial begin #1;
 $display("T|imports %0d %0d %0d %0d %0d",a.values.size(),a.values[2],b.value,$bits(b.value),$bits(a.values[0])); $finish; end endmodule"#;
    assert_eq!(lines(source), ["T|imports 3 5 1 1 32"]);
}
