//! §6.19.6 + §8.10: a method-local declared with an enum TYPEDEF is an enum
//! value — even when the flat by-name maps hold a CLASS type for the same
//! bare name from another scope, and even when the enum's small payload
//! matches a live heap index. Exactly that collision made UVM's
//! `uvm_severity s; s = s.next()` walk dispatch to heap object #1 whenever
//! s==1: the severity seeding loop cycled 0,1,0,… and the report summary
//! dropped its `UVM_ERROR : 0` / `UVM_FATAL : 0` rows (the standard CI grep
//! target). The receiver classifier now lets the declaration binding win.

use xezim::simulate;

const SRC: &str = r#"
package pk;
  typedef enum bit [1:0] { K_A, K_B, K_C, K_D } sev_e;
  class widget;
    function int num(); return 777; endfunction
    function int next(); return 888; endfunction
  endclass
  class user;
    int tally [sev_e];
    function void other_scope();
      widget s;         // class-typed `s` poisons the flat name maps
      s = new;
      void'(s.num());
    endfunction
    function void walk();
      sev_e s;          // enum-typed `s` in a different method
      s = s.first();
      forever begin
        tally[s] = 0;
        if (s == s.last()) break;
        s = s.next();
      end
    endfunction
  endclass
endpackage
module top;
  import pk::*;
  initial begin
    user u = new;
    u.other_scope();    // create a live heap object so handle 1 is valid
    u.walk();
    $display("NOTE: tally=%0d", u.tally.num());
  end
endmodule
"#;

#[test]
fn enum_local_walk_survives_flat_map_collision() {
    let sim = simulate(SRC, 1_000_000).expect("simulate failed");
    let notes: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|l| l.starts_with("NOTE:"))
        .collect();
    assert_eq!(notes, ["NOTE: tally=4"]);
}
