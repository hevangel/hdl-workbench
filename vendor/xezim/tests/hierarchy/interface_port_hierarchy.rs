//! Hierarchical names through interface ports (§25.3 / §23.8).
use std::process::Command;

fn run(name: &str, source: &str) -> (bool, String) {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("interface_port_hierarchy")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("top.sv");
    std::fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--no-cache", "-s", "top"])
        .arg(path)
        .output()
        .unwrap();
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[test]
fn interface_port_hierarchy_wait_and_read() {
    let source = include_str!("../issue_cases/interface_port_hierarchy.sv");
    for (name, source) in [
        ("original", source.to_string()),
        (
            "control",
            source.replace("u_dut.u_regs.bus.state", "u_dut.b.state"),
        ),
    ] {
        let (ok, text) = run(name, &source);
        assert!(ok, "{text}");
        assert!(text.contains("state=01a at 1"), "{text}");
    }
}

#[test]
fn interface_port_hierarchy_connection_forms() {
    for (name, port, conn) in [
        ("typed", "bus_if.slv", ".bus(b)"),
        ("plain", "bus_if", ".bus(b)"),
        ("generic", "interface", ".bus(b.slv)"),
        ("ordered", "bus_if.slv", "b.slv"),
        ("wildcard", "bus_if.slv", ".*"),
        ("implicit", "bus_if.slv", ".bus"),
    ] {
        let source = format!(
            r#"
interface bus_if;
 logic [8:0] state;
 modport slv(output state);
endinterface
module regs({port} bus); initial #1 bus.state = 9'h1a; endmodule
module top;
 bus_if b(), bus();
 regs u_regs({conn});
 initial begin
   wait(u_regs.bus.state === 9'h1a);
   $display("T|read %h %0t", u_regs.bus.state, $time);
   u_regs.bus.state[3:0] = 4'h7;
   #1 $display("T|write %h %h", u_regs.bus.state, {actual}.state);
   $finish;
 end
 initial #5 $fatal(1, "port wait did not resume");
endmodule
"#,
            actual = if name == "wildcard" || name == "implicit" {
                "bus"
            } else {
                "b"
            }
        );
        let (ok, text) = run(name, &source);
        assert!(ok, "{name}: {text}");
        assert!(text.contains("T|read 01a 1"), "{name}: {text}");
        assert!(text.contains("T|write 017 017"), "{name}: {text}");
    }
}

#[test]
fn interface_port_hierarchy_nested_and_indexed_scopes() {
    let source = r#"
interface bus_if;
 logic [8:0] state;
 int words[2];
 modport slv(output state, words);
endinterface
module regs(bus_if.slv bus); endmodule
module bridge(bus_if.slv bus); regs u_regs(.bus(bus)); endmodule
module dut;
 bus_if b[2]();
 for(genvar g=0;g<2;g++) begin: bank
   bridge u_bridge(.bus(b[g]));
 end
endmodule
module top;
 dut u_dut();
 wire [8:0] sampled = u_dut.bank[1].u_bridge.u_regs.bus.state;
 initial begin
   u_dut.bank[0].u_bridge.u_regs.bus.state = 9'h11;
   u_dut.bank[1].u_bridge.u_regs.bus.state = 9'h2a;
   u_dut.bank[1].u_bridge.u_regs.bus.words[1] = 37;
   #1;
   $display("T|nested %h %h %h %h %0d", u_dut.bank[0].u_bridge.bus.state,
     u_dut.bank[1].u_bridge.u_regs.bus.state, u_dut.b[0].state, sampled,
     u_dut.bank[1].u_bridge.u_regs.bus.words[1]);
   $finish;
 end
endmodule
"#;
    let (ok, text) = run("nested", source);
    assert!(ok, "{text}");
    assert!(text.contains("T|nested 011 02a 011 02a 37"), "{text}");
}

#[test]
fn interface_port_hierarchy_missing_members_remain_errors() {
    let source = include_str!("../issue_cases/interface_port_hierarchy.sv");
    for (name, bad, missing) in [
        ("port_typo", "u_dut.u_regs.buss.state", "buss"),
        ("member_typo", "u_dut.u_regs.bus.missing", "missing"),
    ] {
        let (ok, text) = run(name, &source.replace("u_dut.u_regs.bus.state", bad));
        assert!(!ok, "{name}: {text}");
        assert!(
            text.contains(&format!("'{missing}' is not declared")),
            "{name}: {text}"
        );
        assert!(!text.contains("Simulation finished"), "{name}: {text}");
    }
}

#[test]
fn interface_port_hierarchy_event_and_subroutine_paths() {
    let source = r#"
interface bus_if;
 logic flag = 0;
 int value = 7;
 task put(input int n); value = n; endtask
 function int get(); return value; endfunction
 modport slv(output flag, import put, get);
endinterface
module regs(bus_if.slv bus); initial #2 bus.flag = 1; endmodule
module dut;
 bus_if b();
 regs u_regs(b);
 task update(); u_regs.bus.put(23); endtask
endmodule
module top;
 dut u_dut();
 initial begin
   @(posedge $root.top.u_dut.u_regs.bus.flag);
   u_dut.update();
   #1 $display("T|event %0t %0d %0d", $time,
     $root.top.u_dut.u_regs.bus.get(), u_dut.b.value);
   $finish;
 end
 initial #5 $fatal(1, "port event did not fire");
endmodule
"#;
    for (name, source, expected) in [
        ("event", source.to_string(), "T|event 3 23 23"),
        (
            "blocking",
            source.replace("value = n;", "#1 value = n;"),
            "T|event 4 23 23",
        ),
    ] {
        let (ok, text) = run(name, &source);
        assert!(ok, "{name}: {text}");
        assert!(text.contains(expected), "{name}: {text}");
    }
}

#[test]
fn interface_port_hierarchy_instance_port_name_collision() {
    let source = r#"
interface bus_if; logic state = 1; endinterface
module regs(bus_if bus);
 int value = 3;
 function int twice(input int n); return 2*n; endfunction
 initial #1 value = twice(value);
endmodule
module top;
 bus_if b();
 regs bus(b);
 initial begin
   #2 $display("T|collision %0d %0d",bus.value,bus.bus.state);
   $finish;
 end
endmodule
"#;
    let (ok, text) = run("collision", source);
    assert!(ok, "{text}");
    assert!(text.contains("T|collision 6 1"), "{text}");
}
