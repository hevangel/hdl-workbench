//! §23.6: `a.b` needs `a` to be a scope (an instance, a named block, a
//! subroutine) or a struct, interface or class handle. A plain net or port is
//! none of those, and a subroutine's scope holds only its own declarations.
//! The reference simulator rejects both and runs the legal module with the
//! same output.

use xezim::simulate;

#[test]
fn member_of_non_scope() {
    let src = r#"
module module_0 (id_2, id_3, id_18);
  inout id_18;
  input id_3;
  inout id_2;
  assign id_18 = id_3.id_2;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
    let src = r#"
module hier_ref_error();
  task my_task;
    begin : block
    end
  endtask
  initial my_task.missing = 0;
endmodule
"#;
    assert!(simulate(src, 10).is_err());
}

#[test]
fn members_of_scopes() {
    let src = r#"
module leaf;
  reg [3:0] r = 4'd9;
endmodule
module top;
  typedef struct packed { logic [3:0] a; logic [3:0] b; } s_t;
  s_t s = 8'h5c;
  int q[$] = '{1, 2, 3};
  leaf u();
  task my_task;
    reg [3:0] t;
    begin : block
    end
  endtask
  initial begin
    my_task.t = 4'd6;
    #1 $display("M|%0d %h %0d %0d", u.r, s.b, q.size(), my_task.t);
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("scope members are legal");
    assert!(
        sim.output.iter().any(|o| o.message == "M|9 c 3 6"),
        "{:?}",
        sim.output
    );
}
