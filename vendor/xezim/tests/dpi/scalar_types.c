/* §35.5.6.1 canonical scalar types svBit / svLogic (from svdpi.h). Own names. */
#include <svdpi.h>
svBit   st_not(svBit x)    { return x ? sv_0 : sv_1; }
svLogic st_pass(svLogic x) { return x; }
svBit   st_and(svBit a, svBit b) { return (a && b) ? sv_1 : sv_0; }
