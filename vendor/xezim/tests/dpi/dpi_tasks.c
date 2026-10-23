/* IEEE 1800-2017 sec. 35.5: an imported TASK may consume simulation time
 * (through an exported task that delays), returns through its output
 * arguments when it finishes, and may be entered by several processes at
 * once. The C side only calls back; every delay happens in SystemVerilog.
 */
#include "svdpi.h"
#include "vpi_user.h"
#include <stdint.h>

extern int sv_wait(int cycles);          /* exported task: #cycles      */
extern int sv_log(int who, int step);    /* exported task: log a step   */

static uint64_t now_ticks(void) {
    svTimeVal t;
    t.type = vpiSimTime;
    svGetTime(NULL, &t);
    return ((uint64_t)t.high << 32) | t.low;
}

/* Waits `n` times for `period` ticks, logging each step from SystemVerilog;
 * reports the time it started and how long it ran. */
int c_worker(int who, int n, int period, int *started, int *elapsed) {
    uint64_t t0 = now_ticks();
    *started = (int)t0;
    for (int i = 0; i < n; i++) {
        sv_wait(period);
        sv_log(who, i);
    }
    *elapsed = (int)(now_ticks() - t0);
    return 0;
}

/* A task that never waits returns in zero time. */
int c_instant(int x, int *doubled) {
    *doubled = 2 * x;
    return 0;
}
