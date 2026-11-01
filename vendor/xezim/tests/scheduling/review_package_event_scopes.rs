//! §15.5 / §26.3: package event objects keep declaration and alias identity.
fn lines(source: &str) -> Vec<String> {
    let sim = xezim::simulate(source, 200).expect("simulate package events");
    assert_eq!(sim.error_count, 0, "unexpected simulation error");
    sim.output
        .iter()
        .filter(|o| o.message.starts_with("T|"))
        .map(|o| o.message.clone())
        .collect()
}

#[test]
fn package_events_with_same_name_are_independent() {
    let source = r#"package p; event pulse; endpackage
package q; event pulse; endpackage
module top;
 int a,b;
 initial begin @(p::pulse); a=$time; end
 initial begin @(q::pulse); b=$time; end
 initial begin
  #3 ->p::pulse;
  $display("T|p flags %0d %0d",p::pulse.triggered,q::pulse.triggered);
  #4 ->>q::pulse;
  #1 $display("T|times %0d %0d flags %0d %0d",a,b,p::pulse.triggered,q::pulse.triggered);
  $finish;
 end
endmodule"#;
    assert_eq!(lines(source), ["T|p flags 1 0", "T|times 3 7 flags 0 0"]);
}

#[test]
fn imported_and_package_task_triggers_use_their_own_event() {
    let source = r#"package p; event pulse; task fire(); ->pulse; endtask endpackage
package q; event pulse; task fire(); ->>pulse; endtask endpackage
module left_side;
 import p::*; int stamp;
 initial begin @pulse; stamp=$time; end
 initial begin #3 ->>pulse; end
endmodule
module right_side;
 import q::*; int stamp;
 initial begin @(pulse); stamp=$time; end
 initial begin #5 ->pulse; end
endmodule
module top; left_side a(); right_side b(); int ptime,qtime;
 initial begin #6; fork
  begin @(p::pulse); ptime=$time; end
  begin @(q::pulse); qtime=$time; end
 join_none
 #1 p::fire(); #2 q::fire();
 #1 $display("T|imports %0d %0d tasks %0d %0d",a.stamp,b.stamp,ptime,qtime); $finish;
 end
endmodule"#;
    assert_eq!(lines(source), ["T|imports 3 5 tasks 7 9"]);
}

#[test]
fn ordinary_package_variable_does_not_take_event_semantics() {
    let source = r#"package p; event pulse; endpackage
package q; bit pulse=0; endpackage
module top; int a,b;
 initial begin @(p::pulse); a=$time; end
 initial begin @(q::pulse); b=$time; end
 initial begin #3 ->p::pulse; #2 q::pulse=1;
 #1 $display("T|mixed %0d %0d value %0d triggered %0d",a,b,q::pulse,p::pulse.triggered); $finish; end
endmodule"#;
    assert_eq!(lines(source), ["T|mixed 3 5 value 1 triggered 0"]);
}

#[test]
fn package_event_typedefs_keep_their_declaring_package() {
    let source = r#"package p; typedef event token_t; token_t pulse; endpackage
package q; typedef bit token_t; token_t pulse=0; endpackage
module top; import p::*; int stamp;
 initial begin @pulse; stamp=$time; end
 initial begin #4 ->p::pulse; #1
 $display("T|typedef %0d %0d",stamp,pulse.triggered); $finish; end
endmodule"#;
    assert_eq!(lines(source), ["T|typedef 4 0"]);
}

#[test]
fn package_event_aliases_share_only_the_selected_object() {
    let source = r#"package p; event pulse; endpackage
package q; event pulse; endpackage
module top; event alias_event; int a,b,c;
 initial begin #1; alias_event=p::pulse;
  fork
   begin @alias_event; a=$time; end
   begin @(p::pulse); b=$time; end
   begin @(q::pulse); c=$time; end
  join_none
  #2 ->alias_event;
  $display("T|alias flags %0d %0d %0d",alias_event.triggered,p::pulse.triggered,q::pulse.triggered);
  #2 ->q::pulse;
  #1 $display("T|alias times %0d %0d %0d",a,b,c); $finish;
 end
endmodule"#;
    assert_eq!(
        lines(source),
        ["T|alias flags 1 1 0", "T|alias times 3 3 5"]
    );
}

#[test]
fn class_and_package_events_with_same_name_do_not_share_triggered_state() {
    let source = r#"package p; event pulse; endpackage
class C; event pulse; endclass
module top; C obj; int a,b;
 initial begin obj=new;
  fork
   begin @(p::pulse); a=$time; end
   begin @(obj.pulse); b=$time; end
  join_none
  #3 ->p::pulse;
  $display("T|class flags %0d %0d",p::pulse.triggered,obj.pulse.triggered);
  #2 ->obj.pulse;
  $display("T|class flags %0d %0d",p::pulse.triggered,obj.pulse.triggered);
  #1 $display("T|class times %0d %0d",a,b); $finish;
 end
endmodule"#;
    assert_eq!(
        lines(source),
        [
            "T|class flags 1 0",
            "T|class flags 0 1",
            "T|class times 3 5"
        ]
    );
}

#[test]
fn package_task_trigger_ignores_callers_same_named_event() {
    let source = r#"package p; event pulse; task fire(); ->pulse; endtask endpackage
module top; event pulse; int a,b;
 initial begin @(p::pulse); a=$time; end
 initial begin @pulse; b=$time; end
 initial begin #3 p::fire(); #2 ->pulse;
 #1 $display("T|shadow %0d %0d",a,b); $finish; end
endmodule"#;
    assert_eq!(lines(source), ["T|shadow 3 5"]);
}

#[test]
fn package_task_nba_trigger_ignores_callers_same_named_event() {
    let source = r#"package p; event pulse; task fire(); ->>pulse; endtask endpackage
module top; event pulse; int a,b;
 initial begin @(p::pulse); a=$time; end
 initial begin @pulse; b=$time; end
 initial begin #3 p::fire(); #2 ->>pulse;
 #1 $display("T|shadow %0d %0d",a,b); $finish; end
endmodule"#;
    assert_eq!(lines(source), ["T|shadow 3 5"]);
}
