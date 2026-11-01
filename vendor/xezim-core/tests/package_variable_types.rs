//! §26.3: homonymous package variables retain their own declared types.
#[test]
fn package_types_remain_owned_and_typedefs_remain_qualified() {
    use xezim_core::ast::types::{DataType, SimpleType};
    let source = r#"package p; typedef event token_t; token_t pulse; endpackage
package q; typedef bit token_t; token_t pulse; endpackage
module top; bit pulse; endmodule"#;
    let (_, design) =
        xezim_core::parse_and_elaborate_multi(&[source.into()], Some("top"), &[], &[], &[])
            .expect("elaborate package variables");
    let event_type = xezim_core::elaborate::resolve_typedef_chain(
        &design.var_decl_types["p::pulse"],
        &design.typedef_types,
    );
    let value_type = xezim_core::elaborate::resolve_typedef_chain(
        &design.var_decl_types["q::pulse"],
        &design.typedef_types,
    );
    assert!(matches!(
        event_type,
        DataType::Simple {
            kind: SimpleType::Event,
            ..
        }
    ));
    assert!(matches!(value_type, DataType::IntegerVector { .. }));
}
