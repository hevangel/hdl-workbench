//! LRM 6.20.2, 8.25 and A.4.1.1: named inheritance arguments bind by
//! parameter identity before type substitution, preserving omitted defaults.

fn check(source: &str, expected: &[&str]) {
    let sim = xezim::simulate(source, 100).expect("simulate named inheritance");
    let got: Vec<&str> = sim
        .output
        .iter()
        .map(|line| line.message.as_str())
        .filter(|line| line.starts_with("T|"))
        .collect();
    assert_eq!(got, expected);
}

#[test]
fn inherited_task_keeps_its_declaring_type_parameters() {
    check(
        r#"
class task_base #(type ITEM = byte, int MARK = 1);
  task report();
    #1;
    $display("T|task %0d %0d", $bits(ITEM), MARK);
  endtask
endclass
class task_leaf #(type ITEM = shortint) extends task_base #(.MARK(7));
endclass
module top;
  initial begin
    task_leaf #(int) first = new();
    task_leaf #(longint) second = new();
    first.report();
    second.report();
    $finish;
  end
endmodule
"#,
        &["T|task 8 7", "T|task 8 7"],
    );
}

#[test]
fn named_forwarding_preserves_defaults_constructors_and_cast_identity() {
    if !super::compiled_method_test_env::policies(&[("0", "1000"), ("1", "0")]) {
        return;
    }
    check(
        r#"
class evidence;
  int value = 41;
endclass
class foundation;
  int stamp;
  string route;
  evidence proof;
  function new(int marker = 5);
    stamp = marker;
    route = "F";
    proof = new;
  endfunction
endclass
class variant_root extends foundation;
  function new(int marker = 5);
    super.new(marker + 100);
    route = {route, "V"};
  endfunction
endclass
class layer #(type ANCESTOR = foundation, int SCALE = 3,
              type ITEM = byte, type MIRROR = ITEM) extends ANCESTOR;
  function new(int marker = 7);
    super.new(marker);
    route = {route, "L"};
  endfunction
  function void dimensions(string label);
    $display("T|dims %s %0d %0d %0d", label, SCALE, $bits(ITEM), $bits(MIRROR));
  endfunction
endclass
class relay #(type ANCESTOR = foundation, type ITEM = shortint)
    extends layer #(.MIRROR(ITEM), .ANCESTOR(ANCESTOR));
  function new(int marker = 9);
    super.new(marker);
    route = {route, "R"};
  endfunction
endclass
class upper #(type ANCESTOR = foundation)
    extends relay #(.ANCESTOR(ANCESTOR));
  function new(int marker = 11);
    super.new(marker);
    route = {route, "U"};
  endfunction
endclass
class ordered #(type PARENT = foundation) extends layer #(PARENT);
endclass
class renamed #(type PARENT = foundation) extends layer #(.ANCESTOR(PARENT));
endclass
module top;
  initial begin
    relay #(foundation) direct;
    upper #(variant_root) nested;
    ordered #(foundation) positional;
    renamed #(foundation) distinct;
    foundation base_view;
    variant_root saved_view;
    upper #(variant_root) restored;
    int up_ok, down_ok, reject_ok;
    direct = new(17);
    nested = new(19);
    positional = new;
    distinct = new;
    $display("T|chain %0d %s %0d %s", direct.stamp, direct.route, nested.stamp, nested.route);
    $display("T|proof %0d %0d", direct.proof.value, nested.proof.value);
    direct.dimensions("direct");
    nested.dimensions("nested");
    positional.dimensions("positional");
    distinct.dimensions("distinct");
    up_ok = $cast(base_view, nested);
    down_ok = $cast(restored, base_view);
    saved_view = nested;
    reject_ok = $cast(saved_view, direct);
    $display("T|casts %0d %0d %0d %0d %0d", up_ok, down_ok, reject_ok,
             restored == nested, saved_view == nested);
    $finish;
  end
endmodule
"#,
        &[
            "T|chain 17 FLR 119 FVLRU",
            "T|proof 41 41",
            "T|dims direct 3 8 16",
            "T|dims nested 3 8 16",
            "T|dims positional 3 8 8",
            "T|dims distinct 3 8 8",
            "T|casts 1 1 0 1 1",
        ],
    );
}

#[test]
fn omitted_middle_base_default_keeps_its_specialized_type() {
    if !super::compiled_method_test_env::policies(&[("0", "1000"), ("1", "0")]) {
        return;
    }
    check(
        r#"
class word_root #(type WORD = logic [4:0]);
  WORD payload;
  function new(); payload = '1; endfunction
  function int width(); return $bits(WORD); endfunction
endclass
class envelope #(type ANCESTOR = word_root #(logic [12:0]), int MARK = 4)
    extends ANCESTOR;
  function int marker(); return MARK; endfunction
endclass
class sparse_leaf extends envelope #(.MARK(9));
endclass
class forwarded_leaf #(type ANCESTOR = word_root #(logic [6:0]))
    extends envelope #(.MARK(11), .ANCESTOR(ANCESTOR));
endclass
module top;
  initial begin
    sparse_leaf omitted;
    forwarded_leaf forwarded;
    word_root #(logic [12:0]) broad_view;
    word_root #(logic [6:0]) narrow_view;
    int broad_ok, narrow_ok;
    omitted = new;
    forwarded = new;
    broad_ok = $cast(broad_view, omitted);
    narrow_ok = $cast(narrow_view, forwarded);
    $display("T|default %0d %0d %0h", omitted.width(), omitted.marker(), omitted.payload);
    $display("T|forward %0d %0d %0h", forwarded.width(), forwarded.marker(), forwarded.payload);
    $display("T|casts %0d %0d", broad_ok, narrow_ok);
    $finish;
  end
endmodule
"#,
        &["T|default 13 9 1fff", "T|forward 7 11 7f", "T|casts 1 1"],
    );
}
