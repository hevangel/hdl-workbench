// Wide-member randomize shapes: a constraint reaching a
// sub-object of a sub-object, `rand` unpacked struct members, a dynamic
// array element relation through `^`, and `std::randomize` of wide locals.
// The reference simulator prints `r=8 ok=8` on every line.
module randomize_wide_open;
  class L3_nest3_72_impl; rand bit [71:0] x, y, z; endclass class L2_nest3_72_impl; rand L3_nest3_72_impl c; function new(); c = new(); endfunction endclass class L1_nest3_72_impl; rand L2_nest3_72_impl b; rand bit m; function new(); b = new(); endfunction endclass
  class C_nest3_72_impl; rand L1_nest3_72_impl a; constraint c { a.m -> a.b.c.x == 72'h9A0000000000000001; !a.m -> a.b.c.x == 72'h80FFFFFFFFFFFFFFFE; } function new(); a = new(); endfunction function bit ok(); return (a.m ? a.b.c.x == 72'h9A0000000000000001 : a.b.c.x == 72'h80FFFFFFFFFFFFFFFE); endfunction endclass
  typedef struct { rand bit [71:0] x; rand bit [71:0] y; rand bit [71:0] z; rand bit m; } U_ustruct_72_inl;
  class C_ustruct_72_inl; rand U_ustruct_72_inl s; constraint c { s.x inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000}; } function new();  endfunction function bit ok(); return (s.x == 72'h9A0000000000000001 || s.x == 72'h80FFFFFFFFFFFFFFFE || s.x == 72'h010000000000000000); endfunction endclass
  typedef struct { rand bit [71:0] x; rand bit [71:0] y; rand bit [71:0] z; rand bit m; } U_ustruct_72_inr;
  class C_ustruct_72_inr; rand U_ustruct_72_inr s; constraint c { s.x inside {[72'h400000000000000000:72'h4000000000000000FF]}; } function new();  endfunction function bit ok(); return (s.x >= 72'h400000000000000000 && s.x <= 72'h4000000000000000FF); endfunction endclass
  typedef struct { rand bit [71:0] x; rand bit [71:0] y; rand bit [71:0] z; rand bit m; } U_ustruct_72_uniq;
  class C_ustruct_72_uniq; rand U_ustruct_72_uniq s; constraint c { unique {s.x, s.y, s.z}; s.x inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000}; s.y inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000}; s.z inside {72'h9A0000000000000001, 72'h80FFFFFFFFFFFFFFFE, 72'h010000000000000000}; } function new();  endfunction function bit ok(); return (s.x != s.y && s.y != s.z && s.x != s.z && (s.x == 72'h9A0000000000000001 || s.x == 72'h80FFFFFFFFFFFFFFFE || s.x == 72'h010000000000000000) && (s.y == 72'h9A0000000000000001 || s.y == 72'h80FFFFFFFFFFFFFFFE || s.y == 72'h010000000000000000) && (s.z == 72'h9A0000000000000001 || s.z == 72'h80FFFFFFFFFFFFFFFE || s.z == 72'h010000000000000000)); endfunction endclass
  typedef struct { rand bit [71:0] x; rand bit [71:0] y; rand bit [71:0] z; rand bit m; } U_ustruct_72_wrel;
  class C_ustruct_72_wrel; rand U_ustruct_72_wrel s; constraint c {  } function new();  endfunction function bit ok(); return (s.x > s.y && s.y > s.z && s.z > 72'h80FFFFFFFFFFFFFFFE); endfunction endclass
  class C_darr_72_xor; rand bit [71:0] v[]; rand bit m; constraint sz { v.size() == 3; } constraint c { v[2] == (v[0] ^ v[1]); v[0][71] == 1; } function new();  endfunction function bit ok(); return (v[2] == (v[0] ^ v[1]) && v[0][71] == 1); endfunction endclass
  function automatic void p_std_72_inr(); bit [71:0] x, y, z; bit m; int r = 0, k = 0; repeat (8) begin if (std::randomize(x, y, z, m) with { x inside {[72'h400000000000000000:72'h4000000000000000FF]};  }) r++; if ((x >= 72'h400000000000000000 && x <= 72'h4000000000000000FF)) k++; end $display("T|std_72_inr r=%0d ok=%0d", r, k); endfunction
  function automatic void p_std_72_distr(); bit [71:0] x, y, z; bit m; int r = 0, k = 0; repeat (8) begin if (std::randomize(x, y, z, m) with { x dist {[72'h400000000000000000:72'h4000000000000000FF] :/ 1};  }) r++; if ((x >= 72'h400000000000000000 && x <= 72'h4000000000000000FF)) k++; end $display("T|std_72_distr r=%0d ok=%0d", r, k); endfunction
  function automatic void p_std_72_impl(); bit [71:0] x, y, z; bit m; int r = 0, k = 0; repeat (8) begin if (std::randomize(x, y, z, m) with { m -> x == 72'h9A0000000000000001; !m -> x == 72'h80FFFFFFFFFFFFFFFE;  }) r++; if ((m ? x == 72'h9A0000000000000001 : x == 72'h80FFFFFFFFFFFFFFFE)) k++; end $display("T|std_72_impl r=%0d ok=%0d", r, k); endfunction
  function automatic void p_std_72_wrel(); bit [71:0] x, y, z; bit m; int r = 0, k = 0; repeat (8) begin if (std::randomize(x, y, z, m) with {  x > y; y > z; z > 72'h80FFFFFFFFFFFFFFFE; }) r++; if ((x > y && y > z && z > 72'h80FFFFFFFFFFFFFFFE)) k++; end $display("T|std_72_wrel r=%0d ok=%0d", r, k); endfunction
  class N3c; rand bit [31:0] x; endclass
  class N2c; rand N3c c; function new(); c = new(); endfunction endclass
  class N1c; rand N2c b; rand bit m; function new(); b = new(); endfunction endclass
  class C_nest3_32_impl; rand N1c a; constraint k { a.m -> a.b.c.x == 5; !a.m -> a.b.c.x == 7; } function new(); a = new(); endfunction function bit ok(); return (a.m ? a.b.c.x == 5 : a.b.c.x == 7); endfunction endclass
  initial begin
    begin C_nest3_72_impl o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|nest3_72_impl r=%0d ok=%0d", r, k); end
    begin C_ustruct_72_inl o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|ustruct_72_inl r=%0d ok=%0d", r, k); end
    begin C_ustruct_72_inr o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|ustruct_72_inr r=%0d ok=%0d", r, k); end
    begin C_ustruct_72_uniq o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|ustruct_72_uniq r=%0d ok=%0d", r, k); end
    begin C_ustruct_72_wrel o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize() with { s.x > s.y; s.y > s.z; s.z > 72'h80FFFFFFFFFFFFFFFE; }) r++; if (o.ok()) k++; end $display("T|ustruct_72_wrel r=%0d ok=%0d", r, k); end
    begin C_darr_72_xor o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|darr_72_xor r=%0d ok=%0d", r, k); end
    p_std_72_inr();
    p_std_72_distr();
    p_std_72_impl();
    p_std_72_wrel();
    begin C_nest3_32_impl o = new(); int r = 0, k = 0; repeat (8) begin if (o.randomize()) r++; if (o.ok()) k++; end $display("T|nest3_32_impl r=%0d ok=%0d", r, k); end
  end
endmodule
