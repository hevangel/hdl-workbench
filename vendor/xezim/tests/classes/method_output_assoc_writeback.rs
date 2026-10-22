//! §13.5.2: a class METHOD formal declared `output` (or `inout`/`ref`) over an
//! associative array must copy its entries back to the caller's array when the
//! method returns. The class-method plan path tore the callee's queue frame
//! (`pop_and_restore_queue_frame`) BEFORE the associative writeback ran; that
//! unwind deletes every element signal under the formal's dyn key (`@o#N[..]`
//! is registered in the callee's queue frame), so the writeback copied nothing
//! and the caller's array stayed empty.
//!
//! This is the UVM 1800.2 `uvm_domain::get_predecessors_for_successors` /
//! `get_successors_for_predecessors` shape (`output uvm_phase pred[s]` filled
//! by a class method): `uvm_phase::wait_for_self_and_siblings_to_drop` saw an
//! empty sibling set and released `run` early, letting `schedule_phase extract`
//! run ahead of the runtime schedule — a whole cluster of phasing regressions.
//! Reference-validated byte-for-byte.

use xezim::simulate;

fn outs(sim: &xezim::compiler::Simulator) -> Vec<String> {
    sim.output.iter().map(|o| o.message.clone()).collect()
}

/// THE minimal shape: `function void fill(output edges_t o); o[other] = 1;`
/// where the key type is a CLASS (handle-keyed assoc).
#[test]
fn class_method_output_assoc_handle_key() {
    let src = r#"
package P;
  class C; endclass
  typedef bit edges_t[C];
endpackage

module top;
  import P::*;
  class G;
    C other;
    function void fill(output edges_t o);
      o[other] = 1;
    endfunction
  endclass
  G g;
  C c;
  edges_t sib;
  initial begin
    g = new; c = new; g.other = c;
    g.fill(sib);
    $display("IN_CALLER=%0d", sib.num());
    if (sib.num() == 1 && sib.exists(c)) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"IN_CALLER=1".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}

/// int-keyed variant: the bug was key-type-independent (the copy-back itself
/// was dropped, not the key handling).
#[test]
fn class_method_output_assoc_int_key() {
    let src = r#"
module top;
  class G;
    function void fill(output bit o[int]);
      o[42] = 1;
      o[7]  = 1;
    endfunction
  endclass
  G g;
  bit sib[int];
  initial begin
    g = new;
    g.fill(sib);
    $display("IN_CALLER=%0d", sib.num());
    if (sib.num() == 2 && sib[42] == 1 && sib[7] == 1) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"IN_CALLER=2".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}

/// The full UVM pattern: the caller iterates a do-while building the assoc
/// through the output formal, with a delete(this) skip — writes accumulate
/// across iterations and must ALL land in the caller's array.
#[test]
fn class_method_output_assoc_uvm_pred_pattern() {
    let src = r#"
package P;
  class C;
    int id;
    function new(int i); id = i; endfunction
  endclass
  typedef int unsigned edges_t[C];
endpackage

module top;
  import P::*;
  class G;
    C nodes[3];
    function new();
      nodes[0] = new(0);
      nodes[1] = new(1);
      nodes[2] = new(2);
    endfunction
    function void get_predecessors_for_successors(output edges_t o);
      C e;
      int i;
      do begin
        e = nodes[i];
        i++;
        if (e == null || e.id == 1) continue;
        o[e] = 1;
      end while (i < 3);
    endfunction
  endclass
  G g;
  edges_t sib;
  initial begin
    g = new;
    g.get_predecessors_for_successors(sib);
    $display("IN_CALLER=%0d", sib.num());
    if (sib.num() == 2 && sib.exists(g.nodes[0]) && sib.exists(g.nodes[2]))
      $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"IN_CALLER=2".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}

/// `ref` formals bind the caller's namespace directly and must keep working
/// (identity binding), and an `input`-direction assoc formal must NOT write
/// anything back.
#[test]
fn class_method_ref_and_input_assoc() {
    let src = r#"
module top;
  class G;
    function void fill_ref(ref bit o[int]);
      o[5] = 1;
    endfunction
    function void fill_in(input bit o[int], output int n);
      n = o.num();
      o[99] = 1; // must NOT escape to the caller (input = by value)
    endfunction
  endclass
  G g;
  bit r[int], iv[int];
  int n;
  initial begin
    g = new;
    iv[1] = 1; iv[2] = 1;
    g.fill_ref(r);
    g.fill_in(iv, n);
    $display("REF=%0d IN_N=%0d IN_AFTER=%0d", r.num(), n, iv.num());
    if (r.num() == 1 && n == 2 && iv.num() == 2) $display("TAG_PASS");
    else $display("TAG_FAIL");
  end
endmodule
"#;
    let sim = simulate(src, 10).expect("simulate failed");
    let o = outs(&sim);
    assert!(o.contains(&"REF=1 IN_N=2 IN_AFTER=2".to_string()), "{o:?}");
    assert!(o.contains(&"TAG_PASS".to_string()), "{o:?}");
}
