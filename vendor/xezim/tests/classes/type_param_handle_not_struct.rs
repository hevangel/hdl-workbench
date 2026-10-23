//! A class property typed by a type parameter bound to a class (`REQ req;`
//! of a `uvm_sequence #(item)`-style base, §8.25) is a class handle. The
//! assignment path took it for an unpacked struct whenever a same-named
//! struct variable was live elsewhere — the axi4 AVIP's monitor tasks wait
//! with an `output axi4_write_transfer_char_s req` formal — and spread the
//! new handle over struct members, so `req = item::type_id::create()` left
//! `req` unchanged after the first transaction (null in a new sequence).
//!
//! Expected values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const SRC_SEQ: &str = r#"
typedef struct { bit [7:0] a; bit [7:0] b; } pkt_s;
class item; static int n; int id; function new(); id = n++; endfunction
  static function item create(); item t = new(); return t; endfunction endclass
class sbase #(type REQ = int); REQ req; endclass
class seq extends sbase #(item);
  task body();
    repeat (3) begin
      req = item::create();
      $display("T|seq req.id=%0d", req.id);
      #1;
    end
  endtask
endclass
class plain; item req;
  task body(); repeat (2) begin req = item::create(); $display("T|plain req.id=%0d", req.id); #1; end endtask
endclass
module top;
  task automatic sample(output pkt_s req); #2; req.a = 1; #20; req.b = 2; endtask
  initial begin pkt_s x; sample(x); end
  initial begin
    seq s = new(); plain p = new();
    #1;
    s.body(); p.body();
    s = new(); s.body();
  end
endmodule
"#;

const SRC_SHAPES: &str = r#"
typedef struct { bit [7:0] a; bit [7:0] b; } pkt_s;
class item; static int n; int id; function new(); id = n++; endfunction
  static function item create(); item t = new(); return t; endfunction endclass
class sbase #(type REQ = int); REQ req; REQ rq2; endclass
class mid extends sbase #(item); endclass
class seq extends mid;
  task body();
    REQ lx;
    repeat (2) begin
      this.req = item::create(); $display("T|this req.id=%0d", req.id);
      req = new(); $display("T|new req.id=%0d", req.id);
      rq2 = item::create(); $display("T|rq2.id=%0d", rq2.id);
      lx = item::create(); $display("T|lx.id=%0d", lx.id);
      #1;
    end
  endtask
endclass
module top;
  task automatic sample(output pkt_s req, output pkt_s rq2, output pkt_s lx); #2; req.a = 1; rq2.a = 1; lx.a = 1; #20; req.b = 2; endtask
  initial begin pkt_s x, y, z; sample(x, y, z); end
  initial begin seq s = new(); #3; s.body(); end
endmodule
"#;

/// The inherited `req` of two sequence objects and a plain class property,
/// assigned while a task with a struct formal `req` is suspended.
#[test]
fn type_param_handle_property_beside_live_struct_formal() {
    assert_eq!(
        t_lines(SRC_SEQ),
        [
            "T|seq req.id=0",
            "T|seq req.id=1",
            "T|seq req.id=2",
            "T|plain req.id=3",
            "T|plain req.id=4",
            "T|seq req.id=5",
            "T|seq req.id=6",
            "T|seq req.id=7",
        ]
    );
}

/// `this.req`, `req = new()`, a second property and a `REQ` local, two
/// levels below the parameterized base.
#[test]
fn type_param_handle_shapes_beside_live_struct_formals() {
    assert_eq!(
        t_lines(SRC_SHAPES),
        [
            "T|this req.id=0",
            "T|new req.id=1",
            "T|rq2.id=2",
            "T|lx.id=3",
            "T|this req.id=4",
            "T|new req.id=5",
            "T|rq2.id=6",
            "T|lx.id=7",
        ]
    );
}
