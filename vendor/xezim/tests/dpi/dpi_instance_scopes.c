/* Exports from instantiated modules (IEEE 1800-2017 sec. 35.5.3): every
 * instance of `unit` exports sv_get_id, sv_tick and sv_where under one C
 * name each; a call runs the copy in the current DPI scope — the context
 * import's instance, or the one svSetScope selects — or, failing that, in
 * the nearest scope above it. Shared by dpi_instance_scopes_test.sv and
 * dpi_instance_scope_missing_test.sv.
 */
#include "svdpi.h"

extern int sv_get_id(void) __attribute__((weak));
extern int sv_tick(int n) __attribute__((weak));
extern void sv_where(void) __attribute__((weak));
extern int sv_top_val(void) __attribute__((weak));

/* The caller's instance's id. */
int c_ask_id(void) { return sv_get_id(); }

/* The id of the instance at `path`, through svSetScope. */
int c_ask_in(const char *path) {
    svScope prev = svSetScope(svGetScopeFromName(path));
    int v = sv_get_id();
    svSetScope(prev);
    return v;
}

/* An imported task in the instance: waits through the instance's exported
 * task, then names the instance through its exported function. */
int c_tick(int n) {
    sv_tick(n);
    sv_where();
    return 0;
}

/* An export of top, called from an instance's scope. */
int c_call_top(void) { return sv_top_val(); }
