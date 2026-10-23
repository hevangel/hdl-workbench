/* IEEE 1800-2023 §35.5.6 / Annex H: packed structs and unions as svBitVecVal /
 * svLogicVecVal vectors, multi-dimensional open arrays, and output / inout
 * formals of exported subroutines. Pairs with packed_aggregate_dpi_test.sv. */
#include "svdpi.h"

extern void sv_out(int a, int *o, const char *s);
extern void sv_o64(int a, long long *o);
extern void sv_oreal(double *r);
extern void sv_inout(int *v);
extern int sv_tout(int *o);

void pstruct(const svBitVecVal *in, svBitVecVal *out) { out[0] = in[0] + 0x01010101; }
int pstruct_in(const svBitVecVal *in) { return (int)in[0]; }
void ps_io(svBitVecVal *s) { s[0] = s[0] + 0x10101010; }
void ps_out(svBitVecVal *s) { s[0] = 0xa1b2c3d4; }
int pl_in(const svLogicVecVal *s) { return (int)(s[0].aval & 0xff) * 1000 + (int)(s[0].bval & 0xff); }
void pl_out(svLogicVecVal *s) { s[0].aval = 0x5a; s[0].bval = 0x0f; }
void pw_inc(const svBitVecVal *i, svBitVecVal *o) { o[0] = i[0] + 1; o[1] = i[1] + 1; }
int pu_in(const svBitVecVal *u) { return (int)u[0]; }
void pu_out(svBitVecVal *u) { u[0] = 0xcafe; }
int en_in(int e) { return e; }

int arr2d(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++)
        for (int j = svLow(h, 2); j <= svHigh(h, 2); j++)
            s += *(int *)svGetArrElemPtr2(h, i, j) * (i + 1);
    return s * 10 + svDimensions(h);
}
int arr2d_plain(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++)
        for (int j = svLow(h, 2); j <= svHigh(h, 2); j++)
            s += *(int *)svGetArrElemPtr2(h, i, j);
    return s * 10 + svDimensions(h);
}
int arr2d_fixed(const int *a) {
    int s = 0;
    for (int k = 0; k < 6; k++) s = s * 2 + a[k];
    return s;
}
int arr3d(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = 0; i < 2; i++)
        for (int j = 0; j < 2; j++)
            for (int k = 0; k < 2; k++) s = s * 3 + *(char *)svGetArrElemPtr3(h, i, j, k);
    return s * 10 + svSize(h, 3);
}
void arr2d_out(const svOpenArrayHandle h) {
    for (int i = 0; i < 2; i++)
        for (int j = 0; j < 2; j++) *(int *)svGetArrElemPtr2(h, i, j) = i * 10 + j + 1;
}

int call_out(void) { int o = 0; sv_out(5, &o, "hi"); return o; }
int call_tout(int *r) { int t = 0; sv_tout(&t); *r = t; return 0; }
int call_outs(void) {
    long long o = 0; double r = 0; int v = 20;
    sv_o64(5, &o); sv_oreal(&r); sv_inout(&v);
    return (int)(o >> 32) * 100000 + (int)(o & 0xffff) * 1000 + (int)(r * 10) * 10 + v;
}
