// top: frd2
module frd2;
  int dm[];
  logic [7:0] fm [0:3];
  initial begin
    int fd;
    fd = $fopen("frd2.mem", "w"); $fwrite(fd, "@2\n1A\n@0\n11\n"); $fclose(fd);
    dm = new[4];
    $readmemh("frd2.mem", dm);
    $readmemh("frd2.mem", fm);
    $display("T|21.4|dyn=%p fixed=%p", dm, fm);
  end
endmodule
