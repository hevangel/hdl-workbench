/*
 * Regression for the scalar DPI width fix: a 1-bit `bit` formal is crossed as
 * an svBit holding its single meaningful bit, and a real->longint conversion
 * wraps at the 2^63 boundary, both matching the reference simulator.
 *
 * xezim used to route scalar `bit` through the same 32-bit slot as `int`,
 * handing a C svBit (unsigned char) the value's low 8 bits instead of bit 0
 * (bit(2) -> 2, not 0; bit(1.75) -> 2, not 0). And a real 2^63 converted to a
 * longint used to saturate to INT64_MAX where the reference wraps to INT64_MIN.
 */
#include "svdpi.h"
#include <stdio.h>

void bitw_take(svBit a, svBit b, svBit c, svBit d) {
    printf("T|B %d %d %d %d\n", (int)a, (int)b, (int)c, (int)d);
}
void bitw_take_long(long long v) { printf("T|L %lld\n", v); }
void bitw_toggle(svBit* b) { *b = (*b ^ 1) & 1; }
void bitw_set(svBit* b) { *b = 1; }