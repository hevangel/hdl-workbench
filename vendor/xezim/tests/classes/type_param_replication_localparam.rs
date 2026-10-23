//! A localparam in a type-parameterized class whose replication count comes
//! from `$bits` of the type parameter (`{$bits(DT) - 1{1'b1}}`, the sv-tests
//! `class_test_52` shape). The signed count `0 - 1` used to be read as
//! unsigned — ~4 G copies, concatenated one at a time — so elaboration
//! never finished. Run through the binary under a watchdog so a hang fails
//! the test instead of stalling the suite.

use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn bits_of_type_parameter_as_replication_count_elaborates() {
    let dir = std::env::temp_dir().join(format!("xezim_type_param_repl_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temporary directory");
    let path = dir.join("repl.sv");
    std::fs::write(
        &path,
        "class base; endclass\n\
         class how_wide #(type DT=int) extends base;\n\
         \x20 localparam Max_int = {$bits(DT) - 1{1'b1}};\n\
         endclass\n\
         module tb;\n\
         \x20 initial $display(\"DONE\");\n\
         endmodule\n",
    )
    .expect("write design");
    let mut child = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args([
            "--simulate",
            "-s",
            "tb",
            "--no-cache",
            path.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("run xezim");
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("wait") {
            let mut s = String::new();
            use std::io::Read;
            child.stdout.take().unwrap().read_to_string(&mut s).ok();
            assert!(status.success() && s.contains("DONE"), "run failed:\n{s}");
            return;
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            panic!("xezim did not finish within 20 s");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The same localparam in two specializations takes each one's width:
/// `int` gives 31 ones, `byte` gives 7.
#[test]
fn replication_localparam_width_follows_the_specialization() {
    let sim = xezim::simulate(
        r#"
class base; endclass
class how_wide #(type DT=int) extends base;
  localparam Max_int = {$bits(DT) - 1{1'b1}};
  static function int show(); return $bits(Max_int); endfunction
endclass
module tb;
  initial $display("W_int=%0d W_byte=%0d", how_wide#()::show(), how_wide#(byte)::show());
endmodule
"#,
        100,
    )
    .expect("simulate failed");
    let o: Vec<String> = sim.output.iter().map(|o| o.message.clone()).collect();
    assert!(o.iter().any(|l| l == "W_int=31 W_byte=7"), "{o:?}");
}
