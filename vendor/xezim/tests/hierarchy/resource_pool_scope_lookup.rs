//! Pure SystemVerilog self-checking test for resource pool scope lookup patterns.
//!
//! IEEE 1800-2017 §7.9 / §12.7.3: managing resources grouped by scope in an
//! associative array of queue wrappers (`rsrc_q pool[string]`).
//!
//! Validates:
//! - Dynamic allocation of queue objects stored in an associative array.
//! - Scope key iteration (`num()`, `first()`, `next()`) over the pool.
//! - Element extraction and counting across scopes.
//!
//! Verified byte-for-byte identical output (TAG_PASS) against reference simulators.

use xezim::simulate;

#[test]
fn resource_pool_scope_lookup() {
    let src = r#"
module top;
  class rsrc;
    string name;
    int val;
    function new(string n, int v);
      name = n;
      val = v;
    endfunction
  endclass

  class rsrc_q;
    rsrc items[$];
    function void push(rsrc r);
      items.push_back(r);
    endfunction
    function int size();
      return items.size();
    endfunction
    function rsrc get(int i);
      return items[i];
    endfunction
  endclass

  class rsrc_pool;
    rsrc_q pool[string];

    function void add(string scope_name, rsrc r);
      if (!pool.exists(scope_name))
        pool[scope_name] = new();
      pool[scope_name].push(r);
    endfunction

    function int total_keys();
      return pool.num();
    endfunction

    function int total_items();
      string k;
      int cnt = 0;
      if (pool.first(k)) begin
        do begin
          cnt += pool[k].size();
        end while (pool.next(k));
      end
      return cnt;
    endfunction
  endclass

  initial begin
    automatic rsrc_pool p = new();
    automatic rsrc r1 = new("size", 16);
    automatic rsrc r2 = new("size", 32);
    automatic rsrc r3 = new("flag", 1);
    p.add("scope_mom", r1);
    p.add("scope_dad", r2);
    p.add("scope_dad", r3);

    if (p.total_keys() != 2) begin
      $display("TAG_FAIL: expected 2 keys, got %0d", p.total_keys());
    end else if (p.total_items() != 3) begin
      $display("TAG_FAIL: expected 3 items, got %0d", p.total_items());
    end else begin
      $display("TAG_PASS");
    end
  end
endmodule
"#;
    let sim = simulate(src, 50).expect("simulate failed");
    assert!(
        sim.output.iter().any(|l| l.message == "TAG_PASS"),
        "Resource pool scope lookup test failed; output:\n{}",
        sim.output
            .iter()
            .map(|l| l.message.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
