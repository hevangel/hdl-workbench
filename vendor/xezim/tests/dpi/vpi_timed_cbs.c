/* IEEE 1800-2017 §38.36 timed and synchronization callbacks, and the string
 * value formats a VPI client writes 4-state values with.
 *
 * These are the pieces a cocotb testbench stands on, and all of them were
 * missing or inert:
 *
 *   - cbAfterDelay (9), cbReadWriteSynch (6) and cbReadOnlySynch (7) were
 *     rejected outright by vpi_register_cb, so `Timer`, `ReadWrite` and
 *     `ReadOnly` could not be primed at all.
 *   - a pending cbAfterDelay was not future work to the scheduler, so a
 *     testbench driven only from VPI ended at time 0.
 *   - vpi_put_value decoded only Int/Real/Scalar/Vector; a vpiBinStrVal
 *     deposit (what a 4-state client writes) was dropped silently.
 *   - a deposit never marked the signal dirty, so it propagated nowhere.
 *   - writes applied from cbReadWriteSynch landed AFTER edge detection, so a
 *     VPI-driven clock never triggered `always @(posedge clk)` and the DUT
 *     stayed at its reset value forever.
 *
 * The DUT counts posedges of a clock this module drives entirely over VPI, so
 * a regression in any of the above shows up as a wrong count.
 */
#include <stdio.h>
#include <string.h>
#include "vpi_user.h"
#include "sv_vpi_user.h"   /* vpiInstance */

#define TOGGLES 8              /* 4 rising edges */
#define STEP    10             /* ticks between toggles */

static int errors = 0;
static int toggles = 0;
static int rw_fired = 0;
static int ro_fired = 0;

#define CHECK(cond, msg)                                                    \
    do {                                                                    \
        if (!(cond)) {                                                      \
            vpi_printf("FAIL: %s\n", (msg));                                \
            errors++;                                                       \
        }                                                                   \
    } while (0)

static PLI_INT32 tick_cb(p_cb_data cb);

static void arm_timer(void) {
    static s_vpi_time t;
    static s_cb_data cb;
    t.type = vpiSimTime;
    t.high = 0;
    t.low  = STEP;
    memset(&cb, 0, sizeof(cb));
    cb.reason = cbAfterDelay;
    cb.cb_rtn = tick_cb;
    cb.obj    = NULL;
    cb.time   = &t;
    CHECK(vpi_register_cb(&cb) != NULL, "cbAfterDelay must register");
}

static PLI_INT32 rw_cb(p_cb_data cb) { (void)cb; rw_fired++; return 0; }
static PLI_INT32 ro_cb(p_cb_data cb) { (void)cb; ro_fired++; return 0; }

static void arm_synch(void) {
    static s_vpi_time t;
    static s_cb_data rw, ro;
    t.type = vpiSimTime;
    t.high = 0;
    t.low  = 0;
    memset(&rw, 0, sizeof(rw));
    rw.reason = cbReadWriteSynch;
    rw.cb_rtn = rw_cb;
    rw.time   = &t;
    CHECK(vpi_register_cb(&rw) != NULL, "cbReadWriteSynch must register");
    memset(&ro, 0, sizeof(ro));
    ro.reason = cbReadOnlySynch;
    ro.cb_rtn = ro_cb;
    ro.time   = &t;
    CHECK(vpi_register_cb(&ro) != NULL, "cbReadOnlySynch must register");
}

/* Drive `top.clk` with a BINARY STRING deposit, not an integer. */
static void drive_clk(const char *bit) {
    static s_vpi_value v;
    vpiHandle h = vpi_handle_by_name("top.clk", NULL);
    CHECK(h != NULL, "vpi_handle_by_name(top.clk)");
    if (!h) return;
    v.format = vpiBinStrVal;
    v.value.str = (PLI_BYTE8 *)bit;
    vpi_put_value(h, &v, NULL, vpiNoDelay);
}

static PLI_INT32 tick_cb(p_cb_data cb) {
    (void)cb;
    drive_clk((toggles % 2 == 0) ? "1" : "0");
    toggles++;
    if (toggles < TOGGLES) {
        arm_timer();
        return 0;
    }
    /* Every rising edge should have been counted by the DUT. */
    {
        static s_vpi_value v;
        vpiHandle c = vpi_handle_by_name("top.count", NULL);
        CHECK(c != NULL, "vpi_handle_by_name(top.count)");
        if (c) {
            v.format = vpiIntVal;
            vpi_get_value(c, &v);
            vpi_printf("COUNT: %d\n", (int)v.value.integer);
            CHECK(v.value.integer == TOGGLES / 2,
                  "a VPI-driven clock must trigger always @(posedge clk)");
        }
    }
    CHECK(rw_fired > 0, "cbReadWriteSynch must fire");
    CHECK(ro_fired > 0, "cbReadOnlySynch must fire");
    vpi_printf("RW_FIRED: %d RO_FIRED: %d\n", rw_fired, ro_fired);
    vpi_printf(errors ? "RESULT: FAILED\n" : "RESULT: PASSED\n");
    {
        static s_vpi_value fv;
        fv.format = vpiIntVal;
        fv.value.integer = 0;
        vpi_control(vpiFinish, 0);
    }
    return 0;
}

/* Object-model bits a VPI client uses before it can do anything useful.
 * All three were wrong: vpi_iterate did not accept vpiInstance (the generic
 * instance class, and what a client asks for to find the design root),
 * vpi_get_str could not name a vpiType, and vpi_free_object returned 0 —
 * "failure" per §38.24 — for every successful free. */
static void check_object_model(void) {
    vpiHandle it, top, sig;
    PLI_BYTE8 *tn;

    it = vpi_iterate(vpiInstance, NULL);
    CHECK(it != NULL, "vpi_iterate(vpiInstance, NULL) finds the design root");
    if (it) {
        top = vpi_scan(it);
        CHECK(top != NULL, "the root iterator yields the top instance");
        if (top) {
            tn = vpi_get_str(vpiType, top);
            CHECK(tn != NULL && strcmp(tn, "vpiModule") == 0,
                  "vpi_get_str(vpiType) names the top instance vpiModule");
            /* §38.24: 1 on success, not 0. */
            CHECK(vpi_free_object(top) == 1, "vpi_free_object returns 1 on success");
        }
    }

    sig = vpi_handle_by_name("top.count", NULL);
    CHECK(sig != NULL, "vpi_handle_by_name(top.count)");
    if (sig) {
        tn = vpi_get_str(vpiType, sig);
        CHECK(tn != NULL, "vpi_get_str(vpiType) names a signal");
        if (tn) vpi_printf("SIGTYPE: %s\n", tn);
    }
    CHECK(vpi_free_object(NULL) == 0, "vpi_free_object(NULL) is a failure");
}

static PLI_INT32 start_cb(p_cb_data cb) {
    (void)cb;
    check_object_model();
    arm_synch();
    arm_timer();
    return 0;
}

static void register_start(void) {
    static s_cb_data cb;
    memset(&cb, 0, sizeof(cb));
    cb.reason = cbStartOfSimulation;
    cb.cb_rtn = start_cb;
    vpi_register_cb(&cb);
}

void (*vlog_startup_routines[])(void) = {register_start, 0};
