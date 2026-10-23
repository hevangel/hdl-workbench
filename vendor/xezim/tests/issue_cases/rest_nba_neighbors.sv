// §7.6 / §10.4.2: declared ordering, snapshotting, and delayed NBA updates.
module top;
 logic [7:0] src[3:2], dst[-2:-1];
 logic [7:0] grid_src[1:0][4:5], grid_dst[2:3][-1:-2];
 int failures=0;
 initial begin
  src[3]=11;src[2]=22;dst[-2]=0;dst[-1]=0;
  grid_src[1][4]=1;grid_src[1][5]=2;grid_src[0][4]=3;grid_src[0][5]=4;
  dst<=src; grid_dst<=grid_src;
  src[3]=99;grid_src[1][4]=99;
  #0; if(dst[-2]!==0) failures++;
  #1;
  if(dst[-2]!==11 || dst[-1]!==22) failures++;
  if(grid_dst[2][-1]!==1 || grid_dst[2][-2]!==2 || grid_dst[3][-1]!==3 || grid_dst[3][-2]!==4) failures++;
  dst<=#3 src; src[3]=55;
  #2;if(dst[-2]!==11) failures++;
  #2;if(dst[-2]!==99) failures++;
  dst<=src; dst[-2]<=44;
  #1;if(dst[-2]!==44 || dst[-1]!==22) failures++;
  if(failures==0) $display("PASS NBA neighbors");else $display("FAIL NBA neighbors %0d",failures);
  $finish;
 end
endmodule
