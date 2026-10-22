//! IEEE 1800-2023 §26.3: a bare subroutine call inside a package subroutine
//! resolves in that package first. With `g` declared in `$unit`, `PA` and
//! `PB`, `PA::f` calling `g(x)` must run `PA::g` whether `PA::f` is
//! interpreted or runs as bytecode (a compiled body used to bind the shared
//! bare name, i.e. `$unit::g` or the other package's `g`).

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

fn all_policies() -> bool {
    super::compiled_method_test_env::policies(&[("1", "0"), ("1", "1000"), ("0", "1000")])
}

const PKGS: &str = r#"
package PA;
  function automatic int g(int x); return x + 1; endfunction
  function automatic int f(int x); return g(x) * 2; endfunction
endpackage
package PB;
  function automatic int g(int x); return x + 2; endfunction
  function automatic int f(int x); return g(x) * 3; endfunction
endpackage
class K;
  function int g(int x); return x + 50; endfunction
  function int h(int x); return g(x) + PA::f(x) + PB::f(x); endfunction
endclass
"#;

#[test]
fn package_function_calls_its_own_package_first() {
    if !all_policies() {
        return;
    }
    let src = format!(
        "function automatic int g(int x); return x + 1000; endfunction\n{PKGS}{}",
        r#"
module top;
  int s1, s2, s3, s4;
  initial begin
    K k = new;
    for (int i = 0; i < 1500; i++) begin
      s1 += PA::f(i); s2 += PB::f(i); s3 += g(i); s4 += k.h(i);
    end
    $display("T|s1=%0d s2=%0d s3=%0d s4=%0d", s1, s2, s3, s4);
  end
endmodule
"#
    );
    assert_eq!(
        t_lines(&src),
        ["T|s1=2251500 s2=3381750 s3=2624250 s4=6832500"]
    );
}

#[test]
fn package_function_scope_without_unit_declaration() {
    if !all_policies() {
        return;
    }
    let src = format!(
        "{PKGS}{}",
        r#"
module top;
  int s1, s2, s4;
  initial begin
    K k = new;
    for (int i = 0; i < 1500; i++) begin
      s1 += PA::f(i); s2 += PB::f(i); s4 += k.h(i);
    end
    $display("T|s1=%0d s2=%0d s4=%0d", s1, s2, s4);
  end
endmodule
"#
    );
    assert_eq!(t_lines(&src), ["T|s1=2251500 s2=3381750 s4=6832500"]);
}
