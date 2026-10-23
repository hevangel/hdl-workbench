/* DPI audit, family A: unpacked arrays of scalar bit / logic. Each element
 * is one svBit / svLogic byte (IEEE 1800 §35.5.6.1, Annex H.7.4 / H.12),
 * for a fixed-size formal (a plain C array) and an open one (a handle). */
#include "svdpi.h"

static int rec_v[16];
static int rec_n;
int asa_get(int i) { return rec_v[i]; }
static void rec(int v) { rec_v[rec_n++] = v; }

/* logic a[4] / bit b[3] / output logic o[3] */
void asa_fixed(const svLogic *a, const svBit *b, svLogic *o) {
    rec_n = 0;
    for (int i = 0; i < 4; i++) rec(a[i]);
    for (int i = 0; i < 3; i++) rec(b[i]);
    o[0] = sv_x; o[1] = sv_1; o[2] = sv_z;
}

/* logic a[] through element pointers and the whole-array pointer */
void asa_open(const svOpenArrayHandle h) {
    rec_n = 0;
    rec(svDimensions(h));
    rec(svSizeOfArray(h));
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) rec(*(const svLogic *)svGetArrElemPtr1(h, i));
    rec(((const svLogic *)svGetArrayPtr(h))[0]);
}

/* bit a[] read through the accessor, logic a[] written through it */
int asa_bits(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s = s * 10 + svGetBitArrElem1(h, i);
    return s;
}
void asa_put(const svOpenArrayHandle h) {
    int k = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) svPutLogicArrElem1(h, (svLogic)(k++ & 3), i);
}
