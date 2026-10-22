#include "svdpi.h"
#include <stdio.h>

extern long long get_drain_time(void);
extern void sv_report(long long v);

void c_entry(void) {
    long long v = get_drain_time();   /* SV -> C DPI return boundary (buggy in xezim) */
    sv_report(v);                      /* C -> SV callback to report */
}
