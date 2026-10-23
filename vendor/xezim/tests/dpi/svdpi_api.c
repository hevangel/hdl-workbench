/* IEEE 1800-2017 Annex H utilities called from C: open-array queries and
 * element access for each canonical element type, bit/part selects of
 * canonical vectors, per-scope user data, svGetCallerInfo and the disable
 * protocol. Each function reports through its return value; the bench
 * prints it. */
#include <stdio.h>
#include <string.h>
#include "svdpi.h"

static char buf[1024];

/* ---- open-array shape ---------------------------------------------- */

/* The unpacked dimension's shape, then each element by SV index from
 * left to right through svGetArrElemPtr1. */
const char *int_shape(const svOpenArrayHandle h) {
    int n = snprintf(buf, sizeof buf, "l=%d r=%d lo=%d hi=%d inc=%d size=%d dims=%d bytes=%d |",
                     svLeft(h, 1), svRight(h, 1), svLow(h, 1), svHigh(h, 1), svIncrement(h, 1),
                     svSize(h, 1), svDimensions(h), svSizeOfArray(h));
    if (svSize(h, 1) > 0) {
        for (int i = svLeft(h, 1);; i -= svIncrement(h, 1)) {
            const int *p = svGetArrElemPtr1(h, i);
            n += snprintf(buf + n, sizeof buf - n, " %d:%d", i, p ? *p : -999);
            if (i == svRight(h, 1)) break;
        }
    } else {
        n += snprintf(buf + n, sizeof buf - n, " empty ptr=%s",
                      svGetArrElemPtr1(h, 0) ? "set" : "null");
    }
    return buf;
}

/* Out-of-range indices give NULL; the variadic form agrees with the
 * fixed-arity one; the raw handle is the low element. */
const char *int_edges(const svOpenArrayHandle h) {
    int lo = svLow(h, 1), hi = svHigh(h, 1);
    snprintf(buf, sizeof buf, "below=%s above=%s variadic=%s raw=%s dim2=%d",
             svGetArrElemPtr1(h, lo - 1) ? "set" : "null",
             svGetArrElemPtr1(h, hi + 1) ? "set" : "null",
             svGetArrElemPtr(h, hi) == svGetArrElemPtr1(h, hi) ? "same" : "differ",
             svGetArrayPtr(h) == svGetArrElemPtr1(h, lo) ? "low" : "other",
             svSize(h, 2));
    return buf;
}

/* ---- canonical element types --------------------------------------- */

int sum_bytes(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s += *(const signed char *)svGetArrElemPtr1(h, i);
    return s;
}

int sum_shorts(const svOpenArrayHandle h) {
    int s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s += *(const short *)svGetArrElemPtr1(h, i);
    return s;
}

long long sum_longs(const svOpenArrayHandle h) {
    long long s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s += *(const long long *)svGetArrElemPtr1(h, i);
    return s;
}

double sum_reals(const svOpenArrayHandle h) {
    double s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s += *(const double *)svGetArrElemPtr1(h, i);
    return s;
}

double sum_shortreals(const svOpenArrayHandle h) {
    double s = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) s += *(const float *)svGetArrElemPtr1(h, i);
    return s;
}

/* Each element doubled and every double scaled by 1.5, in place. */
void double_bytes(const svOpenArrayHandle h) {
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) *(signed char *)svGetArrElemPtr1(h, i) *= 2;
}

void scale_reals(const svOpenArrayHandle h) {
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) *(double *)svGetArrElemPtr1(h, i) *= 1.5;
}

void scale_ints(const svOpenArrayHandle h, int k) {
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) *(int *)svGetArrElemPtr1(h, i) *= k;
}

/* ---- packed elements ----------------------------------------------- */

/* A `bit [11:0]` array: the packed dimension (0) and each element. */
const char *bitvec_shape(const svOpenArrayHandle h) {
    int n = snprintf(buf, sizeof buf, "dims=%d p.l=%d p.r=%d p.size=%d |", svDimensions(h),
                     svLeft(h, 0), svRight(h, 0), svSize(h, 0));
    for (int i = svLeft(h, 1);; i -= svIncrement(h, 1)) {
        svBitVecVal v = 0;
        svGetBitArrElem1VecVal(&v, h, i);
        n += snprintf(buf + n, sizeof buf - n, " %d:%03x", i, v);
        if (i == svRight(h, 1)) break;
    }
    return buf;
}

/* A `logic [3:0]` array, each element as its four bits (MSB first). */
const char *logicvec_dump(const svOpenArrayHandle h) {
    static const char code[] = "01zx";
    int n = 0;
    buf[0] = 0;
    for (int i = svLeft(h, 1);; i -= svIncrement(h, 1)) {
        svLogicVecVal v;
        svGetLogicArrElem1VecVal(&v, h, i);
        char bits[5];
        for (int b = 0; b < 4; b++) bits[3 - b] = code[svGetBitselLogic(&v, b)];
        bits[4] = 0;
        n += snprintf(buf + n, sizeof buf - n, "%s%d:%s", n ? " " : "", i, bits);
        if (i == svRight(h, 1)) break;
    }
    return buf;
}

/* Element at index i gets i * 0x111, through the canonical copy-in. */
void fill_bitvec(const svOpenArrayHandle h) {
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) {
        svBitVecVal v = (svBitVecVal)(i * 0x111) & 0xfff;
        svPutBitArrElem1VecVal(h, &v, i);
    }
}

/* The low element becomes 1x0z, the others keep their value with bit 0
 * forced to 1. */
void mark_logic(const svOpenArrayHandle h) {
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) {
        svLogicVecVal v;
        if (i == svLow(h, 1)) {
            v.aval = 0;
            v.bval = 0;
            svPutBitselLogic(&v, 3, sv_1);
            svPutBitselLogic(&v, 2, sv_x);
            svPutBitselLogic(&v, 1, sv_0);
            svPutBitselLogic(&v, 0, sv_z);
        } else {
            svGetLogicArrElem1VecVal(&v, h, i);
            svPutBitselLogic(&v, 0, sv_1);
        }
        svPutLogicArrElem1VecVal(h, &v, i);
    }
}

/* ---- scalar elements ----------------------------------------------- */

/* Counts the ones, then inverts each bit. */
int flip_bits(const svOpenArrayHandle h) {
    int ones = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) {
        svBit b = svGetBitArrElem1(h, i);
        ones += b;
        svPutBitArrElem1(h, !b, i);
    }
    return ones;
}

/* Counts the x and z bits, then turns x into 1 and z into 0. */
int resolve_logic(const svOpenArrayHandle h) {
    int unknown = 0;
    for (int i = svLow(h, 1); i <= svHigh(h, 1); i++) {
        svLogic b = svGetLogicArrElem1(h, i);
        if (b == sv_x || b == sv_z) {
            unknown++;
            svPutLogicArrElem(h, b == sv_x ? sv_1 : sv_0, i);
        }
    }
    return unknown;
}

/* ---- bit and part selects of plain vectors ------------------------- */

int get_part(const svBitVecVal *v, int i, int w) {
    svBitVecVal r = 0xdeadbeef;
    svGetPartselBit(&r, v, i, w);
    return (int)r;
}

void put_part(svBitVecVal *v, int i, int w, int val) {
    svPutPartselBit(v, (svBitVecVal)val, i, w);
}

int get_bit(const svBitVecVal *v, int i) {
    return svGetBitselBit(v, i);
}

/* Bits [i+w-1:i] of a 4-state vector: aval in the low 16 bits of the
 * result, bval in the high 16. */
int get_lpart(const svLogicVecVal *v, int i, int w) {
    svLogicVecVal r;
    svGetPartselLogic(&r, v, i, w);
    return (int)((r.bval << 16) | (r.aval & 0xffff));
}

void put_lpart(svLogicVecVal *v, int i, int w, int aval, int bval) {
    svLogicVecVal s;
    s.aval = (uint32_t)aval;
    s.bval = (uint32_t)bval;
    svPutPartselLogic(v, s, i, w);
}

/* ---- user data, caller info, disable state ------------------------- */

static int key_a, key_b;
static int store[8];
static int nstore;

/* Keeps `v` under this scope's key_a; returns svPutUserData's status. */
int ud_put(int v) {
    store[nstore] = v;
    return svPutUserData(svGetScope(), &key_a, &store[nstore++]);
}

/* This scope's key_a value, or -1. */
int ud_get(void) {
    int *p = svGetUserData(svGetScope(), &key_a);
    return p ? *p : -1;
}

/* Misuse and key separation: NULL scope / key are refused, an unused key
 * reads back NULL, and a second put replaces the first. */
const char *ud_rules(void) {
    static int x = 1, y = 2;
    svScope s = svGetScope();
    int put_null_scope = svPutUserData(NULL, &key_b, &x);
    int put_null_key = svPutUserData(s, NULL, &x);
    const char *other = svGetUserData(s, &key_b) ? "set" : "null";
    svPutUserData(s, &key_b, &x);
    svPutUserData(s, &key_b, &y);
    int replaced = *(int *)svGetUserData(s, &key_b);
    snprintf(buf, sizeof buf, "null_scope=%d null_key=%d unused=%s replaced=%d", put_null_scope,
             put_null_key, other, replaced);
    return buf;
}

const char *misc_state(void) {
    const char *file = "unset";
    int line = -1;
    int got = svGetCallerInfo(&file, &line);
    snprintf(buf, sizeof buf, "caller=%d disabled=%d", got, svIsDisabledState());
    svAckDisabledState();
    return buf;
}
