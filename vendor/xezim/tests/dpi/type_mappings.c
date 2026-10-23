/* DPI import type-mapping coverage: scalar atoms, real, out/inout, packed
   2-state (svBitVecVal*) and 4-state (svLogicVecVal*) vectors incl. x/z.
   Standard svdpi.h layouts. Own names. */
#include <svdpi.h>
#include <string.h>

char      tm_byte(char x)         { return x + 1; }
short     tm_short(short x)       { return x + 1; }
int       tm_int(int x)           { return x + 1; }
long long tm_long(long long x)    { return x + 1; }
double    tm_real(double x)       { return x * 2.0; }
float     tm_shortreal(float x)   { return x * 2.0f; }
void      tm_out(int* o)          { *o = 77; }
void      tm_inout(int* io)       { *io += 100; }

/* 2-state packed bit [31:0] -> svBitVecVal* (plain uint32 array) */
int tm_bitvec(const svBitVecVal* v)   { return (int)v[0]; }
/* bit [63:0] -> two words */
int tm_bitvec64(const svBitVecVal* v) { return (int)v[0] + (int)v[1]; }
/* 4-state logic [31:0] -> svLogicVecVal* ({aval,bval} inline) */
int  tm_logic_aval(const svLogicVecVal* v) { return v[0].aval; }
long long tm_logic_ab(const svLogicVecVal* v) {
    return ((long long)(unsigned)v[0].bval << 32) | (unsigned)v[0].aval;
}
int  tm_strlen(const char* s) { return (int)strlen(s); }
