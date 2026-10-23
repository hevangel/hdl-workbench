/* Output array formals whose elements C leaves unwritten (IEEE 1800-2017
 * 35.5.6.1, H.12). C writes only element 0 (or one corner); the rest of
 * the output buffer is copied back as it started. */
#include <svdpi.h>

void oaz_ifix(int *a) { a[0] = 9; }
void oaz_lfix(svLogic *a) { a[0] = sv_x; }
void oaz_oa_i(svOpenArrayHandle o) { *(int *)svGetArrElemPtr1(o, 0) = 9; }
void oaz_oa_l(svOpenArrayHandle o) { svPutLogicArrElem1(o, sv_x, 0); }
void oaz_oa_l8(svOpenArrayHandle o) {
    svLogicVecVal v = {0x77, 0};
    svPutLogicArrElemVecVal(o, &v, 0);
}
void oaz_oa_2d(svOpenArrayHandle o) {
    svPutLogicArrElem2(o, sv_x, 0, 0);
    svPutLogicArrElem2(o, sv_1, 1, 1);
}
int oaz_entry(svOpenArrayHandle o) {
    int r = *(int *)svGetArrElemPtr1(o, 0);
    *(int *)svGetArrElemPtr1(o, 1) = 4;
    return r;
}
int oaz_task(svOpenArrayHandle o) { *(int *)svGetArrElemPtr1(o, 1) = 7; return 0; }
void oaz_io(svOpenArrayHandle o) { *(int *)svGetArrElemPtr1(o, 0) = 9; }
