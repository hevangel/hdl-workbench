//! Name-keyed declaration metadata (struct member layout, declared type, …)
//! that a function's local displaces is put back when the function returns —
//! but only when the name looked live, and a task SUSPENDED in another
//! process is invisible to that check (its frames are swapped out). UVM hit
//! this with `uvm_reg_predictor::write()`'s `uvm_reg_bus_op rw` running while
//! a register access task waited holding `uvm_reg_item rw`: the struct's
//! layout stayed behind, and the task's `rw.status = ...` then wrote a bit
//! slice into the class handle, which read back as null.

use xezim::simulate;

fn run(struct_kw: &str, fn_body: &str) -> Vec<String> {
    let src = format!(
        r#"
typedef enum {{ OK, NOT_OK }} st_e;
typedef struct {struct_kw} {{ int kind; int addr; st_e status; }} op_s;
class item; st_e status = NOT_OK; int kind = 1; endclass
class pred;
  function void write(int x);
    op_s rw;
    {fn_body}
  endfunction
endclass
class map;
  pred p;
  task do_bus(item rw);
    #10;
    rw.status = OK;
    $display("null=%0d status=%s kind=%0d", rw == null, rw.status.name(), rw.kind);
  endtask
endclass
module top;
  initial begin
    map m = new; item it = new;
    m.p = new;
    fork
      #5 m.p.write(3);
      m.do_bus(it);
    join
    $display("item status=%s", it.status.name());
  end
endmodule
"#
    );
    let sim = simulate(&src, 1000).expect("simulate failed");
    sim.output.iter().map(|o| o.message.clone()).collect()
}

fn check(o: &[String]) {
    for w in ["null=0 status=OK kind=1", "item status=OK"] {
        assert!(o.iter().any(|l| l == w), "expected {w:?} in:\n{o:#?}");
    }
}

#[test]
fn unpacked_struct_local_does_not_leak_onto_suspended_handle() {
    check(&run("", "rw.kind = x; rw.status = OK;"));
}

#[test]
fn packed_struct_local_does_not_leak_onto_suspended_handle() {
    check(&run("packed", "rw.kind = x; rw.status = OK;"));
}

/// The declaration alone was enough: the member is never written.
#[test]
fn unwritten_struct_local_does_not_leak_onto_suspended_handle() {
    check(&run("", ""));
}
