/* VPI writes to the unpacked-array ports of tests/dpi/vpi_array_ports.sv
 * (see tests/strings/vpi_array_ports.rs). At time 1 it writes elements of
 * the instances' array ports, one whole port with vpi_put_value_array, and
 * two elements of a top-level array that a port reads; at time 6 it reads
 * the written elements back through the instance's name and the actual's.
 * Prints `CHK|<name>` per read that holds, `FAIL:` otherwise. With
 * AP_BY_NAME set, elements are looked up by name (`top.u1.a[0]`) instead of
 * with vpi_handle_by_index. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "vpi_user.h"
#include "sv_vpi_user.h"

static vpiHandle get(const char *n, int i) {
    char full[64];
    vpiHandle h;
    if (i >= 0 && getenv("AP_BY_NAME")) {
        snprintf(full, sizeof full, "%s[%d]", n, i);
        h = vpi_handle_by_name(full, NULL);
    } else {
        h = vpi_handle_by_name((PLI_BYTE8 *)n, NULL);
        if (h && i >= 0) h = vpi_handle_by_index(h, i);
    }
    if (!h) vpi_printf("FAIL: no handle for %s [%d]\n", n, i);
    return h;
}

static void put(const char *n, int i, int fmt, int v, const char *s) {
    vpiHandle h = get(n, i);
    s_vpi_value val;
    if (!h) return;
    val.format = fmt;
    if (fmt == vpiIntVal) val.value.integer = v;
    else if (fmt == vpiScalarVal) val.value.scalar = v ? vpi1 : vpi0;
    else val.value.str = (PLI_BYTE8 *)s;
    vpi_put_value(h, &val, NULL, vpiNoDelay);
}

static void want(const char *n, int i, int fmt, int v, const char *s) {
    vpiHandle h = get(n, i);
    s_vpi_value val;
    if (!h) return;
    val.format = fmt;
    vpi_get_value(h, &val);
    if (fmt == vpiIntVal ? val.value.integer != v : strcmp(val.value.str, s) != 0) {
        if (fmt == vpiIntVal)
            vpi_printf("FAIL: %s [%d] reads %d, not %d\n", n, i, (int)val.value.integer, v);
        else
            vpi_printf("FAIL: %s [%d] reads %s, not %s\n", n, i, val.value.str, s);
    } else if (i >= 0) {
        vpi_printf("CHK|%s[%d]\n", n, i);
    } else {
        vpi_printf("CHK|%s\n", n);
    }
}

#define W0 "123456789abcdef0123456789"
#define W1 "fedcba9876543210fedcba987"
#define L3 "0123456789abcdef"
#define L5 "fedcba9876543210"

static void at(PLI_INT32 (*rtn)(p_cb_data), unsigned t);

static void override8(int flags, int v) {
    s_vpi_value val;
    val.format = vpiIntVal;
    val.value.integer = v;
    vpi_put_value(get("top.um8.a", 5), &val, NULL, flags);
}

static PLI_INT32 released_again(p_cb_data cb) {
    (void)cb;
    want("top.um8.a", 5, vpiIntVal, 44, 0);
    vpi_printf("UPD|same callback release\n");
    return 0;
}

static PLI_INT32 released(p_cb_data cb) {
    (void)cb;
    want("top.um8.a", 5, vpiIntVal, 44, 0);
    vpi_printf("UPD|release\n");
    /* Release must restore the unchanged source even when no settle has
       occurred since the force. */
    override8(vpiForceFlag, 99);
    override8(vpiReleaseFlag, 0);
    at(released_again, 1);
    return 0;
}

static PLI_INT32 forced(p_cb_data cb) {
    (void)cb;
    want("top.um8.a", 5, vpiIntVal, 123, 0);
    vpi_printf("UPD|force holds\n");
    override8(vpiReleaseFlag, 0);
    at(released, 1);
    return 0;
}

static PLI_INT32 change_forced_source(p_cb_data cb) {
    (void)cb;
    put("top.m8", 5, vpiIntVal, 44, 0);
    at(forced, 1);
    return 0;
}

static PLI_INT32 changed(p_cb_data cb) {
    (void)cb;
    want("top.um8.a", 3, vpiIntVal, 33, 0);
    want("top.um64.a", 3, vpiHexStrVal, 0, "1111222233334444");
    want("top.um1.a", 3, vpiIntVal, 0, 0);
    /* A change to another element must leave these deposits intact. */
    want("top.um8.a", 5, vpiIntVal, 88, 0);
    want("top.um64.a", 5, vpiHexStrVal, 0, L5);
    want("top.um1.a", 5, vpiIntVal, 1, 0);
    vpi_printf("UPD|source changes\n");
    override8(vpiForceFlag, 123);
    at(change_forced_source, 1);
    return 0;
}

static PLI_INT32 change_source(p_cb_data cb) {
    (void)cb;
    put("top.m8", 3, vpiIntVal, 33, 0);
    put("top.m64", 3, vpiHexStrVal, 0, "1111222233334444");
    put("top.m1", 3, vpiIntVal, 0, 0);
    at(changed, 1);
    return 0;
}

static PLI_INT32 late(p_cb_data cb) {
    (void)cb;
    want("top.u1.a", 0, vpiIntVal, 10, 0);
    want("top.a8", 1, vpiIntVal, 20, 0);
    want("top.o.m.l.a", 1, vpiIntVal, 40, 0);
    want("top.o.m.a", 0, vpiIntVal, 30, 0);
    want("top.o.a", 1, vpiIntVal, 40, 0);
    want("top.c8", 0, vpiIntVal, 30, 0);
    want("top.uw.a", 0, vpiHexStrVal, 0, W0);
    want("top.w", 1, vpiHexStrVal, 0, W1);
    want("top.us.a", 2, vpiIntVal, 1, 0);
    want("top.s1", 3, vpiIntVal, 1, 0);
    want("top.ur.a", 0, vpiIntVal, 50, 0);
    want("top.ud.a", 0, vpiIntVal, 70, 0);
    want("top.d8", 1, vpiIntVal, 71, 0);
    want("top.ug.a[0][1]", -1, vpiIntVal, 21, 0);
    want("top.g8[1][2]", -1, vpiIntVal, 12, 0);
    want("top.un.a", 0, vpiIntVal, 40, 0);
    want("top.n8", 3, vpiIntVal, 43, 0);
    want("top.um8.a", 5, vpiIntVal, 88, 0);
    want("top.um64.a", 3, vpiHexStrVal, 0, L3);
    want("top.um1.a", 5, vpiIntVal, 1, 0);
    at(change_source, 2);
    return 0;
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

static PLI_INT32 early(p_cb_data cb) {
    (void)cb;
    put("top.u1.a", 0, vpiIntVal, 10, 0);
    put("top.u1.a", 1, vpiIntVal, 20, 0);
    /* Three levels down, and the level above it. */
    put("top.o.m.l.a", 0, vpiIntVal, 30, 0);
    put("top.o.m.a", 1, vpiIntVal, 40, 0);
    put("top.uw.a", 0, vpiHexStrVal, 0, W0);
    put("top.uw.a", 1, vpiHexStrVal, 0, W1);
    put("top.us.a", 0, vpiScalarVal, 1, 0);
    put("top.us.a", 1, vpiScalarVal, 0, 0);
    put("top.us.a", 2, vpiScalarVal, 1, 0);
    put("top.us.a", 3, vpiScalarVal, 1, 0);
    /* The actual, read by `ur`. */
    put("top.r8", 0, vpiIntVal, 50, 0);
    put("top.r8", 1, vpiIntVal, 60, 0);
    {
        /* §38.35: both elements of `ud.a` in one call. */
        PLI_INT32 words[2] = {70, 71}, idx[1] = {0};
        s_vpi_arrayvalue av;
        memset(&av, 0, sizeof av);
        av.format = vpiIntVal;
        av.value.integers = words;
        vpi_put_value_array(get("top.ud.a", -1), &av, idx, 2);
    }
    put("top.ug.a[0][1]", -1, vpiIntVal, 21, 0);
    put("top.ug.a[1][2]", -1, vpiIntVal, 12, 0);
    put("top.un.a", 3, vpiIntVal, 43, 0);
    put("top.un.a", 0, vpiIntVal, 40, 0);
    put("top.um8.a", 3, vpiIntVal, 77, 0);
    put("top.um8.a", 5, vpiIntVal, 88, 0);
    put("top.um64.a", 3, vpiHexStrVal, 0, L3);
    put("top.um64.a", 5, vpiHexStrVal, 0, L5);
    put("top.um1.a", 3, vpiScalarVal, 1, 0);
    put("top.um1.a", 5, vpiScalarVal, 1, 0);
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
