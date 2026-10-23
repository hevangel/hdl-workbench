/* vpiPort / vpiDirection object model — see vpi_ports.sv. */
#include "vpi_user.h"
#include <stdio.h>
#include <string.h>

static int fails;
#define CHECK(c, m) do { if (!(c)) { vpi_printf("VPI_PORTS FAIL: %s\n", m); fails++; } } while (0)

static void collect(int rel, vpiHandle scope, char *buf, size_t n) {
  buf[0] = 0;
  vpiHandle it = vpi_iterate(rel, scope), h;
  while (it && (h = vpi_scan(it))) {
    char one[80];
    snprintf(one, sizeof one, "%s%s:%d:%d:%d", buf[0] ? "," : "", vpi_get_str(vpiName, h),
             vpi_get(vpiType, h), vpi_get(vpiDirection, h), vpi_get(vpiSize, h));
    strncat(buf, one, n - strlen(buf) - 1);
  }
}

static PLI_INT32 probe(PLI_BYTE8 *ud) {
  (void)ud;
  char buf[512];
  vpiHandle mi = vpi_iterate(vpiModule, NULL), top = vpi_scan(mi); vpi_free_object(mi);
  CHECK(top != NULL, "top module handle");

  /* name:type:direction:size — vpiPort=44, vpiInput=1, vpiOutput=2, vpiInout=3.
   * Ports come in port-list order (vpiPortIndex 0, 1, 2). */
  collect(vpiPort, top, buf, sizeof buf);
  CHECK(strcmp(buf, "clk_in:44:1:1,o_top:44:2:4,io_top:44:3:2") == 0, buf);
  /* The connected objects keep their own types. §23.2.2.3: `input logic`
   * (no net type, no `var`) is a net, `output logic` a variable. */
  collect(vpiNet, top, buf, sizeof buf);
  CHECK(strstr(buf, "w:36:") != NULL && strstr(buf, "clk_in:36:1:1") != NULL, buf);
  collect(vpiReg, top, buf, sizeof buf);
  CHECK(strstr(buf, "o_top:48:2:4") != NULL && strstr(buf, "clk_in") == NULL, buf);

  vpiHandle ii = vpi_iterate(vpiModule, top), sub = vpi_scan(ii); vpi_free_object(ii);
  CHECK(sub != NULL, "sub-instance handle");
  collect(vpiPort, sub, buf, sizeof buf);
  CHECK(strcmp(buf, "clk:44:1:1,i:44:1:4,o:44:2:4,io:44:3:2") == 0, buf);
  collect(vpiReg, sub, buf, sizeof buf);
  CHECK(strstr(buf, "internal:48:5:4") != NULL, buf);   /* a plain variable: vpiNoDirection */

  /* a port handle reads the connected signal's value and names it fully */
  vpiHandle pi = vpi_iterate(vpiPort, sub), ph;
  int seen_i = 0;
  while ((ph = vpi_scan(pi))) {
    if (strcmp(vpi_get_str(vpiName, ph), "i") == 0) {
      s_vpi_value v; v.format = vpiIntVal; vpi_get_value(ph, &v);
      CHECK(v.value.integer == 5, "port i value");
      CHECK(strcmp(vpi_get_str(vpiFullName, ph), "tb.u_sub.i") == 0, vpi_get_str(vpiFullName, ph));
      CHECK(strcmp(vpi_get_str(vpiType, ph), "vpiPort") == 0, "vpiType name");
      seen_i = 1;
    }
  }
  CHECK(seen_i, "port i found");

  /* by-name lookup still yields the net, which reports its port's direction */
  vpiHandle clk = vpi_handle_by_name("tb.u_sub.clk", NULL);
  CHECK(clk && vpi_get(vpiType, clk) == vpiNet && vpi_get(vpiDirection, clk) == vpiInput, "by-name clk");

  vpi_printf(fails ? "VPI_PORTS: %d failure(s)\n" : "VPI_PORTS: all checks passed\n", fails);
  return 0;
}

static void reg(void) {
  s_vpi_systf_data d; memset(&d, 0, sizeof d);
  d.type = vpiSysTask; d.tfname = "$ports_probe"; d.calltf = probe;
  vpi_register_systf(&d);
}
void (*vlog_startup_routines[])(void) = { reg, 0 };
