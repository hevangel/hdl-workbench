//! §25.9/§23.6: a virtual interface assigned a NESTED interface instance by
//! hierarchical path (`px.vif = a.d;`, `px.vif = g[0].a.d;`) bound nothing,
//! so every task called through the handle returned at once — an AVIP proxy
//! waiting on its BFM's reset never waited.
//!
//! Only the head of a two-segment name was tried (as a modport view of an
//! interface instance) before the name was treated as an object handle; a
//! module instance `a` is neither, and three-segment paths were not looked at
//! at all. The reference's interface instance is now found by its path —
//! through generate-block selects, from the top module name, and upward from
//! the scope doing the assignment. A direct call through a generate-block
//! path (`g[0].a.d.wait_rst()`) resolves the same way.
//!
//! Every expectation below was cross-checked against the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

const IFACE: &str = r#"
interface drv_bfm(input bit pclk, input bit preset_n);
  task wait_rst();
    $display("%0t enter preset_n=%b pclk=%b", $time, preset_n, pclk);
    @(negedge preset_n);
    $display("%0t negedge", $time);
    @(posedge preset_n);
    $display("%0t posedge", $time);
  endtask
endinterface
module agent_bfm(input bit clk, input bit rst);
  drv_bfm d(.pclk(clk), .preset_n(rst));
endmodule
package p;
  class proxy;
    virtual drv_bfm vif;
    task run();
      vif.wait_rst();
      $display("%0t proxy done", $time);
    endtask
  endclass
endpackage
"#;

fn tb(body: &str) -> String {
    format!(
        "{IFACE}module top;\n  import p::*;\n  bit clk, rst;\n  initial begin clk = 0; forever #10 clk = ~clk; end\n  initial begin rst = 1; #15 rst = 0; #20 rst = 1; end\n{body}  initial #100 $finish;\nendmodule\n"
    )
}

fn check(o: &[String], done: &str) {
    let want = [
        "0 enter preset_n=1 pclk=0",
        "15 negedge",
        "35 posedge",
        done,
    ];
    let got: Vec<&String> = o.iter().filter(|l| !l.starts_with("Simulation")).collect();
    assert_eq!(got, want.iter().collect::<Vec<_>>(), "{o:?}");
}

#[test]
fn vif_bound_to_nested_instance_path() {
    let o = lines(&tb(
        "  agent_bfm a(.clk(clk), .rst(rst));\n  initial begin\n    proxy px = new;\n    px.vif = a.d;\n    px.run();\n  end\n",
    ));
    check(&o, "35 proxy done");
}

#[test]
fn vif_bound_through_generate_block_path() {
    let o = lines(&tb(
        "  genvar i;\n  generate for (i = 0; i < 1; i++) begin : g\n    agent_bfm a(.clk(clk), .rst(rst));\n  end endgenerate\n  initial begin\n    proxy px = new;\n    px.vif = g[0].a.d;\n    px.run();\n  end\n",
    ));
    check(&o, "35 proxy done");
}

#[test]
fn vif_bound_by_top_qualified_path() {
    let o = lines(&tb(
        "  agent_bfm a(.clk(clk), .rst(rst));\n  initial begin\n    proxy px = new;\n    px.vif = top.a.d;\n    px.run();\n  end\n",
    ));
    check(&o, "35 proxy done");
}

#[test]
fn direct_task_call_through_generate_block_path() {
    let o = lines(&tb(
        "  genvar i;\n  generate for (i = 0; i < 1; i++) begin : g\n    agent_bfm a(.clk(clk), .rst(rst));\n  end endgenerate\n  initial begin\n    g[0].a.d.wait_rst();\n    $display(\"%0t direct done\", $time);\n  end\n",
    ));
    check(&o, "35 direct done");
}

#[test]
fn vif_bound_from_inside_a_submodule() {
    let src = format!(
        "{IFACE}module env(input bit clk, input bit rst);\n  import p::*;\n  agent_bfm a(.clk(clk), .rst(rst));\n  initial begin\n    proxy px = new;\n    px.vif = a.d;\n    #1 px.run();\n  end\nendmodule\nmodule top;\n  bit clk, rst;\n  initial begin clk = 0; forever #10 clk = ~clk; end\n  initial begin rst = 1; #15 rst = 0; #20 rst = 1; end\n  env e(.clk(clk), .rst(rst));\n  initial #100 $finish;\nendmodule\n"
    );
    let o = lines(&src);
    let want = [
        "1 enter preset_n=1 pclk=0",
        "15 negedge",
        "35 posedge",
        "35 proxy done",
    ];
    let got: Vec<&String> = o.iter().filter(|l| !l.starts_with("Simulation")).collect();
    assert_eq!(got, want.iter().collect::<Vec<_>>(), "{o:?}");
}
