/* Issue #306: C writes and returns 4-state svLogic values (x and z) to SV,
 * and an imported function takes and returns an enum (LRM 35.5.5 maps an
 * enum to its base C type). */
#include "svdpi.h"

void dpi_logic_out(svLogic *b) { *b = sv_x; }
void dpi_logic_zout(svLogic *b) { *b = sv_z; }
svLogic dpi_logic_ret(void) { return sv_x; }

int dpi_next_color(int c) { return c >= 3 ? 1 : c + 1; }
