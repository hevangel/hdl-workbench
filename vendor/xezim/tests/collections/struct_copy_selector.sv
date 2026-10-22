module top;
  typedef struct { int tag; int q[$]; } record_t;
  typedef struct { record_t member; } wrapper_t;
  record_t rows[2], dst, targets[2];
  record_t dynamic_rows[], queue_rows[$], mapped_rows[string];
  wrapper_t wrapped;
  int calls = 0, writes = 0, index;
  int selectors[$];
  function automatic int pick(); calls++; return 1; endfunction
  function automatic int target(); writes++; return 0; endfunction
  function automatic string key(); calls++; return "chosen"; endfunction
  initial begin
    rows[0].tag = 7;
    rows[0].q.push_back(31);
    rows[1].tag = 9;
    rows[1].q.push_back(42);
    rows[1].q.push_back(43);

    dst = rows[0];
    calls = 0;
    dst = rows[pick()];
    $display("T|fixed %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], calls);

    dst = rows[0];
    index = 1;
    dst = rows[index++];
    $display("T|increment %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], index);

    dynamic_rows = new[2];
    dynamic_rows[1] = rows[1];
    dst = rows[0];
    calls = 0;
    dst = dynamic_rows[pick()];
    $display("T|dynamic %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], calls);

    queue_rows.push_back(rows[0]);
    queue_rows.push_back(rows[1]);
    dst = rows[0];
    calls = 0;
    dst = queue_rows[pick()];
    $display("T|queue %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], calls);

    mapped_rows["chosen"] = rows[1];
    dst = rows[0];
    calls = 0;
    dst = mapped_rows[key()];
    $display("T|associative %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], calls);

    targets[0] = rows[0];
    calls = 0;
    writes = 0;
    targets[target()] = rows[pick()];
    $display("T|both %0d %0d %0d %0d %0d %0d", targets[0].tag,
      targets[0].q.size(), targets[0].q[0], targets[0].q[1], calls, writes);

    wrapped.member = rows[0];
    calls = 0;
    wrapped.member = rows[pick()];
    $display("T|member %0d %0d %0d %0d %0d", wrapped.member.tag,
      wrapped.member.q.size(), wrapped.member.q[0], wrapped.member.q[1], calls);

    selectors.push_back(1);
    dst = rows[0];
    dst = rows[selectors.pop_front()];
    $display("T|pop %0d %0d %0d %0d %0d", dst.tag, dst.q.size(), dst.q[0], dst.q[1], selectors.size());
  end
endmodule
