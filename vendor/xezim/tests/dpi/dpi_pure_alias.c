/* DPI import forms (IEEE 1800-2017 sec. 35.5.1-35.5.4): `pure` functions
 * in continuous and combinational logic, two SV names linked to one C
 * symbol, a C-owned object carried as a chandle, and output arguments of
 * several types written by one call.
 */
#include "svdpi.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static int pure_calls = 0;

int c_parity(int v) {                       /* pure */
    pure_calls++;
    return __builtin_popcount((unsigned)v) & 1;
}
int c_pure_calls(void) { return pure_calls; }

int c_sum(int a, int b) { return a + b; }   /* linked under two SV names */

/* A counter object owned by C. */
typedef struct { int count; int step; } counter_t;
void *c_counter_new(int step) {
    counter_t *c = malloc(sizeof *c);
    c->count = 0;
    c->step = step;
    return c;
}
int c_counter_bump(void *h) {
    counter_t *c = h;
    c->count += c->step;
    return c->count;
}
void c_counter_free(void *h) { free(h); }

/* Several output types from one call. */
static char text[64];
void c_outputs(int x, double *half, const char **label, svBitVecVal *byte_out,
               svLogicVecVal *nib, svBit *odd) {
    *half = x / 2.0;
    snprintf(text, sizeof text, "x=%d", x);
    *label = text;
    *byte_out = (svBitVecVal)(x & 0xff);
    nib->aval = (unsigned)(x & 0xf);
    nib->bval = (x & 0x10) ? 0x1u : 0u;     /* bit 0 becomes X when x[4] */
    *odd = (svBit)(x & 1);
}
