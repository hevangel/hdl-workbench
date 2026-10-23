/* DPI audit, family C: canonical C elements of unpacked-array formals
 * (IEEE 1800 §35.5.6.1, Annex H.7 / H.12): an enum is its base type, a
 * packed struct and `integer` / `time` svLogicVecVal words, an implicitly
 * typed `[7:0]` a logic vector; and an assignment-pattern actual passes
 * every element. */
#include "svdpi.h"

static int rec_v[16];
static int rec_n;
int ael_get(int i) { return rec_v[i]; }
static void rec(int v) { rec_v[rec_n++] = v; }

int ael_enum(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s = s * 10 + *(int *)svGetArrElemPtr1(h, i);
    return s;
}
void ael_vec(const svOpenArrayHandle h) {
    rec_n = 0;
    svLogicVecVal v;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) {
        svGetLogicArrElemVecVal(&v, h, i);
        rec((int)v.aval); rec((int)v.bval);
    }
}
void ael_fixed_vec(const svLogicVecVal *a) {
    rec_n = 0;
    for (int i = 0; i < 2; i++) { rec((int)a[i].aval); rec((int)a[i].bval); }
}
void ael_time_out(svLogicVecVal *t) {
    t[0].aval = 5; t[0].bval = 1; t[1].aval = 0; t[1].bval = 0;
    t[2].aval = 0xFFFFFFFF; t[2].bval = 0; t[3].aval = 0; t[3].bval = 0;
}
int ael_ints(const int *a) { return a[0] * 10 + a[1]; }
int ael_open(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s = s * 10 + *(int *)svGetArrElemPtr1(h, i);
    return s * 100 + svSize(h, 1);
}
int ael_2d(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++)
        for (int j = svLow(h, 2); j <= svHigh(h, 2); j++) s = s * 10 + *(int *)svGetArrElemPtr2(h, i, j);
    return s * 100 + svSize(h, 1) * 10 + svSize(h, 2);
}
