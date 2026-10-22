//! §6.12.2: a real converted to an integral type rounds to the nearest
//! integer, which then wraps to the target width like any integral value. An
//! out-of-range real saturated instead (a real 2^64 in a `longint` read
//! 2^63-1), and a target wider than 64 bits kept only 64 bits (`1.0e30` in a
//! `bit [127:0]`). The expected lines are the reference simulator's.

use std::process::Command;

fn t_lines(tag: &str, src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("xezim_rtoiwrap_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join("top.sv");
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .args(["--no-cache", "-s", "top"])
        .arg(&sv)
        .current_dir(&dir)
        .output()
        .expect("failed to run xezim");
    let _ = std::fs::remove_dir_all(&dir);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .filter(|l| l.starts_with("T|"))
        .map(str::to_string)
        .collect()
}

const MODULE_SRC: &str = r#"
module top;
  longint l; longint unsigned lu; int i; integer g; time t;
  bit [127:0] w; logic [79:0] w80;
  initial begin
    l = 2.0**64;            $display("T|l 2^64 = %0d", l);
    l = 2.0**64 + 2.0**62;  $display("T|l 2^64+2^62 = %0d", l);
    l = 1.0e30;             $display("T|l 1e30 = %0d", l);
    l = -1.0e30;            $display("T|l -1e30 = %0d", l);
    l = -(2.0**63) - 4096.0; $display("T|l -2^63-4096 = %0d", l);
    l = 2.0**63;            $display("T|l 2^63 = %0d", l);
    lu = 1.0e19;            $display("T|lu 1e19 = %0d", lu);
    lu = 2.0**64;           $display("T|lu 2^64 = %0d", lu);
    i = 1.0e30;             $display("T|i 1e30 = %0d", i);
    i = 2.0**64 + 7.0*(2.0**12); $display("T|i 2^64+7*2^12 = %0d", i);
    g = 1.0e30;             $display("T|integer 1e30 = %0d", g);
    t = 2.0**64;            $display("T|time 2^64 = %0d", t);
    w = 1.0e30;             $display("T|w128 1e30 = %0d", w);
    w = 2.0**100;           $display("T|w128 2^100 = %h", w);
    w80 = 2.0**70 + 2.0**10; $display("T|w80 2^70 = %h", w80);
    $display("T|cast longint'(2^64) = %0d", longint'(2.0**64));
    $display("T|cast longint'(1e30) = %0d", longint'(1.0e30));
  end
endmodule
"#;

#[test]
fn out_of_range_real_wraps_to_the_target_width() {
    assert_eq!(
        t_lines("mod", MODULE_SRC, &[]),
        [
            "T|l 2^64 = 0",
            "T|l 2^64+2^62 = 4611686018427387904",
            "T|l 1e30 = 5076964154930102272",
            "T|l -1e30 = -5076964154930102272",
            "T|l -2^63-4096 = 9223372036854771712",
            "T|l 2^63 = -9223372036854775808",
            "T|lu 1e19 = 10000000000000000000",
            "T|lu 2^64 = 0",
            "T|i 1e30 = 0",
            "T|i 2^64+7*2^12 = 28672",
            "T|integer 1e30 = 0",
            "T|time 2^64 = 0",
            "T|w128 1e30 = 1000000000000000019884624838656",
            "T|w128 2^100 = 00000010000000000000000000000000",
            "T|w80 2^70 = 00400000000000000000",
            "T|cast longint'(2^64) = 0",
            "T|cast longint'(1e30) = 5076964154930102272",
        ]
    );
}

const METHOD_SRC: &str = r#"
class C;
  real r;
  function longint asg(real x); longint l; l = x; return l; endfunction
  function int asgi(real x); int i; i = x; return i; endfunction
  function longint ret(real x); return x; endfunction
  function bit [127:0] wide(real x); bit [127:0] w; w = x; return w; endfunction
  function longint prop(); longint l; l = r; return l; endfunction
endclass
function automatic longint f_asg(real x); longint l; l = x; return l; endfunction
module top;
  initial begin
    C c = new;
    real big = 2.0**64, e30 = 1.0e30, m = -(2.0**63) - 4096.0;
    for (int k = 0; k < 2; k++) begin
      $display("T|%0d asg 2^64=%0d 1e30=%0d m=%0d", k, c.asg(big), c.asg(e30), c.asg(m));
      $display("T|%0d asgi 1e30=%0d 2^32+5=%0d", k, c.asgi(e30), c.asgi(2.0**32 + 5.0));
      $display("T|%0d ret 2^64=%0d 1e30=%0d", k, c.ret(big), c.ret(e30));
      $display("T|%0d wide 1e30=%0d", k, c.wide(e30));
      c.r = e30; $display("T|%0d prop 1e30=%0d", k, c.prop());
      $display("T|%0d free 2^64=%0d 1e30=%0d", k, f_asg(big), f_asg(e30));
    end
  end
endmodule
"#;

/// The same conversions inside methods and a free function, interpreted and
/// compiled on the first call.
#[test]
fn out_of_range_real_wraps_in_methods() {
    let mut want = Vec::new();
    for k in 0..2 {
        want.push(format!(
            "T|{k} asg 2^64=0 1e30=5076964154930102272 m=9223372036854771712"
        ));
        want.push(format!("T|{k} asgi 1e30=0 2^32+5=5"));
        want.push(format!("T|{k} ret 2^64=0 1e30=5076964154930102272"));
        want.push(format!("T|{k} wide 1e30=1000000000000000019884624838656"));
        want.push(format!("T|{k} prop 1e30=5076964154930102272"));
        want.push(format!("T|{k} free 2^64=0 1e30=5076964154930102272"));
    }
    assert_eq!(t_lines("meth", METHOD_SRC, &[]), want);
    assert_eq!(
        t_lines("meth_t0", METHOD_SRC, &[("XEZIM_METHOD_TIER", "0")]),
        want
    );
}
