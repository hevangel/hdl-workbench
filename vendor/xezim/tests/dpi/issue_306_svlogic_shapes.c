/* Issue #306 neighbours: 4-state values crossing the DPI boundary (IEEE 1800
 * §35.5.6.1, Annex H.7.4 / H.10.2). svLogic is an unsigned char: sv_0 = 0,
 * sv_1 = 1, sv_z = 2, sv_x = 3. svLogicVecVal holds {aval, bval} per 32 bits:
 * 0 = (0,0), 1 = (1,0), z = (0,1), x = (1,1). */
#include "svdpi.h"

/* scalar logic: output, result, input, inout */
void s306_lout(int sel, svLogic *b) { *b = (svLogic)sel; }
svLogic s306_lret(int sel) { return (svLogic)sel; }
int s306_lin(svLogic b) { return (int)b; }
void s306_linout(svLogic *b, int *seen) { *seen = (int)*b; *b = sv_z; }
/* an implicitly typed formal is a logic */
int s306_impl(svLogic b, svLogic *o) { *o = sv_x; return (int)b; }
/* imported task with an output logic */
int s306_tout(svLogic *b) { *b = sv_z; return 0; }
/* scalar bit stays two-state */
void s306_bout(svBit *b) { *b = sv_1; }
svBit s306_bret(void) { return sv_1; }

/* packed logic: aval 0xC9 / bval 0x53 is 1x0z_10zx */
void s306_vout(svLogicVecVal *v) { v[0].aval = 0xC9; v[0].bval = 0x53; }
void s306_vinout(svLogicVecVal *v, int *a, int *b) {
    *a = (int)v[0].aval;
    *b = (int)v[0].bval;
    v[0].aval ^= 0x0F;
    v[0].bval = 0x81;
}
void s306_vwide(svLogicVecVal *v) {
    v[0].aval = 0xFFFFFFFF; v[0].bval = 0x0000FFFF;
    v[1].aval = 0x5A;       v[1].bval = 0xF0;
}
void s306_vin(const svLogicVecVal *v, int *a, int *b) { *a = (int)v[0].aval; *b = (int)v[0].bval; }
