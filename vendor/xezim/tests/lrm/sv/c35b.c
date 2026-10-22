#include <stdio.h>
#include <string.h>
#include "svdpi.h"
extern void sv_out(int a, int *o, const char *s);
extern int sv_where(void);
unsigned int u32(unsigned int a) { return a + 1; }
short s16(short a) { return a - 1; }
unsigned char b8(unsigned char a) { return a ^ 0xff; }
void pstruct(const svBitVecVal *in, svBitVecVal *out) { out[0] = in[0] + 0x01010101; }   /* packed struct of 4 bytes */
int arr2d(const svOpenArrayHandle h) { int s = 0; for (int i = svLow(h,1); i <= svHigh(h,1); i++) for (int j = svLow(h,2); j <= svHigh(h,2); j++) s += *(int*)svGetArrElemPtr2(h, i, j); return s * 10 + svDimensions(h); }
int open_bits(const svOpenArrayHandle h) { svBitVecVal v; int s = 0; for (int i = svLow(h,1); i <= svHigh(h,1); i++) { svGetBitArrElem1VecVal(&v, h, i); s += v; } return s; }
void open_logic(const svOpenArrayHandle h) { svLogicVecVal v; for (int i = svLow(h,1); i <= svHigh(h,1); i++) { svGetLogicArrElem1VecVal(&v, h, i); printf("C|ol[%d] a=%x b=%x\n", i, v.aval, v.bval); } fflush(stdout); }
int open_size(const svOpenArrayHandle h) { return svSize(h, 1) * 100 + svLeft(h, 1) * 10 + svRight(h, 1); }
int call_out(void) { int o = 0; sv_out(5, &o, "hi"); return o; }
int where(void) { return sv_where(); }
void dyn_out(const svOpenArrayHandle h) { for (int i = svLow(h,1); i <= svHigh(h,1); i++) *(int*)svGetArrElemPtr1(h, i) = 100 + i; }
int scal_logic(svLogic l) { return l; }
