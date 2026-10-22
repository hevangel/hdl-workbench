#ifndef SVDPI_H
#define SVDPI_H

/* SystemVerilog DPI-C header (IEEE 1800-2017 clause 35 and Annex H) for
 * xezim: the canonical types, the scope and time primitives, the bit and
 * part-select helpers, the open-array queries and element access, per-scope
 * user data, svGetCallerInfo and the disable protocol.
 *
 * Open arrays: an open-array handle points at the element data, laid out in
 * ascending index order in each element's canonical C form, so code that
 * indexes it as a plain C array (`int *p = (int *)h;`) works as well as the
 * svSize/svGetArrElemPtr1/... calls. One unpacked dimension is passed.
 */

#include <stdint.h>
#include <limits.h>

/* s_vpi_vecval is declared in vpi_user.h (IEEE 1800 section 38.25).
 * svdpi.h needs it for svLogicVecVal below. Pulling vpi_user.h in
 * here keeps the dependency explicit instead of relying on
 * ordering between user #include directives. */
#include "vpi_user.h"

/* svBitVecVal - unsigned 32-bit vector element used for svBit types. */
typedef uint32_t svBitVecVal;

/* svLogicVecVal - 4-state vector element. IEEE 1800 section 35.5.5
 * specifies the layout as identical to s_vpi_vecval (aval/bval);
 * UVM source files do plain assignment between the two, so we
 * typedef to keep type compatibility without per-field translation. */
typedef s_vpi_vecval svLogicVecVal;

typedef svLogicVecVal* p_svLogicVecVal;

/* Canonical scalar types - IEEE 1800-2017 section 35.5.6.1. A single
 * `bit` maps to svBit, a single `logic`/`reg` to svLogic; both are a
 * byte-wide value. */
typedef uint8_t svScalar;
typedef svScalar svBit;    /* 2-state: sv_0 / sv_1 */
typedef svScalar svLogic;  /* 4-state: sv_0 / sv_1 / sv_z / sv_x */

/* Scalar bit values (section 35.5.6.1). */
#ifndef sv_0
#define sv_0 0
#define sv_1 1
#define sv_z 2
#define sv_x 3
#endif
/* Alternate spelling used by some codebases. */
#ifndef sv_b_0
#define sv_b_0 sv_0
#define sv_b_1 sv_1
#define sv_b_z sv_z
#define sv_b_x sv_x
#endif

/* Open array types */
typedef struct svOpenArrayType {
    void* dhandle;
    void* dptr;
    int dims[16];
    int static_size[16];
} svOpenArrayType;

typedef void* svOpenArrayHandle;

/* svScope - opaque scope handle. Pointer-only; never dereferenced
 * from C. Forward-declared so callers can store and pass it. */
typedef struct SVScopePlaceholder *svScope;

/* Symbol visibility for export functions. */
#define SV_PUBLIC __attribute__((visibility("default")))

/* DPI context function attribute. C and C++ see different syntax:
 *   C++ : extern "C" __attribute__((visibility("default")))
 *   C   : __attribute__((visibility("default"))) only
 * Both UVM `uvm_common.c` (C) and `uvm_dpi.cc` (C++) need this to
 * compile cleanly against the same header. */
#ifdef __cplusplus
#define DPI_CONTEXT extern "C" SV_PUBLIC
#else
#define DPI_CONTEXT SV_PUBLIC
#endif

/* DPI version query - IEEE 1800 section 35.7. Returns the DPI
 * standard revision as a string ("1800-2005", "1800-2009", etc.).
 * UVM's m_uvm_report_dpi calls this at startup to confirm the
 * simulator supports the DPI version UVM expects. */
DPI_CONTEXT const char *svDpiVersion(void);

/* Scope primitives - IEEE 1800 section 36.6. svSetScope returns the
 * previously-active scope so callers can save/restore the stack:
 *
 *     svScope prev = svSetScope(my_scope);
 *     ... do work ...
 *     svSetScope(prev);
 *
 * IEEE 1800-2005 declared `void svSetScope(svScope)` but 1800-2009+
 * added the svScope return. UVM 1.2 (which calls this pattern) was
 * written against the 1800-2009+ behavior. */
DPI_CONTEXT svScope svGetScopeFromName(const char *scope_name);
DPI_CONTEXT const char *svGetNameFromScope(svScope scope);
DPI_CONTEXT svScope svGetScope(void);
DPI_CONTEXT svScope svSetScope(svScope scope);

/* Simulation time for DPI code. A scope naming a module or instance answers
 * with that module's timescale; NULL answers with the simulation's (its
 * finest precision). Units are powers of ten in seconds (-9 = 1 ns).
 * svGetTime fills `high`/`low` with simulation ticks when time->type is
 * vpiSimTime, or `real` in the scope's time unit when it is
 * vpiScaledRealTime. Each returns 0, or -1 on a NULL output pointer. */
typedef s_vpi_time svTimeVal;
DPI_CONTEXT int svGetTime(const svScope scope, svTimeVal *time);
DPI_CONTEXT int svGetTimeUnit(const svScope scope, int32_t *time_unit);
DPI_CONTEXT int svGetTimePrecision(const svScope scope, int32_t *time_precision);

/* Number of 32-bit chunks a packed vector of `WIDTH` bits occupies (H.7.6). */
#ifndef SV_PACKED_DATA_NELEMS
#define SV_PACKED_DATA_NELEMS(WIDTH) (((WIDTH) + 31) >> 5)
#endif
#define SV_MASK(N) \
    ((N) >= 32 ? UINT32_MAX : ((UINT32_C(1) << (N)) - UINT32_C(1)))
#define SV_GET_UNSIGNED_BITS(VALUE, N) \
    ((N) == 32 ? (VALUE) : ((VALUE) & SV_MASK(N)))
#define SV_GET_SIGNED_BITS(VALUE, N) \
    ((N) == 32 ? (VALUE) : \
     (((VALUE) & (UINT32_C(1) << ((N) - 1))) ? \
      ((VALUE) | ~SV_MASK(N)) : ((VALUE) & SV_MASK(N))))

/* H.10.1: bit selects and part selects of canonical vectors. A part select
 * of `w` bits (1..32) starts at bit `i` and lands in bits [w-1:0]. */
DPI_CONTEXT svBit svGetBitselBit(const svBitVecVal *s, int i);
DPI_CONTEXT svLogic svGetBitselLogic(const svLogicVecVal *s, int i);
DPI_CONTEXT void svPutBitselBit(svBitVecVal *d, int i, svBit s);
DPI_CONTEXT void svPutBitselLogic(svLogicVecVal *d, int i, svLogic s);
DPI_CONTEXT void svGetPartselBit(svBitVecVal *d, const svBitVecVal *s, int i, int w);
DPI_CONTEXT void svGetPartselLogic(svLogicVecVal *d, const svLogicVecVal *s, int i, int w);
DPI_CONTEXT void svPutPartselBit(svBitVecVal *d, const svBitVecVal s, int i, int w);
DPI_CONTEXT void svPutPartselLogic(svLogicVecVal *d, const svLogicVecVal s, int i, int w);

/* H.12.2: open-array queries. Dimension 1 is the (first) unpacked
 * dimension; dimension 0 is a packed element's range. */
DPI_CONTEXT int svLeft(const svOpenArrayHandle h, int d);
DPI_CONTEXT int svRight(const svOpenArrayHandle h, int d);
DPI_CONTEXT int svLow(const svOpenArrayHandle h, int d);
DPI_CONTEXT int svHigh(const svOpenArrayHandle h, int d);
DPI_CONTEXT int svIncrement(const svOpenArrayHandle h, int d);
DPI_CONTEXT int svSize(const svOpenArrayHandle h, int d);
DPI_CONTEXT int svDimensions(const svOpenArrayHandle h);
DPI_CONTEXT void *svGetArrayPtr(const svOpenArrayHandle h);
DPI_CONTEXT int svSizeOfArray(const svOpenArrayHandle h);

/* H.12.3: a pointer to one element, by its SV index; NULL when out of
 * range. */
DPI_CONTEXT void *svGetArrElemPtr(const svOpenArrayHandle h, int indx1, ...);
DPI_CONTEXT void *svGetArrElemPtr1(const svOpenArrayHandle h, int indx1);
DPI_CONTEXT void *svGetArrElemPtr2(const svOpenArrayHandle h, int indx1, int indx2);
DPI_CONTEXT void *svGetArrElemPtr3(const svOpenArrayHandle h, int indx1, int indx2, int indx3);

/* H.12.4/H.12.5: copy a packed element to or from canonical form. */
DPI_CONTEXT void svPutBitArrElemVecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, ...);
DPI_CONTEXT void svPutBitArrElem1VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1);
DPI_CONTEXT void svPutBitArrElem2VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, int indx2);
DPI_CONTEXT void svPutBitArrElem3VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, int indx2, int indx3);
DPI_CONTEXT void svPutLogicArrElemVecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, ...);
DPI_CONTEXT void svPutLogicArrElem1VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1);
DPI_CONTEXT void svPutLogicArrElem2VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, int indx2);
DPI_CONTEXT void svPutLogicArrElem3VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, int indx2, int indx3);
DPI_CONTEXT void svGetBitArrElemVecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, ...);
DPI_CONTEXT void svGetBitArrElem1VecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1);
DPI_CONTEXT void svGetBitArrElem2VecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, int indx2);
DPI_CONTEXT void svGetBitArrElem3VecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, int indx2, int indx3);
DPI_CONTEXT void svGetLogicArrElemVecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, ...);
DPI_CONTEXT void svGetLogicArrElem1VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1);
DPI_CONTEXT void svGetLogicArrElem2VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, int indx2);
DPI_CONTEXT void svGetLogicArrElem3VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, int indx2, int indx3);

/* H.12.6: scalar (single bit/logic) elements. */
DPI_CONTEXT svBit svGetBitArrElem(const svOpenArrayHandle s, int indx1, ...);
DPI_CONTEXT svBit svGetBitArrElem1(const svOpenArrayHandle s, int indx1);
DPI_CONTEXT svBit svGetBitArrElem2(const svOpenArrayHandle s, int indx1, int indx2);
DPI_CONTEXT svBit svGetBitArrElem3(const svOpenArrayHandle s, int indx1, int indx2, int indx3);
DPI_CONTEXT svLogic svGetLogicArrElem(const svOpenArrayHandle s, int indx1, ...);
DPI_CONTEXT svLogic svGetLogicArrElem1(const svOpenArrayHandle s, int indx1);
DPI_CONTEXT svLogic svGetLogicArrElem2(const svOpenArrayHandle s, int indx1, int indx2);
DPI_CONTEXT svLogic svGetLogicArrElem3(const svOpenArrayHandle s, int indx1, int indx2, int indx3);
DPI_CONTEXT void svPutBitArrElem(const svOpenArrayHandle d, svBit value, int indx1, ...);
DPI_CONTEXT void svPutBitArrElem1(const svOpenArrayHandle d, svBit value, int indx1);
DPI_CONTEXT void svPutBitArrElem2(const svOpenArrayHandle d, svBit value, int indx1, int indx2);
DPI_CONTEXT void svPutBitArrElem3(const svOpenArrayHandle d, svBit value, int indx1, int indx2, int indx3);
DPI_CONTEXT void svPutLogicArrElem(const svOpenArrayHandle d, svLogic value, int indx1, ...);
DPI_CONTEXT void svPutLogicArrElem1(const svOpenArrayHandle d, svLogic value, int indx1);
DPI_CONTEXT void svPutLogicArrElem2(const svOpenArrayHandle d, svLogic value, int indx1, int indx2);
DPI_CONTEXT void svPutLogicArrElem3(const svOpenArrayHandle d, svLogic value, int indx1, int indx2, int indx3);

/* H.9.3: data a C library attaches to a scope under a key of its own.
 * svPutUserData returns 0, or -1 for a NULL scope or key. */
DPI_CONTEXT int svPutUserData(const svScope scope, void *userKey, void *userData);
DPI_CONTEXT void *svGetUserData(const svScope scope, void *userKey);

/* H.9.4: the calling statement's file and line; xezim returns 0 (not
 * available). */
DPI_CONTEXT int svGetCallerInfo(const char **fileName, int *lineNumber);

/* H.9.5: the disable protocol; xezim never disables an import mid-call. */
DPI_CONTEXT int svIsDisabledState(void);
DPI_CONTEXT void svAckDisabledState(void);

/* Compatibility marker for tools that test which DPI standard we
 * expose. The 1800-2005 value 0 is the UVM-required minimum. */
#ifndef DPI_COMPATIBILITY_VERSION_1800_2005
#define DPI_COMPATIBILITY_VERSION_1800_2005  0
#endif

#endif /* SVDPI_H */
