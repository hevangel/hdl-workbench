//! §16.5 / §9.4.2: assertion clocking events share the procedural event grammar.

use sv_parser::ast::Description;
use sv_parser::ast::decl::ModuleItem;
use sv_parser::ast::expr::ExprKind;
use sv_parser::ast::stmt::{Edge, EventControl, StatementKind, TimingControl};

#[test]
fn assertion_property_and_sequence_keep_each_event_term() {
    for clock in [
        "posedge clk or negedge rst",
        "posedge clk, negedge rst",
        "(posedge clk or (negedge rst))",
        "((posedge clk), (negedge rst))",
    ] {
        for declaration in [
            format!("assert property (@({clock}) a);"),
            format!("property p; @({clock}) a; endproperty"),
            format!("sequence s; @({clock}) a; endsequence"),
        ] {
            let source = format!("module top; logic clk, rst, a; {declaration} endmodule");
            let result = sv_parser::parse(&source);
            assert!(
                result.errors.is_empty(),
                "{declaration}: {:?}",
                result.errors
            );
            let Description::Module(module) = &result.source.descriptions[0] else {
                panic!()
            };
            let expression = match &module.items[1] {
                ModuleItem::AssertionItem(a) => &a.expr,
                ModuleItem::PropertyDeclaration(p) => p.body.as_ref().unwrap(),
                ModuleItem::SequenceDeclaration(s) => s.body.as_ref().unwrap(),
                other => panic!("{other:?}"),
            };
            let ExprKind::SvaClocked { events, .. } = &expression.kind else {
                panic!()
            };
            assert_eq!(events.len(), 2, "{clock}: {events:?}");
            assert_eq!(events[0].edge, Some(Edge::Posedge));
            assert_eq!(events[1].edge, Some(Edge::Negedge));
        }
    }
}

#[test]
fn guards_and_expression_clocks_keep_their_own_terms() {
    let result = sv_parser::parse(
        "module top; logic a,b,g,h; assert property (@(posedge (a && b) iff g or negedge b iff h or a) 1); endmodule",
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    let Description::Module(module) = &result.source.descriptions[0] else {
        panic!()
    };
    let ModuleItem::AssertionItem(a) = &module.items[1] else {
        panic!()
    };
    let ExprKind::SvaClocked { events, .. } = &a.expr.kind else {
        panic!()
    };
    assert_eq!(events.len(), 3);
    assert!(events[0].iff.is_some());
    assert!(events[1].iff.is_some());
    assert!(events[2].edge.is_none() && events[2].iff.is_none());
}

#[test]
fn ordinary_expression_parentheses_are_not_event_groups() {
    for clock in ["(a + b) * c", "f(a, b)", "((a + b)) * c"] {
        let source = format!("module top; int a,b,c; always @({clock}) a = 1; endmodule");
        let result = sv_parser::parse(&source);
        assert!(result.errors.is_empty(), "{clock}: {:?}", result.errors);
    }
}

#[test]
fn expect_and_always_share_nested_event_lists() {
    for statement in [
        "expect (@((posedge clk), negedge rst) a) a = 1; else a = 0;",
        "@((posedge clk), negedge rst) a = 1;",
    ] {
        let result = sv_parser::parse(&format!(
            "module top; logic clk,rst,a; initial {statement} endmodule"
        ));
        assert!(result.errors.is_empty(), "{statement}: {:?}", result.errors);
        let Description::Module(module) = &result.source.descriptions[0] else {
            panic!()
        };
        let ModuleItem::InitialConstruct(initial) = &module.items[1] else {
            panic!()
        };
        let wait = match &initial.stmt.kind {
            StatementKind::SeqBlock { stmts, .. } => &stmts[0],
            _ => &initial.stmt,
        };
        let StatementKind::TimingControl {
            control: TimingControl::Event(EventControl::EventExpr(events)),
            ..
        } = &wait.kind
        else {
            panic!("{wait:?}")
        };
        assert_eq!(events.len(), 2);
    }
}

#[test]
fn incomplete_event_lists_remain_diagnostics() {
    for clock in [
        "posedge clk or",
        "posedge clk,",
        "posedge clk or negedge",
        "",
    ] {
        let result = sv_parser::parse(&format!(
            "module top; logic clk; assert property (@({clock}) 1); endmodule"
        ));
        assert!(!result.errors.is_empty(), "{clock}");
    }
}
