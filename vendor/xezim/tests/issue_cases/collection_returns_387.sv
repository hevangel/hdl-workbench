typedef byte ba_t[];
typedef int q_t[$];
typedef int fa_t[2];
typedef int ra_t[4:3];
typedef logic [7:0] packed_t;
class Box; int n=9; endclass
module top;
 int calls=0;
 function automatic ba_t empty_self(int n); calls++; return empty_self; endfunction
 function automatic ba_t filled_self(int n); calls++; filled_self=new[n]; filled_self[0]=7; return filled_self; endfunction
 function automatic q_t queue_self(int n); calls++; queue_self.push_back(7); queue_self.push_back(n); return queue_self; endfunction
 function automatic ba_t local_dyn(int n); ba_t t; calls++; t=new[n]; t[0]=7; return t; endfunction
 function automatic fa_t fixed_local(int n); fa_t t; calls++; t[0]=7; return t; endfunction
 function automatic fa_t fixed_self(int n); calls++; fixed_self[0]=7; return fixed_self; endfunction
 function automatic fa_t fixed_fall(int n); calls++; fixed_fall[0]=8; endfunction
 function automatic ra_t ranged_self(int n); calls++; ranged_self[4]=7; ranged_self[3]=8; return ranged_self; endfunction
 function automatic packed_t scalar_self(int n); calls++; scalar_self=13; return scalar_self; endfunction
 function automatic string string_self(int n); calls++; string_self="ok"; return string_self; endfunction
 function automatic Box handle_self(int n); calls++; handle_self=new; return handle_self; endfunction
 initial begin
  ba_t d; q_t q; fa_t a; ra_t b; packed_t p; string s; Box h;
  d=empty_self(2); $display("T|empty|%0d",d.size());
  d=filled_self(2); $display("T|filled|%0d %0d %0d",d.size(),d[0],d[1]);
  q=queue_self(2); $display("T|queue|%0d %0d %0d",q.size(),q[0],q[1]);
  d=local_dyn(2); $display("T|local_dyn|%0d %0d %0d",d.size(),d[0],d[1]);
  a=fixed_local(0); $display("T|fixed_local|%0d %0d",a[0],a[1]);
  a=fixed_self(0); $display("T|fixed_self|%0d %0d",a[0],a[1]);
  a=fixed_fall(0); $display("T|fixed_fall|%0d %0d",a[0],a[1]);
  b=ranged_self(0); $display("T|ranged_self|%0d %0d",b[4],b[3]);
  p=scalar_self(0); s=string_self(0); h=handle_self(0);
  $display("T|controls|%0d %s %0d",p,s,h.n);
  $display("T|calls|%0d",calls);
 end
endmodule
