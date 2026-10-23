module top;
  typedef struct { int value; } level0_t;
  typedef struct { level0_t inner; } level1_t;
  typedef struct { level1_t inner; } level2_t;
  typedef struct { level2_t inner; } level3_t;
  typedef struct { level3_t inner; } level4_t;
  typedef struct { level4_t inner; } level5_t;
  typedef struct { level5_t inner; } level6_t;
  typedef struct { level6_t inner; } level7_t;
  typedef struct { level7_t inner; } level8_t;
  typedef struct { level8_t inner; } level9_t;
  typedef struct { level9_t inner; } level10_t;
  typedef struct { level10_t inner; } level11_t;
  typedef struct { level11_t inner; } level12_t;
  level9_t q9[$];
  level10_t q10[$];
  level12_t q12[$];
  int calls9=0, calls10=0, calls12=0, method_calls=0;
  function automatic level9_t make9(); calls9++; return '{default:0}; endfunction
  function automatic level10_t make10(); calls10++; return '{default:0}; endfunction
  function automatic level12_t make12(); calls12++; return '{default:0}; endfunction
  class factory;
    function level9_t build(); method_calls++; return '{default:0}; endfunction
  endclass
  initial begin
    factory f=new;
    q9.push_back(make9());
    $display("T|back|%0d %0d",calls9,q9.size());
    q10.push_front(make10());
    $display("T|front|%0d %0d",calls10,q10.size());
    q12.insert(0,make12());
    $display("T|insert|%0d %0d",calls12,q12.size());
    q10.push_back(1 ? make10() : make10());
    $display("T|conditional|%0d %0d",calls10,q10.size());
    q12.push_back((make12()));
    $display("T|parenthesized|%0d %0d",calls12,q12.size());
    q9.push_back(f.build());
    $display("T|method|%0d %0d",method_calls,q9.size());
    $finish;
  end
endmodule
