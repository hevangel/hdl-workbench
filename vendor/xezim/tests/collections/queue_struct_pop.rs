//! #272: `x = q.pop_front()` from a class-property queue of an unpacked
//! struct returned zeroed members. §7.10.2.5/§7.10.2.6: the popped element
//! is the method's value, so it must carry every member to the target
//! (local, class property, return value, argument, declaration initializer).
//! Expected values are the reference simulator's.

use xezim::simulate;

#[test]
fn class_queue_pop_front_into_local_struct() {
    let src = r#"
module top;
  class predictor;
    typedef struct { bit [11:0] address; bit accepted; } receipt_t;
    receipt_t receipts[$];
    function void observe_write(bit [11:0] address, bit accepted);
      receipts.push_back('{address, accepted});
      $display("ENQUEUE address=%h accepted=%b size=%0d",address,accepted,receipts.size());
    endfunction
    function void consume(bit [11:0] expected);
      receipt_t receipt;
      $display("PEEK address=%h accepted=%b",receipts[0].address,receipts[0].accepted);
      receipt = receipts.pop_front();
      $display("DEQUEUE address=%h accepted=%b expected=%h",receipt.address,receipt.accepted,expected);
      if(receipt.address != expected || !receipt.accepted) $fatal(1,"receipt corrupted");
    endfunction
  endclass
  predictor p;
  initial begin
    p=new;
    p.observe_write(12'he0,1);
    p.consume(12'he0);
    $display("RECEIPT_QUEUE_PASS"); $finish;
  end
endmodule
"#;
    let sim = simulate(src, 100).expect("simulate failed");
    let msgs: Vec<&str> = sim.output.iter().map(|l| l.message.as_str()).collect();
    assert_eq!(
        msgs,
        [
            "ENQUEUE address=0e0 accepted=1 size=1",
            "PEEK address=0e0 accepted=1",
            "DEQUEUE address=0e0 accepted=1 expected=0e0",
            "RECEIPT_QUEUE_PASS",
        ]
    );
}

#[test]
fn struct_queue_pop_shapes() {
    // pop_front/pop_back; class-property and module-scope queues; nested
    // structs and packed members; targets: local, `this` property, return
    // value, argument, declaration initializer, external `p.q.pop_*()`.
    let sim = simulate(include_str!("queue_struct_pop_shapes.sv"), 100).expect("simulate failed");
    let got: Vec<&str> = sim
        .output
        .iter()
        .filter_map(|l| l.message.strip_prefix("T|"))
        .collect();
    let want = [
        "ENQUEUE address=0e0 accepted=1 size=1",
        "ENQUEUE address=123 accepted=0 size=2",
        "ENQUEUE address=456 accepted=1 size=3",
        "ENQUEUE address=789 accepted=1 size=4",
        "ENQUEUE address=abc accepted=0 size=5",
        "front address=0e0 accepted=1 left=4",
        "back address=abc accepted=0 left=3",
        "decl address=123 accepted=0 left=2",
        "ENQUEUE address=5a5 accepted=1 size=3",
        "into_prop address=456 accepted=1 left=2",
        "take address=789 accepted=1",
        "ext address=5a5 accepted=1 left=0",
        "nested_front id=1 r=111/1 n=5a hi=5 tag=c3 left=2",
        "nested_back id=3 r=333/1 n=77 hi=7 tag=99 left=1",
        "ext_nested id=2 r=222/0 n=a5 tag=3c",
        "mod_front address=0aa accepted=1",
        "mod_back address=0cc accepted=1 left=1",
        "mod_nested id=8 r=888/0 n=56 lo=6 tag=78",
        "mtake address=0bb accepted=0 left=3",
        "arg address=0ff accepted=1",
        "mdecl address=0dd accepted=1 left=1",
        "ENQUEUE address=777 accepted=0 size=1",
        "ext_prop address=777 accepted=0 left=0",
        "mod_nested2 id=7 r=777/1 n=12 lo=2 tag=34 left=0",
    ];
    assert_eq!(got, want);
}
