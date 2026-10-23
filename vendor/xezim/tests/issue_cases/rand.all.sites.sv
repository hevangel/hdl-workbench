`timescale 1ns/1ps
`ifndef z1
`define z1
`define z2 \
int z3=0;
`define z4(expr,msg)\
if(!(expr))begin \
z3++;\
$display("FAIL @%0t : %s",$time,msg);\
end
`define z5 \
if(z3==0)begin \
$display("TEST_PASS");\
end else begin \
$display("TEST_FAIL count=%0d",z3);\
$fatal(1);\
end
`endif
typedef logic[1:0]u2_t;
typedef logic[6:0]u7_t;
typedef logic[10:0]u11_t;
typedef logic[3:0]u4_t;
typedef logic[4:0]u5_t;
typedef logic[6:0]u6_t;
typedef logic[7:0]u8_t;
typedef logic[8:0]u9_t;
typedef logic[15:0]u16_t;
typedef logic[16:0]u17_t;
typedef logic[23:0]u24_t;
typedef logic[31:0]u32_t;
typedef logic[39:0]u40_t;
typedef struct packed{
logic vld;
logic write;
logic[39:0]aIPW61;
logic[2:0]IPW64;
logic[2:0]size;
logic[7:0]len;
logic prot1;
logic[1:0]IPW38;
logic IPW52;
}z6;
typedef struct packed{
logic vld;
logic[127:0]wdata;
logic[15:0]strb;
logic lIPW60;
}z7;
typedef struct packed{
logic vld;
logic[127:0]rdata;
logic lIPW60;
logic[1:0]resp;
}z8;
typedef struct packed{
u24_t z9;
u24_t z10;
}z11;
typedef struct{
u24_t z12;
z11 z13[7:0][$];
}z14;
typedef enum logic[1:0]{z15=2'd0,z16=2'd1}z17;
typedef enum logic[1:0]{
z18=2'b00,z19=2'b10,
z20=2'b11,z21=2'b01
}z22;
typedef enum logic[1:0]{z23=2'b00,z24=2'b01,IDLE=2'b10}z25;
typedef enum logic{z26=1'b0,z27=1'b1}z28;
`define z29 2
`define z30 1
`define z31 16
`define z32 17
`define z33 37
`define z34 4
`define z35 64
`define z36 256
`define z37 32
`define z38 16
`define z39 16
`define z40 32'h8000_0000
typedef int z41[];
class z42#(
type z43=z6,
type z44=z8,
type z45=z7);
rand bit[31:0]z46;
rand bit[31:0]z47;
rand z43 IPW63;
rand bit[39:0]z48;
rand bit[13:0]z49;
rand bit z50;
rand bit z51;
rand bit z52;
rand z45 z53[];
rand bit z54;
rand z44 z55[];
bit[3:0]z56;
bit[12:0]z57;
bit z58,z59;
bit[4:0]z60;
bit[5:0]z61;
bit z62,z63;
int z64;
bit[2:0]z65;
int z66;
z45 z67;
constraint z68{
if(!z52){
z49 dist{
[0:z57>>2]:/3,
[z57>>2:z57]:/5,
z57+1:/2
};
}
IPW63.IPW64<=z56;
z62->(IPW63.write==1);
z63->(IPW63.write==0);
}
constraint z69{
z46 dist{[0:15]:/900,[16:56]:/80,[57:63]:/20,[500:700]:/1};
z47 dist{[0:15]:/900,[16:56]:/80,[57:63]:/20,[200:500]:/1};
}
constraint z70{
z49==((IPW63.len+1)<<IPW63.size);
}
constraint z71{IPW63.size<=z65;}
constraint z72{
if(z59){
IPW63.IPW52 dist{0:=14,1:=1};
IPW63.prot1 dist{1:=9,0:=1};
}else{
IPW63.IPW52==0;
IPW63.prot1==0;
}
if(IPW63.IPW52){
z49<=64;
IPW63.aIPW61[6]==z48[6];
}
}
constraint z73{(IPW63.IPW52==1)->(IPW63.IPW38==1);}
constraint z74{
if(z58){IPW63.IPW38 dist{1:=9,2:=1};}
else{(IPW63.IPW38==1);}
if(IPW63.IPW38==2){z48==IPW63.aIPW61;}
else{(z48==(IPW63.aIPW61+z49-1));}
}
constraint z75{
if(IPW63.IPW38==2){
IPW63.write==0;
IPW63.IPW52==z17'(0);
z49 inside{32,64,128,256};
IPW63.size==z65;
}
}
constraint z76{
(IPW63.aIPW61>>z61)==(z48>>z61);
}
constraint z77{
z53.size()==(IPW63.len+1);
z55.size()==(IPW63.len+1);
}
constraint z78{z51 dist{0:=99,1:=1};}
constraint z79{
foreach(z53[i]){
z53[i].vld==1;
if(z51){
z53[i].strb==(1<<z66)-1;
}else{
z53[i].strb dist{
(1<<z66)-1:/40,
[0:(1<<z66)-2]:/60
};
}
}
foreach(z53[i]){
if(i==(IPW63.len)){z53[i].lIPW60==1;}
else{z53[i].lIPW60==0;}
}
}
constraint z80{
(IPW63.aIPW61&((1<<IPW63.size)-1))==0;
(IPW63.IPW52==1)->((IPW63.aIPW61&(z49-1))==0);
IPW63.len<=255;
}
constraint z81{
solve IPW63.IPW38 before IPW63.len;
solve IPW63.len before IPW63.size;
solve IPW63.size before z49;
solve IPW63.len before z53;
solve IPW63.IPW38,IPW63.size before IPW63.aIPW61;
solve IPW63.aIPW61,IPW63.len before z48;
}
function new(string name="IPW44_transfer_info");
z64=$bits(z67.wdata);
z66=$bits(z67.strb);
z65=$clog2(z64)-3;
z61=16;
endfunction:new
endclass:z42
class z82;
rand bit[31:0]z46;
rand bit[31:0]z83;
rand bit[39:0]z84;
rand bit[12:0]z85;
rand bit z86;
rand bit z87;
rand bit[2:0]z88;
rand bit[39:0]z48;
rand bit z50;
bit[3:0]z56;
bit[12:0]z57;
bit[5:0]z61;
bit z63,z62;
constraint z89{
z85 dist{
[0:z57>>2]:/3,
[z57>>2:z57-1]:/5,
z57:/2
};
z88<=z56;
}
constraint z90{z48==(z84+z85);}
constraint z91{
(z84>>z61)==(z48>>z61);
}
constraint z92{
z86 dist{0:=50,1:=50};
z63->(z86==0);
z62->(z86==1);
z50==0;
}
constraint z93{z87 dist{0:=9,1:=1};}
constraint z94{
z46 dist{0:/800,[1:15]:/100,[16:56]:/90,[57:63]:/9,[500:700]:/1};
z83 dist{0:=700,[1:50]:/200,[50:100]:/10,[500:700]:/1};
}
function new(string name="IPW45_transfer");
z61=16;
endfunction:new
endclass:z82
class z95;
rand logic[31:0]z46;
rand logic[31:8]z84;
rand bit[3:0]z88;
rand bit z96;
bit[3:0]z56;
bit z50;
constraint z97{
z46 dist{0:=200,[1:5]:/200,[2:127]:/600,[500:700]:/1};
}
constraint z98{
z88<=z56;
z96==1;
}
function new(string name="IPW59_transfer");
endfunction:new
endclass:z95
class z99;
rand static int z100;
rand static bit[3:0]z101;
rand static int z102;
rand static bit[3:0]z103;
rand static int z104;
rand static bit[3:0]z105;
rand static int z106;
rand static bit[3:0]z107;
rand static int z108;
rand static bit[3:0]z109;
rand static int z110;
rand static bit[3:0]z111;
bit z112;
bit z113;
int z114;
string z115;
constraint z116{
z100 dist{
900:/1,[900+1:1200-1]:/2,1200:/9,
[1200+1:1800-1]:/3,1800:/5
};
z112->(z100==1200);
}
constraint z117{
z102 dist{
400:/1,[400+1:1200-1]:/2,1200:/9,
[1200+1:1400-1]:/3,1400:/5
};
z112->(z102==1200);
}
constraint z118{
z104 dist{
400:/1,[400+1:600-1]:/2,600:/9,
[600+1:720-1]:/3,720:/5
};
z112->(z104==600);
}
constraint z119{
z106 dist{
400:/1,[400+1:1200-1]:/2,1200:/9,
[1200+1:1400-1]:/3,1400:/5
};
z112->(z106==1200);
}
constraint z120{
z110 dist{
400:/1,[400+1:1200-1]:/2,1200:/9,
[1200+1:1400-1]:/3,1400:/5
};
z112->(z110==1200);
}
constraint z121{
if(z113){
z108==z114;
}
}
constraint z122{
z108 dist{
400:/1,[400+1:1200-1]:/2,1200:/9,
[1200+1:1400-1]:/3,1400:/5
};
z112->(z108==1200);
}
function void pre_randomize();
z112=$test$plusargs("IPW47_perf_test");
z113=$test$plusargs("IPW61_IPW54");
z115="MIPW615";
void'($value$plusargs("IPW61_type=%s",z115));
if(z113)begin
z114=804;
void'($value$plusargs("IPW61_IPW54=%d",z114));
if(z115=="MIPW614")begin
z114>>=1;
end
end
if($value$plusargs("gclk_IPW47_IPW54=%d",z100))begin
z116.constraint_mode(0);
z100.rand_mode(0);
end
if($value$plusargs("gclk_if_IPW45_IPW47_IPW54=%d",z110))begin
z120.constraint_mode(0);
z110.rand_mode(0);
end
endfunction:pre_randomize
function new();
endfunction:new
endclass:z99
class z123;
int z124;
int z125;
int z126;
int z127;
int z128;
logic[9:0]z129[9];
function new();
z129='{2,3,4,6,8,12,16,24,32};
z124=-1;
z125=-1;
z126=-1;
z127=-1;
z128=-1;
endfunction:new
rand bit[1:0]z130;
rand bit[3:0]z131;
rand bit z132;
rand bit z133;
rand bit z134;
constraint z135{
z130 dist{0:=1,1:=49};
z131 inside{[0:8]};
z132 inside{[0:1]};
z133 inside{[0:1]};
z134 inside{[0:1]};
}
constraint z136{
if(z124!=-1){z130==z124;}
if(z125!=-1){z131==z125;}
if(z126!=-1){z132==z126;}
if(z127!=-1){z133==z127;}
if(z128!=-1){z134==z128;}
}
endclass:z123
typedef struct packed{
bit[3:0]z137;
bit[10:0]z138;
bit[8:0]z139;
bit[7:0]z140;
}z141;
typedef struct packed{
bit[1:0]z142;
bit[1:0]z143;
bit[1:0]IPW09;
bit[1:0]z144;
bit[3:0]z145;
bit[3:0]z146;
}z147;
typedef struct packed{
z147 z148;
bit[31:0]z149;
bit[(`z32*18)-1:0]z150;
z141 z151;
z141 z152;
bit[31:0]z153;
bit[24:0]z154;
bit[63:0][23:0]z155;
bit[63:0][23:0]z156;
bit[63:0][3:0]z157;
bit[23:0]z158;
}z159;
typedef struct packed{
z147 z160;
bit[(`z32*32)-1:0]z161;
}z162;
class z163;
rand z159 z164;
z41 z165;
bit z166;
bit[`z32-1:0]z167;
int z168=-1;
int z169=-1;
function new();
z166=$test$plusargs("cov_tweaks");
endfunction:new
constraint z170{
z164.z149>0;
if(z166){
z164.z149 dist{
0:/2,
[32'h1:32'h1851eb85]:/1,
[32'h1851eb86:32'h48f5c28f]:/1,
[32'h48f5c290:32'h79999998]:/1,
[32'h79999999:32'h86666665]:/2,
[32'h86666666:32'hb70a3d6f]:/1,
[32'hb70a3d70:32'he7ae1479]:/1,
[32'he7ae147a:32'hfffffffe]:/1,
32'hffffffff:/2
};
}else{
z164.z149 dist{
[1:10]:/1,[11:100]:/9,[101:1000]:/30,
[1000:30000]:/40,[30000:60000]:/10
};
}
}
constraint z171{
foreach(z167[i]){
z164.z150[(i*18)+8]dist{1:=9,0:=1};
z164.z150[(i*18)+17]dist{1:=9,0:=1};
}
(z164.z151.z140==0)->
(z164.z151.z139==0);
(z164.z152.z140==0)->
(z164.z152.z139==0);
(z164.z151.z140<=
z164.z151.z139);
(z164.z152.z140<=
z164.z152.z139);
}
constraint z172{
z164.z153[15:0]dist{[32:63]:/25,64:/50,[65:192]:/25};
z164.z153[31:16]dist{[32:63]:/25,64:/50,[65:192]:/25};
z164.z154[8:0]dist{
0:/1,
[1:((`z36-64)>>1)]:/5,
[((`z36-64)>>1)+1:((`z36-64)-1)]:/11,
[(`z36-64):9'h1FF]:/3
};
z164.z154[24:16]dist{
0:/1,
[1:((`z37-16)>>1)]:/5,
[((`z37-16)>>1)+1:((`z37-16))]:/11,
[(`z37-16)+1:9'h1FF]:/3
};
}
constraint z173{
if(z168!=-1){
z164.z149==z168;
}
if(z169!=-1){
z164.z153==z169;
}
}
endclass:z163
class z174;
rand z162 z175;
z41 z165;
rand u4_t z176;
u5_t z177,z178;
u16_t z179;
bit[`z32-1:0]z167;
int z180=-1;
function new();
endfunction:new
constraint z181{
z176 dist{
0:/15,1:/7,2:/2,3:/1
};
(z175.z160.z145==(4'hf>>z176));
}
constraint z182{
({1'b0,z175.z160.z145}+
{1'b0,z175.z160.z146})<=(`z29-1);
z175.z160.z142==0;
}
endclass:z174
class z183;
rand z163 z184[`z29];
rand z174 z185;
rand z123 z186;
rand bit[15:0]z187;
rand bit[15:0]z188;
rand bit[39:0]z189,z190;
rand bit[15:0]z191[0:3];
rand bit[3:0]z192[0:3];
rand bit[39:0]z193[0:3];
rand bit[39:0]z194[0:3];
bit z195;
u40_t z196[];
u40_t z197[];
int ctr;
bit z198;
bit z112;
bit z199;
bit[`z29-1:1]z200;
bit[15:1]z201;
bit[0:(`z34-1)]z202;
bit[`z33-1:0]z203;
bit[`z32-1:0]z167;
function new(bit dprp=0);
z186=new();
z185=new();
foreach(z184[i])begin
z184[i]=new();
end
z198=dprp;
z112=$test$plusargs("IPW47_perf_test");
z199=$test$plusargs("disable_IPW60")|z112;
endfunction:new
function void pre_randomize();
if(z112|z199)begin
z204.constraint_mode(0);
z205.constraint_mode(0);
z206.constraint_mode(0);
end
endfunction:pre_randomize
constraint z207{
if(z199){
foreach(z202[d]){
z184[0].z164.z156[(d<<4)+0]==0;
z184[0].z164.z157[(d<<4)+0]==4'b1011;
z184[0].z164.z155[(d<<4)+15]==z189[39:16];
foreach(z201[i]){
z184[0].z164.z156[(d<<4)+i]==0;
z184[0].z164.z157[(d<<4)+i]==4'b1011;
z184[0].z164.z155[(d<<4)+i-1]==24'h0;
}
}
}
}
constraint z208{
(z185.z175.z160.z145==1)->
(z185.z175.z160.z143==1);
}
constraint z209{
z185.z175.z160.IPW09==z186.z130;
z185.z175.z160.z143==z186.z134;
}
constraint z210{
foreach(z167[i]){
z185.z175.z161[int'(i<<5)+:16]dist{
0:/2,[16:1023]:/2,[1024:4095]:/5,[4096:65535]:/1
};
}
}
constraint z211{
if(z112){
(z186.z124==-1)->(z186.z130==1);
if(z185.z180==-1){
(z185.z175.z160.z145==(`z29-1));
}
}
}
constraint z212{
solve z186.z131 before z187;
solve z186.z132 before z187;
solve z185.z175.z160.z145 before z187;
solve z187 before z189;
solve z188 before z187;
foreach(z192[i]){
solve z192[i]before z184[0].z164.z157;
}
solve z185.z175.z160 before z184[0].z164.z148;
foreach(z200[i]){
solve z184[0].z164.z148 before z184[i].z164.z148;
solve z184[0].z164.z155 before z184[i].z164.z155;
solve z184[0].z164.z156 before z184[i].z164.z156;
solve z184[0].z164.z157 before z184[i].z164.z157;
solve z184[0].z164.z158 before z184[i].z164.z158;
}
}
constraint z213{
z188==(z186.z129[z186.z131]
<<z186.z132);
z187==z188*(z185.z175.z160.z145+1);
z189==((40'h800_0000*z187)-1);
z190==((40'h800_0000*z188)-1);
}
constraint z204{
foreach(z192[i]){
z184[0].z164.z157[(i<<4)+z192[i]]==4'b1011;
z184[0].z164.z155[(i<<4)+z192[i]][23:16]==8'h0;
z191[i][z192[i]]==1'b0;
}
}
constraint z205{
foreach(z191[i]){
z191[i]dist{0:/9,[1:16'hFFFF]:/1};
$countones(z191[i])<=15;
}
}
constraint z206{
foreach(z202[i]){
z191[i][0]->(z184[0].z164.z155[(i<<4)+0]==24'h0);
!z191[i][0]->(z184[0].z164.z155[(i<<4)+0]!=24'h0);
foreach(z201[j]){
if(z191[i][j]){
(z184[0].z164.z155[(i<<4)+j]==
z184[0].z164.z155[(i<<4)+j-1]);
}else{
(z184[0].z164.z155[(i<<4)+j]>
z184[0].z164.z155[(i<<4)+j-1]);
}
}
}
foreach(z202[i]){
z184[0].z164.z155[(i<<4)]<z189[39:16];
z184[0].z164.z156[(i<<4)]<z189[39:16];
z184[0].z164.z156[(i<<4)][4:0]==0;
(z184[0].z164.z155[(i<<4)]+
z184[0].z164.z156[(i<<4)])<z189[39:16];
foreach(z201[j]){
z184[0].z164.z155[(i<<4)+j]<z189[39:16];
z184[0].z164.z156[(i<<4)+j]<z189[39:16];
z184[0].z164.z156[(i<<4)+j][4:0]==0;
(z184[0].z164.z155[(i<<4)+j]+
z184[0].z164.z156[(i<<4)+j])<z189[39:16];
}
}
}
constraint z214{
foreach(z202[i]){
if(z192[i]==0){
z194[i][39:16]inside{[
0:z184[0].z164.z155[(i<<4)+z192[i]]
]};
}else{
z194[i][39:16]inside{[
(z184[0].z164.z155[(i<<4)+z192[i]-1]+1)
:z184[0].z164.z155[(i<<4)+z192[i]]
]};
}
z194[i][15:0]==0;
z193[i][15:0]==0;
z193[i][39:16]==z194[i][39:16]+
z184[0].z164.z156[(i<<4)+z192[i]];
z193[i][39:16]<z189[39:16];
}
z193[0]==z193[1];
z193[1]==z193[2];
z193[2]==z193[3];
z193[3]==z193[0];
}
constraint z215{
z184[0].z164.z148==z185.z175.z160;
}
endclass:z183
module tb_all_randomize;
`z2
int z216=0;
string z217="";
task automatic z218(int n,string nm);
z216=n;z217=nm;
$display("[%0t] SITE %0d BEGIN : %s",$time,n,nm);
$fflush;
endtask
task automatic z219();
$display("[%0t] SITE %0d DONE  : %s",$time,z216,z217);
$fflush;
endtask
task automatic chk(bit ok,string msg);
if(!ok)begin
z3++;
$display("FAIL @%0t [SITE %0d %s] : %s",$time,z216,z217,msg);
$fflush;
end
endtask
typedef struct{
int z220;
int z221;
int z57;
int z58;
int z222;
int z60;
int z59;
int z63;
int z62;
int z61;
bit[39:0]z223;
int z224;
}z225;
z225 z226;
bit z227=0;
bit z228=0;
bit z166=0;
bit z112=0;
bit z229=0;
bit z230=0;
bit[2:0]z231=0;
u32_t z232[$];
u32_t z233[$];
u32_t z234[$];
u32_t z235[$];
u32_t z236[$];
u32_t z237[$];
u32_t z238[$];
u32_t z239[$];
z14 z240[4];
int z241=2;
typedef enum logic[1:0]{z242=2'd0,z243=2'd1,z244=2'd2}z245;
typedef struct{
z245 cmd;
bit[39:0]IPW43;
bit[12:0]z246;
bit[7:0]len;
bit[2:0]size;
struct{bit[3:0]IPW51;}z247;
}z248;
z248 z249[$];
task automatic z250();
z226.z220=3;
z226.z221=7;
z226.z57=127;
z226.z58=1;
z226.z222=10;
z226.z60=17;
z226.z59=1;
z226.z63=0;
z226.z62=0;
z226.z61=16;
z226.z223=40'h0000_1234_0000;
z226.z224=128;
z232='{32'h0000_0100,32'h0000_0200,32'h0000_0300};
z233='{32'h0000_1010};
z234='{32'h0000_2000,32'h0000_2004};
z235='{32'h0000_3000};
z236='{32'h0000_0180,32'h0000_0184};
z237='{32'h0000_4000};
z238='{32'h0000_5000,32'h0000_5004,32'h0000_5008,
32'h0000_500C,32'h0000_5010,32'h0000_5014};
z239='{32'h0000_0100,32'h0000_0200,32'h0000_0300,32'h0000_0400};
foreach(z240[d])begin
z240[d].z12=24'h00_1000*(d+1);
z240[d].z13[0].push_back('{24'h00_0000,24'hFF_FFFF});
z240[d].z13[1].push_back('{24'h00_0000,24'h00_0FFF});
z240[d].z13[1].push_back('{24'hF0_0000,24'hFF_FFFF});
z240[d].z13[2].push_back('{24'h80_0000,24'h8F_FFFF});
z240[d].z13[3].push_back('{24'h00_1000,24'h00_1FFF});
end
z249.push_back('{z242,40'h0000_0000_1000,13'd63,8'd3,3'd4,'{IPW51:4'd2}});
z249.push_back('{z243,40'h0000_0000_2000,13'd15,8'd0,3'd4,'{IPW51:4'd3}});
z249.push_back('{z244,40'h0000_0000_3000,13'd31,8'd1,3'd4,'{IPW51:4'd5}});
endtask
task automatic z251();
z218(1,"driving_IPW57per:2025 IPW27.randomize()");
begin
z99 IPW27=new;
chk(IPW27.randomize()==1,"randomize returned 0");
chk(IPW27.z100 inside{[900:1800]},"gclk_IPW47_IPW54 out of range");
chk(IPW27.z104 inside{[400:720]},"config IPW54 out of range");
chk(IPW27.z108 inside{[400:1400]},"IPW460 IPW54 out of range");
end
z219();
endtask
task automatic z252();
z218(2,"driving_IPW57per:2036 StartCfgForDUT.randomize()");
begin
z183 z253=new;
chk(z253.randomize()==1,"randomize returned 0");
chk(z253.z187>0,"total_IPW10_in_Gbits == 0");
chk(z253.z189==((40'h800_0000*z253.z187)-1),
"max_aIPW61ess mismatch");
chk(z253.z184[0].z164.z149>0,"IPW19_IPW62 == 0");
chk(z253.z185.z175.z160.z142==0,"IPW40_TYPE != 0");
end
z219();
endtask
task automatic z254();
z218(3,"driving_IPW57per:2056 StartCfgForDUT.randomize() (re-rand)");
begin
z183 z253=new;
void'(z253.randomize());
z253.z186.z124=1;
z253.z185.z180=1;
chk(z253.randomize()==1,"re-randomize returned 0");
chk(z253.z186.z130==1,"IPW09_ds != runarg 1");
chk(z253.z185.z175.z160.z145==1,
"NUM_IPW55S != runarg 1");
end
z219();
endtask
task automatic z255();
z218(4,"driving_IPW57per:1948 num_transactions randomize");
begin
int unsigned z256;
int cts_tp;
int z257;
int unsigned z258=100;
int unsigned z259=10;
int unsigned z260=1000;
int unsigned z261=100;
chk(std::randomize(z256)with{
z256 dist{
[z259:z258]:/50,
[z258+1:z261]:/20,
[z261+1:z260>>1]:/10,
[(z260>>1)+1:z260-1]:/5,
z260:/15
};
}==1,"std::randomize num_transactions_nd failed");
chk(z256 inside{[z259:z260]},
"num_transactions_nd out of dist range");
cts_tp=2;
z257=z256*cts_tp;
chk(z257==z256*2,"num_transactions scaling wrong");
end
z219();
endtask
task automatic z262();
z218(5,"IPW02:1131 IPW46_idx/aIPW61_offset randomize");
begin
int z263;
bit[9:0]z264;
chk(std::randomize(z263,z264)with{
z263 inside{[0:`z30]};
z264!=10'd1;
}==1,"std::randomize IPW46_idx/aIPW61_offset failed");
chk(z263 inside{[0:`z30]},"IPW46_idx out of range");
chk(z264!=10'd1,"aIPW61_offset == 10'd1");
end
z219();
endtask
function automatic z42 z265();
z42 IPW63=new;
IPW63.z56=0;
IPW63.z58=0;
IPW63.z60=17;
IPW63.z59=0;
IPW63.z57=3;
return IPW63;
endfunction
task automatic z266();
z218(6,"IPW44_seq_lib:81 std::randomize(rand_waIPW61_msb)");
begin
logic[12:0]z267;
logic[31:0]z268;
logic[31:0]z269=32'h0000_1234;
chk(std::randomize(z267)==1,"randomize failed");
z268={z267,z269[18:0]};
chk(z268[18:0]==z269[18:0],"fin_waIPW61 low bits mismatch");
end
z219();
endtask
task automatic z270();
z218(7,"IPW44_seq_lib:90 IPW63.randomize() with (WRITE_REG)");
begin
logic[31:0]z269=32'h0000_1234;
logic[31:0]wdata=32'hDEAD_BEEF;
logic[12:0]z267;
logic[31:0]z268;
z42 IPW63;
void'(std::randomize(z267));
z268={z267,z269[18:0]};
IPW63=z265();
chk(IPW63.randomize()with
{
IPW63.aIPW61[39:32]==0;IPW63.size==2;IPW63.len==0;IPW63.write==1;
IPW63.aIPW61[31:0]==z268;z53[0].wdata==wdata;z53[0].strb==4'hf;
z52==1;z54==0;IPW63.vld==1;z50==0;
}==1,"IPW63.randomize() with failed");
chk(IPW63.IPW63.aIPW61[31:0]==z268,"aIPW61 mismatch");
chk(IPW63.IPW63.size==2&&IPW63.IPW63.len==0&&IPW63.IPW63.write==1,"size/len/write mismatch");
chk(IPW63.z53[0].wdata==wdata,"wdata mismatch");
end
z219();
endtask
task automatic z271();
z218(8,"IPW44_seq_lib:107 std::randomize(rand_raIPW61_msb)");
begin
logic[12:0]z272;
logic[31:0]z273;
logic[31:0]z274=32'h0000_0200;
chk(std::randomize(z272)==1,"randomize failed");
z273={z272,z274[18:0]};
chk(z273[18:0]==z274[18:0],"fin_raIPW61 low bits mismatch");
end
z219();
endtask
task automatic z275();
z218(9,"IPW44_seq_lib:116 IPW63.randomize() with (READ_REG)");
begin
logic[31:0]z274=32'h0000_0200;
logic[12:0]z272;
logic[31:0]z273;
z42 IPW63;
void'(std::randomize(z272));
z273={z272,z274[18:0]};
IPW63=z265();
chk(IPW63.randomize()with
{
IPW63.aIPW61[39:32]==0;IPW63.size==2;IPW63.len==0;IPW63.write==0;
IPW63.aIPW61[31:0]==z273;z53[0].strb==4'hf;
z52==1;z54==1;IPW63.vld==1;z50==0;
}==1,"IPW63.randomize() with failed");
chk(IPW63.IPW63.aIPW61[31:0]==z273,"aIPW61 mismatch");
chk(IPW63.IPW63.write==0&&IPW63.z54==1,"write/wait mismatch");
end
z219();
endtask
task automatic z276();
z218(10,"IPW44_seq_lib:134 std::randomize(rand_raIPW61_msb)");
begin
logic[12:0]z272;
chk(std::randomize(z272)==1,"randomize failed");
end
z219();
endtask
task automatic z277();
z218(11,"IPW44_seq_lib:143 IPW63.randomize() with (READ_REG_NO_WAIT)");
begin
logic[31:0]z274=32'h0000_0300;
logic[12:0]z272;
logic[31:0]z273;
z42 IPW63;
void'(std::randomize(z272));
z273={z272,z274[18:0]};
IPW63=z265();
chk(IPW63.randomize()with
{
IPW63.aIPW61[39:32]==0;IPW63.size==2;IPW63.len==0;IPW63.write==0;
IPW63.aIPW61[31:0]==z273;z53[0].strb==4'hf;
z52==1;z54==0;IPW63.vld==1;z50==0;
}==1,"IPW63.randomize() with failed");
chk(IPW63.IPW63.aIPW61[31:0]==z273,"aIPW61 mismatch");
chk(IPW63.z54==0,"wait_for_rdata != 0");
end
z219();
endtask
task automatic z278();
z218(12,"IPW44_seq_lib:257 std::randomize(cfg_wr,wdata,bcIPW60_op,aIPW61,IPW46_num)");
begin
bit cfg_wr;
logic[31:0]wdata;
bit z279;
logic[31:0]aIPW61;
int z280;
int i=1;
chk(std::randomize(cfg_wr,wdata,z279,aIPW61,z280)with{
z279 dist{0:=4,1:=1};
if(z279){
aIPW61 inside{[32'h8000:32'hBFFF]};
}else{
aIPW61 inside{[`z40+(z280<<16):(`z40+(z280<<16)+32'h3FFF)]};
}
aIPW61[1:0]==0;
(z279==1)->(cfg_wr==1);
z280==(i&((`z29>>1)-1));
}==1,"std::randomize cfg_wr... failed");
chk(z279 inside{0,1},"bcIPW60_op not 0/1");
chk(aIPW61[1:0]==0,"aIPW61 not word aligned");
chk(z280==(i&((`z29>>1)-1)),"IPW46_num mismatch");
if(z279)begin
chk(aIPW61 inside{[32'h8000:32'hBFFF]},"bcIPW60 aIPW61 out of range");
chk(cfg_wr==1,"bcIPW60_op but !cfg_wr");
end else begin
chk(aIPW61 inside{[`z40+(z280<<16):`z40+(z280<<16)+32'h3FFF]},
"IPW50 cfg aIPW61 out of range");
end
end
z219();
endtask
task automatic z281();
z218(13,"IPW44_seq_lib:289 std::randomize(run_intr_service,read_IPW36,reprogram_cfg)");
begin
bit z282,z283,z284;
u2_t z285=2'd2;
u2_t z286=2'd2;
u2_t z287=2'd2;
chk(std::randomize(z282,z283,z284)with{
(z285==2'd0)->(z282==0);
(z285==2'd1)->(z282==1);
z282 dist{1:=60,0:=40};
(z286==2'd0)->(z283==0);
(z286==2'd1)->(z283==1);
z283 dist{1:=60,0:=40};
(z287==2'd0)->(z284==0);
(z287==2'd1)->(z284==1);
z284 dist{1:=80,0:=20};
}==1,"std::randomize isr/IPW36/reprog failed");
chk(z282 inside{0,1},"run_intr_service not 0/1");
chk(z283 inside{0,1},"read_IPW36 not 0/1");
chk(z284 inside{0,1},"reprogram_cfg not 0/1");
z285=2'd1;z286=2'd1;z287=2'd1;
chk(std::randomize(z282,z283,z284)with{
(z285==2'd0)->(z282==0);
(z285==2'd1)->(z282==1);
z282 dist{1:=60,0:=40};
(z286==2'd0)->(z283==0);
(z286==2'd1)->(z283==1);
z283 dist{1:=60,0:=40};
(z287==2'd0)->(z284==0);
(z287==2'd1)->(z284==1);
z284 dist{1:=80,0:=20};
}==1,"std::randomize isr/IPW36/reprog (fIPW59ed) failed");
chk(z282==1&&z283==1&&z284==1,
"fIPW59ed ra values not honored");
end
z219();
endtask
task automatic z288();
z218(14,"IPW44_seq_lib:321 std::randomize(aIPW61) with aIPW61 inside queues");
begin
u32_t aIPW61;
chk(std::randomize(aIPW61)with{
aIPW61 inside{z232,z233,z234,z235};
}==1,"std::randomize aIPW61 failed");
chk(aIPW61 inside{z232}||aIPW61 inside{z233}||
aIPW61 inside{z234}||aIPW61 inside{z235},
"aIPW61 not a member of any queue");
end
z219();
endtask
task automatic z289();
z218(15,"IPW44_seq_lib:336 IPW63.randomize() (final, no with)");
begin
z42 IPW63=z265();
chk(IPW63.randomize()==1,"IPW63.randomize() failed");
IPW63.z54=1;
IPW63.z50=1;
chk(IPW63.IPW63.IPW64<=IPW63.z56,"IPW64 > max_IPW63_IPW51");
chk(IPW63.z49==((IPW63.IPW63.len+1)<<IPW63.IPW63.size),"xfer_byte_cnt mismatch");
end
z219();
endtask
task automatic z290();
z218(16,"IPW44_seq_lib:491 std::randomize(bcIPW60_IPW36_ctrl,bcIPW60_IPW36_op)");
begin
bit z291;
z22 z292;
bit z293,z294;
z293=1'b1;z294=1'b0;
chk(std::randomize(z291,z292)with{
if(z293){
z291 dist{1:=4,0:=1};
z292 dist{
z20:=4,
z19:=1
};
}else if(z294){
z291 dist{1:=1,0:=4};
z292 dist{
z18:=4,
z21:=4,
z20:=1,
z19:=1
};
}else{
z291 dist{1:=1,0:=9};
z292 dist{
z20:=4,
z19:=1
};
}
}==1,"std::randomize bcIPW60_IPW36 (not_inited) failed");
chk(z292 inside{z20,z19},
"not_inited branch op out of set");
z293=1'b0;z294=1'b1;
chk(std::randomize(z291,z292)with{
if(z293){
z291 dist{1:=4,0:=1};
z292 dist{z20:=4,z19:=1};
}else if(z294){
z291 dist{1:=1,0:=4};
z292 dist{
z18:=4,z21:=4,
z20:=1,z19:=1
};
}else{
z291 dist{1:=1,0:=9};
z292 dist{z20:=4,z19:=1};
}
}==1,"std::randomize bcIPW60_IPW36 (inited) failed");
z293=1'b0;z294=1'b0;
chk(std::randomize(z291,z292)with{
if(z293){
z291 dist{1:=4,0:=1};
z292 dist{z20:=4,z19:=1};
}else if(z294){
z291 dist{1:=1,0:=4};
z292 dist{
z18:=4,z21:=4,
z20:=1,z19:=1
};
}else{
z291 dist{1:=1,0:=9};
z292 dist{z20:=4,z19:=1};
}
}==1,"std::randomize bcIPW60_IPW36 (mixed) failed");
end
z219();
endtask
task automatic z295();
z218(17,"IPW44_seq_lib:551 std::randomize(IPW49_read_IPW56)");
begin
z28 z296;
chk(std::randomize(z296)==1,"std::randomize IPW49_read_IPW56 failed");
chk(z296 inside{z26,z27},
"IPW49_read_IPW56 invalid");
end
z219();
endtask
task automatic z297();
z218(18,"IPW44_seq_lib:559 std::randomize(IPW49_read_hc_select)");
begin
u4_t z298;
chk(std::randomize(z298)with{
z298 inside{[0:(`z29-1)]};
}==1,"std::randomize IPW49_read_hc_select failed");
chk(z298<=`z29-1,"IPW49_read_hc_select out of range");
end
z219();
endtask
task automatic z299();
z218(19,"IPW44_seq_lib:574 std::randomize(IPW36_op) UNINITIALIZED");
begin
z22 z300;
chk(std::randomize(z300)with{
z300 dist{z20:=4,z19:=1};
}==1,"std::randomize IPW36_op (uninit) failed");
chk(z300 inside{z20,z19},"IPW36_op out of set");
end
z219();
endtask
task automatic z301();
z218(20,"IPW44_seq_lib:579 std::randomize(IPW36_op) COLLECTING");
begin
z22 z300;
chk(std::randomize(z300)with{
z300 dist{
z18:=17,
z20:=1,
z19:=1,
z21:=1
};
}==1,"std::randomize IPW36_op (collecting) failed");
chk(z300 inside{z18,z20,z19,z21},
"IPW36_op out of set");
end
z219();
endtask
task automatic z302();
z218(21,"IPW44_seq_lib:593 std::randomize(IPW63_client_list,...,IPW36_op)");
begin
u7_t z303[],z304[],z305[];
u9_t z306[];
z22 z300;
z28 z296;
z296=z26;
chk(std::randomize(z303,z304,z305,z306,z300)with{
if(z296==z26){
z303.size()==`z33;
z304.size()==`z33;
z305.size()==`z33;
z306.size()==(`z32*`z31);
foreach(z303[i]){
z303[i]==i;
z304[i]==i;
z305[i]==i;
}
foreach(z306[i]){
z306[i]==i;
}
}else{
z303.size()==1;
z304.size()==1;
z305.size()==1;
z306.size()==1;
foreach(z303[i]){
z303[i]inside{[0:`z33-1]};
z304[i]inside{[0:`z33-1]};
z305[i]inside{[0:`z33-1]};
}
foreach(z306[i]){
z306[i]inside{[0:`z32*`z31]};
}
}
z300 dist{
z18:=1,
z20:=12,
z19:=1,
z21:=7
};
}==1,"std::randomize IPW36 lists (full) failed");
chk(z303.size()==`z33,"IPW63_client_list size mismatch");
chk(z306.size()==`z32*`z31,"bank_idx_list size mismatch");
foreach(z303[i])chk(z303[i]==i,"IPW63_client_list[i] != i");
z296=z27;
chk(std::randomize(z303,z304,z305,z306,z300)with{
if(z296==z26){
z303.size()==`z33;
z304.size()==`z33;
z305.size()==`z33;
z306.size()==(`z32*`z31);
foreach(z303[i]){
z303[i]==i;
z304[i]==i;
z305[i]==i;
}
foreach(z306[i]){z306[i]==i;}
}else{
z303.size()==1;
z304.size()==1;
z305.size()==1;
z306.size()==1;
foreach(z303[i]){
z303[i]inside{[0:`z33-1]};
z304[i]inside{[0:`z33-1]};
z305[i]inside{[0:`z33-1]};
}
foreach(z306[i]){
z306[i]inside{[0:`z32*`z31]};
}
}
z300 dist{
z18:=1,
z20:=12,
z19:=1,
z21:=7
};
}==1,"std::randomize IPW36 lists (single) failed");
chk(z303.size()==1,"IPW63_client_list size != 1");
chk(z303[0]<`z33,"IPW63_client out of range");
end
z219();
endtask
function automatic z42 z307();
z42 IPW63=new;
IPW63.z56=z226.z221;
IPW63.z61=z226.z61;
IPW63.z63=z226.z63;
IPW63.z62=z226.z62;
IPW63.z58=z226.z58;
IPW63.z60=z226.z60;
IPW63.z59=z226.z59;
IPW63.z57=z226.z57;
return IPW63;
endfunction
task automatic z308();
z218(22,"IPW44_seq_lib:766 tmp_cfg.randomize() (IPW47IPW55CtrlConfig)");
begin
z163 z309;
z309=new;
chk(z309.randomize()==1,"tmp_cfg.randomize() failed");
chk(z309.z164.z149>0,"IPW19_IPW62 == 0");
end
z219();
endtask
task automatic z310();
z218(23,"IPW44_seq_lib:768 std::randomize(num_reprog_regs,bcIPW60_cfg,cfg_dest_aIPW61,cfg_dest_IPW55)");
begin
bit z311[];
u32_t z312[];
u4_t z313[];
u4_t z314;
chk(std::randomize(z314,z311,z312,z313)with{
z314>=4;
z312.size()==z314;
z311.size()==z314;
z313.size()==z314;
foreach(z313[i]){
z313[i]<`z29;
}
foreach(z311[i]){
z311[i]dist{1:=2,0:=1};
!(z312[i]inside{z236,z237});
z312[i]inside{z238};
}
}==1,"std::randomize reprog lists failed");
chk(z314>=4,"num_reprog_regs < 4");
chk(z312.size()==z314,"cfg_dest_aIPW61 size mismatch");
foreach(z311[i])begin
chk(!(z312[i]inside{z236,z237}),"aIPW61 in nowr/w1c set");
chk(z312[i]inside{z238},"aIPW61 not in on_the_fly set");
chk(z313[i]<`z29,"cfg_dest_IPW55 >= `NUM_IPW50");
end
end
z219();
endtask
task automatic z315();
z218(24,"IPW44_seq_lib:869 std::randomize(cov_hnum)");
begin
bit[3:0]z316;
chk(std::randomize(z316)with{z316 inside{[0:(`z29-1)]};}==1,
"std::randomize cov_hnum failed");
chk(z316<=`z29-1,"cov_hnum out of range");
end
z219();
endtask
task automatic z317();
z218(25,"IPW44_seq_lib:898 IPW63.randomize() with (rdbufs_full 64B)");
begin
bit[3:0]z316=0;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
z46==0;z47==0;
z229->(IPW63.aIPW61[13:10]==z316);
IPW63.aIPW61[5:0]==0;
z52==0;z54==0;IPW63.vld==1;z50==0;
z49==64;
IPW63.write==0;
}==1,"IPW63.randomize() with (64B) failed");
chk(IPW63.z49==64,"xfer_byte_cnt != 64");
chk(IPW63.IPW63.aIPW61[5:0]==0,"aIPW61 not 64B aligned");
chk(IPW63.IPW63.write==0,"write != 0");
end
z219();
endtask
task automatic z318();
z218(26,"IPW44_seq_lib:922 IPW63.randomize() with (rdbufs_full 8B)");
begin
bit[3:0]z316=0;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
z46==0;z47==0;
z229->(IPW63.aIPW61[13:10]==z316);
IPW63.aIPW61[2:0]==0;
z52==0;z54==0;IPW63.vld==1;z50==0;
z49==8;
IPW63.write==0;
}==1,"IPW63.randomize() with (8B) failed");
chk(IPW63.z49==8,"xfer_byte_cnt != 8");
chk(IPW63.IPW63.aIPW61[2:0]==0,"aIPW61 not 8B aligned");
end
z219();
endtask
task automatic z319();
z218(27,"IPW44_seq_lib:946 std::randomize(IPW51_to_use)");
begin
bit[2:0]z320;
chk(std::randomize(z320)with{
(z320<=z226.z221);
}==1,"std::randomize IPW51_to_use failed");
chk(z320<=z226.z221,"IPW51_to_use > max_IPW51");
end
z219();
endtask
task automatic z321();
z218(28,"IPW44_seq_lib:954 std::randomize(actual_IPW41_pattern,trigger_IPW48,...)");
begin
bit[2:0]z320=3'd2;
bit[2:0]z241;
bit[2:0]z322,z323;
bit z324;
bit[4:0]z325;
z11 z326;
bit[23:0]z327;
bit z328;
z241=3'd2;
z322=3'b101;
chk(std::randomize(z323,z324,z325,z326,z327,z328)with{
z324 dist{0:=97,1:=3};
(z324==1)->(z323==0);
foreach(z322[i]){
if(z322[i]){
z323[i]dist{0:=98,1:=2};
}else{
z323[i]==0;
}
}
z240[z241].z13[z323].size()!=0;
z325<=(z240[z241].z13[z323].size()-1);
if(z324){
z326.z9==z240[z241].z12;
z326.z10==24'hFF_FFFF;
}else{
z326==z240[z241].z13[z323][z325];
}
((z323==0)&!z324&z226.z59&z227)->(z328 dist{0:=4,1:=1});
((z323==0)&!z324&z226.z59&z227)->(z328 dist{0:=19,1:=1});
((z323!=0)|z324|!z226.z59)->(z328==0);
(z228==1)->(z328==0);
(z328==1)->(z327==z226.z223[39:16]);
(z328==0)->z327 inside{[z326.z9:z326.z10]};
}==1,"std::randomize IPW41 pattern failed");
chk(z324 inside{0,1},"trigger_IPW48 not 0/1");
if(z324)chk(z323==0,"IPW48 but pattern != 0");
foreach(z322[i])begin
if(!z322[i])chk(z323[i]==0,"pattern bit set where not possible");
end
chk(z325<=31,"IPW41_region_idx > 31");
if(z328==1)begin
chk(z327==z226.z223[39:16],"excl region aIPW61 mismatch");
end else begin
chk(z327>=z326.z9&&
z327<=z326.z10,"IPW28 out of region");
end
end
z219();
endtask
task automatic z329();
z218(29,"IPW44_seq_lib:988 std::randomize(generate_back2back_IPW63uests,...)");
begin
bit z330;
bit z331;
int z332;
int z333;
int z334;
bit[7:0]z335;
bit z336;
bit[3:0]z337;
bit[4:0]z338;
bit z339;
bit[39:0]z340;
bit z328,z341;
bit[3:0]z316;
bit[4:0]z342[`z29];
bit[4:0]z343[`z31];
bit[23:0]z327=24'h00_1234;
chk(std::randomize(z330,z331,z334,z340,
z336,z337,z338,z335,
z342,z343,z339,z316,
z332,z333,z341)with{
$countones(
{z330,z331,z339,z336}
)inside{0,1};
z338 inside{[5:30]};
z330 dist{0:/49,1:/1};
z331 dist{0:/60,1:/40};
z226.z63->(z341==0);
z341 dist{0:=9,1:=1};
z332 inside{[10:30]};
z333 inside{[10:100]};
z334 inside{[11:15]};
z316 inside{[0:(`z29-1)]};
z340[39:16]==z327;
z229->(z340[13:10]==z316);
foreach(z342[i]){
z342[i]==i;
}
foreach(z343[i]){
z343[i]==i;
}
if(z230|(z231!=0)){
z338<=7;
z332<=15;
z333<=15;
}
}==1,"std::randomize seq shape failed");
chk($countones({z330,z331,z339,z336})inside{0,1},
"more than one IPW56 bit set");
chk(z332 inside{[10:30]},"num_back2back out of range");
chk(z334 inside{[11:15]},"shift_amt out of range");
chk(z340[39:16]==z327,"aIPW61_base page mismatch");
foreach(z342[i])chk(z342[i]==i,"IPW55s_order[i] != i");
foreach(z343[i])chk(z343[i]==i,"IPW37_order[i] != i");
end
z219();
endtask
task automatic z344();
z218(30,"IPW44_seq_lib:1029 std::randomize(bnumt)");
begin
bit[3:0]bnumt;
chk(std::randomize(bnumt)==1,"std::randomize bnumt failed");
end
z219();
endtask
task automatic z345();
z218(31,"IPW44_seq_lib:1042 IPW63.randomize() with (test_bankq_full)");
begin
bit[3:0]bnumt=4'd5;
bit[3:0]z316=0;
bit[23:0]z327=24'h00_1234;
bit[6:0]z222=7'd10;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
IPW63.IPW52==0;
IPW63.aIPW61[13:10]==z316;
IPW63.aIPW61[9:6]==bnumt;
IPW63.aIPW61[39:16]==z327;
z49==64;
z52==0;z54==0;IPW63.vld==1;z50==0;
(z58&z228)->(
IPW63.IPW38 dist{2:=z222,1:=(100-z222)}
);
}==1,"IPW63.randomize() with (bankq) failed");
chk(IPW63.IPW63.aIPW61[13:10]==z316,"IPW55 bits mismatch");
chk(IPW63.IPW63.aIPW61[9:6]==bnumt,"bank bits mismatch");
chk(IPW63.z49==64,"xfer_byte_cnt != 64");
end
z219();
endtask
task automatic z346();
z218(32,"IPW44_seq_lib:1069 std::randomize(do_wr,this_IPW63_set_IPW57)");
begin
bit do_wr;
bit z347;
bit[6:0]z222=7'd10;
chk(std::randomize(do_wr,z347)with{
z226.z63->(do_wr==0);
z226.z62->(do_wr==1);
do_wr dist{0:/9,1:/1};
if(z226.z58&!do_wr){
z347 dist{1:=z222,0:=(100-z222)};
}else{
z347==0;
}
}==1,"std::randomize do_wr/IPW57 failed");
chk(do_wr inside{0,1},"do_wr not 0/1");
if(do_wr)chk(z347==0,"IPW57 set on write");
end
z219();
endtask
task automatic z348();
z218(33,"IPW44_seq_lib:1081 std::randomize(IPW552favor)");
begin
bit[3:0]z337;
chk(std::randomize(z337)with{z337<`z29;}==1,
"std::randomize IPW552favor failed");
chk(z337<`z29,"IPW552favor >= `NUM_IPW50");
end
z219();
endtask
task automatic z349();
z218(34,"IPW44_seq_lib.sv:1088 std::randomize(this_IPW63_IPW58_m1,this_IPW63_len,this_IPW63_size)");
begin
bit[12:0]z350,z351;
bit[8:0]z352,z353;
bit z354=0;
int hnum=0,z337=0;
z351=(z226.z224==64)?7:(z226.z224==128)?
15:(z226.z224==256)?31:0;
chk(std::randomize(z350,z352,z353)with{
z353<=5;
(z226.z224==256)->(z353==5);
(z226.z224==128)->(z353==4);
(z226.z224==64)->(z353==3);
z350<=z226.z57;
if(z354){
(z350 inside{31,63,127,255});
}
z352<=255;
if(hnum==z337){
if(z226.z57>=255){
z350>=255;
z350<=1023;
}else{
z350==z226.z57;
}
}else{
if(z354){
z350==31;
}else{
z350<=z351;
}
}
(z350>255)->(z350[0]==1);
(z350>511)->(z350[1:0]==2'b11);
(z350>1023)->(z350[2:0]==3'b111);
(z350>2047)->(z350[3:0]==4'b1111);
(z350==(((z352+1)<<z353)-1));
}==1,"std::randomize this_IPW63 shape failed");
chk(z353<=5,"this_IPW63_size > 5");
chk(z350==(((z352+1)<<z353)-1),"IPW58/len/size inconsistent");
end
z219();
endtask
task automatic z355();
z218(35,"IPW44_seq_lib:1128 IPW63.randomize() with (favor_one_IPW55)");
begin
bit do_wr=0;
bit[12:0]z350=13'd15;
bit[8:0]z352=9'd0,z353=9'd4;
int hnum=0;
bit[39:0]z340=40'h0000_1234_0000;
bit[39:0]z356=40'hFFFF_FFFF_FFFF;
bit[6:0]z222=7'd10;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
IPW63.IPW52==0;
IPW63.write==do_wr;
IPW63.aIPW61[13:10]==hnum;
(IPW63.aIPW61&{z356[39:14],14'h0})==(z340&{z356[39:14],14'h0});
z49==(z350+14'd1);
IPW63.len==z352;
IPW63.size==z353;
z52==0;z54==0;IPW63.vld==1;z50==0;
(z58&z228&!IPW63.write)->(
IPW63.IPW38 dist{2:=z222,1:=(100-z222)}
);
}==1,"IPW63.randomize() with (favor IPW55) failed");
chk(IPW63.IPW63.write==do_wr,"write mismatch");
chk(IPW63.IPW63.aIPW61[13:10]==hnum,"IPW55 bits mismatch");
chk(IPW63.z49==(z350+1),"xfer_byte_cnt mismatch");
end
z219();
endtask
task automatic z357();
z218(36,"IPW44_seq_lib:1169 IPW63.randomize() with (back2back)");
begin
bit z328=0;
bit[2:0]z320=3'd2;
bit[3:0]z316=0;
bit[23:0]z327=24'h00_1234;
bit[6:0]z222=7'd10;
bit z358=0;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
z46==0;z47==0;
if(z226.z59){
IPW63.IPW52==z328;
IPW63.IPW52->(IPW63.aIPW61==z226.z223);
}
!IPW63.IPW52->(IPW63.aIPW61[39:16]==z327);
z229->(IPW63.aIPW61[13:10]==z316);
IPW63.IPW64==z320;
z52==0;z54==0;IPW63.vld==1;z50==0;
(z58&z228&(IPW63.write==0))->(
IPW63.IPW38 dist{2:=z222,1:=(100-z222)}
);
!(z49 inside{32,64,128,256})->(IPW63.IPW38!=2);
if(z358){
z49==z226.z57+1;
IPW63.write==0;
}
}==1,"IPW63.randomize() with (b2b) failed");
chk(IPW63.IPW63.IPW64==z320,"IPW64 mismatch");
chk(IPW63.IPW63.aIPW61[39:16]==z327,"aIPW61 page mismatch");
end
z219();
endtask
task automatic z359();
z218(37,"IPW44_seq_lib:1212 IPW63.randomize() with (single page)");
begin
bit[2:0]z320=3'd2;
bit[3:0]z316=0;
bit[39:0]z340=40'h0000_1234_0000;
bit[39:0]z356=40'hFFFF_FFFF_FFFF;
bit[6:0]z222=7'd10;
bit z358=0;
z42 IPW63=z307();
IPW63.z59=0;
chk(IPW63.randomize()with
{
(IPW63.aIPW61&z356)==(z340&z356);
IPW63.IPW64==z320;
z52==0;z54==0;IPW63.vld==1;z50==0;
if(z358){
z49==z226.z57+1;
IPW63.write==0;
}
z229->(IPW63.aIPW61[13:10]==z316);
!(z49 inside{32,64,128,256})->(IPW63.IPW38!=2);
(z58&z228&(IPW63.write==0))->(
IPW63.IPW38 dist{2:=z222,1:=(100-z222)}
);
}==1,"IPW63.randomize() with (single page) failed");
chk((IPW63.IPW63.aIPW61&z356)==(z340&z356),"aIPW61 mask mismatch");
chk(IPW63.IPW63.IPW64==z320,"IPW64 mismatch");
end
z219();
endtask
task automatic z360();
z218(38,"IPW44_seq_lib:1250 IPW63.randomize() with (deIPW41)");
begin
bit z328=0;
bit z341=0;
bit[2:0]z320=3'd2;
bit[3:0]z316=0;
bit[23:0]z327=24'h00_1234;
bit[6:0]z222=7'd10;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
if(z226.z59){
IPW63.IPW52==z328;
IPW63.IPW52->(IPW63.aIPW61==z226.z223);
}
if(z341&!z328){
IPW63.IPW52==0;
IPW63.aIPW61[39:6]==z226.z223[39:6];
IPW63.write==1;
}else{
IPW63.aIPW61[39:16]==z327;
z229->(IPW63.aIPW61[13:10]==z316);
}
IPW63.IPW64==z320;
z52==0;z54==0;IPW63.vld==1;z50==0;
(z58&z228&(IPW63.write==0))->(
IPW63.IPW38 dist{2:=z222,1:=(100-z222)}
);
}==1,"IPW63.randomize() with (deIPW41) failed");
chk(IPW63.IPW63.IPW64==z320,"IPW64 mismatch");
chk(IPW63.IPW63.aIPW61[39:16]==z327,"aIPW61 page mismatch");
end
z219();
endtask
task automatic z361();
z218(39,"IPW44_seq_lib:1292 std::randomize(IPW51_to_use) (interference)");
begin
bit[2:0]z320;
chk(std::randomize(z320)with{z320<=z226.z221;}==1,
"std::randomize IPW51_to_use (interf) failed");
chk(z320<=z226.z221,"IPW51_to_use > max_IPW51");
end
z219();
endtask
task automatic z362();
z218(40,"IPW44_seq_lib:1297 std::randomize(actual_IPW41_pattern,IPW41_region_idx,...)");
begin
bit[2:0]z241=3'd2;
bit[2:0]z323;
bit[4:0]z325;
z11 z326;
bit[23:0]z327;
chk(std::randomize(z323,z325,z326,z327)with{
z323==0;
z325<=(z240[z241].z13[z323].size()-1);
z326==z240[z241].z13[z323][z325];
z327 inside{[z326.z9:z326.z10]};
}==1,"std::randomize interf IPW41 region failed");
chk(z323==0,"actual_IPW41_pattern != 0");
chk(z327>=z326.z9&&
z327<=z326.z10,"IPW28 out of region");
end
z219();
endtask
task automatic z363();
z218(41,"IPW44_seq_lib:1314 IPW63.randomize() with (interference)");
begin
bit[2:0]z320=3'd2;
bit[23:0]z327=24'h00_1234;
z42 IPW63=z307();
chk(IPW63.randomize()with
{
if(z226.z59){
IPW63.IPW52->(IPW63.aIPW61==z226.z223);
}
!IPW63.IPW52->(IPW63.aIPW61[39:16]==z327);
IPW63.IPW64==z320;
z52==0;z54==0;IPW63.vld==1;z50==0;
}==1,"IPW63.randomize() with (interference) failed");
chk(IPW63.IPW63.IPW64==z320,"IPW64 mismatch");
chk(IPW63.IPW63.aIPW61[39:16]==z327,"aIPW61 page mismatch");
end
z219();
endtask
task automatic z364();
z218(42,"IPW44_seq_lib:1340 IPW63.randomize() (lIPW60 xfer)");
begin
z42 IPW63=z307();
chk(IPW63.randomize()==1,"IPW63.randomize() failed");
IPW63.z54=0;
IPW63.z50=1;
chk(IPW63.IPW63.IPW64<=IPW63.z56,"IPW64 > max_IPW63_IPW51");
end
z219();
endtask
task automatic z365();
z218(43,"IPW44_seq_lib:1432 IPW63.randomize() with (IPW42 IPW35)");
begin
bit[5:0]z366=6'd3;
bit[39:0]z367=40'h0000_0000_1000;
bit[12:0]z368=13'd63;
bit z369=0;
bit z370=0;
bit z371=0;
int z353;
z42 IPW63=z307();
IPW63.z59=z369;
z353=$clog2((z368+1)&(~(z368+1)+1));
chk(IPW63.randomize()with
{
IPW63.vld==1;
z46==0;
z47==0;
IPW63.aIPW61==z367;
z49==(z368+1);
IPW63.write==z371;
z51==1;
IPW63.IPW64==4'd2;
IPW63.IPW52==z369;
if(z366==63){
IPW63.size==2;
}else{
if(z353>z65){
IPW63.size==z65;
}else{
IPW63.size==z353;
}
}
z50==0;
z52==(z366==63);
IPW63.IPW38==({z370,!z370});
z54==0;
}==1,"IPW63.randomize() with (IPW42 IPW35) failed");
chk(IPW63.IPW63.aIPW61==z367,"IPW43 mismatch");
chk(IPW63.z49==(z368+1),"IPW58 mismatch");
chk(IPW63.IPW63.IPW38=={z370,!z370},"IPW38 mismatch");
end
z219();
endtask
task automatic z372();
z218(44,"IPW44_seq_lib:1490 IPW63.randomize() (IPW42 lIPW60 xfer)");
begin
bit[5:0]z366=6'd3;
z42 IPW63=new;
IPW63.z56=(z366==63)?0:z226.z221;
IPW63.z61=(z366==63)?0:z226.z61;
IPW63.z63=(z366==63)?0:z226.z63;
IPW63.z62=(z366==63)?0:z226.z62;
IPW63.z58=(z366==63)?0:z226.z58;
IPW63.z60=(z366==63)?17:z226.z60;
IPW63.z57=(z366==63)?3:z226.z57;
IPW63.z59=(z366==63)?0:z226.z59;
chk(IPW63.randomize()==1,"IPW63.randomize() failed");
IPW63.z54=0;
IPW63.z50=1;
chk(IPW63.IPW63.IPW64<=IPW63.z56,"IPW64 > max_IPW63_IPW51");
end
z219();
endtask
function automatic z82 z373();
z82 IPW63=new;
IPW63.z56=z226.z221;
IPW63.z61=z226.z61;
IPW63.z57=z226.z57;
IPW63.z63=z226.z63;
IPW63.z62=z226.z62;
return IPW63;
endfunction
task automatic z374();
z218(45,"IPW45_seq_lib:61 std::randomize(cov_hnum)");
begin
bit[3:0]z316;
chk(std::randomize(z316)with{z316 inside{[0:(`z29-1)]};}==1,
"std::randomize cov_hnum failed");
chk(z316<=`z29-1,"cov_hnum out of range");
end
z219();
endtask
task automatic z375();
z218(46,"IPW45_seq_lib:90 std::randomize(start_aIPW61)");
begin
bit[39:0]z376;
chk(std::randomize(z376)with{
z376[15:0]==0;
z376<=32'hFFF_FFFF;
}==1,"std::randomize start_aIPW61 failed");
chk(z376[15:0]==0,"start_aIPW61 not 64K aligned");
chk(z376<=32'hFFF_FFFF,"start_aIPW61 too big");
end
z219();
endtask
task automatic z377();
z218(47,"IPW45_seq_lib:103 IPW63.randomize() with (perf wr)");
begin
bit[39:0]z376=40'h0000_1000_0000;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z84==z376;
z50==0;
z85==(z226.z57);
z46==0;
z83==0;
z86==1;
z87==0;
}==1,"IPW63.randomize() with (perf wr) failed");
chk(IPW63.z84==z376,"IPW63_aIPW61 mismatch");
chk(IPW63.z86==1,"IPW63_write != 1");
chk(IPW63.z85==z226.z57,"IPW63_IPW58_m1 mismatch");
end
z219();
endtask
task automatic z378();
z218(48,"IPW45_seq_lib:129 IPW63.randomize() with (perf flush rd)");
begin
bit[39:0]z376=40'h0000_1000_0000;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z84==z376;
z50==0;
z85==0;
z46==0;
z83==0;
z86==0;
z87==0;
}==1,"IPW63.randomize() with (perf flush) failed");
chk(IPW63.z84==z376,"IPW63_aIPW61 mismatch");
chk(IPW63.z85==0,"IPW63_IPW58_m1 != 0");
end
z219();
endtask
task automatic z379();
z218(49,"IPW45_seq_lib:152 IPW63.randomize() with (perf rd)");
begin
bit[39:0]z376=40'h0000_1000_0000;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z84==z376;
z50==0;
z85==(z226.z57);
z46==0;
z83==0;
z86==0;
z87==0;
}==1,"IPW63.randomize() with (perf rd) failed");
chk(IPW63.z84==z376,"IPW63_aIPW61 mismatch");
chk(IPW63.z86==0,"IPW63_write != 0");
end
z219();
endtask
task automatic z380();
z218(50,"IPW45_seq_lib:181 IPW63.randomize() with (rand_perf wr)");
begin
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
z85==0;
z46==0;
z83==0;
z86==1;
z87==0;
}==1,"IPW63.randomize() with (rand_perf wr) failed");
chk(IPW63.z86==1,"IPW63_write != 1");
chk(IPW63.z85==0,"IPW63_IPW58_m1 != 0");
end
z219();
endtask
task automatic z381();
z218(51,"IPW45_seq_lib:204 IPW63.randomize() with (rand_perf flush)");
begin
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
z85==0;
z46==0;
z83==0;
z86==0;
z87==0;
}==1,"IPW63.randomize() with (rand_perf flush) failed");
chk(IPW63.z86==0,"IPW63_write != 0");
end
z219();
endtask
task automatic z382();
z218(52,"IPW45_seq_lib:225 IPW63.randomize() with (rand_perf rd)");
begin
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
z85==0;
z46==0;
z83==0;
z86==0;
z87==0;
}==1,"IPW63.randomize() with (rand_perf rd) failed");
chk(IPW63.z86==0,"IPW63_write != 0");
chk(IPW63.z85==0,"IPW63_IPW58_m1 != 0");
end
z219();
endtask
task automatic z383();
z218(53,"IPW45_seq_lib:255 IPW63.randomize() with (rdbufs_full 64B)");
begin
bit[3:0]z316=0;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
z85==(z226.z57);
z229->(z84[13:10]==z316);
z46==0;
z83==0;
z86==0;
z87==0;
}==1,"IPW63.randomize() with (rdbufs 64B) failed");
chk(IPW63.z85==z226.z57,"IPW63_IPW58_m1 mismatch");
chk(IPW63.z86==0,"IPW63_write != 0");
end
z219();
endtask
task automatic z384();
z218(54,"IPW45_seq_lib:277 IPW63.randomize() with (rdbufs_full 8B)");
begin
bit[3:0]z316=0;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
z85==7;
z46==0;
z83==0;
z86==0;
z229->(z84[13:10]==z316);
z87==0;
}==1,"IPW63.randomize() with (rdbufs 8B) failed");
chk(IPW63.z85==7,"IPW63_IPW58_m1 != 7");
chk(IPW63.z86==0,"IPW63_write != 0");
end
z219();
endtask
task automatic z385();
z218(55,"IPW45_seq_lib:304 std::randomize(IPW51_to_use)");
begin
bit[2:0]z320;
chk(std::randomize(z320)with{z320<=z226.z221;}==1,
"std::randomize IPW51_to_use failed");
chk(z320<=z226.z221,"IPW51_to_use > max_IPW51");
end
z219();
endtask
task automatic z386();
z218(56,"IPW45_seq_lib:310 std::randomize(actual_IPW41_pattern,trigger_IPW48,...)");
begin
bit[2:0]z241=3'd2;
bit[2:0]z322,z323;
bit z324;
bit[4:0]z325;
z11 z326;
bit[23:0]z327;
z322=3'b101;
chk(std::randomize(z323,z324,z325,z326,z327)with{
z324 dist{0:=97,1:=3};
(z324==1)->(z323==0);
foreach(z322[i]){
if(z322[i]){
z323[i]dist{0:=98,1:=2};
}else{
z323[i]==0;
}
}
z240[z241].z13[z323].size()!=0;
z325<=(z240[z241].z13[z323].size()-1);
if(z324){
z326.z9==z240[z241].z12;
z326.z10==24'hFF_FFFF;
}else{
z326==z240[z241].z13[z323][z325];
}
z327 inside{[z326.z9:z326.z10]};
}==1,"std::randomize IPW45 IPW41 pattern failed");
chk(z327>=z326.z9&&
z327<=z326.z10,"IPW28 out of region");
end
z219();
endtask
task automatic z387();
z218(57,"IPW45_seq_lib:336 std::randomize(generate_back2back_IPW63uests,...)");
begin
bit z330;
bit z331;
int z332;
int z333;
int z334;
bit[7:0]z335;
bit z336;
bit[3:0]z337;
bit[4:0]z338;
bit z339;
bit[39:0]z340;
bit z341;
bit[3:0]z316;
bit[4:0]z342[16];
bit[4:0]z343[16];
bit[23:0]z327=24'h00_1234;
chk(std::randomize(z330,z331,z334,z340,
z336,z337,z338,z335,
z342,z343,z339,z316,
z332,z333,z341)with{
$countones(
{z330,z331,z339,z336}
)inside{0,1};
z338 inside{[5:30]};
z330 dist{0:/49,1:/1};
z331 dist{0:/60,1:/40};
z341 dist{0:=9,1:=1};
z226.z63->(z341==0);
z332 inside{[10:100]};
z333 inside{[10:50]};
z334 inside{[11:15]};
z316 inside{[0:(`z29-1)]};
z340[39:16]==z327;
foreach(z342[i]){
z342[i]==i;
}
foreach(z343[i]){
z343[i]==i;
}
}==1,"std::randomize IPW45 seq shape failed");
chk(z332 inside{[10:100]},"num_back2back out of range");
chk(z340[39:16]==z327,"aIPW61_base page mismatch");
end
z219();
endtask
task automatic z388();
z218(58,"IPW45_seq_lib:384 std::randomize(bnumt)");
begin
bit[3:0]bnumt;
chk(std::randomize(bnumt)==1,"std::randomize bnumt failed");
end
z219();
endtask
task automatic z389();
z218(59,"IPW45_seq_lib:393 IPW63.randomize() with (test_bankq_full)");
begin
bit[3:0]bnumt=4'd5;
bit[3:0]z316=0;
bit[23:0]z327=24'h00_1234;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
z85==63;
IPW63.z84[13:10]==z316;
IPW63.z84[9:6]==bnumt;
IPW63.z84[39:16]==z327;
z46==0;
z83==0;
z87==0;
}==1,"IPW63.randomize() with (bankq) failed");
chk(IPW63.z84[13:10]==z316,"IPW55 bits mismatch");
chk(IPW63.z84[9:6]==bnumt,"bank bits mismatch");
chk(IPW63.z85==63,"IPW63_IPW58_m1 != 63");
end
z219();
endtask
task automatic z390();
z218(60,"IPW45_seq_lib:417 std::randomize(do_wr)");
begin
bit do_wr;
chk(std::randomize(do_wr)with{
z226.z63->(do_wr==0);
z226.z62->(do_wr==1);
do_wr dist{0:/9,1:/1};
}==1,"std::randomize do_wr failed");
chk(do_wr inside{0,1},"do_wr not 0/1");
end
z219();
endtask
task automatic z391();
z218(61,"IPW45_seq_lib:423 std::randomize(IPW552favor)");
begin
bit[3:0]z337;
chk(std::randomize(z337)==1,"std::randomize IPW552favor failed");
end
z219();
endtask
task automatic z392();
z218(62,"IPW45_seq_lib:428 std::randomize(this_IPW63_IPW58_m1)");
begin
bit[12:0]z350,z351;
int hnum=0,z337=0;
z351=(!z226.z62)?((z226.z224==64)?7:(z226.z224==128)?
15:(z226.z224==256)?31:0):31;
chk(std::randomize(z350)with{
z350<=z226.z57;
if(hnum==z337){
if(z226.z57>=255){
z350>=255;
}else{
z350==z226.z57;
}
}else{
z350<=z351;
}
}==1,"std::randomize this_IPW63_IPW58_m1 failed");
chk(z350<=z226.z57,"IPW58 > max");
end
z219();
endtask
task automatic z393();
z218(63,"IPW45_seq_lib:448 IPW63.randomize() with (favor_one_IPW55)");
begin
bit do_wr=0;
bit[12:0]z350=13'd15;
int hnum=0;
bit[23:0]z327=24'h00_1234;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
z50==0;
IPW63.z85==z350;
IPW63.z84[13:10]==hnum;
IPW63.z84[39:16]==z327;
z46==0;
z83==0;
z86==do_wr;
z87==0;
}==1,"IPW63.randomize() with (favor IPW55) failed");
chk(IPW63.z84[13:10]==hnum,"IPW55 bits mismatch");
chk(IPW63.z85==z350,"IPW58 mismatch");
chk(IPW63.z86==do_wr,"write mismatch");
end
z219();
endtask
task automatic z394();
z218(64,"IPW45_seq_lib:478 IPW63.randomize() with (back2back)");
begin
bit[2:0]z320=3'd2;
bit[23:0]z327=24'h00_1234;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
IPW63.z46==0;
IPW63.z84[39:16]==z327;
IPW63.z88==z320;
IPW63.z83==0;
}==1,"IPW63.randomize() with (b2b) failed");
chk(IPW63.z84[39:16]==z327,"aIPW61 page mismatch");
chk(IPW63.z88==z320,"IPW51 mismatch");
end
z219();
endtask
task automatic z395();
z218(65,"IPW45_seq_lib:503 IPW63.randomize() with (single page)");
begin
bit[2:0]z320=3'd2;
bit[23:0]z327=24'h00_1234;
bit[39:0]z340=40'h0000_1234_0000;
bit[39:0]z356=40'hFFFF_FFFF_FFFF;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
(IPW63.z84&z356)==(z340&z356);
IPW63.z84[39:16]==z327;
IPW63.z88==z320;
}==1,"IPW63.randomize() with (single page) failed");
chk((IPW63.z84&z356)==(z340&z356),"aIPW61 mask mismatch");
chk(IPW63.z88==z320,"IPW51 mismatch");
end
z219();
endtask
task automatic z396();
z218(66,"IPW45_seq_lib:526 IPW63.randomize() with (deIPW41)");
begin
bit[2:0]z320=3'd2;
bit z341=0;
bit[23:0]z327=24'h00_1234;
z82 IPW63=z373();
chk(IPW63.randomize()with{
IPW63.z88==z320;
if(z341){
IPW63.z84[39:8]==z226.z223[39:8];
IPW63.z86==1;
}else{
IPW63.z84[39:16]==z327;
}
}==1,"IPW63.randomize() with (deIPW41) failed");
chk(IPW63.z88==z320,"IPW51 mismatch");
chk(IPW63.z84[39:16]==z327,"aIPW61 page mismatch");
end
z219();
endtask
task automatic z397();
z218(67,"IPW45_seq_lib:554 std::randomize(IPW51_to_use) (interference)");
begin
bit[2:0]z320;
chk(std::randomize(z320)with{z320<=z226.z221;}==1,
"std::randomize IPW51_to_use (interf) failed");
chk(z320<=z226.z221,"IPW51_to_use > max_IPW51");
end
z219();
endtask
task automatic z398();
z218(68,"IPW45_seq_lib:559 std::randomize(actual_IPW41_pattern,IPW41_region_idx,...)");
begin
bit[2:0]z241=3'd2;
bit[2:0]z323;
bit[4:0]z325;
z11 z326;
bit[23:0]z327;
chk(std::randomize(z323,z325,z326,z327)with{
z323==0;
z325<=(z240[z241].z13[z323].size()-1);
z326==z240[z241].z13[z323][z325];
z327 inside{[z326.z9:z326.z10]};
}==1,"std::randomize IPW45 interf IPW41 region failed");
chk(z323==0,"actual_IPW41_pattern != 0");
end
z219();
endtask
task automatic z399();
z218(69,"IPW45_seq_lib:572 IPW63.randomize() with (interference)");
begin
bit[2:0]z320=3'd2;
bit[23:0]z327=24'h00_1234;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
IPW63.z84[39:16]==z327;
IPW63.z88==z320;
}==1,"IPW63.randomize() with (interference) failed");
chk(IPW63.z84[39:16]==z327,"aIPW61 page mismatch");
chk(IPW63.z88==z320,"IPW51 mismatch");
end
z219();
endtask
task automatic z400();
z218(70,"IPW45_seq_lib:592 IPW63.randomize() (lIPW60 xfer)");
begin
z82 IPW63=z373();
chk(IPW63.randomize()==1,"IPW63.randomize() failed");
IPW63.z50=1;
chk(IPW63.z88<=IPW63.z56,"IPW51 > max");
end
z219();
endtask
task automatic z401();
z218(71,"IPW45_seq_lib:671 IPW63.randomize() with (IPW42 IPW35)");
begin
bit[39:0]z367=40'h0000_0000_1000;
bit[12:0]z368=13'd63;
bit z371=0;
z82 IPW63=z373();
chk(IPW63.randomize()with
{
IPW63.z46==0;
IPW63.z83==0;
IPW63.z84==z367;
IPW63.z85==z368;
IPW63.z86==z371;
IPW63.z88==3'd2;
IPW63.z50==0;
}==1,"IPW63.randomize() with (IPW42 IPW35) failed");
chk(IPW63.z84==z367,"IPW43 mismatch");
chk(IPW63.z85==z368,"IPW58 mismatch");
end
z219();
endtask
task automatic z402();
z218(72,"IPW45_seq_lib:700 IPW63.randomize() (IPW42 lIPW60 xfer)");
begin
z82 IPW63=z373();
chk(IPW63.randomize()==1,"IPW63.randomize() failed");
IPW63.z50=1;
chk(IPW63.z88<=IPW63.z56,"IPW51 > max");
end
z219();
endtask
function automatic z95 z403();
z95 IPW63=new;
IPW63.z56=z226.z221;
return IPW63;
endfunction
task automatic z404();
z218(73,"IPW59_seq_lib:57 std::randomize(IPW51_to_use)");
begin
bit[3:0]z320;
chk(std::randomize(z320)with{z320<=z226.z221;}==1,
"std::randomize IPW51_to_use failed");
chk(z320<=z226.z221,"IPW51_to_use > max_IPW51");
end
z219();
endtask
task automatic z405();
z218(74,"IPW59_seq_lib:63 std::randomize(actual_IPW41_pattern,trigger_IPW48,...)");
begin
bit[2:0]z241=3'd2;
bit[2:0]z322,z323;
bit z324;
bit[4:0]z325;
z11 z326;
bit[23:0]z327;
z322=3'b101;
chk(std::randomize(z323,z324,z325,z326,z327)with{
z324 dist{0:=97,1:=3};
(z324==1)->(z323==0);
foreach(z322[i]){
if(z322[i]){
z323[i]dist{0:=98,1:=2};
}else{
z323[i]==0;
}
}
z240[z241].z13[z323].size()!=0;
z325<=(z240[z241].z13[z323].size()-1);
if(z324){
z326.z9==z240[z241].z12;
z326.z10==24'hFF_FFFF;
}else{
z326==z240[z241].z13[z323][z325];
}
z326.z9[23:16]==0;
z327 inside{[z326.z9:z326.z10]};
z327[23:16]==0;
}==1,"std::randomize IPW59 IPW41 pattern failed");
chk(z327[23:16]==0,"IPW28 [23:16] != 0");
end
z219();
endtask
task automatic z406();
z218(75,"IPW59_seq_lib:91 std::randomize(gen_b2b_IPW63s,gen_IPW63s_in_single_page,num_IPW63s,...)");
begin
bit z407,z408;
int z409;
bit[39:0]z340;
int z334;
bit[23:0]z327=24'h00_1234;
chk(std::randomize(z407,z408,z409,
z340,z334)with{
$countones({z407,z408})inside{0,1};
z407 dist{0:/49,1:/1};
z408 dist{0:/60,1:/40};
z409 inside{[10:50]};
z340[39:16]==z327;
z334 inside{[11:15]};
}==1,"std::randomize IPW59 seq shape failed");
chk(z409 inside{[10:50]},"num_IPW63s out of range");
chk(z340[39:16]==z327,"aIPW61_base page mismatch");
end
z219();
endtask
task automatic z410();
z218(76,"IPW59_seq_lib:110 IPW63.randomize() with (b2b)");
begin
bit[3:0]z320=4'd2;
bit[23:0]z327=24'h00_1234;
z95 IPW63=z403();
chk(IPW63.randomize()with
{
IPW63.z46==0;
IPW63.z84[31:16]==z327[15:0];
IPW63.z88==z320;
}==1,"IPW63.randomize() with (b2b) failed");
chk(IPW63.z84[31:16]==z327[15:0],"aIPW61 page mismatch");
chk(IPW63.z88==z320,"IPW51 mismatch");
end
z219();
endtask
task automatic z411();
z218(77,"IPW59_seq_lib:130 IPW63.randomize() with (single page)");
begin
bit[3:0]z320=4'd2;
bit[23:0]z327=24'h00_1234;
bit[39:0]z340=40'h0000_1234_0000;
bit[39:0]z356=40'hFF_FFFF_FFFF<<4'd12;
z95 IPW63=z403();
chk(IPW63.randomize()with
{
({8'd0,IPW63.z84,8'd0}&z356)==(z340&z356);
IPW63.z84[31:16]==z327[15:0];
IPW63.z88==z320;
}==1,"IPW63.randomize() with (single page) failed");
chk(IPW63.z84[31:16]==z327[15:0],"aIPW61 page mismatch");
end
z219();
endtask
task automatic z412();
z218(78,"IPW59_seq_lib:149 IPW63.randomize() with (deIPW41)");
begin
bit[3:0]z320=4'd2;
bit[23:0]z327=24'h00_1234;
z95 IPW63=z403();
chk(IPW63.randomize()with
{
IPW63.z84[31:16]==z327[15:0];
IPW63.z88==z320;
}==1,"IPW63.randomize() with (deIPW41) failed");
chk(IPW63.z84[31:16]==z327[15:0],"aIPW61 page mismatch");
chk(IPW63.z88==z320,"IPW51 mismatch");
end
z219();
endtask
task automatic z413();
z218(79,"IPW59_seq_lib:173 std::randomize(IPW51_to_use) (interference)");
begin
bit[3:0]z320;
chk(std::randomize(z320)with{z320<=z226.z221;}==1,
"std::randomize IPW51_to_use (interf) failed");
chk(z320<=z226.z221,"IPW51_to_use > max_IPW51");
end
z219();
endtask
task automatic z414();
z218(80,"IPW59_seq_lib:178 std::randomize(actual_IPW41_pattern,IPW41_region_idx,...)");
begin
bit[2:0]z241=3'd2;
bit[2:0]z323;
bit[4:0]z325;
z11 z326;
bit[23:0]z327;
chk(std::randomize(z323,z325,z326,z327)with{
z323==0;
z325<=(z240[z241].z13[z323].size()-1);
z326==z240[z241].z13[z323][z325];
z326.z9[23:16]==0;
z327 inside{[z326.z9:z326.z10]};
z327[23:16]==0;
}==1,"std::randomize IPW59 interf IPW41 region failed");
chk(z323==0,"actual_IPW41_pattern != 0");
chk(z327[23:16]==0,"IPW28 [23:16] != 0");
end
z219();
endtask
task automatic z415();
z218(81,"IPW59_seq_lib:190 IPW63.randomize() with (interference)");
begin
bit[3:0]z320=4'd2;
bit[23:0]z327=24'h00_1234;
z95 IPW63=z403();
chk(IPW63.randomize()with
{
IPW63.z84[31:16]==z327[15:0];
IPW63.z88==z320;
}==1,"IPW63.randomize() with (interference) failed");
chk(IPW63.z88==z320,"IPW51 mismatch");
end
z219();
endtask
task automatic z416();
z218(82,"IPW59_seq_lib:205 void'(IPW63.randomize()) (lIPW60 xfer)");
begin
z95 IPW63=z403();
void'(IPW63.randomize());
IPW63.z50=1;
chk(IPW63.z88<=IPW63.z56,"IPW51 > max");
end
z219();
endtask
task automatic z417();
z218(83,"IPW59_seq_lib:274 IPW63.randomize() with (IPW42 IPW35)");
begin
bit[39:0]z367=40'h0000_0000_1000;
z95 IPW63=z403();
chk(IPW63.randomize()with
{
IPW63.z46==0;
IPW63.z84[31:8]==z367[31:8];
IPW63.z88==4'd2;
}==1,"IPW63.randomize() with (IPW42 IPW35) failed");
chk(IPW63.z84[31:8]==z367[31:8],"IPW43 mismatch");
end
z219();
endtask
task automatic z418();
z218(84,"IPW59_seq_lib:296 IPW63.randomize() (IPW42 lIPW60 xfer)");
begin
z95 IPW63=z403();
chk(IPW63.randomize()==1,"IPW63.randomize() failed");
IPW63.z50=1;
chk(IPW63.z88<=IPW63.z56,"IPW51 > max");
end
z219();
endtask
task automatic z419();
z218(85,"IPW44_driver:98 std::randomize(slow_wresp_IPW56)");
begin
bit z420;
chk(std::randomize(z420)with{
z420 dist{0:=2,1:=3};
}==1,"std::randomize slow_wresp_IPW56 failed");
chk(z420 inside{0,1},"slow_wresp_IPW56 not 0/1");
end
z219();
endtask
task automatic z421();
z218(86,"IPW44_driver:288 std::randomize(curr_wresp_cdt_delay,slow_wresp_IPW56)");
begin
bit[7:0]z422;
bit z420=0;
bit z112=0,z423=0,z424=0;
chk(std::randomize(z422,z420)with{
(z112&!z423)->(z422==0);
z424->(z420==0);
if(!z424&z423){
z422 dist{[16:63]:/1,[64:127]:/4,[128:255]:/5};
}else if(z420){
z422 dist{[0:8]:/1,[9:63]:/3,[64:255]:/7};
}else{
z422 dist{[0:8]:/7,[9:15]:/3,[16:63]:/1};
}
}==1,"std::randomize wresp delay failed");
chk(std::randomize(z422,z420)with{
(z112&!z423)->(z422==0);
z424->(z420==0);
if(!z424&z423){
z422 dist{[16:63]:/1,[64:127]:/4,[128:255]:/5};
}else if(z420){
z422 dist{[0:8]:/1,[9:63]:/3,[64:255]:/7};
}else{
z422 dist{[0:8]:/7,[9:15]:/3,[16:63]:/1};
}
}==1,"std::randomize wresp delay (2nd) failed");
end
z219();
endtask
task automatic z425();
z218(87,"IPW44_driver:355 std::randomize(curr_rdata_cdt_delay)");
begin
bit[7:0]z426;
bit z112=0,z427=0;
chk(std::randomize(z426)with{
(z112&!z427)->(z426==0);
if(z166){
z426 dist{[0:3]:/1,[4:15]:/7,[16:63]:/1,[500:1000]:/1};
}else{
if(z427){
z426 dist{[16:63]:/1,[63:127]:/9};
}else{
z426 dist{[0:3]:/95,[4:15]:/4,[16:63]:/1};
}
}
}==1,"std::randomize rdata delay failed");
z166=1;
chk(std::randomize(z426)with{
(z112&!z427)->(z426==0);
if(z166){
z426 dist{[0:3]:/1,[4:15]:/7,[16:63]:/1,[500:1000]:/1};
}else{
if(z427){
z426 dist{[16:63]:/1,[63:127]:/9};
}else{
z426 dist{[0:3]:/95,[4:15]:/4,[16:63]:/1};
}
}
}==1,"std::randomize rdata delay (cov_tweaks) failed");
z166=0;
end
z219();
endtask
task automatic z428();
z218(88,"IPW45_driver:77 std::randomize(IPW63_delay,IPW63_aIPW61,...,IPW63_id) in new()");
begin
logic[31:0]z46;
logic[39:0]z84;
logic[12:0]z85;
logic z86;
logic z87;
logic[2:0]z88;
int z429;
chk(std::randomize(z46,z84,z85,z86,z87,z88,z429)==1,
"std::randomize IPW45_IPW63_info fields failed");
end
z219();
endtask
task automatic z430();
z218(89,"IPW45_driver:129 std::randomize(fIPW60_reads,fIPW60_writes)");
begin
bit z431,z432;
chk(std::randomize(z431,z432)with{
z431 dist{0:/2,1:/8};
z432 dist{0:/2,1:/8};
}==1,"std::randomize fIPW60_reads/writes failed");
chk(z431 inside{0,1}&&z432 inside{0,1},"fIPW60_rw not 0/1");
end
z219();
endtask
task automatic z433();
z218(90,"IPW45_driver:247 std::randomize(num_cyc,num_cdts_this_cyc)");
begin
int z434,z435;
int z436=64;
bit z437=0;
chk(std::randomize(z434,z435)with{
z434>=(z436/`z38);
if(z166){
z434<=16'd2000;
}else{
z434<=(z436<<1);
}
if(z437){
if((z436%`z38)==0){
z434==(z436/`z38);
}else{
z434==((z436/`z38)+1);
}
}
}==1,"std::randomize num_cyc failed");
chk(z434>=(z436/`z38),"num_cyc too small");
end
z219();
endtask
typedef struct packed{
logic[64*`z39-1:0]wdata;
logic[2:0]wid;
}z438;
task automatic z439();
z218(91,"IPW45_driver:440 std::randomize(nxt_cyc_wpkt,tmp_wIPW63_wid)");
begin
z438 z440;
logic[2:0]z441;
chk(std::randomize(z440,z441)==1,
"std::randomize nxt_cyc_wpkt failed");
end
z219();
endtask
task automatic z442();
z218(92,"IPW45_driver:446 std::randomize(tmp_wdata)");
begin
logic[64*`z39-1:0]z443;
chk(std::randomize(z443)==1,"std::randomize tmp_wdata failed");
end
z219();
endtask
task automatic z444();
z218(93,"IPW45_driver:474 std::randomize(this_wdata_beat_num_beats)");
begin
int z445;
bit z432=0;
int z446=8;
chk(std::randomize(z445)with{
if(z432==1){
z445==z446;
}else{
z445 dist{0:/10,[1:z446]:/90};
}
}==1,"std::randomize beat_num_beats failed");
chk(z445 inside{[0:z446]},"beats out of range");
z432=1;
chk(std::randomize(z445)with{
if(z432==1){
z445==z446;
}else{
z445 dist{0:/10,[1:z446]:/90};
}
}==1,"std::randomize beat_num_beats (fIPW60) failed");
chk(z445==z446,"fIPW60 writes beats != limit");
end
z219();
endtask
task automatic z447();
z218(94,"IPW45_driver:547 std::randomize(tmp_wdata)");
begin
logic[64*`z39-1:0]z443;
chk(std::randomize(z443)==1,"std::randomize tmp_wdata (2nd) failed");
end
z219();
endtask
task automatic z448();
z218(95,"IPW59_driver:111 std::randomize(rdata_cdt_ret_dly)");
begin
int z449;
bit z450=0;
chk(std::randomize(z449)with{
if(z166){
z449 dist{[3:8]:/8,[9:31]:/1,[1000:2000]:/1};
}else if(z450){
z449 dist{[32:127]:/1,[128:512]:/9};
}else{
z449 dist{[3:8]:/8,[9:31]:/2};
}
}==1,"std::randomize rdata_cdt_ret_dly failed");
chk(z449>=3,"rdata_cdt_ret_dly < 3");
end
z219();
endtask
task automatic z451();
z218(96,"IPW46_driver:313 std::randomize(cfg_IPW63_cdt_retdelay)");
begin
bit[11:0]z452;
chk(std::randomize(z452)with{
if(z166){
z452 dist{[4:16]:/1,[17:48]:/8,[1000:2000]:/1};
}else{
z452 dist{[4:16]:/2,[17:48]:/8};
}
}==1,"std::randomize cfg_IPW63_cdt_retdelay failed");
chk(z452>=4,"cfg_IPW63_cdt_retdelay < 4");
end
z219();
endtask
task automatic z453();
z218(97,"IPW46_driver:336 std::randomize(cfg_rdata_retdelay,cfg_rdata_ret)");
begin
bit[11:0]z454;
bit[31:0]z455;
chk(std::randomize(z454,z455)with{
if(z166){
z454 dist{[4:16]:/1,[17:48]:/8,[1000:2000]:/1};
}
else{z454 dist{[4:16]:/2,[17:48]:/8};}
}==1,"std::randomize cfg_rdata_retdelay failed");
chk(z454>=4,"cfg_rdata_retdelay < 4");
end
z219();
endtask
task automatic z456();
z218(98,"IPW46_driver:448 std::randomize(wcdt_dly)");
begin
u11_t z457;
bit z458=0;
int z459=24;
bit z460=0;
chk(std::randomize(z457)with{
if(z458){
z457==z459;
}else if(z460){
z457==8;
}else{
z457 dist{
[16:32]:/4,
[33:64]:/1
};
}
}==1,"std::randomize wcdt_dly failed");
chk(z457 inside{[16:64]},"wcdt_dly out of range");
end
z219();
endtask
task automatic z461();
z218(99,"IPW46_driver:479 std::randomize(num_cyc) [4:32]");
begin
bit[15:0]z434;
chk(std::randomize(z434)with{z434 inside{[4:32]};}==1,
"std::randomize num_cyc (wcdt) failed");
chk(z434 inside{[4:32]},"num_cyc out of range");
end
z219();
endtask
task automatic z462();
z218(100,"IPW46_driver:488 std::randomize(num_cyc,ncdts_this_cyc)");
begin
bit[15:0]z434;
bit[1:0]z463[];
bit[6:0]z464=7'd10;
bit z460=0;
chk(std::randomize(z434,z463)with{
z434>(z464>>1);
z434<(z464<<1);
z460->(z434<=z464);
z463.size()==z434;
foreach(z463[i]){
z463[i]inside{[0:2]};
}
z463.sum()with(16'(item))==z464;
}==1,"std::randomize num_cyc,ncdts_this_cyc failed");
chk(z463.size()==z434,"ncdts size mismatch");
chk(z463.sum()with(16'(item))==z464,"ncdts sum mismatch");
end
z219();
endtask
task automatic z465();
z218(101,"IPW46_driver:651 std::randomize(nxt_arb_dly)");
begin
int z466;
int z467=40;
chk(std::randomize(z466)with{
if(z467<16){
(z466==0);
}else if(z467<32){
z466 inside{[1:63]};
}else if(z467<96){
z466 inside{[64:127]};
}else{
z466 inside{[128:1023]};
}
}==1,"std::randomize nxt_arb_dly failed");
chk(z466 inside{[64:127]},"nxt_arb_dly out of branch range");
end
z219();
endtask
task automatic z468();
z218(102,"IPW46_driver:689 std::randomize(curr_line)");
begin
logic[8*128-1:0]z469;
chk(std::randomize(z469)==1,"std::randomize curr_line failed");
end
z219();
endtask
task automatic z470();
z218(103,"IPW46_driver:801 std::randomize(sim_refresh_resp,sim_refresh_data,nxt_resp_dly,resp2rd_dly)");
begin
bit z471,z472;
int z473,z474;
bit fresp=0;
bit z460=0;
bit[3:0]z475=4'd3,z476=4'd5;
chk(std::randomize(z471,z472,z473,z474)with{
z471 dist{0:=999,1:=1};
z472 dist{0:=999,1:=1};
fresp->(z471==0);
fresp->(z472==0);
if(fresp){
z473 inside{[16:32]};
z474 inside{[32:64]};
}else if(z460){
z473==0;
z474==16;
}else if(z471){
z473 inside{[512:2047]};
z474 inside{[32:64]};
}else if(z472){
z473 inside{[32:64]};
z474 inside{[512:2047]};
}else if(z476==z475){
z473 dist{0:/9,[1:15]:/1};
z474 dist{16:/9,[17:32]:/1};
}else{
z473 dist{[2:15]:/8,[15:63]:/2};
z474 dist{[18:127]:/8,[128:511]:/1};
}
}==1,"std::randomize resp dlys failed");
chk(z473>=0&&z474>=0,"negative delay");
end
z219();
endtask
task automatic z477();
z218(104,"IPW46_driver:892 std::randomize(num_cyc,nrb_this_cyc,rbeat_this_cyc)");
begin
int z434;
bit[1:0]z478[];
bit[64:0]z479[];
int z480=12;
bit z460=0;
bit z481=0;
chk(std::randomize(z434,z478,z479)with{
if(z460){
if(!z481){
z434==(((z480)>>1)+z480[0]);
}else{
z434==z480;
}
}else{
z434>=(z480>>1);
z434<=(z480<<1);
}
z478.size()==z434;
z479.size()==z434;
foreach(z478[i]){
if(z481){
z478[i]dist{0:=1,1:=75};
}else{
z478[i]inside{[0:2]};
}
}
z478.sum()with(16'(item))==z480;
}==1,"std::randomize rdata sched failed");
chk(z478.size()==z434,"nrb size mismatch");
chk(z478.sum()with(16'(item))==z480,"nrb sum mismatch");
end
z219();
endtask
task automatic z482();
z218(105,"IPW46_driver:956 std::randomize(num_cyc) (blank IPW33)");
begin
int z434;
bit z460=0;
chk(std::randomize(z434)with{
if(z460){
z434==1;
}else{
z434 inside{[16:64]};
}
}==1,"std::randomize num_cyc (blank) failed");
chk(z434 inside{[16:64]},"num_cyc out of range");
end
z219();
endtask
task automatic z483();
z218(106,"IPW16_mem_bfms_pkg:103 std::randomize(dly) [0:max_val]");
begin
int dly;
int z484=10;
chk(std::randomize(dly)with{dly inside{[0:z484]};}==1,
"std::randomize dly [0:max] failed");
chk(dly inside{[0:z484]},"dly out of range");
end
z219();
endtask
task automatic z485();
z218(107,"IPW16_mem_bfms_pkg:105 std::randomize(dly) [16:max_val]");
begin
int dly;
int z484=200;
chk(std::randomize(dly)with{dly inside{[16:z484]};}==1,
"std::randomize dly [16:max] failed");
chk(dly inside{[16:z484]},"dly out of range");
end
z219();
endtask
task automatic z486();
z218(108,"IPW16_mem_bfms_pkg:112 std::randomize(rand_pick) [0:19]");
begin
int z487;
chk(std::randomize(z487)with{z487 inside{[0:19]};}==1,
"std::randomize rand_pick failed");
chk(z487 inside{[0:19]},"rand_pick out of range");
end
z219();
endtask
task automatic z488();
z218(109,"IPW16_mem_bfms_pkg:119 std::randomize(dly) [127:max_dly]");
begin
int dly;
int z489=5000;
chk(std::randomize(dly)with{dly inside{[127:z489]};}==1,
"std::randomize dly [127:max_dly] failed");
chk(dly inside{[127:z489]},"dly out of range");
end
z219();
endtask
task automatic z490();
z218(110,"IPW16_mem_bfms_pkg:121 std::randomize(dly) [16:127]");
begin
int dly;
chk(std::randomize(dly)with{dly inside{[16:127]};}==1,
"std::randomize dly [16:127] failed");
chk(dly inside{[16:127]},"dly out of range");
end
z219();
endtask
task automatic z491();
z218(111,"IPW16_mem_bfms_pkg:125 std::randomize(dly) [16:127] (base)");
begin
int dly;
chk(std::randomize(dly)with{dly inside{[16:127]};}==1,
"std::randomize dly [16:127] (base) failed");
chk(dly inside{[16:127]},"dly out of range");
end
z219();
endtask
task automatic z492();
z218(112,"IPW16_mem_bfms_pkg:300 std::randomize(rand_val) [0:refresh_interval]");
begin
int z493;
int z494=1000;
chk(std::randomize(z493)with{z493 inside{[0:z494]};}==1,
"std::randomize rand_val failed");
chk(z493 inside{[0:z494]},"rand_val out of range");
end
z219();
endtask
task automatic z495();
z218(113,"IPW16_mem_bfms_pkg:303 std::randomize(refresh_dur) [1:max_dur]");
begin
int z496;
int z497=64;
chk(std::randomize(z496)with{z496 inside{[1:z497]};}==1,
"std::randomize refresh_dur failed");
chk(z496 inside{[1:z497]},"refresh_dur out of range");
end
z219();
endtask
task automatic z498();
z218(114,"IPW16_mem_bfms_pkg:753 std::randomize(cdt_return_IPW33)");
begin
bit[9:0]z499=10'd20;
bit z500[];
chk(std::randomize(z500)with{
z500.size()>=z499;
z500.size()<=z499<<1;
z500.sum()with(10'(item))==z499;
}==1,"std::randomize cdt_return_IPW33 failed");
chk(z500.sum()with(10'(item))==z499,"cdt sum mismatch");
end
z219();
endtask
task automatic z501();
z218(115,"IPW16_mem_bfms_pkg:1021 std::randomize(trbr)");
begin
bit[255:0]trbr;
chk(std::randomize(trbr)==1,"std::randomize trbr failed");
end
z219();
endtask
task automatic z502();
z218(116,"IPW16_mem_bfms_pkg:1456 std::randomize(num_cyc,num_cdts_this_cyc)");
begin
int z434;
bit[15:0]z435[];
int z436=48;
chk(std::randomize(z434,z435)with{
z434>=(z436/`z39);
z434<=(z436<<1);
z435.size()==z434;
foreach(z435[i]){
z435[i]inside{[0:`z39]};
}
z435.sum()with(16'(item))==z436;
}==1,"std::randomize num_cyc,num_cdts_this_cyc (IPW45) failed");
chk(z435.sum()with(16'(item))==z436,"cdt sum mismatch");
end
z219();
endtask
task automatic z503();
z218(117,"IPW16_mem_bfms_pkg:1473 std::randomize(num_blank_cdt_IPW33)");
begin
int z504;
chk(std::randomize(z504)with{z504>=4;}==1,
"std::randomize num_blank_cdt_IPW33 failed");
chk(z504>=4,"num_blank_cdt_IPW33 < 4");
end
z219();
endtask
task automatic z505();
z218(118,"IPW16_mem_bfms_pkg:1588 std::randomize(num_cyc,num_rdata_this_cyc)");
begin
int z434;
bit[15:0]z506[];
int z507=64;
chk(std::randomize(z434,z506)with{
z434>=(z507/`z38);
z434<=(z507<<1);
z506.size()==z434;
foreach(z506[i]){
z506[i]inside{[0:`z38]};
}
z506.sum()with(16'(item))==z507;
}==1,"std::randomize num_cyc,num_rdata_this_cyc failed");
chk(z506.sum()with(16'(item))==z507,"rdata sum mismatch");
end
z219();
endtask
task automatic z508();
z218(119,"IPW16_mem_bfms_pkg:1786 std::randomize(IPW59_rdata_delay) [0:3]");
begin
int z509;
chk(std::randomize(z509)with{z509 inside{[0:3]};}==1,
"std::randomize IPW59_rdata_delay failed");
chk(z509 inside{[0:3]},"IPW59_rdata_delay out of range");
end
z219();
endtask
task automatic z510();
z218(120,"fc_IPW47_bfm:1249 std::randomize(tmp_mem_line) (rd path)");
begin
logic[255:0]z511;
bit z512=0;
chk(std::randomize(z511)with{
z512->(z511==0);
}==1,"std::randomize tmp_mem_line (rd) failed");
z512=1;
chk(std::randomize(z511)with{
z512->(z511==0);
}==1,"std::randomize tmp_mem_line (rd, x0) failed");
chk(z511==0,"treat_x_as_0 but line != 0");
end
z219();
endtask
task automatic z513();
z218(121,"fc_IPW47_bfm:1316 std::randomize(tmp_mem_line) (wr path)");
begin
logic[255:0]z511;
bit z512=0;
chk(std::randomize(z511)with{
z512->(z511==0);
}==1,"std::randomize tmp_mem_line (wr) failed");
end
z219();
endtask
task automatic z514();
z218(122,"fc_IPW47_bfm:1369 std::randomize(cts_tp)");
begin
bit[15:0]cts_tp[];
int z515=1000;
int z516,z517;
z516=z515;
z517=1000000/z516;
chk(std::randomize(cts_tp)with{
cts_tp.size()==16;
foreach(cts_tp[i]){
cts_tp[i]>=0.9*z517;
cts_tp[i]<=1.1*z517;
}
}==1,"std::randomize cts_tp failed");
chk(cts_tp.size()==16,"cts_tp size != 16");
foreach(cts_tp[i])begin
chk(cts_tp[i]>=0.9*z517&&cts_tp[i]<=1.1*z517,"cts_tp out of range");
end
end
z219();
endtask
task automatic z518();
z218(123,"IPW17_cfg_seq:96 std::randomize(trigger_cfg_reads,num_cfg_reads,...)");
begin
bit z519;
int z520;
bit[39:0]z521[];
chk(std::randomize(z519,z520,z521)with{
if(z239.size()==0){
z519==0;
}else{
z519 dist{0:=9,1:=1};
}
z520>=0;
z520<=15;
z520<=z239.size();
z521.size()==z520;
foreach(z521[j]){
z521[j]inside{z239};
}
}==1,"std::randomize mss cfg reads failed");
chk(z520>=0&&z520<=15,"num_cfg_reads out of range");
chk(z521.size()==z520,"intermediate size mismatch");
foreach(z521[j])begin
chk(z521[j]inside{z239},
"intermediate aIPW61 not in written set");
end
end
z219();
endtask
initial begin
$display("=== tb_all_randomize : ALL IPW03 randomize sites ===");
z250();
z251();
z252();
z254();
z255();
z262();
z266();z270();
z271();z275();
z276();z277();
z278();z281();
z288();z289();
z290();z295();
z297();z299();
z301();z302();
z308();z310();
z315();z317();
z318();z319();
z321();z329();
z344();z345();
z346();z348();
z349();z355();
z357();z359();
z360();z361();
z362();z363();
z364();z365();
z372();
z374();z375();
z377();z378();
z379();z380();
z381();z382();
z383();z384();
z385();z386();
z387();z388();
z389();z390();
z391();z392();
z393();z394();
z395();z396();
z397();z398();
z399();z400();
z401();z402();
z404();z405();
z406();z410();
z411();z412();
z413();z414();
z415();z416();
z417();z418();
z419();z421();
z425();
z428();z430();
z433();z439();
z442();z444();
z447();z448();
z451();z453();
z456();z461();
z462();z465();
z468();z470();
z477();z482();
z483();z485();
z486();z488();
z490();z491();
z492();z495();
z498();z501();
z502();z503();
z505();z508();
z510();z513();
z514();
z518();
$display("=== tb_all_randomize : all %0d sites executed ===",123);
`z5
$finish;
end
endmodule:tb_all_randomize
