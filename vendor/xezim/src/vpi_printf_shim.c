/* vpi_printf / vpi_vprintf — IEEE 1800-2017 section 38.34.
 *
 * These live in C rather than Rust because defining a C-variadic function
 * requires the unstable `c_variadic` feature. They are trivial, so the
 * shim stays trivial: forward to vprintf and flush, since VPI output is
 * expected to interleave with $display.
 *
 * Compiled and linked by build.rs.
 */
#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>

int vpi_vprintf(char *format, va_list ap) {
    int n = vprintf(format, ap);
    fflush(stdout);
    return n;
}

int vpi_printf(char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int n = vprintf(format, ap);
    va_end(ap);
    fflush(stdout);
    return n;
}

/* vpi_mcd_printf / vpi_mcd_vprintf: bit 0 of the descriptor is stdout, printed
 * here like vpi_printf; the other bits are files opened with vpi_mcd_open or
 * $fopen, written by the Rust backend through the same channel table. */
extern int xezim_vpi_mcd_write(unsigned int mcd, const char *buf, int len);

int vpi_mcd_vprintf(unsigned int mcd, char *format, va_list ap) {
    int n = 0;
    if (mcd & 1u) {
        va_list out;
        va_copy(out, ap);
        n = vprintf(format, out);
        va_end(out);
        fflush(stdout);
    }
    if (mcd & ~1u) {
        char small[1024];
        va_list sized;
        va_copy(sized, ap);
        int len = vsnprintf(small, sizeof small, format, sized);
        va_end(sized);
        if (len < 0)
            return len;
        if ((size_t)len < sizeof small) {
            xezim_vpi_mcd_write(mcd, small, len);
        } else {
            char *big = malloc((size_t)len + 1);
            if (!big)
                return -1;
            va_list again;
            va_copy(again, ap);
            vsnprintf(big, (size_t)len + 1, format, again);
            va_end(again);
            xezim_vpi_mcd_write(mcd, big, len);
            free(big);
        }
        n = len;
    }
    return n;
}

int vpi_mcd_printf(unsigned int mcd, char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int n = vpi_mcd_vprintf(mcd, format, ap);
    va_end(ap);
    return n;
}

/* vpi_control — IEEE 1800-2017 section 38.14. Variadic, so it lives here too;
 * the operation's optional argument is unpacked and forwarded to the Rust
 * backend: the diagnostic level of vpiStop/vpiFinish, or the scope handle of
 * vpiSetInteractiveScope. vpiReset's three arguments are not read — xezim
 * refuses the operation. */
extern int xezim_vpi_control(int operation, int arg, void *handle);

#define XEZIM_vpiStop                  66
#define XEZIM_vpiFinish                67
#define XEZIM_vpiSetInteractiveScope   69

int vpi_control(int operation, ...) {
    int arg = 0;
    void *handle = 0;
    if (operation == XEZIM_vpiStop || operation == XEZIM_vpiFinish) {
        va_list ap;
        va_start(ap, operation);
        arg = va_arg(ap, int);
        va_end(ap);
    } else if (operation == XEZIM_vpiSetInteractiveScope) {
        va_list ap;
        va_start(ap, operation);
        handle = va_arg(ap, void *);
        va_end(ap);
    }
    return xezim_vpi_control(operation, arg, handle);
}

/* vpi_handle_multi — IEEE 1800-2017 section 38.22. Variadic, so it lives here
 * too. The one relation the standard defines through it, vpiInterModPath,
 * takes exactly two reference handles (an output port and an input port), so
 * only ref1 and ref2 are read: the standard gives no terminator by which a
 * longer list could be found. */
extern void *xezim_vpi_handle_multi(int type, void *ref1, void *ref2);

void *vpi_handle_multi(int type, void *ref1, void *ref2, ...) {
    return xezim_vpi_handle_multi(type, ref1, ref2);
}
