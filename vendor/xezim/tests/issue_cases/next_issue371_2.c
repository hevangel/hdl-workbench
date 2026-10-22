// Drives top.clk with immediate (vpiNoDelay) writes every 5 time units and calls
// vpi_control(vpiFinish) from the cbValueChange callback of the third rising
// edge. vpiFinish runs $finish when the callback returns (IEEE 1800-2017 38.4),
// so the processes that edge wakes must not run: top.n ends at 2.
#include <stddef.h>
#include <vpi_user.h>

static vpiHandle clk;
static int rises;

static PLI_INT32 toggle(p_cb_data cb);

static void after(PLI_UINT32 d) {
  static s_vpi_time t;
  t.type = vpiSimTime; t.high = 0; t.low = d;
  s_cb_data c = {0};
  c.reason = cbAfterDelay; c.cb_rtn = toggle; c.time = &t;
  vpi_register_cb(&c);
}

static PLI_INT32 toggle(p_cb_data cb) {
  (void)cb;
  s_vpi_value v = {vpiIntVal};
  vpi_get_value(clk, &v);
  v.value.integer = !v.value.integer;
  vpi_put_value(clk, &v, NULL, vpiNoDelay);
  after(5);
  return 0;
}

static PLI_INT32 change(p_cb_data cb) {
  if (cb->value->value.integer == 1 && ++rises == 3) {
    vpi_printf("rising edge %d at %u: vpi_control(vpiFinish)\n", rises, cb->time->low);
    vpi_control(vpiFinish, 0);
  }
  return 0;
}

static PLI_INT32 start(p_cb_data cb) {
  (void)cb;
  static s_vpi_time t = {vpiSimTime};
  static s_vpi_value v = {vpiIntVal};
  clk = vpi_handle_by_name("top.clk", NULL);
  s_cb_data c = {0};
  c.reason = cbValueChange; c.cb_rtn = change; c.obj = clk; c.time = &t; c.value = &v;
  vpi_register_cb(&c);
  after(5);
  return 0;
}

static void setup(void) {
  s_cb_data c = {0};
  c.reason = cbStartOfSimulation; c.cb_rtn = start;
  vpi_register_cb(&c);
}

void (*vlog_startup_routines[])(void) = {setup, 0};
