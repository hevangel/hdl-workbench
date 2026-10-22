// top: rbi
module rbi;
  bit [3:0] m_b4 = 4'b1x0z;
  byte unsigned m_bu = 200;
  int m_i = 'x;
  initial begin
    bit [3:0] l_b4 = 4'b1x0z;
    byte unsigned l_bu = 200;
    int l_i = 'x;
    automatic bit [3:0] a_b4 = 4'b1x0z;
    automatic byte unsigned a_bu = 200;
    bit [3:0] p_b4;
    byte unsigned p_bu;
    p_b4 = 4'b1x0z; p_bu = 200;
    $display("T|r1|module %b %0d %0d", m_b4, m_bu, m_i);
    $display("T|r2|static-local %b %0d %0d", l_b4, l_bu, l_i);
    $display("T|r3|auto-local %b %0d", a_b4, a_bu);
    $display("T|r4|assigned %b %0d", p_b4, p_bu);
  end
endmodule
