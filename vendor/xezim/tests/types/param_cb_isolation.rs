use xezim::simulate;

const SRC: &str = r#"
module top;
    virtual class BaseCB;
        pure virtual function void do_cb(ref string trace[$]);
    endclass

    virtual class ParamCB #(int N = 0) extends BaseCB;
        virtual function void do_param_cb(ref string trace[$], int val);
        endfunction
    endclass

    class CB_Registry #(type T = int, type CB = BaseCB);
        static CB tw_cbs[$];

        static function void add_tw(CB cb);
            tw_cbs.push_back(cb);
        endfunction

        static function void execute_all(ref string trace[$]);
            foreach (tw_cbs[i]) begin
                tw_cbs[i].do_cb(trace);
            end
        endfunction
    endclass

    class ConcreteParamCB #(int N = 0) extends ParamCB #(N);
        int m_n;
        function new();
            m_n = N;
        endfunction

        virtual function void do_cb(ref string trace[$]);
            trace.push_back($sformatf("CB_N=%0d", m_n));
        endfunction
    endclass

    int pass = 1;
    initial begin
        string trace1[$];
        string trace2[$];

        ConcreteParamCB#(1) cb1 = new();
        ConcreteParamCB#(2) cb2 = new();

        CB_Registry#(int, ParamCB#(1))::add_tw(cb1);
        CB_Registry#(int, ParamCB#(2))::add_tw(cb2);

        CB_Registry#(int, ParamCB#(1))::execute_all(trace1);
        CB_Registry#(int, ParamCB#(2))::execute_all(trace2);

        $display("=== N=1 Queue Trace ===");
        foreach (trace1[i]) $display("  [%0d]: %s", i, trace1[i]);

        $display("=== N=2 Queue Trace ===");
        foreach (trace2[i]) $display("  [%0d]: %s", i, trace2[i]);

        if (trace1.size() != 1 || trace1[0] != "CB_N=1") begin
            $display("ERROR: N=1 queue corrupted! Expected [CB_N=1], got %p", trace1);
            pass = 0;
        end

        if (trace2.size() != 1 || trace2[0] != "CB_N=2") begin
            $display("ERROR: N=2 queue corrupted! Expected [CB_N=2], got %p", trace2);
            pass = 0;
        end

        if (pass) begin
            $display("STATUS: PASS - Value-parameterized static callback queues are properly isolated.");
        end else begin
            $display("STATUS: FAIL - Cross-specialization leakage detected!");
        end
    end
endmodule
"#;

fn u(sim: &xezim::compiler::Simulator, n: &str) -> u64 {
    sim.get_signal(&format!("top.{}", n))
        .or_else(|| sim.get_signal(n))
        .unwrap_or_else(|| panic!("signal not found: {}", n))
        .to_u64()
        .unwrap_or_else(|| panic!("{} not u64-able", n))
        & 0xFFFF_FFFF
}

#[test]
fn test_param_cb_isolation() {
    let sim = simulate(SRC, 100).expect("simulate failed");
    assert_eq!(
        u(&sim, "pass"),
        1,
        "Value-parameterized static callback queues must isolate per specialization"
    );
}
