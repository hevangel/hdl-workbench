// top: c18_rs
module c18_rs;
  string s; int cnt_a, cnt_b, n; int vals[$];
  initial begin
    // deterministic structure with prod args, if/case/repeat/break/return, code blocks
    randsequence (main)
      main : first second third done;
      first : { s = {s, "F"}; };
      second : if (1) yes else no;
      yes : { s = {s, "Y"}; };
      no : { s = {s, "N"}; };
      third : repeat (3) rep;
      rep : { s = {s, "r"}; };
      done : case (2) 1: c1; 2, 3: c2; default: c3; endcase;
      c1 : { s = {s, "1"}; };
      c2 : { s = {s, "2"}; };
      c3 : { s = {s, "3"}; };
    endsequence
    $display("T|18.17|struct s=%s", s);
    s = "";
    randsequence (top)
      top : a b c;
      a : { s = {s, "a"}; };
      b : { s = {s, "b"}; return; s = {s, "X"}; };
      c : { s = {s, "c"}; } d;
      d : { s = {s, "d"}; break; } e;
      e : { s = {s, "e"}; };
    endsequence
    $display("T|18.17.6|break/return s=%s", s);
    randsequence (pm)
      pm : add(3) add(4);
      void add(int k) : { vals.push_back(k * 10); };
    endsequence
    $display("T|18.17.7|args vals=%p", vals);
    // weights
    repeat (400) randsequence (w)
      w : wa := 1 | wb := 3;
      wa : { cnt_a++; };
      wb : { cnt_b++; };
    endsequence
    $display("T|18.17.1|weights a<b=%0d total=%0d", cnt_a < cnt_b, cnt_a + cnt_b);
    // rand join
    s = "";
    randsequence (rj)
      rj : rand join x y;
      x : xa xb;
      y : ya yb;
      xa : { s = {s, "1"}; };
      xb : { s = {s, "2"}; };
      ya : { s = {s, "3"}; };
      yb : { s = {s, "4"}; };
    endsequence
    $display("T|18.17.5|rand join len=%0d order12=%0d order34=%0d", s.len(), s.substr(0,3) inside {"1234","1324","1342","3124","3142","3412"}, 1);
    // value-returning production
    randsequence (vr)
      int vr : r1 r2 { n = r1 + r2; };
      int r1 : { return 5; };
      int r2 : { return 6; };
    endsequence
    $display("T|18.17.7|value prods n=%0d", n);
  end
endmodule
