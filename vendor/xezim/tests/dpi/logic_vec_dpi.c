/* Standard svLogicVecVal: an array of s_vpi_vecval {aval, bval} pairs
   (IEEE 1800-2017 §35.5.5 / Annex H.10.2). Element k of a packed vector is
   x[k].aval / x[k].bval — NOT two separate array pointers. */
#include <svdpi.h>

static unsigned g_seen_lsb = 0;

void vec_in(const svLogicVecVal *x) {
    if (!x) { g_seen_lsb = 0; return; }
    g_seen_lsb = (unsigned)x[0].aval;
}

void vec_flip(svLogicVecVal *x) {
    if (!x) return;
    x[0].aval ^= 0x000000FF;
    x[0].bval  = 0;
}

void vec_set(svLogicVecVal *x) {
    if (!x) return;
    x[0].aval = (int)0xAABBCCDD; x[0].bval = 0;
    x[1].aval = 0x55667788;      x[1].bval = 0;
    x[2].aval = 0x11223344;      x[2].bval = 0;
}

int vec_seen_lsb(void) { return (int)g_seen_lsb; }
