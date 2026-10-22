/* DPI scopes (IEEE 1800-2017 sec. 35.5.3, 36.6): a context import sees
 * the scope of the instance it was called through; svGetScopeFromName /
 * svGetNameFromScope round-trip; svSetScope returns the scope it replaces;
 * and C -> SV -> C recursion through a context import and an export.
 * dpi_instance_exports_test.sv also calls sv_get_id, exported by every
 * instance of `unit`.
 */
#include "svdpi.h"
#include <string.h>
#include <stdio.h>

extern int sv_fact(int n);               /* exported from top */

static char name_buf[256];

/* The caller's scope name. */
const char *c_scope_name(void) {
    snprintf(name_buf, sizeof name_buf, "%s", svGetNameFromScope(svGetScope()));
    return name_buf;
}

/* Look `path` up and name it back; "<null>" when there is no such scope. */
const char *c_lookup(const char *path) {
    svScope s = svGetScopeFromName(path);
    snprintf(name_buf, sizeof name_buf, "%s", s ? svGetNameFromScope(s) : "<null>");
    return name_buf;
}

/* The caller's scope name, through a second import in top. */
const char *c_lookup_self(void) { return c_scope_name(); }

/* Switch to `path`, report the scope now current and the one svSetScope
 * returned, restore, and report the current scope again. */
const char *c_switch(const char *path) {
    char cur[100], prev_name[100];
    svScope prev = svSetScope(svGetScopeFromName(path));
    snprintf(cur, sizeof cur, "%s", svGetNameFromScope(svGetScope()));
    snprintf(prev_name, sizeof prev_name, "%s", svGetNameFromScope(prev));
    svSetScope(prev);
    snprintf(name_buf, sizeof name_buf, "in=%s prev=%s back=%s", cur, prev_name,
             svGetNameFromScope(svGetScope()));
    return name_buf;
}

/* n! through SV: c_fact(n) = n * sv_fact(n - 1), sv_fact(k) = c_fact(k). */
int c_fact(int n) { return n <= 1 ? 1 : n * sv_fact(n - 1); }

/* Per-instance exports (dpi_instance_exports_test.sv). */
extern int sv_get_id(void) __attribute__((weak));
int c_ask_id(void) { return sv_get_id ? sv_get_id() : -1; }
