//! §6.19.3: cast target identities survive package/class qualification.

#[test]
fn qualified_package_enum_keeps_identity() {
    let bad = r#"package p; typedef enum logic[1:0] {A=0,B=1} e_t; endpackage
module top; typedef enum logic[1:0] {X=0,Y=1} f_t; f_t fv;
initial begin fv=p::e_t'(1); $finish; end endmodule"#;
    let error = xezim::simulate(bad, 10)
        .err()
        .expect("cross-enum cast must be rejected");
    assert!(
        error.contains("an enum takes only its own members"),
        "{error}"
    );
    let good = r#"package p; typedef enum logic[1:0] {A=0,B=1} e_t; endpackage
module top; p::e_t fv; initial begin fv=p::e_t'(1); $display("T|value %0d",fv); $finish; end endmodule"#;
    let result = xezim::simulate(good, 10).expect("same enum cast is legal");
    assert!(result.output.iter().any(|o| o.message == "T|value 1"));
}

#[test]
fn qualified_package_alias_keeps_identity() {
    let bad = r#"package p; typedef enum logic[1:0] {A=0,B=1} e_t; typedef e_t alias_t; endpackage
module top; typedef enum logic[1:0] {X=0,Y=1} f_t; f_t fv;
initial begin fv=p::alias_t'(1); $finish; end endmodule"#;
    let error = xezim::simulate(bad, 10)
        .err()
        .expect("cross-enum cast must be rejected");
    assert!(
        error.contains("an enum takes only its own members"),
        "{error}"
    );
    let good = r#"package p; typedef enum logic[1:0] {A=0,B=1} e_t; typedef e_t alias_t; endpackage
module top; p::alias_t fv; initial begin fv=p::alias_t'(1); $display("T|value %0d",fv); $finish; end endmodule"#;
    let result = xezim::simulate(good, 10).expect("same enum cast is legal");
    assert!(result.output.iter().any(|o| o.message == "T|value 1"));
}

#[test]
fn qualified_class_enum_keeps_identity() {
    let bad = r#"class C; typedef enum logic[1:0] {A=0,B=1} e_t; endclass
module top; typedef enum logic[1:0] {X=0,Y=1} f_t; f_t fv;
initial begin fv=C::e_t'(1); $finish; end endmodule"#;
    let error = xezim::simulate(bad, 10)
        .err()
        .expect("cross-enum cast must be rejected");
    assert!(
        error.contains("an enum takes only its own members"),
        "{error}"
    );
    let good = r#"class C; typedef enum logic[1:0] {A=0,B=1} e_t; endclass
module top; C::e_t fv; initial begin fv=C::e_t'(1); $display("T|value %0d",fv); $finish; end endmodule"#;
    let result = xezim::simulate(good, 10).expect("same enum cast is legal");
    assert!(result.output.iter().any(|o| o.message == "T|value 1"));
}

#[test]
fn package_class_enum_keeps_identity() {
    let bad = r#"package p; class C; typedef enum logic[1:0] {A=0,B=1} e_t; endclass endpackage
module top; typedef enum logic[1:0] {X=0,Y=1} f_t; f_t fv;
initial begin fv=p::C::e_t'(1); $finish; end endmodule"#;
    let error = xezim::simulate(bad, 10)
        .err()
        .expect("cross-enum cast must be rejected");
    assert!(
        error.contains("an enum takes only its own members"),
        "{error}"
    );
    let good = r#"package p; class C; typedef enum logic[1:0] {A=0,B=1} e_t; endclass endpackage
module top; p::C::e_t fv; initial begin fv=p::C::e_t'(1); $display("T|value %0d",fv); $finish; end endmodule"#;
    let result = xezim::simulate(good, 10).expect("same enum cast is legal");
    assert!(result.output.iter().any(|o| o.message == "T|value 1"));
}

#[test]
fn same_named_package_classes_keep_distinct_enum_identity() {
    let src = r#"
package p; class C; typedef enum logic[1:0] {A=0,B=1} E; endclass endpackage
package q; class C; typedef enum logic[1:0] {X=0,Y=1} E; endclass endpackage
module top; q::C::E fv; initial begin fv=p::C::E'(1); $finish; end endmodule
"#;
    let error = xezim::simulate(src, 10)
        .err()
        .expect("different qualified enum types");
    assert!(
        error.contains("an enum takes only its own members"),
        "{error}"
    );
}
