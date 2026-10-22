//! Typed unpacked parameter elements remain constants in every scope (§6.20.2 / §26.3).
fn elaborate(source: &str) -> xezim_core::ElaboratedModule {
    xezim_core::parse_and_elaborate_multi(
        &[source.to_string()],
        Some("top"),
        &[],
        &["top.sv".to_string()],
        &[],
    )
    .expect("elaborate")
    .1
}

#[test]
fn package_array_bounds_and_dependent_constants() {
    let design = elaborate(
        r#"
package p;
 localparam int A[4:5]='{17,29};
 localparam int B=A[5]+2;
 localparam int M[2][2]='{'{3,5},'{7,11}};
 localparam int N=M[0][1];
endpackage
module top; int memory[p::B]; endmodule
"#,
    );
    assert_eq!(design.arrays["p::A"], (4, 5, 32));
    assert_eq!(design.unpacked_decl_dims["p::A"], vec![(4, 5)]);
    assert_eq!(design.parameters["p::B"].to_i64(), Some(31));
    assert_eq!(design.parameters["p::N"].to_i64(), Some(5));
    assert_eq!(
        design.arrays["memory"].1 - design.arrays["memory"].0 + 1,
        31
    );
}

#[test]
fn negative_and_descending_package_indices() {
    let design = elaborate(
        r#"
package p; localparam int A[-2:-1]='{17,29}; localparam int B[5:4]='{31,43}; endpackage
module top; localparam int N=p::A[-1]; localparam int M=p::B[4]; endmodule
"#,
    );
    assert_eq!(design.parameters["N"].to_i64(), Some(29));
    assert_eq!(design.parameters["M"].to_i64(), Some(43));
    assert_eq!(design.unpacked_decl_dims["p::B"], vec![(5, 4)]);
}

#[test]
fn body_array_override_sizes_dependent_declarations() {
    let design = elaborate(
        r#"
module leaf;
 parameter int A[2]='{3,5}; localparam int N=A[1]+2; logic[N-1:0] q;
endmodule
module top; leaf u_default(); leaf #(.A('{7,9})) u_override(); endmodule
"#,
    );
    assert_eq!(design.parameters["u_default.N"].to_i64(), Some(7));
    assert_eq!(design.parameters["u_override.N"].to_i64(), Some(11));
    assert_eq!(design.signals["u_default.q"].width, 7);
    assert_eq!(design.signals["u_override.q"].width, 11);
    assert_eq!(design.parameters["u_override.A[1]"].to_i64(), Some(9));
}

#[test]
fn imported_array_elements_size_constants() {
    let design = elaborate(
        r#"
package p; localparam int A[2]='{3,5}; endpackage
module top; import p::A; localparam int N=A[1]+2; logic[N-1:0] q; endmodule
"#,
    );
    assert_eq!(design.parameters["N"].to_i64(), Some(7));
    assert_eq!(design.signals["q"].width, 7);
}

#[test]
fn qualified_out_of_bounds_does_not_select_local_homonym() {
    let design = elaborate(
        r#"
package p; localparam int A[1]='{17}; endpackage
module top; localparam int A[2]='{91,93}; localparam int N=p::A[1]; endmodule
"#,
    );
    assert_eq!(design.parameters["N"].to_u64(), Some(0));
    let four_state = elaborate(
        r#"
package p; localparam integer A[1]='{17}; endpackage
module top; localparam integer A[2]='{91,93}; localparam integer N=p::A[1]; endmodule
"#,
    );
    assert!(four_state.parameters["N"].has_xz());
    assert!(!design.parameters["N"].has_xz());
}

#[test]
fn imported_array_uses_exporting_package_bounds_and_values() {
    let design = elaborate(
        r#"
package p; localparam int L=4, R=5, V=17; localparam int A[L:R]='{V,V+12}; endpackage
module top; localparam int L=8, R=9, V=91; import p::A; localparam int N=A[5]; logic [N-1:0] data; endmodule
"#,
    );
    assert_eq!(design.arrays["A"], (4, 5, 32));
    assert_eq!(design.parameters["N"].to_u64(), Some(29));
    assert_eq!(design.signals["data"].width, 29);
}
