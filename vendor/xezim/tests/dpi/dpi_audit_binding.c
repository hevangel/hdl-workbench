/* DPI audit, family E: argument binding of import calls (IEEE 1800
 * §13.5.3 / §13.5.4) and the scope a context import runs in (§35.5.3):
 * where the import is declared — its package, or the instance, interface or
 * generate block whose code calls it. */
#include "svdpi.h"
#include <string.h>

int abd_args(int a, int b) { return a * 10 + b; }

static char nm[256];
const char *abd_where(void) {
    svScope s = svGetScope();
    const char *n = s ? svGetNameFromScope(s) : "null";
    strcpy(nm, n ? n : "nullname");
    return nm;
}
const char *abd_pkg_where(void) { return abd_where(); }
