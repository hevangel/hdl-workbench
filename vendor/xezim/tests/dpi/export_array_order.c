#include <stdio.h>
extern void put(const char *s);
extern void f(const int *a, const int *b, const int *c, int *o);
void c_main(void) {
    char buf[100];
    int a[4] = {0, 1, 2, 3}, b[4] = {10, 11, 12, 13}, c[6] = {20, 21, 22, 23, 24, 25};
    int o[3] = {0, 0, 0};
    f(a, b, c, o);
    snprintf(buf, sizeof buf, "o=%d %d %d", o[0], o[1], o[2]);
    put(buf);
}
