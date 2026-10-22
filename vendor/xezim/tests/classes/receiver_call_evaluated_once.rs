//! A CALL in receiver position of a member or select chain — `me().x`,
//! `me().o.data`, `me().vif.data`, `q.pop_front().addr`, `getp().b` — runs
//! exactly once per evaluation, as the reference simulator does (§13.4: a
//! function call is one operand; §8.5: the member is selected from the one
//! object it returns).
//!
//! The member paths re-evaluate their receiver as they try each storage shape
//! (an object handle, a packed or unpacked struct value, a collection store),
//! and a statement probes its operands again (collection copies, fixed-array
//! shapes, the lvalue width), so a chain rooted at a call ran it up to a dozen
//! times: every side effect repeated, and `q.pop_front().addr` drained the
//! whole queue and returned the wrong element. The receiver's value is now
//! held for the evaluation and for the statement that owns it.

use xezim::simulate;

fn out(src: &str) -> String {
    let sim = simulate(src, 100).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

const SRC: &str = r#"
typedef struct packed { logic [7:0] a; logic [7:0] b; } ps_t;
typedef struct { int a; int b; } us_t;
interface bus_if; logic [7:0] data; endinterface
class item; int addr; function new(int a); addr = a; endfunction endclass
class other;
  int data; int arr[4];
  function new(); data = 77; arr[2] = 5; endfunction
  function int get(); return data + 1; endfunction
endclass
class cfg_c; virtual bus_if vif; endclass
class wrap;
  other o; other vif; int x; int calls;
  ps_t p; us_t u;
  item iq[$];
  function new();
    o = new(); vif = o; x = 9; p = 16'h1234; u.a = 5; u.b = 6;
    for (int i = 0; i < 4; i++) begin item it = new(i * 10); iq.push_back(it); end
  endfunction
  function wrap me(); calls++; return this; endfunction
  function ps_t getp(); calls++; return p; endfunction
  function us_t getu(); calls++; return u; endfunction
  function int r_vif(); return me().vif.data; endfunction
  function int r_prop(); return me().x; endfunction
  function int r_chain(); return me().o.data; endfunction
  function int r_elem(); return me().o.arr[2]; endfunction
  function int r_meth(); return me().o.get(); endfunction
  function int r_expr(); return me().x + 1; endfunction
  function void w_prop(); me().x = 4; endfunction
  function void w_chain(); me().o.data = 6; endfunction
endclass
module tb;
  bus_if bus();
  cfg_c cc;
  wrap w;
  int v;
  initial begin
    cc = new(); cc.vif = bus;
    w = new();
    w.calls = 0; v = w.r_vif();   $display("r_vif=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.r_prop();  $display("r_prop=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.r_chain(); $display("r_chain=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.r_elem();  $display("r_elem=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.r_meth();  $display("r_meth=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.r_expr();  $display("r_expr=%0d calls=%0d", v, w.calls);
    w.calls = 0; w.w_prop();  $display("w_prop x=%0d calls=%0d", w.x, w.calls);
    w.calls = 0; w.w_chain(); $display("w_chain d=%0d calls=%0d", w.o.data, w.calls);
    w.calls = 0; v = w.me().o.data; $display("m_chain=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.me().x;      $display("m_prop=%0d calls=%0d", v, w.calls);
    w.calls = 0; v = w.getp().b;    $display("m_packed=%h calls=%0d", v, w.calls);
    w.calls = 0; v = w.getu().b;    $display("m_unpacked=%0d calls=%0d", v, w.calls);
    v = w.iq.pop_front().addr;      $display("m_pop addr=%0d size=%0d", v, w.iq.size());
    if (w.me().x == 4) $display("m_cond calls=%0d", w.calls);
  end
endmodule
"#;

#[test]
fn receiver_call_with_side_effects_runs_once() {
    let o = out(SRC);
    for expect in [
        "r_vif=77 calls=1",
        "r_prop=9 calls=1",
        "r_chain=77 calls=1",
        "r_elem=5 calls=1",
        "r_meth=78 calls=1",
        "r_expr=10 calls=1",
        "w_prop x=4 calls=1",
        "w_chain d=6 calls=1",
        "m_chain=6 calls=1",
        "m_prop=4 calls=1",
        "m_packed=00000034 calls=1",
        "m_unpacked=6 calls=1",
        "m_pop addr=0 size=3",
        "m_cond calls=2",
    ] {
        assert!(
            o.lines().any(|l| l.trim_end() == expect),
            "expected `{expect}` in:\n{o}"
        );
    }
}
