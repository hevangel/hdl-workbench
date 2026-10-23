//! #255: the constraint shapes of a nested configuration class, with every
//! random variable 64 bits or narrower. The reference simulator solves each
//! class; xezim's joint solver failed them in turn:
//!
//! - a packed-struct field set from another variable (`h.NUM == (4'hf >>
//!   pre)`) was judged against the struct's stale value, because the field
//!   is a segment variable that was never written into its struct while the
//!   search ran (§7.2.1, §11.5.1), so the solver reported the set
//!   unsatisfiable;
//! - bits and parts of array elements (`cpa[i][39:16]`, `zsd[i][j]`) were not
//!   split into segments the way selects of a scalar are (§7.4, §11.5.1);
//! - an element or bit at a position that depends on random variables
//!   (`rights[(i << 4) + id[i]]`, `zsd[i][id[i]]`) was only judged once
//!   everything it reads was decided (§7.4, §11.5.1);
//! - an equality whose side wraps at the compared width (`cpa[i][39:16] ==
//!   ofs[i][39:16] + rel[i]`, 24 bits) did not propagate (§11.4.5, §11.8.2);
//! - `$countones` of such a split variable was not modelled (§20.9);
//! - a `foreach` over a state vector used as an iterator (`bit [0:3] it`)
//!   was judged only as a whole (§18.5.8.1);
//! - bounds propagation around an unsatisfiable cycle of equalities stepped
//!   through a 40-bit range one value per round.

use xezim::simulate;

const SRC: &str = r#"
typedef logic [3:0] u4_t;
typedef struct packed {
   bit [1:0] TYPE; bit [1:0] SIZE; bit [1:0] P09; bit [1:0] GATE; bit [3:0] NUM; bit [3:0] FIRST;
} hdr_t;

class sregs;
   int p09_ra = -1, size_ra = -1;
   logic [9:0] gb_tab [9];
   function new(); gb_tab = '{2,3,4,6,8,12,16,24,32}; endfunction
   rand bit [1:0] p09_ds; rand bit [3:0] gb_ds; rand bit en_ds; rand bit size_ds;
   constraint a { p09_ds dist { 0 := 1, 1 := 49 }; gb_ds inside {[0:8]}; }
   constraint r { if (p09_ra != -1) { p09_ds == p09_ra; } if (size_ra != -1) { size_ds == size_ra; } }
endclass

class client;
   rand hdr_t h; rand u4_t pre;
   constraint c1 { pre dist { 0 :/ 15, 1 :/ 7, 2 :/ 2, 3 :/ 1 }; (h.NUM == (4'hf >> pre)); }
   constraint c2 { ({1'b0,h.NUM} + {1'b0,h.FIRST}) <= 1; h.TYPE == 0; }
endclass

// The S02/S03 core: nested rand handles, struct fields, solve...before on
// sub-object members, a state array indexed by a rand, 40-bit products.
class top_cfg;
   rand client cl; rand sregs sr;
   rand bit [15:0] total, per;
   rand bit [39:0] max_a;
   function new(); cl = new(); sr = new(); endfunction
   constraint av { (cl.h.NUM == 1) -> (cl.h.SIZE == 1); }
   constraint same { cl.h.P09 == sr.p09_ds; cl.h.SIZE == sr.size_ds; }
   constraint ord { solve sr.gb_ds before total; solve cl.h.NUM before total; solve total before max_a; }
   constraint comp { per == (sr.gb_tab[sr.gb_ds] << sr.en_ds);
                     total == per * (cl.h.NUM + 1);
                     max_a == ((40'h800_0000 * total) - 1); }
endclass

// Parts of 40-bit elements tied across elements through a wrapping sum.
class parts;
   rand bit [39:0] cpa [0:3];
   rand bit [39:0] ofs [0:3];
   rand bit [23:0] lim [4];
   rand bit [23:0] rel [4];
   constraint c {
      foreach (cpa[i]) {
         ofs[i][39:16] inside { [0 : lim[i]] };
         ofs[i][15:0] == 0;
         cpa[i][15:0] == 0;
         cpa[i][39:16] == ofs[i][39:16] + rel[i];
         cpa[i][39:16] < 24'h1000;
      }
      cpa[0] == cpa[1]; cpa[1] == cpa[2]; cpa[2] == cpa[3];
   }
endclass

// Elements and bits at positions chosen by random indices.
class positions;
   rand bit [3:0] fa [0:3];
   rand bit [3:0] rights [64];
   rand bit [23:0] lim [64];
   rand bit [15:0] zsd [0:3];
   constraint e {
      foreach (fa[i]) {
         rights[(i << 4) + fa[i]] == 4'b1011;
         lim[(i << 4) + fa[i]][23:16] == 8'h0;
         zsd[i][fa[i]] == 1'b0;
      }
      foreach (zsd[i]) $countones(zsd[i]) >= 15;
   }
endclass

// The segment-table blocks, iterated over state vectors as in the report:
// zero-sized segments chosen by dist, monotonic limits, full-access
// segments at random positions, and a common address through a wrapping
// 24-bit sum.
class seg_cfg;
   rand bit [15:0] total;
   rand bit [39:0] max_a;
   rand bit [15:0] zsd [0:3];
   rand bit [3:0]  fa_ids [0:3];
   rand bit [39:0] cpa [0:3];
   rand bit [39:0] ofs [0:3];
   rand bit [23:0] seg_limit [64];
   rand bit [23:0] seg_reloc [64];
   rand bit [3:0]  seg_rights [64];
   bit [0:3] g_it;
   bit [15:1] s_it;
   constraint t { total inside {[2:64]}; max_a == ((40'h800_0000 * total) - 1); solve total before max_a; }
   constraint e {
      foreach (fa_ids[i]) {
         seg_rights[(i << 4) + fa_ids[i]] == 4'b1011;
         seg_limit[(i << 4) + fa_ids[i]][23:16] == 8'h0;
         zsd[i][fa_ids[i]] == 1'b0;
      }
   }
   constraint z {
      foreach (zsd[i]) {
         zsd[i] dist { 0 :/ 9, [1:16'hFFFF] :/ 1 };
         $countones(zsd[i]) <= 15;
      }
   }
   constraint l {
      foreach (g_it[i]) {
         zsd[i][0]  -> (seg_limit[(i << 4) + 0] == 24'h0);
         !zsd[i][0] -> (seg_limit[(i << 4) + 0] != 24'h0);
         foreach (s_it[j]) {
            if (zsd[i][j]) { seg_limit[(i << 4) + j] == seg_limit[(i << 4) + j - 1]; }
            else { seg_limit[(i << 4) + j] > seg_limit[(i << 4) + j - 1]; }
         }
      }
      foreach (g_it[i]) {
         seg_limit[(i << 4)] < max_a[39:16];
         seg_reloc[(i << 4)] < max_a[39:16];
         seg_reloc[(i << 4)][4:0] == 0;
         (seg_limit[(i << 4)] + seg_reloc[(i << 4)]) < max_a[39:16];
         foreach (s_it[j]) {
            seg_limit[(i << 4) + j] < max_a[39:16];
            seg_reloc[(i << 4) + j] < max_a[39:16];
            seg_reloc[(i << 4) + j][4:0] == 0;
            (seg_limit[(i << 4) + j] + seg_reloc[(i << 4) + j]) < max_a[39:16];
         }
      }
   }
   constraint c {
      foreach (g_it[i]) {
         if (fa_ids[i] == 0) {
            ofs[i][39:16] inside { [0 : seg_limit[(i << 4) + fa_ids[i]]] };
         } else {
            ofs[i][39:16] inside { [(seg_limit[(i << 4) + fa_ids[i] - 1] + 1) : seg_limit[(i << 4) + fa_ids[i]]] };
         }
         ofs[i][15:0] == 0;
         cpa[i][15:0] == 0;
         cpa[i][39:16] == ofs[i][39:16] + seg_reloc[(i << 4) + fa_ids[i]];
         cpa[i][39:16] < max_a[39:16];
      }
      cpa[0] == cpa[1]; cpa[1] == cpa[2]; cpa[2] == cpa[3]; cpa[3] == cpa[0];
   }
endclass

// A rand bit position in a rand vector, with $countones.
class one_hot;
   rand bit [15:0] m; rand bit [3:0] p;
   constraint c { m[p] == 1'b1; $countones(m) == 1; p > 9; }
endclass

module top;
  initial begin
    top_cfg t = new();
    parts a = new();
    positions b = new();
    one_hot o = new();
    seg_cfg g = new();
    int ok, bad;
    repeat (3) begin
      ok = t.randomize();
      $display("top_cfg ok=%0d num=%0d first=%0d size=%0d same=%0d max=%0d", ok, t.cl.h.NUM,
               t.cl.h.FIRST, t.cl.h.SIZE, t.cl.h.P09 == t.sr.p09_ds && t.cl.h.SIZE == t.sr.size_ds,
               t.max_a == (40'h800_0000 * t.total) - 1 && t.total == t.per * 2);
      ok = a.randomize();
      bad = 0;
      foreach (a.cpa[i]) begin
        if (a.cpa[i] != a.cpa[0] || a.cpa[i][15:0] != 0 || a.ofs[i][15:0] != 0) bad++;
        if (a.ofs[i][39:16] > a.lim[i] || a.cpa[i][39:16] >= 24'h1000) bad++;
        if (a.cpa[i][39:16] != 24'(a.ofs[i][39:16] + a.rel[i])) bad++;
      end
      $display("parts ok=%0d bad=%0d", ok, bad);
      ok = b.randomize();
      bad = 0;
      foreach (b.fa[i]) begin
        if (b.rights[(i << 4) + b.fa[i]] != 4'b1011 || b.lim[(i << 4) + b.fa[i]][23:16] != 0) bad++;
        if (b.zsd[i][b.fa[i]] != 0 || $countones(b.zsd[i]) != 15) bad++;
      end
      $display("positions ok=%0d bad=%0d", ok, bad);
      ok = o.randomize();
      $display("one_hot ok=%0d good=%0d", ok, o.m == (16'h1 << o.p) && o.p > 9);
      ok = g.randomize();
      bad = 0;
      foreach (g.g_it[i]) begin
        if (g.seg_rights[(i << 4) + g.fa_ids[i]] != 4'b1011) bad++;
        if (g.cpa[i] != g.cpa[0] || g.cpa[i][39:16] >= g.max_a[39:16]) bad++;
        for (int j = 1; j < 16; j++) begin
          if (g.zsd[i][j] && g.seg_limit[(i << 4) + j] != g.seg_limit[(i << 4) + j - 1]) bad++;
          if (!g.zsd[i][j] && g.seg_limit[(i << 4) + j] <= g.seg_limit[(i << 4) + j - 1]) bad++;
        end
      end
      $display("seg_cfg ok=%0d bad=%0d", ok, bad);
    end
    t.sr.p09_ra = 1;
    t.sr.size_ra = 1;
    ok = t.randomize();
    $display("rerandomize ok=%0d p09=%0d num=%0d", ok, t.sr.p09_ds, t.cl.h.NUM);
  end
endmodule
"#;

#[test]
fn issue_255_constraint_shapes_randomize() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    let round = [
        "top_cfg ok=1 num=1 first=0 size=1 same=1 max=1",
        "parts ok=1 bad=0",
        "positions ok=1 bad=0",
        "one_hot ok=1 good=1",
        "seg_cfg ok=1 bad=0",
    ];
    let mut expected: Vec<&str> = Vec::new();
    for _ in 0..3 {
        expected.extend(round);
    }
    expected.push("rerandomize ok=1 p09=1 num=1");
    assert_eq!(out, expected, "{out:?}");
}
