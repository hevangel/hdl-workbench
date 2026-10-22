// IEEE 1800-2017 9.2.3: final procedures run before the end-of-simulation
// callbacks. Records whether the final block has run when cbEndOfSimulation fires.
#include <stddef.h>
#include <vpi_user.h>

static PLI_INT32 eos(p_cb_data cb) {
  (void)cb;
  vpiHandle h = vpi_handle_by_name("top.final_ran", NULL);
  s_vpi_value v = {vpiIntVal};
  vpi_get_value(h, &v);
  vpi_printf("cbEndOfSimulation: top.final_ran = %d\n", (int)v.value.integer);
  vpi_printf("%s\n", v.value.integer == 1 ? "PASS" : "FAIL: cbEndOfSimulation fired before the final block");
  return 0;
}

static void setup(void) {
  s_cb_data cb = {0};
  cb.reason = cbEndOfSimulation;
  cb.cb_rtn = eos;
  vpi_register_cb(&cb);
}

void (*vlog_startup_routines[])(void) = {setup, 0};
