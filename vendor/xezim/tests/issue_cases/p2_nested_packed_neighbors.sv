module top;
  typedef struct packed { logic [8:1] a; logic [0:3] b; } item_t;
  typedef struct packed { item_t x; logic [3:0] tail; } outer_t;
  logic [0:1][2:1][0:3] global_bits;
  typedef logic signed [1:0][3:0] octet_t;
  typedef octet_t alias_t;
  typedef struct packed { bit [7:0] lo; bit [7:0] hi; } pair_t;
  pair_t rows[2][$];
  int errors;
  task automatic ck(string label, logic [63:0] got, logic [63:0] expected);
    if (got === expected) $display("PASS %s",label);
    else begin errors++; $display("FAIL %s got=%h expected=%h",label,got,expected); end
  endtask
  task automatic fill(output logic [1:0][3:0] result[2]);
    foreach (result[i]) result[i]='0;
    result[1][1][2 +: 2]=2'b10;
  endtask
  task automatic touch(ref logic [0:1][2:1][0:3] value);
    value[1][1][0:3]=4'ha;
  endtask
  task automatic work;
    logic [0:1][2:1][0:3] local_bits;
    logic [1:0][7:0] arr[2];
    item_t [0:1] records;
    outer_t nested;
    logic [0:1][4:1] ascending_dynamic[];
    alias_t aliases[2];
    logic signed [1:0] narrow_unknown;
    pair_t pair_value;
    octet_t signed_dynamic[];
    logic [1:0][3:0] matrix[2][2];
    logic [1:0][3:0] formal_result[2];
    local_bits='0;
    local_bits[0][2][0:1]=2'b11;
    ck("ascending dimensions",local_bits,16'hc000);
    local_bits[1][1][1 +: 2]=2'b10;
    ck("indexed ascending inner",local_bits,16'hc004);
    local_bits[1][2:1]=8'h5a;
    ck("middle range",local_bits,16'hc05a);
    ck("middle range read",local_bits[1][2:1],8'h5a);
    records='0; records[1].a[4:1]=4'hb; records[1].b[0:1]=2'b11;
    ck("ascending structs",records,24'h0000bc);
    nested='0; nested.x.a[4:1]=4'hb; nested.x.b[0:1]=2'b11;
    ck("nested members",nested,16'h0bc0);
    foreach (arr[i]) arr[i]='0;
    arr[1][1][6 -: 3]=3'b101;
    ck("unpacked selected element",arr[1],16'h5000);
    ascending_dynamic=new[2]; ascending_dynamic[1]='0;
    ascending_dynamic[1][0][3 -: 2]=2'b11;
    ascending_dynamic[1][1]=4'h5;
    ck("dynamic ascending dimensions",ascending_dynamic[1],8'h65);
    foreach (aliases[i]) aliases[i]='0;
    aliases[1][1]=4'h8; aliases[1][0][3:1]=3'b101;
    ck("typedef chain private storage",aliases[1][1],4'h8);
    signed_dynamic=new[2]; signed_dynamic[1]=8'h80;
    ck("dynamic signed whole element",signed_dynamic[1],64'hffff_ffff_ffff_ff80);
    foreach (matrix[i,j]) matrix[i][j]='0;
    matrix[1][0][1][1 +: 2]=2'b11;
    ck("multidimensional private storage",matrix[1][0],8'h60);
    fill(formal_result);
    ck("output formal packed element",formal_result[1],8'h80);
    narrow_unknown=2'bx1; local_bits[0][2]=narrow_unknown;
    ck("signed unknown assignment extension",local_bits[0][2],4'bxxx1);
    local_bits[1][1][0:2]='1;
    ck("selected fill literal",local_bits[1][1][0:2],3'b111);
    touch(local_bits);
    ck("selected ref formal",local_bits[1][1],4'ha);
    pair_value.lo=8'h12; pair_value.hi=8'h34;
    rows[1].push_back(pair_value);
    ck("packed member below nested queue",rows[1][0].lo,8'h12);
    ck("packed element inside nested queue",rows[1][0] inside {rows[1]},1);
  endtask
  initial begin
    global_bits='0; global_bits[1][1][1:3]=3'b101;
    ck("global ascending",global_bits,16'h0005);
    work;
    if(errors==0) $display("ALL PASS");
    $finish;
  end
endmodule
