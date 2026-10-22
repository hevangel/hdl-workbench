module top;
  typedef struct { bit [11:0] address; bit accepted; } receipt_t;
  typedef struct packed { bit [3:0] hi; bit [3:0] lo; } nib_t;
  typedef struct { int id; receipt_t r; nib_t n; logic [7:0] tag; } outer_t;
  receipt_t mq[$];
  outer_t mo[$];
  class predictor;
    receipt_t receipts[$];
    outer_t outers[$];
    function void observe_write(bit [11:0] address, bit accepted);
      receipts.push_back('{address, accepted});
      $display("T|ENQUEUE address=%h accepted=%b size=%0d", address, accepted, receipts.size());
    endfunction
    function void consume_front();
      receipt_t receipt;
      receipt = receipts.pop_front();
      $display("T|front address=%h accepted=%b left=%0d", receipt.address, receipt.accepted, receipts.size());
    endfunction
    function void consume_back();
      receipt_t receipt;
      receipt = this.receipts.pop_back();
      $display("T|back address=%h accepted=%b left=%0d", receipt.address, receipt.accepted, receipts.size());
    endfunction
    function void consume_decl();
      receipt_t receipt = receipts.pop_front();
      $display("T|decl address=%h accepted=%b left=%0d", receipt.address, receipt.accepted, receipts.size());
    endfunction
    receipt_t last;
    function void into_prop();
      last = receipts.pop_front();
      $display("T|into_prop address=%h accepted=%b left=%0d", last.address, last.accepted, receipts.size());
    endfunction
    function receipt_t take();
      return receipts.pop_front();
    endfunction
    function void nested();
      outer_t o;
      o = outers.pop_front();
      $display("T|nested_front id=%0d r=%h/%b n=%h hi=%h tag=%h left=%0d", o.id, o.r.address, o.r.accepted, o.n, o.n.hi, o.tag, outers.size());
      o = outers.pop_back();
      $display("T|nested_back id=%0d r=%h/%b n=%h hi=%h tag=%h left=%0d", o.id, o.r.address, o.r.accepted, o.n, o.n.hi, o.tag, outers.size());
    endfunction
  endclass
  predictor p;
  receipt_t x;
  outer_t y;
  function automatic receipt_t mtake();
    return mq.pop_front();
  endfunction
  function automatic void show(receipt_t r);
    $display("T|arg address=%h accepted=%b", r.address, r.accepted);
  endfunction
  initial begin
    p = new;
    p.observe_write(12'he0, 1);
    p.observe_write(12'h123, 0);
    p.observe_write(12'h456, 1);
    p.observe_write(12'h789, 1);
    p.observe_write(12'habc, 0);
    p.consume_front();
    p.consume_back();
    p.consume_decl();
    p.observe_write(12'h5a5, 1);
    p.into_prop();
    x = p.take();
    $display("T|take address=%h accepted=%b", x.address, x.accepted);
    x = p.receipts.pop_front();
    $display("T|ext address=%h accepted=%b left=%0d", x.address, x.accepted, p.receipts.size());
    p.outers.push_back('{1, '{12'h111, 1}, 8'h5a, 8'hc3});
    p.outers.push_back('{2, '{12'h222, 0}, 8'ha5, 8'h3c});
    p.outers.push_back('{3, '{12'h333, 1}, 8'h77, 8'h99});
    p.nested();
    y = p.outers.pop_front();
    $display("T|ext_nested id=%0d r=%h/%b n=%h tag=%h", y.id, y.r.address, y.r.accepted, y.n, y.tag);
    mq.push_back('{12'h0aa, 1});
    mq.push_back('{12'h0bb, 0});
    mq.push_back('{12'h0cc, 1});
    x = mq.pop_front();
    $display("T|mod_front address=%h accepted=%b", x.address, x.accepted);
    x = mq.pop_back();
    $display("T|mod_back address=%h accepted=%b left=%0d", x.address, x.accepted, mq.size());
    mo.push_back('{7, '{12'h777, 1}, 8'h12, 8'h34});
    mo.push_back('{8, '{12'h888, 0}, 8'h56, 8'h78});
    y = mo.pop_back();
    $display("T|mod_nested id=%0d r=%h/%b n=%h lo=%h tag=%h", y.id, y.r.address, y.r.accepted, y.n, y.n.lo, y.tag);
    y = mo.pop_front();
    mq.push_back('{12'h0dd, 1});
    mq.push_back('{12'h0ee, 0});
    mq.push_back('{12'h0ff, 1});
    x = mtake();
    $display("T|mtake address=%h accepted=%b left=%0d", x.address, x.accepted, mq.size());
    show(mq.pop_back());
    begin
      automatic receipt_t z = mq.pop_front();
      $display("T|mdecl address=%h accepted=%b left=%0d", z.address, z.accepted, mq.size());
    end
    p.observe_write(12'h777, 0);
    p.last = p.receipts.pop_back();
    $display("T|ext_prop address=%h accepted=%b left=%0d", p.last.address, p.last.accepted, p.receipts.size());
    $display("T|mod_nested2 id=%0d r=%h/%b n=%h lo=%h tag=%h left=%0d", y.id, y.r.address, y.r.accepted, y.n, y.n.lo, y.tag, mo.size());
  end
endmodule
