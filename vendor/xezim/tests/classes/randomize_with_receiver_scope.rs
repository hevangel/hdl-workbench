//! §18.7 — the receiver of `obj.randomize() with {…}` names the
//! randomized object only inside the inline block, and only when the object
//! has no property of that name (names resolve in the object's scope
//! first). `pre_randomize`/`post_randomize` read `req` as an ordinary name:
//! a property `req`, or a local `req` of the method. Expected values from
//! the reference simulator.
use xezim::simulate;

fn tagged(src: &str) -> Vec<String> {
    let sim = simulate(src, 10_000).expect("simulate failed");
    sim.output
        .iter()
        .filter_map(|o| o.message.strip_prefix("T|").map(str::to_string))
        .collect()
}

const SHADOW_POST: &str = r#"
module top;
  class B; rand bit [7:0] data[]; constraint c { data.size() == 2; } endclass
  class A;
    rand bit [7:0] data[];
    B req;
    rand bit [7:0] x;
    constraint c { data.size() == 3; }
    function new(); req = new; void'(req.randomize()); req.data[0] = 8'h5a; endfunction
    function void post_randomize();
      $display("T|post req.data[0]=%h data.size=%0d", req.data[0], data.size());
    endfunction
  endclass
  initial begin
    A req = new;
    if (!req.randomize() with { x == 3; }) $display("T|FAIL1");
    $display("T|x=%0d", req.x);
    if (!req.randomize()) $display("T|FAIL2");
  end
endmodule
"#;

const LOCAL_PRE_POST: &str = r#"
module top;
  class B; rand bit [7:0] data[]; endclass
  B other;
  class A;
    rand bit [7:0] data[];
    rand bit [7:0] x;
    constraint c { data.size() == 3; foreach (data[i]) data[i] == 8'h11 * (i + 1); }
    function void pre_randomize();
      B req = new;
      req.data = new[2];
      req.data[0] = 8'h3c;
      $display("T|pre req.data[0]=%h", req.data[0]);
    endfunction
    function void post_randomize();
      B req = new;
      req.data = new[2];
      req.data[0] = 8'h5a;
      $display("T|post req.data[0]=%h data[0]=%h", req.data[0], data[0]);
    endfunction
  endclass
  initial begin
    A req = new;
    if (!req.randomize() with { x == 3; req.data[1] == 8'h22; }) $display("T|FAIL1");
    $display("T|x=%0d d1=%h", req.x, req.data[1]);
  end
endmodule
"#;

const MEMBER_IN_CONSTRAINT: &str = r#"
module top;
  class B; rand bit [7:0] data[]; rand bit [7:0] y; endclass
  class A;
    rand bit [7:0] data[];
    B req;
    rand bit [7:0] x;
    constraint c { data.size() == 3; x == req.data[0]; }
    function new(); req = new; req.data = new[2]; req.data[0] = 8'h5a; req.data[1] = 8'h21; endfunction
  endclass
  initial begin
    A req = new;
    int ok = 1;
    if (!req.randomize()) $display("T|FAIL0");
    $display("T|plain x=%h", req.x);
    if (!req.randomize() with { data[0] == 8'h07; }) $display("T|FAIL1");
    $display("T|w1 x=%h d0=%h", req.x, req.data[0]);
    // §18.7: `req` resolves in the object's scope first: A.req (B), not the caller's handle.
    if (!req.randomize() with { data[1] == req.data[1]; }) $display("T|FAIL2");
    $display("T|w2 x=%h d1=%h", req.x, req.data[1]);
  end
endmodule
"#;

#[test]
fn post_randomize_reads_property_named_like_receiver() {
    assert_eq!(
        tagged(SHADOW_POST),
        [
            "post req.data[0]=5a data.size=3",
            "x=3",
            "post req.data[0]=5a data.size=3"
        ]
    );
}

#[test]
fn pre_post_randomize_read_local_named_like_receiver() {
    assert_eq!(
        tagged(LOCAL_PRE_POST),
        [
            "pre req.data[0]=3c",
            "post req.data[0]=5a data[0]=11",
            "x=3 d1=22"
        ]
    );
}

#[test]
fn property_named_like_receiver_wins_in_constraints() {
    assert_eq!(
        tagged(MEMBER_IN_CONSTRAINT),
        ["plain x=5a", "w1 x=5a d0=07", "w2 x=5a d1=21"]
    );
}
