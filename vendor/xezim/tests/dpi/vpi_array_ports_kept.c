/* VPI writes to tests/dpi/vpi_array_ports_kept.sv (see
 * tests/strings/vpi_array_ports.rs). At time 1 it writes an element of the
 * actual of the port with a second driver and an element of the port that
 * collapses; at time 6 it reads them back.
 * Prints `CHK|<name>` per read that holds, `FAIL:` otherwise. */
#include <string.h>
#include "vpi_user.h"

static vpiHandle get(const char *n, int i) {
    vpiHandle h = vpi_handle_by_name((PLI_BYTE8 *)n, NULL);
    if (h) h = vpi_handle_by_index(h, i);
    if (!h) vpi_printf("FAIL: no handle for %s [%d]\n", n, i);
    return h;
}

static void put(const char *n, int i, int v) {
    vpiHandle h = get(n, i);
    s_vpi_value val;
    if (!h) return;
    val.format = vpiIntVal;
    val.value.integer = v;
    vpi_put_value(h, &val, NULL, vpiNoDelay);
}

static void want(const char *n, int i, const char *bits) {
    vpiHandle h = get(n, i);
    s_vpi_value val;
    if (!h) return;
    val.format = vpiBinStrVal;
    vpi_get_value(h, &val);
    if (strcmp(val.value.str, bits) != 0)
        vpi_printf("FAIL: %s [%d] reads %s, not %s\n", n, i, val.value.str, bits);
    else
        vpi_printf("CHK|%s[%d]\n", n, i);
}

static void at(PLI_INT32 (*rtn)(p_cb_data), unsigned t) {
    s_cb_data cb;
    s_vpi_time tm = {vpiSimTime, 0, t, 0.0};
    memset(&cb, 0, sizeof cb);
    cb.reason = cbAfterDelay;
    cb.cb_rtn = rtn;
    cb.time = &tm;
    vpi_register_cb(&cb);
}

static PLI_INT32 late(p_cb_data cb) {
    (void)cb;
    /* The port with a second driver follows its actual through its
     * connection. */
    want("top.ue.a", 0, "00110100");
    want("top.uk.a", 0, "01010110");
    want("top.k", 0, "01010110");
    return 0;
}

static PLI_INT32 early(p_cb_data cb) {
    (void)cb;
    put("top.e", 0, 0x34);
    put("top.uk.a", 0, 0x56);
    at(late, 5);
    return 0;
}

static PLI_INT32 start(p_cb_data cb) {
    (void)cb;
    at(early, 1);
    return 0;
}

static void reg(void) {
    s_cb_data cb;
    memset(&cb, 0, sizeof cb);
    cb.reason = cbStartOfSimulation;
    cb.cb_rtn = start;
    vpi_register_cb(&cb);
}

void (*vlog_startup_routines[])(void) = {reg, 0};
