//! §13.4/§6.21: a subroutine-local declaration does not change the type of a
//! same-named variable in its caller. The width/signedness/declared-type
//! tables are keyed by bare name, so a callee's `logic [7:0] r;` (or a class
//! handle or `string r`) overwrote the entries of the caller's `int r`, which
//! kept the callee's type after the return: a later `r = -1` read back
//! 4294967295, an `inout int` copied back through such a call — UVM's
//! `uvm_resource_db#(int)::read_by_name` — printed unsigned, and a caller's
//! `C1 o; o = new;` constructed the callee's `D1`. Expected output
//! cross-checked against the reference simulator.

use xezim::simulate;

const SRC: &str = r#"
module tb;
  class H; int v; endclass
  class C1; int v = 1; endclass
  class D1; int v = 2; endclass
  class rsrc #(type T = int);
    T val;
    function T read(); return val; endfunction
  endclass
  class db #(type T = int);
    static rsrc #(T) store;
    static function void set(T v);
      store = new;
      store.val = v;
    endfunction
    static function bit read_by_name(input string s, inout T val);
      rsrc #(T) r = store;
      val = r.read();
      return 1;
    endfunction
  endclass
  function automatic void f8();
    logic [7:0] r;
    r = 1;
  endfunction
  function automatic void fh();
    H r;
    r = new;
  endfunction
  function automatic void fs();
    string r;
    r = "hello";
  endfunction
  function automatic bit mrd(inout int val);
    logic [7:0] r;
    val = -1;
    return 1;
  endfunction
  task automatic tio(inout int val);
    logic [3:0] r;
    val = -2;
  endtask
  function automatic void fd();
    D1 o;
    o = new;
  endfunction
  initial begin
    int r;
    bit ok;
    C1 o;
    f8(); r = -1; $display("A r=%0d", r);
    fh(); r = -3; $display("B r=%0d", r);
    fs(); r = 300; $display("C r=%0d", r);
    r = 0; ok = mrd(r); $display("D r=%0d", r);
    r = 0; tio(r); $display("E r=%0d", r);
    db#(int)::set(-1);
    r = 0; ok = db#(int)::read_by_name("x", r); $display("F r=%0d ok=%0d", r, ok);
    r = r - 1; $display("G r=%0d", r);
    fd(); o = new; $display("H o.v=%0d", o.v);
  end
endmodule
"#;

#[test]
fn callee_local_does_not_retype_caller_variable() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    let msgs: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert_eq!(
        msgs,
        [
            "A r=-1",
            "B r=-3",
            "C r=300",
            "D r=-1",
            "E r=-2",
            "F r=-1 ok=1",
            "G r=-2",
            "H o.v=1",
        ],
    );
}
