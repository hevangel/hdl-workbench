// top: c12
module c12;
  logic [3:0] sel;
  int r;
  initial begin
    // 12.4 if-else with x condition -> else
    sel = 4'bx; if (sel) r = 1; else r = 2; $display("T|12.4a|%0d", r);
    sel = 4'b0x10; if (sel) r = 1; else r = 2; $display("T|12.4b|%0d", r);
    // 12.4.2 unique-if / priority-if violations (reported as warnings)
    sel = 3;
    unique if (sel == 3) r = 1; else if (sel > 2) r = 2;   // overlap -> violation
    $display("T|12.4.2a|%0d", r);
    sel = 9;
    unique if (sel == 3) r = 1; else if (sel == 4) r = 2;  // no match, no else -> violation
    $display("T|12.4.2b|%0d", r);
    priority if (sel == 3) r = 5; else if (sel == 4) r = 6; // no match -> violation
    $display("T|12.4.2c|%0d", r);
    unique0 if (sel == 3) r = 7;                             // no violation
    $display("T|12.4.2d|%0d", r);
    // 12.5 case
    sel = 4'b1x00;
    case (sel) 4'b1x00: r = 1; 4'b1000: r = 2; default: r = 3; endcase $display("T|12.5a|%0d", r);
    casez (4'b1z01) 4'b1?0?: r = 1; default: r = 2; endcase $display("T|12.5b|%0d", r);
    casez (4'b1x01) 4'b1?01: r = 1; default: r = 2; endcase $display("T|12.5c|%0d", r);
    casex (4'b1x01) 4'b1001: r = 1; default: r = 2; endcase $display("T|12.5d|%0d", r);
    casex (4'b1001) 4'b1x0z: r = 1; default: r = 2; endcase $display("T|12.5e|%0d", r);
    // case with sized items: width extension to largest
    case (3'b101) 5'b00101: r = 1; default: r = 2; endcase $display("T|12.5f|%0d", r);
    case (-1) 4'hf: r = 1; 32'hffffffff: r = 2; default: r = 3; endcase $display("T|12.5g|%0d", r);
    // constant expression case (case (1'b1))
    sel = 4'b0100;
    case (1'b1) sel[0]: r = 0; sel[1]: r = 1; sel[2]: r = 2; default: r = 9; endcase $display("T|12.5h|%0d", r);
    // first match wins
    case (2) 1, 2: r = 10; 2: r = 20; endcase $display("T|12.5i|%0d", r);
    // string case
    begin string s = "foo"; case (s) "bar": r = 1; "foo": r = 2; default: r = 3; endcase $display("T|12.5j|%0d", r); end
    // 12.5.3 unique/priority case violations
    sel = 2;
    unique case (sel) 1: r = 1; 2: r = 2; 2: r = 3; endcase $display("T|12.5.3a|%0d", r);
    unique case (sel) 5: r = 5; 6: r = 6; endcase $display("T|12.5.3b|%0d", r);
    priority casez (sel) 4'b00??: r = 7; 4'b0010: r = 8; endcase $display("T|12.5.3c|%0d", r);
    unique0 case (sel) 7: r = 0; endcase $display("T|12.5.3d|%0d", r);
    // 12.5.4 case inside
    sel = 4'd6;
    case (sel) inside [0:3]: r = 1; [4:7], 9: r = 2; default: r = 3; endcase $display("T|12.5.4a|%0d", r);
    case (4'b1011) inside 4'b1?11: r = 1; default: r = 2; endcase $display("T|12.5.4b|%0d", r);
    case (4'b1x11) inside 4'b1011: r = 1; 4'b1?11: r = 2; default: r = 3; endcase $display("T|12.5.4c|%0d", r);
    // 12.6 pattern matching (case matches) - see c07 tagged unions; if matches
    // 12.7 loops
    r = 0; for (int i = 0, j = 10; i < j; i += 2, j--) r++; $display("T|12.7.1|%0d", r);
    r = 0; repeat (3'b101) r++; $display("T|12.7.2a|%0d", r);
    r = 0; repeat (-1) r++; $display("T|12.7.2b|%0d", r);
    r = 0; repeat (4'bx) r++; $display("T|12.7.2c|%0d", r);
    r = 0; while (r < 5) r += 2; $display("T|12.7.3|%0d", r);
    r = 10; do r++; while (r < 5); $display("T|12.7.4|%0d", r);
    begin int md[2][3]; int cnt = 0; string ord = "";
      foreach (md[i, j]) begin md[i][j] = i * 10 + j; ord = {ord, $sformatf("%0d%0d,", i, j)}; end
      $display("T|12.7.5a|%s %p", ord, md); end
    begin int dn[3:1]; string ord = ""; foreach (dn[k]) ord = {ord, $sformatf("%0d", k)}; $display("T|12.7.5b|%s", ord); end
    begin int aa[string] = '{"z":1, "a":2, "m":3}; string ord = ""; foreach (aa[k]) ord = {ord, k}; $display("T|12.7.5c|%s", ord); end
    begin int q[$] = {7,8}; int dd[][]; string ord = ""; dd = new[2]; dd[0] = new[1]; dd[1] = new[2];
      foreach (dd[a1, b1]) ord = {ord, $sformatf("%0d%0d ", a1, b1)}; $display("T|12.7.5d|%s", ord);
      foreach (q[k]) q[k] *= 2; $display("T|12.7.5e|%p", q); end
    begin logic [3:0][1:0] pk; string ord = ""; foreach (pk[m, n]) ord = {ord, $sformatf("%0d%0d ", m, n)}; $display("T|12.7.5f|%s", ord); end
    begin int u[2][2]; string ord = ""; foreach (u[, j]) ord = {ord, $sformatf("%0d ", j)}; $display("T|12.7.5g|%s", ord); end
    r = 0; forever begin r++; if (r == 4) break; end $display("T|12.7.6|%0d", r);
    // 12.8 jump
    r = 0; for (int i = 0; i < 10; i++) begin if (i % 2) continue; if (i > 6) break; r += i; end $display("T|12.8a|%0d", r);
    begin int k; for (k = 0; k < 5; k++) if (k == 3) break; $display("T|12.8b|%0d", k); end
    r = 0; repeat (5) begin r++; if (r == 2) continue; end $display("T|12.8c|%0d", r);
    $display("T|12.8d|%0d", early(5));
    early_t(2);
    // 12.6 if matches
    begin typedef union tagged {int A; bit [3:0] B;} tu_t; tu_t tu = tagged B (4'd9);
      if (tu matches tagged B .v) $display("T|12.6.2|B=%0d", v); else $display("T|12.6.2|noB"); end
  end
  function int early(int n); for (int i = 0; i < 100; i++) if (i == n) return i * 2; return -1; endfunction
  task early_t(int n); if (n > 1) begin $display("T|12.8e|ret"); return; end $display("T|12.8e|noret"); endtask
endmodule
