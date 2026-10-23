//! §23.6 / §6.21: a variable of a named block is reachable by its
//! hierarchical name (`inst.blk.var`), and each instance of a module has its
//! own. Storing an instance process's block locals per instance must keep
//! the named-block path resolving.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1_000_000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// Read and write `a.nb.inner` / `b.nb.inner` from the top; each instance
/// sees its own value.
#[test]
fn named_block_locals_per_instance() {
    let src = r#"// Named-block locals of two instances, read and written hierarchically.
module leaf #(parameter int K = 1);
  initial begin : nb
    int inner;
    inner = 10 * K;
    #2 $display("T|%m sees inner=%0d", inner);
  end
endmodule
module top;
  leaf #(1) a();
  leaf #(2) b();
  initial begin
    #1 $display("T|r a=%0d b=%0d", a.nb.inner, b.nb.inner);
    a.nb.inner = 5;
    $display("T|w a=%0d b=%0d", a.nb.inner, b.nb.inner);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|r a=10 b=20",
        "T|w a=5 b=20",
        "T|top.a.nb sees inner=5",
        "T|top.b.nb sees inner=20",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Nested named blocks, a named block in a task, a named fork block,
/// generate-instance paths and `$root.` paths, read and written.
#[test]
fn named_block_paths_reach_instance_locals() {
    let src = r#"// Hierarchical names into named blocks of multiply-instantiated modules.
module leaf #(parameter int K = 1);
  int tsum;
  initial begin : nb
    int inner;
    inner = 10 * K;
    begin : deep
      int dv;
      dv = 100 * K;
      #3 $display("T|dv=%0d", dv);
    end
  end
  task automatic tk();
    begin : tb
      int tv;
      tv = 7 * K;
      tsum = tv;
    end
  endtask
  initial begin
    fork : fk
      begin : fb
        int fv;
        fv = 1000 * K;
        #3 $display("T|fv=%0d", fv);
      end
    join
  end
  initial #1 tk();
endmodule
module mid #(parameter int K = 1);
  leaf #(K) l();
endmodule
module top;
  leaf #(1) a();
  leaf #(2) b();
  genvar g;
  generate for (g = 0; g < 2; g++) begin : gen
    mid #(g + 5) m();
  end endgenerate
  initial begin
    #2;
    $display("T|r a=%0d b=%0d", a.nb.inner, b.nb.inner);
    $display("T|nest a=%0d b=%0d", a.nb.deep.dv, b.nb.deep.dv);
    $display("T|gen %0d %0d", gen[0].m.l.nb.inner, gen[1].m.l.nb.deep.dv);
    $display("T|root %0d %0d", $root.top.a.nb.inner, $root.top.gen[1].m.l.nb.inner);
    $display("T|fork %0d %0d", a.fk.fb.fv, b.fk.fb.fv);
    $display("T|task %0d %0d", a.tsum, b.tsum);
    a.nb.inner = 3;
    b.nb.deep.dv = 4;
    gen[1].m.l.nb.inner = 9;
    $display("T|w %0d %0d %0d %0d", a.nb.inner, b.nb.inner, b.nb.deep.dv, gen[1].m.l.nb.inner);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|r a=10 b=20",
        "T|nest a=100 b=200",
        "T|gen 50 600",
        "T|root 10 60",
        "T|fork 1000 2000",
        "T|task 7 14",
        "T|w 3 20 4 9",
        "T|dv=100",
        "T|fv=1000",
        "T|dv=4",
        "T|fv=2000",
        "T|dv=500",
        "T|fv=5000",
        "T|dv=600",
        "T|fv=6000",
    ];
    assert_eq!(t_lines(src), expected);
}

/// §21.2.1.7 / §9.3.5: `%m` inside a nested named block or a named fork
/// block of an instance, and inside a named fork block of the top. A block
/// the process waits in kept its name only while it ran straight through.
#[test]
fn percent_m_in_nested_named_blocks() {
    let src = r#"// %m inside nested named blocks and named fork blocks of an instance.
module leaf;
  initial begin : nb
    begin : deep
      #1 $display("T|%m");
    end
  end
  initial begin
    fork : fk
      begin : fb
        #2 $display("T|%m");
      end
    join
  end
endmodule
module top;
  leaf a();
  initial begin : tnb
    fork : tfk
      begin : tfb
        #3 $display("T|%m");
      end
    join
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|top.a.nb.deep", "T|top.a.fk.fb", "T|top.tnb.tfk.tfb"];
    assert_eq!(t_lines(src), expected);
}

/// Neighbours: a named block nested in one the process waits in, an
/// unnamed block inside a named one, named and unnamed fork children, a
/// named fork inside a task's named block, an edge block and a function.
#[test]
fn percent_m_named_block_and_fork_shapes() {
    let src = r#"module leaf;
  reg x = 0;
  initial begin : nb
    $display("T|a0 %m");
    begin : deep
      #1 $display("T|a1 %m");
      begin : deeper
        $display("T|a2 %m");
      end
    end
    begin
      $display("T|a3 %m");
    end
  end
  initial begin
    fork : fk
      begin : fb
        #2 $display("T|b1 %m");
      end
      begin
        #2 $display("T|b2 %m");
      end
    join
    fork
      begin : fb2
        #1 $display("T|b3 %m");
      end
    join
  end
  task automatic t1;
    begin : tb
      $display("T|c1 %m");
      fork : tf
        begin : tfb
          $display("T|c2 %m");
        end
      join
    end
  endtask
  initial #5 t1();
  always @(posedge x) begin : ab
    begin : ab2
      $display("T|d1 %m");
    end
  end
  initial #6 x = 1;
  function void f1;
    begin : fb
      $display("T|e1 %m");
    end
  endfunction
  initial #7 f1();
endmodule
module top;
  leaf a();
  initial begin : tnb
    fork : tfk
      begin : tfb
        #3 $display("T|t1 %m");
      end
    join
  end
  initial begin
    fork : tfk2
      #4 $display("T|t2 %m");
    join_none
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|a0 top.a.nb",
        "T|a1 top.a.nb.deep",
        "T|a2 top.a.nb.deep.deeper",
        "T|a3 top.a.nb",
        "T|b1 top.a.fk.fb",
        "T|b2 top.a.fk",
        "T|t1 top.tnb.tfk.tfb",
        "T|b3 top.a.fb2",
        "T|t2 top.tfk2",
        "T|c1 top.a.t1.tb",
        "T|c2 top.a.t1.tb.tf.tfb",
        "T|d1 top.a.ab.ab2",
        "T|e1 top.a.f1.fb",
    ];
    let mut got = t_lines(src);
    let mut want: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    got.sort();
    want.sort();
    assert_eq!(got, want);
}
