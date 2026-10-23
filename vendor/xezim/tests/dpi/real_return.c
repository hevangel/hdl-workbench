/*
 * Regression for #324 (and its cousin paths): a real-valued expression to/from
 * a DPI integral scalar OR packed-vector formal must be VALUE-converted
 * (rounding to nearest, ties away from zero) in both directions, not
 * bit-reinterpret its IEEE-754 bits. xezim used to return the bit pattern of
 * real 100.0 (0x4059000000000000) instead of 100, passed the same garbage (0)
 * into a C int formal, and handed a C time/integer/logic packed formal the raw
 * f64 bits (half of double 3.75 -> 0 in the low word, up to 0x400E... whole).
 */
#include "svdpi.h"
#include <stdio.h>

/* Exports: return types declared in the SV; read here as C integers. */
extern long long get_drain_time(void);
extern int get_i32(void);
extern short get_i16(void);
extern void sv_report(long long v);

/* Imports called from SV with real actuals into integral/packed formals. */
int c_take_int(int v) { return v; }
void c_take_time(const svLogicVecVal* p) { printf("T|T%lld\n", (long long)(p[0].aval | ((unsigned long long)p[1].aval << 32))); }
void c_take_integer(const svLogicVecVal* p) { printf("T|I%u\n", p[0].aval); }
void c_take_bit(const svBitVecVal* p) { printf("T|B%u\n", p[0]); }

void c_entry(void) {
    long long v64 = get_drain_time(); /* 100ns / 1ns -> real 100.0 -> long long 100 */
    int       v32 = get_i32();        /* real 3.75 -> int 4 (round half away) */
    short     v16 = get_i16();        /* real -3.5 -> short -4 */
    printf("T|%lld %d %d\n", v64, v32, (int)v16);
    sv_report(v64);
}