#include <stdio.h>
#include <string.h>
#include "svdpi.h"
typedef struct { int a; char b; double r; const char *s; void *h; } us_t;
typedef struct { short s; svLogicVecVal v[1]; svBitVecVal w[3]; svLogic l; int arr[3]; } um_t;
typedef struct { int x; char y; } s1_t;
extern void put(const char *s);
extern void *f_ch(void *h, void **o, void **io);
extern const char *f_str(const char *s, const char **o, const char **io);
extern void f_l(const svLogicVecVal *a, const svLogicVecVal *b, svLogicVecVal *o, svLogicVecVal *io);
extern void f_b(const svBitVecVal *a, svBitVecVal *o, svBitVecVal *io);
extern char f_sc(char a, short b, float c, svBit d, svLogic e, char *oa, short *ob, float *oc, svBit *od, svLogic *oe);
extern svBit f_rb(int i);
extern svLogic f_rl(int i);
extern float f_rf(float x);
extern int f_us(const us_t *a, us_t *o, um_t *m);
extern void f_ua(const int *a, int *o, char *io, const svLogicVecVal *lv, const s1_t *sa, s1_t *so);
extern int f_2d(const int *a, double *r);
extern int t_w(void *h, const svLogicVecVal *w, int *o);
static char b[400];
#define P(...) do { snprintf(b, sizeof b, __VA_ARGS__); put(b); } while (0)
void c_main(void) {
    int x1, x2, x3;
    void *o = 0, *io = &x2;
    void *r = f_ch(&x1, &o, &io);
    P("f_ch r=%d o=%d io=%d", r == &x1, o == &x2, io == &x1);
    const char *so = "unset", *sio = "base";
    const char *sr = f_str("abc", &so, &sio);
    P("f_str r=%s o=%s io=%s", sr, so, sio);
    svLogicVecVal a[3] = {{0x89abcdef, 0}, {0x01234567, 0}, {1, 1}};
    svLogicVecVal bb[4] = {{0xaaaa5555, 0}, {0x0000ffff, 0xff}, {3, 0}, {0xf, 0}};
    svLogicVecVal lo[4], lio[3] = {{0xffffffff, 0}, {0xffffffff, 0}, {0x3f, 0}};
    memset(lo, 0x5a, sizeof lo);
    f_l(a, bb, lo, lio);
    P("f_l o=%08x/%08x %08x/%08x %08x/%08x %08x/%08x io=%08x/%08x %08x/%08x %08x/%08x",
      lo[3].aval, lo[3].bval, lo[2].aval, lo[2].bval, lo[1].aval, lo[1].bval, lo[0].aval, lo[0].bval,
      lio[2].aval, lio[2].bval, lio[1].aval, lio[1].bval, lio[0].aval, lio[0].bval);
    svBitVecVal ba[4] = {0x11112222, 0x33334444, 0x55556666, 0xf}, bo[3] = {7, 7, 7}, bio[4] = {0, 1, 2, 3};
    f_b(ba, bo, bio);
    P("f_b o=%08x %08x %08x io=%08x %08x %08x %08x", bo[2], bo[1], bo[0], bio[3], bio[2], bio[1], bio[0]);
    char oa; short ob; float oc; svBit od; svLogic oe;
    char rr = f_sc(-5, -300, 1.25f, 1, sv_x, &oa, &ob, &oc, &od, &oe);
    P("f_sc r=%d oa=%d ob=%d oc=%.2f od=%d oe=%d", rr, oa, ob, oc, od, oe);
    P("f_rb %d %d f_rl %d %d %d %d f_rf %.2f", f_rb(4), f_rb(5), f_rl(0), f_rl(1), f_rl(2), f_rl(3), f_rf(1.25f));
    us_t ua = {21, 4, 1.5, "hi", 0}, uo;
    memset(&uo, 0, sizeof uo);
    um_t m; memset(&m, 0, sizeof m);
    m.s = 9; m.v[0].aval = 0x3c; m.v[0].bval = 0x01; m.w[0] = 0x12345678; m.w[1] = 0x9abcdef0; m.w[2] = 0x2a; m.l = sv_1;
    m.arr[0] = 1; m.arr[1] = 2; m.arr[2] = 3;
    int ur = f_us(&ua, &uo, &m);
    P("f_us r=%d o=%d %d %.2f %s %d m=%d %08x/%08x %08x %08x %08x %d %d %d %d", ur, uo.a, uo.b, uo.r, uo.s ? uo.s : "(null)", uo.h == 0,
      m.s, m.v[0].aval, m.v[0].bval, m.w[2], m.w[1], m.w[0], m.l, m.arr[0], m.arr[1], m.arr[2]);
    int ia[4] = {1, 2, 3, 4}, iao[3] = {0, 0, 0};
    char cio[4] = {10, 11, 12, 13};
    svLogicVecVal lv[2] = {{0xab, 0}, {0xcd, 0x0f}};
    s1_t sa[2] = {{100, 1}, {200, 2}}, sso[2] = {{0, 0}, {0, 0}};
    f_ua(ia, iao, cio, lv, sa, sso);
    P("f_ua o=%d %d %d io=%d %d %d %d so=%d/%d %d/%d", iao[0], iao[1], iao[2], cio[0], cio[1], cio[2], cio[3], sso[0].x, sso[0].y, sso[1].x, sso[1].y);
    int a2[6] = {1, 2, 3, 4, 5, 6}; double r2[2] = {0, 0};
    int r2d = f_2d(a2, r2);
    P("f_2d r=%d r2=%.1f %.1f", r2d, r2[0], r2[1]);
}
int c_tmain(void) {
    svLogicVecVal w[4] = {{1, 0}, {2, 0}, {3, 0}, {0x77, 0}};
    int o = 0, x;
    t_w(&x, w, &o);
    P("t_w o=%d", o);
    o = 0;
    t_w(0, w, &o);
    P("t_w null o=%d", o);
    return 0;
}
