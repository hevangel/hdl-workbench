#include "svdpi.h"
#include <stdint.h>

int main(void) {
    for (int width = 1; width <= 32; ++width) {
        uint32_t mask = UINT32_MAX >> (32 - width);
        uint32_t sign = UINT32_C(1) << (width - 1);
        if ((uint32_t)SV_MASK(width) != mask) return 1;
        if ((uint32_t)SV_GET_UNSIGNED_BITS(UINT32_MAX, width) != mask) return 2;
        if ((uint32_t)SV_GET_SIGNED_BITS(sign, width) != (sign | ~mask)) return 3;
        if ((uint32_t)SV_GET_SIGNED_BITS(sign - 1, width) != sign - 1) return 4;
    }
    return 0;
}
