#include <string.h>
#include "vpi_user.h"
static PLI_INT32 put_calltf(PLI_BYTE8 *ud) {
  vpiHandle a = vpi_handle_by_name("t38.a", NULL); s_vpi_value v; s_vpi_time dt;
  v.format = vpiIntVal; v.value.integer = 99; dt.type = vpiSimTime; dt.high = 0; dt.low = 5;
  vpi_put_value(a, &v, &dt, vpiTransportDelay);
  v.value.integer = 55; vpi_put_value(vpi_handle_by_name("t38.b", NULL), &v, &dt, vpiInertialDelay);
  return 0;
}
static void reg(void) { s_vpi_systf_data tf; memset(&tf, 0, sizeof tf); tf.type = vpiSysTask; tf.tfname = "$put_delayed"; tf.calltf = put_calltf; vpi_register_systf(&tf); }
void (*vlog_startup_routines[])(void) = { reg, 0 };
