/* Scalar 4-state svLogic marshalling, enum-typed prototypes, and open
 * arrays of scalar logic (IEEE 1800-2017 35.5.5/35.5.6, Annex H.7).
 * Every direction of a 1-bit `logic` formal/return is a svLogic CELL:
 * 0/1/2/3 = 0/1/z/x. Enum formals and returns map to the base C type.
 * An unpacked array of scalar logic has 1-byte svLogic elements. */
#include <svdpi.h>

void sle_out_x(svLogic *b) { *b = sv_x; }
void sle_out_z(svLogic *b) { *b = sv_z; }
svLogic sle_ret_x(void) { return sv_x; }

/* The cell an `input logic` formal arrives as, straight back. */
int sle_sees(svLogic b) { return (int)b; }

/* What an `inout logic` reads on entry; writes z back. */
int sle_io_sees(svLogic *b) {
    int v = (int)*b;
    *b = sv_z;
    return v;
}

/* enum base type is int (35.5.5): plain int in, int out. */
int sle_next_color(int c) { return c >= 3 ? 1 : c + 1; }

/* byte-based enum: the C ABI promotes the return; SV sees 8-bit signed. */
int sle_byte_enum(void) { return 200; }

/* output enum formal arrives as an int* the C side stores through. */
void sle_color_out(int *c) { *c = 3; }

/* The first cell of an INPUT open array (actual values pass through). */
int sle_oa_entry(const svOpenArrayHandle o) { return (int)svGetLogicArrElem1(o, 0); }

/* Fill an OUTPUT open array of scalar logic three ways: a raw element
 * pointer, svPutLogicArrElem1 (note: value BEFORE index), and reading
 * back what the output buffer started as. */
int sle_oa_cells(svOpenArrayHandle o) {
    int entry = (int)svGetLogicArrElem1(o, 0);
    *(svLogic *)svGetArrElemPtr1(o, 0) = sv_x;
    *(svLogic *)svGetArrElemPtr1(o, 1) = sv_z;
    svPutLogicArrElem1(o, sv_1, 2);
    svPutLogicArrElem1(o, sv_0, 3);
    return entry;
}
