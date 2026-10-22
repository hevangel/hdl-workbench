#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "svdpi.h"
typedef struct { int a; char b; double r; } st_t;
int add_i(int a, int b) { return a + b; }
char add_byte(char a) { return a + 1; }
long long add_ll(long long a) { return a * 2; }
double mul_r(double a, double b) { return a * b; }
unsigned char ret_bit(unsigned char b) { return !b; }
void out_args(int *o, double *r, unsigned char *bt) { *o = 77; *r = 2.5; *bt = 1; }
void inout_i(int *io) { *io = *io * 3; }
const char *ret_str(const char *s) { static char buf[256]; snprintf(buf, sizeof buf, "<%s:%d>", s, (int)strlen(s)); return buf; }
void str_out(const char **s) { *s = "fromC"; }
void bitvec(const svBitVecVal *in, svBitVecVal *out) { out[0] = ~in[0]; out[1] = in[1] ^ 0x5; }   /* bit [39:0] */
void logvec(const svLogicVecVal *in, svLogicVecVal *out) { out[0].aval = in[0].aval; out[0].bval = in[0].bval; printf("C|logvec aval=%08x bval=%08x\n", in[0].aval, in[0].bval); fflush(stdout); }
int sum_open(const svOpenArrayHandle h) { int s = 0; int lo = svLow(h, 1), hi = svHigh(h, 1); for (int i = lo; i <= hi; i++) s += *(int*)svGetArrElemPtr1(h, i); return s * 100 + svSize(h, 1); }
void fill_open(const svOpenArrayHandle h) { for (int i = svLow(h,1); i <= svHigh(h,1); i++) *(int*)svGetArrElemPtr1(h, i) = i * 10; }
int sum_fixed(const int *a) { return a[0] + a[1] + a[2] + a[3]; }
void struct_in(const st_t *s, st_t *o) { o->a = s->a + 1; o->b = s->b + 1; o->r = s->r * 2; }
void *mk_handle(int v) { int *p = malloc(sizeof(int)); *p = v; return p; }
int rd_handle(void *h) { return h ? *(int*)h : -1; }
int pure_sq(int x) { return x * x; }
extern int sv_double(int);
extern void sv_wait(int);
extern int sv_scoped(void);
int call_export(int x) { return sv_double(x) + 1; }
int c_task_waits(int n) { sv_wait(n); return 0; }
int scope_test(void) {
  svScope s = svGetScope(); const char *nm = svGetNameFromScope(s);
  svScope o = svGetScopeFromName("c35.sub");
  printf("C|scope cur=%s other=%s\n", nm ? nm : "(null)", o ? svGetNameFromScope(o) : "(null)"); fflush(stdout);
  svScope prev = svSetScope(o); int r = sv_scoped(); svSetScope(prev); return r;
}
int bit_sel(const svBitVecVal *v) { return svGetBitselBit(v, 3) * 10 + svGetBitselBit(v, 0); }
void put_logic(svLogicVecVal *v) { v[0].aval = 0x0000000A; v[0].bval = 0x0000000C; }  /* 4-bit: bits 2,3 x/z */
