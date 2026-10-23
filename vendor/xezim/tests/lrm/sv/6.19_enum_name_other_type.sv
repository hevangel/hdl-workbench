// top: ren
module ren;
  typedef enum logic [2:0] {IDLE = 3'b000, RUN = 3'b011, STOP = 3'b111} st_t;
  typedef enum logic [1:0] {XA = 2'b00, XB = 2'bx1} xe_t;
  st_t st;
  initial begin
    st = st_t'(3'b001);
    $display("T|r1|[%s] %b", st.name(), st);
    st = st_t'(3'b101);
    $display("T|r2|[%s] %b", st.name(), st);
  end
endmodule
