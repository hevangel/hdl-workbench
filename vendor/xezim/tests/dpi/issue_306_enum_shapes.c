/* Issue #306 neighbours: enum-typed DPI formals and results. IEEE 1800
 * §35.5.6 interprets an enum as its base type, so the C side uses that type's
 * mapping (Annex H.7): int -> int, byte -> char, logic -> svLogic,
 * bit [7:0] -> svBitVecVal*, logic [3:0] -> svLogicVecVal*. */
#include "svdpi.h"

int e306_next(int c) { return c >= 3 ? 1 : c + 1; }
void e306_out(int *o, int *io) { *o = 3; *io = *io + 1; }
int e306_pkg(int c) { return c + 1; }
char e306_byte(char x) { return (char)(x + 1); }
svLogic e306_scalar(svLogic x) { return x == sv_0 ? sv_1 : x; }
int e306_bv_in(const svBitVecVal *x) { return (int)x[0]; }
void e306_bv_out(svBitVecVal *o, svBitVecVal *io) { o[0] = 0x20; io[0] = io[0] + 0x10; }
void e306_lv_in(const svLogicVecVal *x, int *a, int *b) { *a = (int)x->aval; *b = (int)x->bval; }
void e306_lv_out(svLogicVecVal *o, svLogicVecVal *io) {
    o->aval = 2; o->bval = 0;
    io->aval ^= 3; io->bval = 0;
}
