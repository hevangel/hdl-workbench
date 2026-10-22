//! §6.18: a local declared through a typedef with unpacked dims
//! (`typedef int q_t[$]; q_t q;`) is a collection in a compiled body too.
//! The declarator carries no dims, and the compiled path used to keep such a
//! local in a scalar register, so `q[1]` bit-selected the call result.
//! Expected values are the reference simulator's.

use xezim::simulate;

const SRC: &str = r#"
typedef int intq_t[$];
class Reg;
  int base;
  function intq_t get(); return '{base, base + 1}; endfunction
  function intq_t get2(); intq_t t; t.push_back(base); t.push_back(base + 1); return t; endfunction
  function int meth(); intq_t q; q = get(); return q[0] * 10 + q[1]; endfunction
endclass
function automatic int f2(Reg r); intq_t q; int a, b; q = r.get(); a = q[0]; b = q[1]; return a * 10 + b; endfunction
function automatic int f3(Reg r); intq_t q; q = r.get2(); return q.size() * 100 + q[1]; endfunction
function automatic longint loop(Reg r, int n);
  longint acc = 0;
  intq_t q;
  for (int i = 0; i < n; i++) begin
    r.base = i;
    q = r.get();
    acc += q[0] + q[1];
  end
  return acc;
endfunction
module top;
  initial begin
    Reg r = new;
    r.base = 4;
    $display("T|meth=%0d f2=%0d f3=%0d", r.meth(), f2(r), f3(r));
    $display("T|loop=%0d", loop(r, 2000));
  end
endmodule
"#;

#[test]
fn compiled_typedef_local_queue() {
    let _ = super::compiled_method_test_env::eager();
    let sim = simulate(SRC, 10).expect("simulate failed");
    let o: Vec<String> = sim
        .output
        .iter()
        .map(|o| o.message.trim().to_string())
        .collect();
    for want in ["T|meth=45 f2=45 f3=205", "T|loop=4000000"] {
        assert!(o.iter().any(|l| l == want), "missing {want:?} in {o:?}");
    }
}
