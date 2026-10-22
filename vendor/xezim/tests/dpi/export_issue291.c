#include <stdio.h>
#include "svdpi.h"
extern void put(const char *s);
extern int ok_add(int a, int b);
extern int add2_with_h(int a, int b, void *h);
extern const char *str_ret(void);
extern void wide(const svLogicVecVal *v, svLogicVecVal *o);
void c_main(void) {
    char b[200]; int x = 7;
    snprintf(b, sizeof b, "ok_add(2,3)=%d", ok_add(2, 3)); put(b);
    snprintf(b, sizeof b, "add2_with_h(2,3,0)=%d", add2_with_h(2, 3, (void *)0)); put(b);
    snprintf(b, sizeof b, "add2_with_h(2,3,&x)=%d", add2_with_h(2, 3, &x)); put(b);
    const char *s = str_ret();
    snprintf(b, sizeof b, "str_ret()=%s", s ? s : "(null)"); put(b);
    svLogicVecVal v[4], o[4];
    for (int i = 0; i < 4; i++) { v[i].aval = 0x11111111u * (i + 1); v[i].bval = i == 3 ? 0xF0000000u : 0; o[i].aval = o[i].bval = 0xdead; }
    wide(v, o);
    snprintf(b, sizeof b, "wide o=%08x/%08x %08x/%08x %08x/%08x %08x/%08x",
             o[3].aval, o[3].bval, o[2].aval, o[2].bval, o[1].aval, o[1].bval, o[0].aval, o[0].bval); put(b);
}
