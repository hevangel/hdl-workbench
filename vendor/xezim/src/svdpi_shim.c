/* IEEE 1800-2017 Annex H: the DPI-C utility functions that operate on data
 * in C memory - bit/part selects of canonical vectors, the open-array
 * queries and element access, per-scope user data and svGetCallerInfo.
 *
 * They live in C because several are C-variadic (svGetArrElemPtr,
 * svPutBitArrElemVecVal, ...), which Rust cannot define on stable. Compiled
 * and linked by build.rs next to vpi_printf_shim.c.
 *
 * OPEN ARRAYS. xezim passes an open-array argument as a pointer to the
 * element data (so code that indexes the handle as a plain C array keeps
 * working), with an `xz_oa_hdr` placed immediately BEFORE the data. The
 * header layout is mirrored by `DpiOpenArrayHeader` in simulator.rs. The
 * data holds the elements in ascending SV index order, each element
 * `elem_bytes` long in its canonical C form (H.7).
 */
#include <stdint.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

typedef uint32_t svBitVecVal;
typedef struct { uint32_t aval, bval; } svLogicVecVal;
typedef uint8_t svScalar;
typedef svScalar svBit;
typedef svScalar svLogic;
typedef void *svOpenArrayHandle;
typedef void *svScope;

#define sv_0 0
#define sv_1 1
#define sv_z 2
#define sv_x 3

#define XZ_OA_MAGIC 0x414f5a58u /* "XZOA" */
#define XZ_OA_MAX_DIMS 4

typedef struct {
    uint32_t magic;
    uint32_t elem_bytes;  /* bytes per element in the data              */
    int32_t ndims;        /* unpacked dimensions described below          */
    int32_t packed;       /* 1 when the element is a packed bit/logic vec */
    int32_t packed_left;  /* dimension 0, for a packed element            */
    int32_t packed_right;
    int32_t left[XZ_OA_MAX_DIMS];
    int32_t right[XZ_OA_MAX_DIMS];
    uint32_t total_bytes; /* bytes of element data                        */
    uint32_t elem_count;  /* elements in the data (0 for an empty array)  */
} xz_oa_hdr;              /* 64 bytes; data follows, 8-byte aligned       */

static const xz_oa_hdr *oa_hdr(const svOpenArrayHandle h) {
    if (!h) return NULL;
    const xz_oa_hdr *hd = (const xz_oa_hdr *)((const char *)h - sizeof(xz_oa_hdr));
    return hd->magic == XZ_OA_MAGIC ? hd : NULL;
}

static int imin(int a, int b) { return a < b ? a : b; }
static int imax(int a, int b) { return a > b ? a : b; }

/* ---- H.10.1: bit selects and part selects of canonical vectors -------- */

svBit svGetBitselBit(const svBitVecVal *s, int i) {
    return (svBit)((s[i / 32] >> (i % 32)) & 1u);
}

svLogic svGetBitselLogic(const svLogicVecVal *s, int i) {
    uint32_t a = (s[i / 32].aval >> (i % 32)) & 1u;
    uint32_t b = (s[i / 32].bval >> (i % 32)) & 1u;
    return (svLogic)(b ? (a ? sv_x : sv_z) : a);
}

void svPutBitselBit(svBitVecVal *d, int i, svBit s) {
    uint32_t m = 1u << (i % 32);
    d[i / 32] = (s & 1u) ? (d[i / 32] | m) : (d[i / 32] & ~m);
}

void svPutBitselLogic(svLogicVecVal *d, int i, svLogic s) {
    uint32_t m = 1u << (i % 32);
    uint32_t a = (s == sv_1 || s == sv_x) ? m : 0u;
    uint32_t b = (s == sv_z || s == sv_x) ? m : 0u;
    d[i / 32].aval = (d[i / 32].aval & ~m) | a;
    d[i / 32].bval = (d[i / 32].bval & ~m) | b;
}

/* d[0 +: w] = s[i +: w], w <= 32 */
void svGetPartselBit(svBitVecVal *d, const svBitVecVal *s, int i, int w) {
    uint32_t v = 0;
    for (int k = 0; k < w; k++) v |= (uint32_t)svGetBitselBit(s, i + k) << k;
    *d = v;
}

void svGetPartselLogic(svLogicVecVal *d, const svLogicVecVal *s, int i, int w) {
    uint32_t a = 0, b = 0;
    for (int k = 0; k < w; k++) {
        a |= ((s[(i + k) / 32].aval >> ((i + k) % 32)) & 1u) << k;
        b |= ((s[(i + k) / 32].bval >> ((i + k) % 32)) & 1u) << k;
    }
    d->aval = a;
    d->bval = b;
}

/* d[i +: w] = s, w <= 32 */
void svPutPartselBit(svBitVecVal *d, const svBitVecVal s, int i, int w) {
    for (int k = 0; k < w; k++) svPutBitselBit(d, i + k, (svBit)((s >> k) & 1u));
}

void svPutPartselLogic(svLogicVecVal *d, const svLogicVecVal s, int i, int w) {
    for (int k = 0; k < w; k++) {
        uint32_t m = 1u << ((i + k) % 32);
        svLogicVecVal *t = &d[(i + k) / 32];
        t->aval = (t->aval & ~m) | (((s.aval >> k) & 1u) ? m : 0u);
        t->bval = (t->bval & ~m) | (((s.bval >> k) & 1u) ? m : 0u);
    }
}

/* ---- H.12.2: open-array queries ----------------------------------------
 * Dimension d: 0 is the packed dimension of a bit/logic-vector element,
 * 1..n the unpacked ones. Out-of-range queries answer 0 (svSize -1 would be
 * as defensible; 0 never sends a caller's loop past the data).            */

static int oa_bounds(const xz_oa_hdr *hd, int d, int *l, int *r) {
    if (!hd) return 0;
    if (d == 0) {
        if (!hd->packed) return 0;
        *l = hd->packed_left;
        *r = hd->packed_right;
        return 1;
    }
    if (d < 1 || d > hd->ndims) return 0;
    *l = hd->left[d - 1];
    *r = hd->right[d - 1];
    return 1;
}

int svLeft(const svOpenArrayHandle h, int d) {
    int l, r;
    return oa_bounds(oa_hdr(h), d, &l, &r) ? l : 0;
}
int svRight(const svOpenArrayHandle h, int d) {
    int l, r;
    return oa_bounds(oa_hdr(h), d, &l, &r) ? r : 0;
}
/* An empty dynamic array or queue is the range [0:-1]: low 0, high -1 (as
 * $low/$high answer, 20.7), so `for (i = svLow; i <= svHigh; i++)` visits
 * nothing; it counts as ascending, like any [0:n-1] range. */
static int oa_empty(const xz_oa_hdr *hd, int d) {
    return d >= 1 && hd->elem_count == 0;
}
int svLow(const svOpenArrayHandle h, int d) {
    const xz_oa_hdr *hd = oa_hdr(h);
    int l, r;
    if (!oa_bounds(hd, d, &l, &r)) return 0;
    return oa_empty(hd, d) ? l : imin(l, r);
}
int svHigh(const svOpenArrayHandle h, int d) {
    const xz_oa_hdr *hd = oa_hdr(h);
    int l, r;
    if (!oa_bounds(hd, d, &l, &r)) return 0;
    return oa_empty(hd, d) ? r : imax(l, r);
}
/* $increment: 1 when left >= right, -1 otherwise */
int svIncrement(const svOpenArrayHandle h, int d) {
    const xz_oa_hdr *hd = oa_hdr(h);
    int l, r;
    if (!oa_bounds(hd, d, &l, &r)) return 0;
    return oa_empty(hd, d) ? -1 : (l >= r ? 1 : -1);
}
int svSize(const svOpenArrayHandle h, int d) {
    const xz_oa_hdr *hd = oa_hdr(h);
    int l, r;
    if (!oa_bounds(hd, d, &l, &r)) return 0;
    if (oa_empty(hd, d)) return 0;
    return imax(l, r) - imin(l, r) + 1;
}
int svDimensions(const svOpenArrayHandle h) {
    const xz_oa_hdr *hd = oa_hdr(h);
    return hd ? hd->ndims + hd->packed : 0;
}

void *svGetArrayPtr(const svOpenArrayHandle h) {
    return oa_hdr(h) ? (void *)h : NULL;
}
int svSizeOfArray(const svOpenArrayHandle h) {
    const xz_oa_hdr *hd = oa_hdr(h);
    return hd ? (int)hd->total_bytes : 0;
}

/* Element pointer for SV indices idx[0..n): NULL when n is not the number
 * of unpacked dimensions or an index is out of range. Row-major over the
 * dimensions, each in ascending index order. */
static void *oa_elem(const svOpenArrayHandle h, int n, const int *idx) {
    const xz_oa_hdr *hd = oa_hdr(h);
    if (!hd || n != hd->ndims || hd->elem_count == 0) return NULL;
    size_t off = 0;
    for (int k = 0; k < n; k++) {
        int lo = imin(hd->left[k], hd->right[k]);
        int hi = imax(hd->left[k], hd->right[k]);
        if (idx[k] < lo || idx[k] > hi) return NULL;
        off = off * (size_t)(hi - lo + 1) + (size_t)(idx[k] - lo);
    }
    return (char *)h + off * hd->elem_bytes;
}

static int oa_collect(const svOpenArrayHandle h, int first, va_list ap, int *idx) {
    const xz_oa_hdr *hd = oa_hdr(h);
    int n = hd ? hd->ndims : 1;
    if (n < 1 || n > XZ_OA_MAX_DIMS) return 0;
    idx[0] = first;
    for (int k = 1; k < n; k++) idx[k] = va_arg(ap, int);
    return n;
}

void *svGetArrElemPtr(const svOpenArrayHandle h, int indx1, ...) {
    int idx[XZ_OA_MAX_DIMS];
    va_list ap;
    va_start(ap, indx1);
    int n = oa_collect(h, indx1, ap, idx);
    va_end(ap);
    return n ? oa_elem(h, n, idx) : NULL;
}
void *svGetArrElemPtr1(const svOpenArrayHandle h, int indx1) {
    int idx[1] = {indx1};
    return oa_elem(h, 1, idx);
}
void *svGetArrElemPtr2(const svOpenArrayHandle h, int indx1, int indx2) {
    int idx[2] = {indx1, indx2};
    return oa_elem(h, 2, idx);
}
void *svGetArrElemPtr3(const svOpenArrayHandle h, int indx1, int indx2, int indx3) {
    int idx[3] = {indx1, indx2, indx3};
    return oa_elem(h, 3, idx);
}

/* ---- H.12.4/H.12.5: copying canonical elements in and out --------------- */

static int oa_words(const svOpenArrayHandle h) {
    const xz_oa_hdr *hd = oa_hdr(h);
    if (!hd) return 0;
    int w = imax(hd->packed_left, hd->packed_right) - imin(hd->packed_left, hd->packed_right) + 1;
    return hd->packed ? (w + 31) / 32 : (int)(hd->elem_bytes / 4);
}

static void put_bit_vec(void *e, const svOpenArrayHandle h, const svBitVecVal *s) {
    if (e) memcpy(e, s, (size_t)oa_words(h) * sizeof(svBitVecVal));
}
static void get_bit_vec(svBitVecVal *d, const void *e, const svOpenArrayHandle h) {
    if (e) memcpy(d, e, (size_t)oa_words(h) * sizeof(svBitVecVal));
}
static void put_logic_vec(void *e, const svOpenArrayHandle h, const svLogicVecVal *s) {
    if (e) memcpy(e, s, (size_t)oa_words(h) * sizeof(svLogicVecVal));
}
static void get_logic_vec(svLogicVecVal *d, const void *e, const svOpenArrayHandle h) {
    if (e) memcpy(d, e, (size_t)oa_words(h) * sizeof(svLogicVecVal));
}

#define ELEM_VARIADIC(h, first, out)                         \
    void *out;                                               \
    do {                                                     \
        int idx_[XZ_OA_MAX_DIMS];                            \
        va_list ap_;                                         \
        va_start(ap_, first);                                \
        int n_ = oa_collect(h, first, ap_, idx_);            \
        va_end(ap_);                                         \
        out = n_ ? oa_elem(h, n_, idx_) : NULL;              \
    } while (0)

void svPutBitArrElemVecVal(const svOpenArrayHandle d, const svBitVecVal *s, int indx1, ...) {
    ELEM_VARIADIC(d, indx1, e);
    put_bit_vec(e, d, s);
}
void svPutBitArrElem1VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int i1) {
    put_bit_vec(svGetArrElemPtr1(d, i1), d, s);
}
void svPutBitArrElem2VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int i1, int i2) {
    put_bit_vec(svGetArrElemPtr2(d, i1, i2), d, s);
}
void svPutBitArrElem3VecVal(const svOpenArrayHandle d, const svBitVecVal *s, int i1, int i2, int i3) {
    put_bit_vec(svGetArrElemPtr3(d, i1, i2, i3), d, s);
}
void svPutLogicArrElemVecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int indx1, ...) {
    ELEM_VARIADIC(d, indx1, e);
    put_logic_vec(e, d, s);
}
void svPutLogicArrElem1VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int i1) {
    put_logic_vec(svGetArrElemPtr1(d, i1), d, s);
}
void svPutLogicArrElem2VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int i1, int i2) {
    put_logic_vec(svGetArrElemPtr2(d, i1, i2), d, s);
}
void svPutLogicArrElem3VecVal(const svOpenArrayHandle d, const svLogicVecVal *s, int i1, int i2, int i3) {
    put_logic_vec(svGetArrElemPtr3(d, i1, i2, i3), d, s);
}
void svGetBitArrElemVecVal(svBitVecVal *d, const svOpenArrayHandle s, int indx1, ...) {
    ELEM_VARIADIC(s, indx1, e);
    get_bit_vec(d, e, s);
}
void svGetBitArrElem1VecVal(svBitVecVal *d, const svOpenArrayHandle s, int i1) {
    get_bit_vec(d, svGetArrElemPtr1(s, i1), s);
}
void svGetBitArrElem2VecVal(svBitVecVal *d, const svOpenArrayHandle s, int i1, int i2) {
    get_bit_vec(d, svGetArrElemPtr2(s, i1, i2), s);
}
void svGetBitArrElem3VecVal(svBitVecVal *d, const svOpenArrayHandle s, int i1, int i2, int i3) {
    get_bit_vec(d, svGetArrElemPtr3(s, i1, i2, i3), s);
}
void svGetLogicArrElemVecVal(svLogicVecVal *d, const svOpenArrayHandle s, int indx1, ...) {
    ELEM_VARIADIC(s, indx1, e);
    get_logic_vec(d, e, s);
}
void svGetLogicArrElem1VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int i1) {
    get_logic_vec(d, svGetArrElemPtr1(s, i1), s);
}
void svGetLogicArrElem2VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int i1, int i2) {
    get_logic_vec(d, svGetArrElemPtr2(s, i1, i2), s);
}
void svGetLogicArrElem3VecVal(svLogicVecVal *d, const svOpenArrayHandle s, int i1, int i2, int i3) {
    get_logic_vec(d, svGetArrElemPtr3(s, i1, i2, i3), s);
}

/* Scalar (1-bit) elements. An unpacked array of scalar bit / logic holds one
 * svBit / svLogic byte per element (H.7.4); an element of a packed vector
 * array is read and written at its bit 0. */
static int oa_byte_elems(const svOpenArrayHandle h) {
    const xz_oa_hdr *hd = oa_hdr(h);
    return hd && hd->elem_bytes == 1;
}
static svBit oa_get_bit(const svOpenArrayHandle h, const void *e) {
    if (!e) return 0;
    if (oa_byte_elems(h)) return (svBit)(*(const unsigned char *)e & 1u);
    return (svBit)(*(const svBitVecVal *)e & 1u);
}
static svLogic oa_get_logic(const svOpenArrayHandle h, const void *e) {
    if (!e) return sv_x;
    if (oa_byte_elems(h)) return (svLogic)(*(const unsigned char *)e & 3u);
    return svGetBitselLogic((const svLogicVecVal *)e, 0);
}
static void oa_put_bit(const svOpenArrayHandle h, void *e, svBit v) {
    if (!e) return;
    if (oa_byte_elems(h)) *(unsigned char *)e = (unsigned char)(v & 1u);
    else svPutBitselBit((svBitVecVal *)e, 0, v);
}
static void oa_put_logic(const svOpenArrayHandle h, void *e, svLogic v) {
    if (!e) return;
    if (oa_byte_elems(h)) *(unsigned char *)e = (unsigned char)(v & 3u);
    else svPutBitselLogic((svLogicVecVal *)e, 0, v);
}
svBit svGetBitArrElem(const svOpenArrayHandle s, int indx1, ...) {
    ELEM_VARIADIC(s, indx1, e);
    return oa_get_bit(s, e);
}
svBit svGetBitArrElem1(const svOpenArrayHandle s, int i1) {
    return oa_get_bit(s, svGetArrElemPtr1(s, i1));
}
svBit svGetBitArrElem2(const svOpenArrayHandle s, int i1, int i2) {
    return oa_get_bit(s, svGetArrElemPtr2(s, i1, i2));
}
svBit svGetBitArrElem3(const svOpenArrayHandle s, int i1, int i2, int i3) {
    return oa_get_bit(s, svGetArrElemPtr3(s, i1, i2, i3));
}
svLogic svGetLogicArrElem(const svOpenArrayHandle s, int indx1, ...) {
    ELEM_VARIADIC(s, indx1, e);
    return oa_get_logic(s, e);
}
svLogic svGetLogicArrElem1(const svOpenArrayHandle s, int i1) {
    return oa_get_logic(s, svGetArrElemPtr1(s, i1));
}
svLogic svGetLogicArrElem2(const svOpenArrayHandle s, int i1, int i2) {
    return oa_get_logic(s, svGetArrElemPtr2(s, i1, i2));
}
svLogic svGetLogicArrElem3(const svOpenArrayHandle s, int i1, int i2, int i3) {
    return oa_get_logic(s, svGetArrElemPtr3(s, i1, i2, i3));
}
void svPutBitArrElem(const svOpenArrayHandle d, svBit value, int indx1, ...) {
    ELEM_VARIADIC(d, indx1, e);
    oa_put_bit(d, e, value);
}
void svPutBitArrElem1(const svOpenArrayHandle d, svBit value, int i1) {
    oa_put_bit(d, svGetArrElemPtr1(d, i1), value);
}
void svPutBitArrElem2(const svOpenArrayHandle d, svBit value, int i1, int i2) {
    oa_put_bit(d, svGetArrElemPtr2(d, i1, i2), value);
}
void svPutBitArrElem3(const svOpenArrayHandle d, svBit value, int i1, int i2, int i3) {
    oa_put_bit(d, svGetArrElemPtr3(d, i1, i2, i3), value);
}
void svPutLogicArrElem(const svOpenArrayHandle d, svLogic value, int indx1, ...) {
    ELEM_VARIADIC(d, indx1, e);
    oa_put_logic(d, e, value);
}
void svPutLogicArrElem1(const svOpenArrayHandle d, svLogic value, int i1) {
    oa_put_logic(d, svGetArrElemPtr1(d, i1), value);
}
void svPutLogicArrElem2(const svOpenArrayHandle d, svLogic value, int i1, int i2) {
    oa_put_logic(d, svGetArrElemPtr2(d, i1, i2), value);
}
void svPutLogicArrElem3(const svOpenArrayHandle d, svLogic value, int i1, int i2, int i3) {
    oa_put_logic(d, svGetArrElemPtr3(d, i1, i2, i3), value);
}

/* ---- H.9.3: per-scope user data ----------------------------------------
 * A small list keyed by (scope, userKey): svPutUserData replaces, a NULL
 * scope is an error (-1). Process-wide, as the data are C pointers.       */

typedef struct ud_entry { svScope scope; void *key; void *data; struct ud_entry *next; } ud_entry;
static ud_entry *ud_head = NULL;

int svPutUserData(const svScope scope, void *userKey, void *userData) {
    if (!scope || !userKey) return -1;
    for (ud_entry *e = ud_head; e; e = e->next) {
        if (e->scope == scope && e->key == userKey) {
            e->data = userData;
            return 0;
        }
    }
    ud_entry *e = malloc(sizeof *e);
    if (!e) return -1;
    e->scope = scope;
    e->key = userKey;
    e->data = userData;
    e->next = ud_head;
    ud_head = e;
    return 0;
}

void *svGetUserData(const svScope scope, void *userKey) {
    if (!scope || !userKey) return NULL;
    for (ud_entry *e = ud_head; e; e = e->next)
        if (e->scope == scope && e->key == userKey) return e->data;
    return NULL;
}

/* ---- H.9.4 / H.9.5 -------------------------------------------------------
 * svGetCallerInfo: the calling SV statement's file and line are not
 * tracked for DPI calls, so it reports "not available" (0), as the LRM
 * allows. The disable protocol (H.9.5): xezim does not interrupt an
 * imported task or function with `disable`, so a call is never in the
 * disabled state.                                                          */

int svGetCallerInfo(const char **fileName, int *lineNumber) {
    (void)fileName;
    (void)lineNumber;
    return 0;
}

/* svIsDisabledState / svAckDisabledState live with the imported-task
 * machinery in src/compiler/simulator/dpi_task.rs (sec. 35.9). */
