//! IEEE 1800-2023 §7.4.5/§7.5/§7.8: a dynamic element retains its
//! associative-array type in class, local, and unpacked-struct storage.
use std::process::Command;

fn run(source: &str, top: &str, tag: &str, tier: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "xezim_dynamic_assoc_{}_{}_{}",
        std::process::id(),
        tag,
        tier
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("design.sv");
    std::fs::write(&file, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "-s", top])
        .arg(&file)
        .env("XEZIM_METHOD_TIER", tier)
        .output()
        .expect("run xezim");
    let stdout = String::from_utf8_lossy(&result.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        result.status.success(),
        "simulation failed:\n{stdout}\n{stderr}"
    );
    std::fs::remove_dir_all(dir).unwrap();
    stdout
}

const ORIGINAL: &str = r#"// dyn_outer_in_class.sv
module top;

    int checked  = 0;
    int failures = 0;

    function automatic void report(string what, bit ok);
        checked++;
        if (ok) $display("PASS: %s", what);
        else begin
            failures++;
            $display("FAIL: %s", what);
        end
    endfunction

    // ---- the affected storage locations, dynamic outer dimension ----
    class prop_dyn;
        int v[][string];
    endclass

    class prop_dyn2;
        int v[][string][string];
    endclass

    class local_dyn;
        function void go();
            int v[][string];
            v = new[2];
            v[0]["A"] = 7;
            report("class method local, dynamic outer", v[0].size() == 1 && v[0]["A"] == 7);
        endfunction
    endclass

    typedef struct { int v[][string]; } st_dyn_t;

    // ---- controls: same storage, fixed and associative outer ----
    class prop_fixed;
        int v[2][string];
    endclass

    class prop_assoc;
        int v[int][string];
    endclass

    class local_fixed;
        function void go();
            int v[2][string];
            v[0]["A"] = 7;
            report("control: class method local, fixed outer", v[0].size() == 1 && v[0]["A"] == 7);
        endfunction
    endclass

    typedef struct { int v[2][string]; } st_fixed_t;

    // ---- control: dynamic outer at module scope ----
    int module_dyn[][string];

    // instances
    prop_dyn     p1;
    prop_dyn2    p2;
    local_dyn    l1;
    st_dyn_t     s1;
    prop_fixed   f1;
    prop_assoc   f2;
    local_fixed  l2;
    st_fixed_t   s2;

    initial begin
        // --- affected ---
        p1 = new();
        p1.v = new[2];
        p1.v[0]["A"] = 7;
        $display("       class property, dynamic outer     : size=%0d exists=%0d",
                 p1.v[0].size(), p1.v[0].exists("A"));
        report("class property, dynamic outer",
               p1.v[0].size() == 1 && p1.v[0]["A"] == 7);

        p2 = new();
        p2.v = new[2];
        p2.v[0]["A"]["B"] = 7;
        $display("       class property, dynamic, 2 levels : size=%0d", p2.v[0].size());
        report("class property, dynamic outer, two associative levels", p2.v[0].size() == 1);

        l1 = new();
        l1.go();

        s1.v = new[2];
        s1.v[0]["A"] = 7;
        $display("       struct member, dynamic outer      : size=%0d", s1.v[0].size());
        report("struct member, dynamic outer", s1.v[0].size() == 1);

        // --- controls ---
        f1 = new();
        f1.v[0]["A"] = 7;
        report("control: class property, fixed outer",
               f1.v[0].size() == 1 && f1.v[0]["A"] == 7);

        f2 = new();
        f2.v[0]["A"] = 7;
        report("control: class property, associative outer",
               f2.v[0].size() == 1 && f2.v[0]["A"] == 7);

        l2 = new();
        l2.go();

        s2.v[0]["A"] = 7;
        report("control: struct member, fixed outer",
               s2.v[0].size() == 1 && s2.v[0]["A"] == 7);

        module_dyn = new[2];
        module_dyn[0]["A"] = 7;
        report("control: module-scope variable, dynamic outer",
               module_dyn[0].size() == 1 && module_dyn[0]["A"] == 7);

        $display("=== %0d check(s), %0d failure(s) ===", checked, failures);
    end

endmodule
"#;

#[test]
fn dynamic_outer_associative_storage_locations() {
    for tier in ["0", "1"] {
        let output = run(ORIGINAL, "top", "original", tier);
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with("PASS:"))
                .count(),
            9,
            "tier={tier}:\n{output}"
        );
        assert!(
            output.contains("=== 9 check(s), 0 failure(s) ==="),
            "tier={tier}:\n{output}"
        );
        assert!(!output.contains("FAIL:"), "tier={tier}:\n{output}");
    }
}

const NEIGHBORS: &str = r#"typedef byte signed key_t;
class matrix;
  int a[][string];
  int b[][string];
  int deep[][string][int];
  int numeric[][key_t];
  function void init(int n);
    a=new[2]; this.b=new[2]; deep=new[2]; numeric=new[2];
    a[0]["A"]=n; this.b[1]["B"]=n+1;
    deep[0]["A"][-3]=n+2;
    numeric[0][511]=n+3;
  endfunction
endclass
class local_matrix;
  function int visit(int n);
    int v[][string];
    v=new[2]; v[0]["A"]=n; v[1]["B"]=n+1;
    return v[0]["A"]+v[1]["B"]+v[0].size()+v[1].size();
  endfunction
endclass
typedef struct { int v[][string]; int w[][int]; } pair_t;
module neighbors;
  int checked=0; int failed=0;
  matrix first; matrix second; local_matrix worker;
  pair_t left; pair_t right;
  task check(string label, bit ok);
    checked++;
    if(ok) $display("T|PASS %s",label);
    else begin failed++; $display("T|FAIL %s",label); end
  endtask
  initial begin
    first=new; second=new; worker=new;
    first.init(10); second.init(20);
    check("instance row and field isolation",first.a[0]["A"]==10 && second.a[0]["A"]==20 && first.a[1].size()==0 && first.b[1]["B"]==11 && second.b[1]["B"]==21);
    check("two associative key levels",first.deep[0].size()==1 && first.deep[0]["A"].size()==1 && first.deep[0]["A"][-3]==12 && second.deep[0]["A"][-3]==22);
    check("signed narrow numeric key conversion",first.numeric[0].size()==1 && first.numeric[0][-1]==13 && second.numeric[0][-1]==23);
    check("method local repeated activation",worker.visit(30)==63 && worker.visit(40)==83 && first.a[0]["A"]==10);
    left.v=new[2]; right.v=new[2]; left.w=new[2]; right.w=new[2];
    left.v[0]["A"]=51; right.v[1]["B"]=52; left.w[1][-2]=53; right.w[0][3]=54;
    check("struct field and instance isolation",left.v[0]["A"]==51 && left.v[1].size()==0 && right.v[0].size()==0 && right.v[1]["B"]==52 && left.w[1][-2]==53 && right.w[0][3]==54);
    first.a=new[3](first.a);
    check("resize preserves rows and initializes tail",first.a.size()==3 && first.a[0]["A"]==10 && first.a[1].size()==0 && first.a[2].size()==0);
    first.a[2]["C"]=61; first.a=new[1](first.a); first.a=new[3](first.a);
    check("shrink discards nested keys",first.a[0]["A"]==10 && first.a[1].size()==0 && first.a[2].size()==0);
    first.a=new[2];
    check("fresh allocation empties retained index",first.a[0].size()==0 && first.a[1].size()==0 && second.a[0]["A"]==20);
    first.a[0]["D"]=71; first.a.delete(); first.a=new[1];
    check("delete and reallocation empty rows",first.a.size()==1 && first.a[0].size()==0 && second.a[0]["A"]==20);
    left.v=new[3](right.v);
    right.v[1]["B"]=99;
    check("cross source resize copies by value",left.v[1]["B"]==52 && left.v[0].size()==0 && left.v[2].size()==0 && right.v[1]["B"]==99);
    first.deep=new[3](second.deep);
    first.deep[0]["A"][-3]=81; second.deep[0]["A"][-3]=82;
    first.deep[1]["X"][260]=83;
    check("two level copy and independent mutations",first.deep[0]["A"][-3]==81 && second.deep[0]["A"][-3]==82 && first.deep[1]["X"][260]==83 && second.deep[1].size()==0 && first.deep[2].size()==0);
    first.numeric=new[2](second.numeric);
    first.numeric[0][255]=91; second.numeric[0][-1]=92;
    check("copied rows preserve key widths",first.numeric[0].size()==1 && first.numeric[0][-1]==91 && second.numeric[0].size()==1 && second.numeric[0][255]==92);
    $display("T|SUMMARY checked=%0d failed=%0d",checked,failed);
  end
endmodule
"#;

#[test]
fn dynamic_associative_rows_lifetime_and_isolation() {
    for tier in ["0", "1"] {
        let output = run(NEIGHBORS, "neighbors", "neighbors", tier);
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with("T|PASS"))
                .count(),
            12,
            "tier={tier}:\n{output}"
        );
        assert!(
            output.contains("T|SUMMARY checked=12 failed=0"),
            "tier={tier}:\n{output}"
        );
        assert!(!output.contains("T|FAIL"), "tier={tier}:\n{output}");
    }
}

const BOUNDS: &str = r#"class holder_bounds;
  int v[][string];
  function bit local_bounds();
    int v[][string];
    v=new[1]; v[0]["A"]=3;
    v[-1]["bad"]=9; v[1]["bad"]=10;
    return v.size()==1 && v[0].size()==1 && v[0]["A"]==3 && v[-1].size()==0 && v[1].size()==0;
  endfunction
  function bit local_deleted();
    int v[][string];
    v=new[1]; v[0]["A"]=3; v.delete(); v[0]["bad"]=10;
    return v.size()==0 && v[0].size()==0;
  endfunction
endclass
typedef struct { int v[][string]; } bounds_t;
module bounds;
  bounds_t s; holder_bounds h;
  int checked=0; int failed=0;
  task check(string label,bit ok);
    checked++;
    if(ok) $display("T|PASS %s",label);
    else begin failed++; $display("T|FAIL %s",label); end
  endtask
  initial begin
    h=new;
    s.v=new[1]; s.v[0]["A"]=7;
    s.v[-1]["bad"]=1; s.v[1]["bad"]=2;
    check("struct negative and size index ignored",s.v.size()==1 && s.v[0]["A"]==7 && s.v[-1].size()==0 && s.v[1].size()==0);
    s.v=new[0]; s.v[0]["bad"]=3;
    check("struct zero size has no rows",s.v.size()==0 && s.v[0].size()==0);
    s.v=new[1]; s.v[0]["A"]=4; s.v.delete(); s.v[0]["bad"]=5;
    check("struct deleted rows cannot be recreated",s.v.size()==0 && s.v[0].size()==0);
    h.v=new[1]; h.v[0]["A"]=11; h.v[-1]["bad"]=12; h.v[1]["bad"]=13;
    check("class negative and size index ignored",h.v.size()==1 && h.v[0]["A"]==11 && h.v[-1].size()==0 && h.v[1].size()==0);
    h.v.delete(); h.v[0]["bad"]=14;
    check("class deleted row remains empty",h.v.size()==0 && h.v[0].size()==0);
    check("local negative and size index ignored",h.local_bounds());
    check("local deleted row remains empty",h.local_deleted());
    $display("T|SUMMARY checked=%0d failed=%0d",checked,failed);
  end
endmodule
"#;

#[test]
fn dynamic_associative_row_bounds() {
    for tier in ["0", "1"] {
        let output = run(BOUNDS, "bounds", "bounds", tier);
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with("T|PASS"))
                .count(),
            7,
            "tier={tier}:\n{output}"
        );
        assert!(
            output.contains("T|SUMMARY checked=7 failed=0"),
            "tier={tier}:\n{output}"
        );
        assert!(!output.contains("T|FAIL"), "tier={tier}:\n{output}");
    }
}
