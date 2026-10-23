#include "svdpi.h"
extern void sv_out(int a, int *o, const char *s);
void pstruct(const svBitVecVal *in, svBitVecVal *out) { out[0] = in[0] + 0x01010101; }
int pstruct_in(const svBitVecVal *in) { return (int)in[0]; }
int arr2d(const svOpenArrayHandle h) { int s = 0; for (int i = svLow(h,1); i <= svHigh(h,1); i++) for (int j = svLow(h,2); j <= svHigh(h,2); j++) s += *(int*)svGetArrElemPtr2(h, i, j); return s * 10 + svDimensions(h); }
int call_out(void) { int o = 0; sv_out(5, &o, "hi"); return o; }
