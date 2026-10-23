//! §22.5.1: undefined invocations keep one diagnostic and preserve following code.

use sv_parser::ast::Description;
use sv_parser::ast::decl::ModuleItem;
use sv_parser::ast::stmt::StatementKind;

#[test]
fn reported_invocation_has_no_derivative_parse_errors() {
    let parsed = sv_parser::parse(
        "`define FOO 1\nmodule m; initial begin\n`UNDEFINED_MACRO(1, 2)\nx = 1;\nend endmodule",
    );
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
}

#[test]
fn undefined_invocation_preserves_the_following_assignment() {
    for invocation in [
        "`MISSING(1, 2)",
        "`MISSING()",
        "`MISSING(f(1, g(2, 3)), '{4, 5})",
        "`MISSING(\"), (\", 2)",
        "`MISSING(begin x = 7; end, 2)",
        "`MISSING(1, 2);",
        "`MISSING;",
    ] {
        let source =
            format!("module top; int x; initial begin {invocation}\n x = 1; end endmodule");
        let parsed = sv_parser::parse(&source);
        assert!(
            parsed.errors.is_empty(),
            "{invocation}: {:?}",
            parsed.errors
        );
        assert!(parsed.source_text.contains(invocation));
        let Description::Module(module) = &parsed.source.descriptions[0] else {
            panic!("module missing");
        };
        let ModuleItem::InitialConstruct(initial) = &module.items[1] else {
            panic!("initial missing");
        };
        let StatementKind::SeqBlock { stmts, .. } = &initial.stmt.kind else {
            panic!("block missing");
        };
        assert_eq!(stmts.len(), 2, "{invocation}: {stmts:?}");
        assert!(matches!(stmts[0].kind, StatementKind::Null));
        assert!(matches!(
            stmts[1].kind,
            StatementKind::BlockingAssign { .. }
        ));
    }
}

#[test]
fn invocation_is_one_statement_in_if_and_fork() {
    for body in [
        "if (x) `MISSING(1, 2) else x = 1;",
        "fork `MISSING(1, 2) x = 1; join",
        "begin : scope `MISSING(1, 2) end",
    ] {
        let source = format!("module top; int x; initial {body} endmodule");
        let parsed = sv_parser::parse(&source);
        assert!(parsed.errors.is_empty(), "{body}: {:?}", parsed.errors);
        assert!(parsed.source_text.contains("`MISSING"));
    }
}

#[test]
fn invocation_recovers_at_declaration_boundaries() {
    for source in [
        "`MISSING(1, 2) module top; endmodule",
        "module top; `MISSING(1, 2) int x; endmodule",
        "class agent; `MISSING(1, 2) int x; endclass",
        "module top; function void f(); `MISSING(1, 2) endfunction endmodule",
        "module top; task t(); `MISSING(1, 2) endtask endmodule",
    ] {
        let parsed = sv_parser::parse(source);
        assert!(parsed.errors.is_empty(), "{source}: {:?}", parsed.errors);
        assert!(parsed.source_text.contains("`MISSING"));
    }
}

#[test]
fn unterminated_invocation_does_not_consume_block_terminators() {
    let parsed = sv_parser::parse(
        "module top; initial begin `MISSING(1, 2 end endmodule module after_error; endmodule",
    );
    assert_eq!(parsed.errors.len(), 1, "{:?}", parsed.errors);
    assert!(
        parsed.errors[0]
            .message
            .contains("unterminated undefined macro")
    );
    assert_eq!(parsed.source.descriptions.len(), 2);
}
