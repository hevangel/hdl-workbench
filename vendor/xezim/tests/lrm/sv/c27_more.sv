// top: c27x
module leafg #(parameter int ID = 0) (output int idv);
  assign idv = ID;
endmodule
module c27x;
  localparam int N = 4;
  int ids [N];
  // 27.4 generate for with instances and hierarchical names
  for (genvar g = 0; g < N; g++) begin : gl
    leafg #(.ID(g * 3)) u(.idv(ids[g]));
    if (g % 2 == 0) begin : even
      int ev = g;
    end
  end
  // generate over a parameter array via constant function
  function automatic int sq(int x); return x * x; endfunction
  for (genvar k = 1; k <= 3; k++) begin : sqb
    localparam int S = sq(k);
  end
  // 27.5 conditional generate selecting a module
  localparam bit USE_A = 0;
  if (USE_A) begin : sel
    leafg #(100) u(.idv());
  end else begin : sel
    leafg #(200) u(.idv());
  end
  // defparam into generate scope
  defparam gl[1].u.ID = 77;
  // generate inside generate with genvar used in name expression
  for (genvar a = 0; a < 2; a++) begin : ga
    for (genvar b = a; b < 2; b++) begin : gb
      localparam int AB = a * 10 + b;
    end
  end
  // 27.4 loop generate with genvar step other than 1 and descending
  for (genvar d = 6; d > 0; d -= 2) begin : desc
    localparam int D = d;
  end
  initial begin
    #1;
    $display("T|27.4d|%p", ids);
    $display("T|27.4e|%0d %0d", gl[2].even.ev, gl[3].u.idv);
    $display("T|27.4f|%0d %0d", sqb[2].S, sqb[3].S);
    $display("T|27.5c|%0d", sel.u.idv);
    $display("T|27.4g|%0d %0d", ga[0].gb[1].AB, ga[1].gb[1].AB);
    $display("T|27.4h|%0d %0d %0d", desc[6].D, desc[4].D, desc[2].D);
    $display("T|23.10.1b|%0d", gl[1].u.idv);
  end
endmodule
