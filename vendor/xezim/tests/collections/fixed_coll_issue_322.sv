
typedef struct packed { bit [23:0] span_start; bit [23:0] span_end; } addr_range_t;
typedef struct {
  bit [23:0] ipw34_start;
  addr_range_t ipw33_spans [7:0] [$];
} ipw07_addr_regions_t;

module tb;
  ipw07_addr_regions_t ipw07_filtered_regions [4];
  addr_range_t tmp_span;
  int errors = 0;

  initial begin
    func_fill();
    $finish ;
  end

  function func_fill();
    for (int di = 0; di < 4; di++) begin
      for (int si = 0; si < 16; si++) begin
        tmp_span.span_start = si;
        tmp_span.span_end   = si + 3;
        if (si % 2 == 0)  // 8 spans into [1]
          ipw07_filtered_regions[di].ipw33_spans[1].push_back(tmp_span);
        if (si % 3 == 0)  // 6 spans into [2]
          ipw07_filtered_regions[di].ipw33_spans[2].push_back(tmp_span);
        if (si == 5 || si == 11)  // 2 spans into [0]  (bit-mask-filter analog)
          ipw07_filtered_regions[di].ipw33_spans[0].push_back(tmp_span);
      end
      // production's find-with-inside chain
      ipw07_filtered_regions[di].ipw33_spans[3] =
        ipw07_filtered_regions[di].ipw33_spans[2].find with (
          item inside {ipw07_filtered_regions[di].ipw33_spans[1]});
    end

    for (int di = 0; di < 4; di++) begin
      if (ipw07_filtered_regions[di].ipw33_spans[0].size() != 2) begin
        $display("FAIL: d%0d spans[0].size()=%0d (exp 2)", di,
                 ipw07_filtered_regions[di].ipw33_spans[0].size());
        errors++;
      end
      if (ipw07_filtered_regions[di].ipw33_spans[1].size() != 8) begin
        $display("FAIL: d%0d spans[1].size()=%0d (exp 8)", di,
                 ipw07_filtered_regions[di].ipw33_spans[1].size());
        errors++;
      end
      if (ipw07_filtered_regions[di].ipw33_spans[2].size() != 6) begin
        $display("FAIL: d%0d spans[2].size()=%0d (exp 6)", di,
                 ipw07_filtered_regions[di].ipw33_spans[2].size());
        errors++;
      end
      // [3] = [2] filtered by membership in [1]: si%3==0 AND si%2==0 -> si in {0,6,12} -> 3
      if (ipw07_filtered_regions[di].ipw33_spans[3].size() != 3) begin
        $display("FAIL: d%0d spans[3].size()=%0d (exp 3)", di,
                 ipw07_filtered_regions[di].ipw33_spans[3].size());
        errors++;
      end else if (ipw07_filtered_regions[di].ipw33_spans[3][0].span_start != 0) begin
        $display("FAIL: d%0d spans[3][0].span_start=%0d (exp 0)", di,
                 ipw07_filtered_regions[di].ipw33_spans[3][0].span_start);
        errors++;
      end
    end
    if (errors == 0) $display("TEST PASSED");
    else             $display("TEST FAILED: %0d errors", errors);
  endfunction: func_fill

endmodule

