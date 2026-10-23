#include <stdio.h>
extern void put(const char *s);
extern int f_ev(int a, void *e);
extern int ok(int a);
void c_main(void) {
    char buf[100];
    snprintf(buf, sizeof buf, "f_ev=%d ok=%d", f_ev(7, 0), ok(4));
    put(buf);
}
