//! §11.4.13 / §18.5: integral inside bounds share a comparison context,
//! and an unweighted set samples distinct legal values uniformly (#336).
//! The reference simulator verified every neighboring domain and the
//! expected distributions; random sequences need not match between tools.

use std::collections::HashMap;
use xezim::simulate;

fn records(src: &str) -> HashMap<String, HashMap<String, u64>> {
    let sim = simulate(src, 100).expect("simulate");
    assert!(!sim.output.iter().any(|o| o.message.contains("[fatal]")));
    sim.output
        .iter()
        .filter_map(|o| {
            let mut words = o.message.strip_prefix("T|")?.split_whitespace();
            let name = words.next()?.to_string();
            let fields = words
                .filter_map(|w| {
                    let (k, v) = w.split_once('=')?;
                    Some((k.to_string(), v.parse().ok()?))
                })
                .collect();
            Some((name, fields))
        })
        .collect()
}

fn check(c: &HashMap<String, HashMap<String, u64>>, name: &str, lo: u64, hi: u64) {
    let r = c
        .get(name)
        .unwrap_or_else(|| panic!("missing {name}: {c:?}"));
    assert_eq!(r["failed"], 0, "{name}: {r:?}");
    assert_eq!(r["other"], 0, "{name}: {r:?}");
    assert_eq!(r["a"] + r["b"], 4000, "{name}: {r:?}");
    assert!(
        (lo..=hi).contains(&r["b"]),
        "{name}: {r:?}, expected b within {lo}..={hi}"
    );
}

#[test]
fn reported_size_cast_range_remains_reachable() {
    let sim = simulate(ORIGINAL, 100).expect("simulate");
    let lines: Vec<_> = sim.output.iter().map(|o| o.message.trim()).collect();
    assert!(lines.contains(&"PASS"), "{lines:?}");
    for prefix in ["literal bound:", "cast bound:"] {
        let line = lines
            .iter()
            .find(|l| l.starts_with(prefix))
            .expect("range counts");
        let values: Vec<u64> = line
            .split_whitespace()
            .filter_map(|w| w.parse().ok())
            .collect();
        assert!(values[0] >= 180 && values[1] <= 20, "{line}");
    }
}

#[test]
fn seeded_inside_bounds_and_union_values_are_uniform() {
    let c = records(NEIGHBORS);
    // Four thousand seeded draws: the 1/150 singleton has mean26.7;
    // these broad bounds allow sampling noise while rejecting item50/50.
    for name in [
        "literal",
        "cast",
        "wider",
        "unsigned_cast",
        "unsigned_input",
        "signed_literal",
        "mixed_width",
        "dist_each",
    ] {
        check(&c, name, 1, 90);
    }
    check(&c, "positive", 3, 110); // 1/102
    check(&c, "signed_negative", 1, 80); // 1/202
    check(&c, "overlap", 450, 700); // one of seven distinct values, not two list entries
    for name in [
        "dist_total",
        "unsigned_full",
        "signed_full",
        "signed_alias",
        "signed_unsigned",
    ] {
        check(&c, name, 1750, 2250);
    }
}

#[test]
fn scope_parenthesized_foreach_and_inline_sets_are_uniform() {
    let c = records(INLINE);
    for name in [
        "scope_scalar",
        "scope_paren",
        "scope_foreach",
        "class_inline",
        "scope_word",
    ] {
        check(&c, name, 1, 90);
    }
    check(&c, "scope_signed", 1750, 2250);
}

const ORIGINAL: &str = r#"
// a size-cast range bound in an `inside` constraint -- [8'd0 : 8'(148)]
// -- never yields a value from the range; every solution is the other set
// member. The same constraint with a literal bound (8'd148) does.
// (Also visible with the literal: ~50% of draws pick 0xfe, where uniform
// over the 150 legal values -- IEEE 1800-2017 18.5 -- gives ~0.7%.)
class lit_c;
  rand logic [7:0] v;
  constraint c { v inside {[8'd0 : 8'd148], 8'hfe}; }
endclass

class cast_c;
  rand logic [7:0] v;
  constraint c { v inside {[8'd0 : 8'(148)], 8'hfe}; }
endclass

module top;
  initial begin
    lit_c  l = new();
    cast_c c = new();
    int lit_in = 0, lit_fe = 0, cast_in = 0, cast_fe = 0;
    repeat (200) begin
      void'(l.randomize());
      void'(c.randomize());
      if (l.v <= 148) lit_in++;  else if (l.v == 8'hfe) lit_fe++;
      if (c.v <= 148) cast_in++; else if (c.v == 8'hfe) cast_fe++;
    end
    $display("literal bound: %0d in [0:148], %0d 0xfe", lit_in, lit_fe);
    $display("cast bound:    %0d in [0:148], %0d 0xfe", cast_in, cast_fe);
    if (cast_in > 0) $display("PASS");
    else $display("FAIL: cast bound never picks from the range");
    $finish;
  end
endmodule

"#;
const NEIGHBORS: &str = r#"
class literal_c; rand logic [7:0] v; constraint c { v inside {[8'd0:8'd148],8'hfe}; } endclass
class cast_c; rand logic [7:0] v; constraint c { v inside {[8'd0:8'(148)],8'hfe}; } endclass
class positive_c; rand logic [7:0] v; constraint c { v inside {[8'd0:8'(100)],8'hfe}; } endclass
class wider_c; rand logic [7:0] v; constraint c { v inside {[8'd0:9'(148)],8'hfe}; } endclass
class unsigned_cast_c; rand logic [7:0] v; constraint c { v inside {[8'd0:$unsigned(8'(148))],8'hfe}; } endclass
class unsigned_input_c; rand logic [7:0] v; constraint c { v inside {[8'd0:8'($unsigned(148))],8'hfe}; } endclass
class signed_literal_c; rand logic [7:0] v; constraint c { v inside {[8'd0:8'sd148],8'hfe}; } endclass
class mixed_width_c; rand bit [15:0] v; constraint c { v inside {[16'd0:8'(148)],16'hfffe}; } endclass
class overlap_c; rand bit [7:0] v; constraint c { v inside {[0:3],[2:5],5,9,9}; } endclass
class signed_negative_c; rand integer v; constraint c { v inside {[-100:100],-128}; } endclass
class dist_total_c; rand bit [7:0] v; constraint c { v dist {[0:148]:/1,254:/1}; } endclass
class dist_each_c; rand bit [7:0] v; constraint c { v dist {[0:148]:=1,254:=1}; } endclass
class unsigned_full_c; rand bit [63:0] v; constraint c { v inside {[64'd0:64'hffff_ffff_ffff_ffff]}; } endclass
class signed_full_c; rand longint v; constraint c { v inside {[64'sh8000_0000_0000_0000:64'sh7fff_ffff_ffff_ffff]}; } endclass
class signed_alias_c; rand byte v; constraint c { v inside {-1,0,255}; } endclass
class signed_unsigned_c; rand byte v; constraint c { v inside {8'd255,8'd0}; } endclass
module top; initial begin
begin
 automatic literal_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|literal a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic cast_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|cast a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic positive_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=100)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|positive a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic wider_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|wider a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic unsigned_cast_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|unsigned_cast a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic unsigned_input_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|unsigned_input a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic signed_literal_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|signed_literal a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic mixed_width_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==65534)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|mixed_width a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic overlap_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=5)hit_a++; else if(h.v==9)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|overlap a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic signed_negative_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v>=-100 && h.v<=100)hit_a++; else if(h.v==-128)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|signed_negative a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic dist_total_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|dist_total a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic dist_each_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<=148)hit_a++; else if(h.v==254)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|dist_each a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic unsigned_full_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(!h.v[63])hit_a++; else if(h.v[63])hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|unsigned_full a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic signed_full_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v<0)hit_a++; else if(h.v>=0)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|signed_full a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic signed_alias_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v==-1)hit_a++; else if(h.v==0)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|signed_alias a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
begin
 automatic signed_unsigned_c h=new; automatic int hit_a=0,hit_b=0,failed=0,other=0; automatic longint unsigned top_seen=0;
 h.srandom(123);
 repeat(4000) begin
 if(!h.randomize())failed++;
 if(h.v==-1)hit_a++; else if(h.v==0)hit_b++; else other++;
 if(h.v>top_seen)top_seen=h.v;
 end
 $display("T|signed_unsigned a=%0d b=%0d failed=%0d other=%0d max=%0h",hit_a,hit_b,failed,other,top_seen);
 end
$finish;end endmodule"#;
const INLINE: &str = r#"
class inline_c; randc bit [1:0] cycle; rand bit [7:0] v; endclass
module top;
bit[7:0] v, arr[2]; byte signed_value; bit[63:0] word;
initial begin
 automatic inline_c h=new;
 automatic int a,b,f,other;
 void'($urandom(456));
 h.srandom(123);
a=0;b=0;f=0;other=0; repeat(4000)begin if(!(std::randomize(v) with { v inside {[8'd0:8'(148)],8'hfe}; }))f++; if(v<=148)a++; else if(v==254)b++; else other++; end
$display("T|scope_scalar a=%0d b=%0d failed=%0d other=%0d",a,b,f,other);
a=0;b=0;f=0;other=0; repeat(4000)begin if(!(std::randomize(v) with { (v inside {[8'd0:8'(148)],8'hfe}); }))f++; if(v<=148)a++; else if(v==254)b++; else other++; end
$display("T|scope_paren a=%0d b=%0d failed=%0d other=%0d",a,b,f,other);
a=0;b=0;f=0;other=0; repeat(4000)begin if(!(std::randomize(arr) with { foreach(arr[i]) (arr[i] inside {[8'd0:8'(148)],8'hfe}); }))f++; if(arr[0]<=148)a++; else if(arr[0]==254)b++; else other++; end
$display("T|scope_foreach a=%0d b=%0d failed=%0d other=%0d",a,b,f,other);
a=0;b=0;f=0;other=0; repeat(4000)begin if(!(h.randomize() with { (v inside {[8'd0:8'(148)],8'hfe}); }))f++; if(h.v<=148)a++; else if(h.v==254)b++; else other++; end
$display("T|class_inline a=%0d b=%0d failed=%0d other=%0d",a,b,f,other);
a=0;b=0;f=0;other=0; repeat(4000)begin if(!(std::randomize(signed_value) with { signed_value inside {-1,0,255}; }))f++; if(signed_value==-1)a++; else if(signed_value==0)b++; else other++; end
$display("T|scope_signed a=%0d b=%0d failed=%0d other=%0d",a,b,f,other);
a=0;b=0;f=0;other=0; repeat(4000)begin if(!(std::randomize(word) with { word inside {[64'h100000000:64'h100000094],64'h1000000fe}; }))f++; if(word<=64'h100000094)a++; else if(word==64'h1000000fe)b++; else other++; end
$display("T|scope_word a=%0d b=%0d failed=%0d other=%0d",a,b,f,other);
$finish;end endmodule
"#;
