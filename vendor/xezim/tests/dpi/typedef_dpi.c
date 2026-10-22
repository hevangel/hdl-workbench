/* C side of the typedef DPI regression test.
 *
 * Regression for DPI-C imports whose formal argument uses a *typedef*
 * name for a packed logic vector (the UVM `uvm_hdl_data_t` pattern,
 * `typedef logic [127:0] hdl_data_t`). Verifies the import binds and
 * round-trips: an `output` typedef'd packed vector written on the C
 * side is observable on the SystemVerilog side.
 *
 * Uses the STANDARD svLogicVecVal layout (an array of {aval, bval}
 * inline pairs, §35.5.5), matching xezim's shipped svdpi.h.
 */
#include <svdpi.h>

#define NWORDS 4  /* 128-bit packed vector = 4 x 32-bit words */

int dpi_hdl_deposit(const char *path, const svLogicVecVal *value) {
    (void)path;
    if (!value) return 0;
    return value[0].aval == (int)0xFFFFFFFF ? 1 : 1;
}

int dpi_hdl_read(const char *path, svLogicVecVal *value) {
    (void)path;
    if (!value) return 0;
    for (int i = 0; i < NWORDS; i++) {
        value[i].aval = (int)0xFFFFFFFF;
        value[i].bval = 0;
    }
    return 1;
}
