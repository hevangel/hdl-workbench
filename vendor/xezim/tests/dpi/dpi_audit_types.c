/* DPI audit, family B: `integer` / `time` formals and unpacked structs
 * (IEEE 1800 §35.5.6, Annex H.7). integer and time are 4-state packed
 * vectors, so svLogicVecVal* ({aval, bval}: z = (0,1), x = (1,1)); an
 * unpacked struct is a pointer to the C struct of its members, a packed
 * member an inline svLogicVecVal[] / svBitVecVal[], a string a const char*. */
#include "svdpi.h"
#include <string.h>

void aty_integer(const svLogicVecVal *i, svLogicVecVal *o, svLogicVecVal *io, int *a, int *b) {
    *a = (int)i->aval; *b = (int)i->bval;
    o->aval = 0x12; o->bval = 0x3;               /* ...0001_00xz */
    io->aval = ~io->aval; io->bval = 0;
}
void aty_time(const svLogicVecVal *t, svLogicVecVal *o, int *lo, int *hi) {
    *lo = (int)t[0].aval; *hi = (int)t[1].aval;
    o[0].aval = 7; o[0].bval = 0; o[1].aval = 1; o[1].bval = 0;
}

typedef struct { int a; char b; double c; svLogic d; } s1_t;
typedef struct { short h; svLogicVecVal v[1]; svBitVecVal w[3]; const char *s; int arr[3]; } s2_t;
typedef struct { s1_t in; long long z; } s3_t;
int aty_s_in(const s1_t *s) { return s->a * 1000 + s->b * 10 + (int)s->c + (s->d == sv_x ? 7 : 0); }
void aty_s_out(s1_t *s) { s->a = 11; s->b = -2; s->c = 2.75; s->d = sv_z; }
void aty_s_io(s1_t *s) { s->a += 1; s->d = s->d == sv_x ? sv_1 : sv_0; }
int aty_s2_in(const s2_t *s) {
    return s->h + (int)s->v[0].aval * 2 + (int)s->v[0].bval * 3 + (int)s->w[2]
         + (int)strlen(s->s) * 100 + s->arr[0] + s->arr[2] * 1000;
}
void aty_s2_out(s2_t *s) {
    s->h = -9; s->v[0].aval = 0xA5; s->v[0].bval = 0x0F;
    s->w[0] = 1; s->w[1] = 2; s->w[2] = 0x3; s->s = "c-str";
    s->arr[0] = 4; s->arr[1] = 5; s->arr[2] = 6;
}
long long aty_s3_in(const s3_t *s) { return s->in.a + s->z; }
int aty_sa_in(const s1_t *a) { return a[0].a * 10 + a[1].a; }
void aty_sa_out(s1_t *a) { a[0].a = 70; a[1].a = 80; a[0].d = sv_x; a[1].d = sv_1; }
