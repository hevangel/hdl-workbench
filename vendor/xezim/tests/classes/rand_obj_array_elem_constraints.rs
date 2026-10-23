//! §18.4 / §18.5.9: the objects held in a `rand` array of class handles are
//! randomized like a single rand handle, and the enclosing class may
//! constrain their members through an element select (`c[0].x == 6`,
//! `foreach (c[i]) c[i].x == …`). Every such randomize() used to fail:
//!   * the handles of a fixed `rand obj c[N]` were drawn as integers, so the
//!     elements pointed at no object after the first trial, and
//!   * an item on `c[k].fld` was neither pushed into element k's own solve
//!     nor repaired by the enclosing one, so it failed every trial.
//! The element handles themselves are never changed by randomize() (§18.4).
//! Non-rand members of the enclosing object are state variables of the
//! solve (§18.3), so `c[1].x == n` and `if (en) …` are solved as constants.

use xezim::simulate;

const SRC: &str = r#"
class leaf_c;
  rand int z;
endclass
class child_c;
  rand int x;
  rand int y;
  rand leaf_c sub;
  constraint c_y { y inside {[0:100]}; }
  function new(); sub = new(); endfunction
endclass
class fixed_c;
  rand child_c c[2];
  constraint k { c[0].x == 6; c[1].x inside {[3:5]}; c[1].y > 90; }
  function new(); c[0] = new(); c[1] = new(); endfunction
endclass
class one_c;
  rand child_c c[1];
  constraint k { c[0].x == 7; }
  function new(); c[0] = new(); endfunction
endclass
class foreach_c;
  rand child_c c[3];
  constraint k { foreach (c[i]) { c[i].x == i + 10; c[i].sub.z == 2 * i; } }
  function new(); foreach (c[i]) c[i] = new(); endfunction
endclass
class dyn_c;
  rand child_c d[];
  constraint k { d[1].x == 8; foreach (d[i]) d[i].y < 5; }
  function new(); d = new[2]; d[0] = new(); d[1] = new(); endfunction
endclass
class state_c;
  rand child_c c[2];
  int n;
  bit en;
  constraint k { c[1].x == n; if (en) c[0].y == 1; }
  function new(); c[0] = new(); c[1] = new(); endfunction
endclass
class unsat_c;
  rand child_c c[2];
  constraint k { c[0].x == 1; c[0].x == 2; }
  function new(); c[0] = new(); c[1] = new(); endfunction
endclass
module top;
  int fails, moved;
  initial begin
    fixed_c f = new();
    one_c o = new();
    foreach_c fe = new();
    dyn_c d = new();
    state_c t = new();
    unsat_c u = new();
    child_c h0, h1;
    for (int i = 0; i < 20; i++) begin
      h0 = f.c[0]; h1 = f.c[1];
      if (!f.randomize() || f.c[0].x != 6 || !(f.c[1].x inside {[3:5]}) || f.c[1].y <= 90
          || !(f.c[0].y inside {[0:100]})) fails++;
      if (f.c[0] != h0 || f.c[1] != h1) moved++;
      if (!o.randomize() || o.c[0].x != 7) fails++;
      if (!fe.randomize()) fails++;
      foreach (fe.c[j]) if (fe.c[j].x != j + 10 || fe.c[j].sub.z != 2 * j) fails++;
      if (!d.randomize() || d.d[1].x != 8 || d.d[0].y >= 5 || d.d[1].y >= 5) fails++;
      t.n = 20 + i; t.en = i[0];
      if (!t.randomize() || t.c[1].x != 20 + i || (t.en && t.c[0].y != 1)) fails++;
    end
    h0 = u.c[0]; h1 = u.c[1];
    $display("fails=%0d moved=%0d unsat=%0d unsat_moved=%0d", fails, moved, u.randomize(),
             u.c[0] != h0 || u.c[1] != h1);
  end
endmodule
"#;

#[test]
fn rand_obj_array_element_member_constraints() {
    let out: Vec<String> = simulate(SRC, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(out, ["fails=0 moved=0 unsat=0 unsat_moved=0"], "{out:?}");
}

#[test]
fn rand_object_array_state_guards_and_atomic_failure() {
    let src = r#"
class bounded_leaf;
  rand int payload = 0;
  constraint own_rule { payload == 0; }
endclass
class bounded_owner;
  rand bounded_leaf entries[1];
  int requested = 1;
  constraint outer_rule { entries[0].payload == requested; }
  function new(); entries[0] = new(); endfunction
endclass
class state_leaf;
  int payload = 7;
  rand bit toggle;
endclass
class state_owner;
  rand state_leaf entries[1];
  constraint outer_rule { entries[0].payload == 42; }
  function new(); entries[0] = new(); endfunction
endclass
class free_leaf;
  rand int payload = 77;
endclass
class mode_owner;
  rand free_leaf entries[1];
  constraint outer_rule { entries[0].payload == 42; }
  function new(); entries[0] = new(); endfunction
endclass
class conflict_owner;
  rand free_leaf entries[2];
  constraint outer_rule { entries[0].payload == 1; entries[0].payload == 2; }
  function new(); entries[0] = new(); entries[1] = new(); endfunction
endclass
class branch_record;
  rand free_leaf child;
  rand int stamp = 88;
  function new(); child = new(); endfunction
endclass
class shared_owner;
  rand branch_record entries[2];
  constraint outer_rule { entries[0].stamp == 1; entries[0].stamp == 2; }
  function new(); entries[0] = new(); entries[1] = entries[0]; endfunction
endclass
module top;
  initial begin
    bounded_owner bound = new();
    state_owner frozen = new();
    mode_owner disabled = new();
    conflict_owner conflict = new();
    shared_owner shared = new();
    free_leaf mode_leaf;
    int ok;
    ok = bound.randomize();
    $display("own ok=%0d payload=%0d", ok, bound.entries[0].payload);
    bound.requested = 0;
    ok = bound.randomize();
    $display("own_satisfiable ok=%0d payload=%0d", ok, bound.entries[0].payload);
    ok = frozen.randomize();
    $display("state ok=%0d payload=%0d", ok, frozen.entries[0].payload);
    mode_leaf = disabled.entries[0];
    mode_leaf.payload.rand_mode(0);
    ok = disabled.randomize();
    $display("mode ok=%0d payload=%0d", ok, disabled.entries[0].payload);
    mode_leaf.payload.rand_mode(1);
    ok = disabled.randomize();
    $display("mode_enabled ok=%0d payload=%0d", ok, disabled.entries[0].payload);
    ok = conflict.randomize();
    $display("rollback ok=%0d first=%0d second=%0d", ok,
      conflict.entries[0].payload, conflict.entries[1].payload);
    ok = shared.randomize();
    $display("nested_rollback ok=%0d stamp=%0d payload=%0d alias=%0d", ok,
      shared.entries[0].stamp, shared.entries[0].child.payload,
      shared.entries[0] == shared.entries[1]);
    $finish;
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        [
            "own ok=0 payload=0",
            "own_satisfiable ok=1 payload=0",
            "state ok=0 payload=7",
            "mode ok=0 payload=77",
            "mode_enabled ok=1 payload=42",
            "rollback ok=0 first=77 second=77",
            "nested_rollback ok=0 stamp=88 payload=77 alias=1",
        ],
        "{out:?}"
    );
}

#[test]
fn rand_object_array_collection_rollback_and_callback_writes() {
    let src = r#"
class collection_leaf;
  rand int fixed_words[2] = '{11, 12};
  rand int flex_words[] = '{21, 22};
  rand int serial = 77;
  constraint dimensions { flex_words.size() == 3; }
endclass
class collection_owner;
  rand collection_leaf entries[1];
  int requested = 1;
  constraint conflict { entries[0].serial == requested; entries[0].serial == requested + 1; }
  function new(); entries[0] = new(); endfunction
endclass
class mode_leaf;
  rand int serial = 77;
endclass
class mode_owner;
  rand mode_leaf entries[1];
  constraint pin { entries[0].serial == 42; }
  function new(); entries[0] = new(); endfunction
endclass
class hook_leaf;
  rand int serial = 77;
  int marker;
  function void pre_randomize(); serial = 88; marker = 99; endfunction
endclass
class hook_owner;
  rand hook_leaf entries[1];
  constraint conflict { entries[0].serial == 1; entries[0].serial == 2; }
  function new(); entries[0] = new(); endfunction
endclass
module top;
  initial begin
    collection_owner pack = new();
    mode_owner held = new();
    hook_owner hooks = new();
    mode_leaf leaf;
    int ok;
    ok = pack.randomize();
    $display("arrays ok=%0d serial=%0d fixed=%0d,%0d dynamic=%0d,%0d size=%0d", ok,
      pack.entries[0].serial, pack.entries[0].fixed_words[0], pack.entries[0].fixed_words[1],
      pack.entries[0].flex_words[0], pack.entries[0].flex_words[1], pack.entries[0].flex_words.size());
    leaf = held.entries[0];
    leaf.rand_mode(0);
    ok = held.randomize();
    $display("disabled ok=%0d serial=%0d", ok, leaf.serial);
    leaf.rand_mode(1);
    ok = held.randomize();
    $display("enabled ok=%0d serial=%0d", ok, leaf.serial);
    ok = hooks.randomize();
    $display("hooks ok=%0d serial=%0d marker=%0d", ok,
      hooks.entries[0].serial, hooks.entries[0].marker);
    $finish;
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        [
            "arrays ok=0 serial=77 fixed=11,12 dynamic=21,22 size=2",
            "disabled ok=0 serial=77",
            "enabled ok=1 serial=42",
            "hooks ok=0 serial=88 marker=99",
        ],
        "{out:?}"
    );
}

#[test]
fn rand_object_array_unpacked_rand_member_rollback() {
    let src = r#"
typedef struct {
  rand int active;
  int stable;
} record_t;
class record_leaf;
  rand record_t record;
  rand int serial = 77;
  function new(); record.active = 11; record.stable = 22; endfunction
endclass
class record_owner;
  rand record_leaf entries[1];
  int requested = 1;
  constraint conflict { entries[0].serial == requested; entries[0].serial == requested + 1; }
  function new(); entries[0] = new(); endfunction
endclass
module top;
  initial begin
    record_owner data_set = new();
    int ok;
    ok = data_set.randomize();
    $display("record ok=%0d active=%0d stable=%0d serial=%0d", ok,
      data_set.entries[0].record.active, data_set.entries[0].record.stable,
      data_set.entries[0].serial);
    $finish;
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        ["record ok=0 active=11 stable=22 serial=77"],
        "{out:?}"
    );
}

#[test]
fn rand_object_array_nested_callback_collection_writes() {
    let src = r#"
class leaf_record;
  rand int serial = 77;
  rand int fixed_words[2] = '{11, 12};
  rand int flex_words[] = '{21, 22};
endclass
class branch_record;
  rand leaf_record child;
  rand int stamp = 66;
  function new(); child = new(); endfunction
  function void pre_randomize();
    child.serial = 88;
    child.fixed_words[0] = 33;
    child.flex_words = new[3](child.flex_words);
    child.flex_words[2] = 44;
  endfunction
endclass
class owner_record;
  rand branch_record entries[1];
  int requested = 1;
  constraint conflict { entries[0].stamp == requested; entries[0].stamp == requested + 1; }
  function new(); entries[0] = new(); endfunction
endclass
module top;
  initial begin
    owner_record pack = new();
    int ok;
    ok = pack.randomize();
    $display("nested ok=%0d stamp=%0d serial=%0d fixed=%0d,%0d dynamic=%0d,%0d,%0d size=%0d", ok,
      pack.entries[0].stamp, pack.entries[0].child.serial,
      pack.entries[0].child.fixed_words[0], pack.entries[0].child.fixed_words[1],
      pack.entries[0].child.flex_words[0], pack.entries[0].child.flex_words[1],
      pack.entries[0].child.flex_words[2], pack.entries[0].child.flex_words.size());
    $finish;
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        ["nested ok=0 stamp=66 serial=88 fixed=33,12 dynamic=21,22,44 size=3"],
        "{out:?}"
    );
}

#[test]
fn rand_object_array_callback_rebound_child_rollback() {
    let src = r#"
class leaf_record;
  rand int serial;
  function new(int initial_value = 77); serial = initial_value; endfunction
endclass
class branch_record;
  rand leaf_record child;
  rand int stamp = 66;
  bit rebound;
  function new(); child = new(11); endfunction
  function void pre_randomize();
    if (!rebound) begin child = new(); rebound = 1; end
  endfunction
endclass
class owner_record;
  rand branch_record entries[1];
  constraint conflict { entries[0].stamp == 1; entries[0].stamp == 2; }
  function new(); entries[0] = new(); endfunction
endclass
module top;
  initial begin
    owner_record pack = new();
    leaf_record original;
    int ok;
    original = pack.entries[0].child;
    ok = pack.randomize();
    $display("rebind ok=%0d stamp=%0d serial=%0d original=%0d changed=%0d", ok,
      pack.entries[0].stamp, pack.entries[0].child.serial, original.serial,
      original != pack.entries[0].child);
    $finish;
  end
endmodule
"#;
    let out: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect();
    assert_eq!(
        out,
        ["rebind ok=0 stamp=66 serial=77 original=11 changed=1"],
        "{out:?}"
    );
}
