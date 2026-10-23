/* Imported tasks that consume time (IEEE 1800-2017 sec. 35.5.2, 35.9): the
 * kinds of wait an exported task can make, recursion through an export that
 * waits at every level, and the disable protocol. Every delay happens in
 * SystemVerilog through an export; the externs are weak because each bench
 * exports only the ones it uses. Shared by the dpi_task_*_test.sv benches.
 * Times come from vpi_get_time, which both simulators provide. */
#include "svdpi.h"
#include "vpi_user.h"
#include <stdint.h>

extern int sv_wait(int cycles) __attribute__((weak));
extern int sv_log(int who, int step) __attribute__((weak));
extern int sv_edge(void) __attribute__((weak));              /* @(posedge clk)        */
extern int sv_until(int value) __attribute__((weak));        /* wait (count >= value) */
extern int sv_pair(int a, int b) __attribute__((weak));      /* fork #a; #b; join     */
extern int sv_spawn_wait(int d) __attribute__((weak));       /* fork #d join_none; wait fork */
extern int sv_nest(int who, int depth, int *levels) __attribute__((weak)); /* #1, then c_nest */
extern int sv_get(int *v) __attribute__((weak));             /* output through an export task */

static uint64_t now(void) {
    s_vpi_time t;
    t.type = vpiSimTime;
    vpi_get_time(NULL, &t);
    return ((uint64_t)t.high << 32) | t.low;
}

/* n steps of `period`, logging each one. */
int c_steps(int who, int n, int period, int *finished_at) {
    for (int i = 0; i < n; i++) {
        sv_wait(period);
        sv_log(who, i);
    }
    *finished_at = (int)now();
    return 0;
}

/* Each kind of wait an exported task can make. */
int c_kinds(int who, int *finished_at) {
    sv_edge();
    sv_log(who, 100 + (int)now());
    sv_until(3);
    sv_log(who, 200 + (int)now());
    sv_pair(2, 5);
    sv_log(who, 300 + (int)now());
    sv_spawn_wait(4);
    sv_log(who, 400 + (int)now());
    int v = 0;
    sv_get(&v);
    sv_log(who, 500 + v);
    *finished_at = (int)now();
    return 0;
}

/* C -> SV -> C recursion through an export task that waits at each level:
 * returns the number of levels. */
int c_nest(int who, int depth, int *levels) {
    int below = 0;
    if (depth > 0) sv_nest(who, depth - 1, &below);
    *levels = below + 1;
    return 0;
}

/* sec. 35.9: a waiter that notices the disable. */
static int seen[16];
int c_guarded(int who, int n, int period) {
    for (int i = 0; i < n; i++) {
        if (sv_wait(period)) {
            seen[who] = 10 + svIsDisabledState();
            svAckDisabledState();
            return 1;
        }
        sv_log(who, i);
    }
    seen[who] = 1;
    return 0;
}
int c_seen(int who) { return seen[who]; }

/* sec. 35.9 protocol violations: returning 0 after a disable, and calling an
 * export after a disable. */
int c_bad_return(int period) {
    if (sv_wait(period)) return 0;
    return 0;
}
int c_bad_export(int period) {
    if (sv_wait(period)) {
        svAckDisabledState();
        sv_log(99, 99);
        return 1;
    }
    return 0;
}
