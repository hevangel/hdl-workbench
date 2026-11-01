//! §6.18 / §26.3: package aliases retain dimensions and element geometry.
#[test]
fn qualified_package_typedefs_keep_collection_dimensions() {
    use xezim_core::ast::types::UnpackedDimension;
    let source = r#"package p;
 typedef int q_t[$]; typedef q_t alias_t;
 typedef int d_t[]; typedef int matrix_t[int][string];
 typedef logic [1:0][7:0] lane_t;
endpackage
package q; typedef bit q_t; typedef bit lane_t; endpackage
module top; endmodule"#;
    let (_, design) =
        xezim_core::parse_and_elaborate_multi(&[source.into()], Some("top"), &[], &[], &[])
            .expect("elaborate qualified collection aliases");
    for name in ["q_t", "alias_t"] {
        assert!(matches!(
            design.typedef_unpacked_dims[&format!("p::{name}")].as_slice(),
            [UnpackedDimension::Queue { .. }]
        ));
    }
    assert!(matches!(
        design.typedef_unpacked_dims["p::d_t"].as_slice(),
        [UnpackedDimension::Unsized(_)]
    ));
    assert_eq!(design.typedef_unpacked_dims["p::matrix_t"].len(), 2);
    assert_eq!(design.typedef_elem_widths["p::lane_t"], 8);
    assert!(!design.typedef_unpacked_dims.contains_key("q::q_t"));
    assert!(!design.typedef_elem_widths.contains_key("q::lane_t"));
}
