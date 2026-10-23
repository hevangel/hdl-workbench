# Scope: `Arc<[Statement]>` for AST block bodies

**Status: SCOPED, NOT STARTED. Do the M0 in §6 first — it can kill this for one
day of work, and the two most recent attempts in this area both measured
negative.**

Prerequisite identified by an earlier offload study, where the
frame-chain rewrite was built, proved correct, and measured **+5% slower**
because it added `Arc::from(vec)` per *splice* to remove a deep clone per
*suspension* — and splices are far more frequent.

---

## 1. The actual cost this targets

`run_process_stmts` executes a synthesized statement list. Whenever it meets a
blocking `begin/end` it does:

```rust
if self.stmts_have_blocking(inner) {
    let mut expanded = inner.clone();          // <-- DEEP CLONE of the block body
    expanded.extend_from_slice(&stmts[i + 1..]);
    self.run_process_stmts(pid, &expanded);
```

`inner` is `&Vec<Statement>` borrowed from the AST, and `Statement` owns boxed
sub-trees, so `inner.clone()` is a full recursive copy. A blocking block inside
a `forever` re-clones its whole body **on every iteration**. Blocking task-call
inlining has the same shape.

That is the cost. The `Statement` payload never changes during a run — it is
re-cloned only because the type is `Vec<Statement>` and the executor needs an
owned list.

## 2. The change

In `xezim-core/xezim-parser/src/ast/stmt.rs`:

```rust
SeqBlock { name: Option<Identifier>, stmts: Arc<[Statement]> },
ParBlock { name: Option<Identifier>, join_type: JoinType, stmts: Arc<[Statement]> },
```

That is the entire type change — **two field declarations.** The splice then
becomes `Arc::clone(inner)`: one refcount bump instead of a recursive copy of
the whole body, per iteration.

## 3. Blast radius (measured, not estimated)

| category | count | note |
|---|---|---|
| AST field declarations | **2** | the change itself |
| Construction sites (`stmts:` in a `SeqBlock`/`ParBlock` literal) | 7 | wrap with `Arc::from(vec)` |
| `for s in stmts` | 27 | `Arc` is not `IntoIterator`; becomes `stmts.iter()` |
| `.clone()` on a matched body | 23 | **the audit** — see §4 |
| Total `SeqBlock`/`ParBlock` match sites | 79 (9 parser / 27 elaborate / 43 sim) | most compile unchanged: `Arc<[T]>` derefs to `[T]`, so `.iter()`, `.len()`, `.first()`, indexing and slicing all still work |
| Mutable AST passes over block bodies | **1** | see §5 |

`Vec<Statement>` totals per crate today: parser 6, xezim-core 4, xezim 42 — but
the great majority are the simulator's own synthesized lists, which stay `Vec`.
Only the two AST fields change type.

## 4. The one real hazard: `.clone()` changes meaning

`stmts.clone()` currently produces an INDEPENDENT deep copy. After the change it
produces a SHARED handle. For the 10 sites in `simulator.rs` that is exactly the
point. For the 13 sites in the parser and elaborator it must be checked
individually: any of them that clones a body in order to then MUTATE the copy
becomes a silent aliasing bug — the mutation would be visible through every
other holder.

This is the whole review burden of the change. It is 23 sites, each a
two-line judgement, and it is mechanical but must not be skipped or scripted.

`Arc::make_mut` is **not** available for `Arc<[T]>`, so the compiler will not
catch a would-be mutation for you at those sites — it will simply refuse to
compile the mutation at all, which is the good case. The bad case is code that
clones, does not mutate, and relies on identity somewhere downstream.

## 5. The one mutable pass

`xezim-core/src/elaborate.rs:8030 rewrite_stmt_delays(stmt: &mut Statement, ...)`
— per-module timescale rewriting (§3.14). It recurses with
`for s in stmts.iter_mut()`, which `Arc<[Statement]>` cannot provide.

Fix: rebuild instead of mutating.

```rust
StatementKind::SeqBlock { stmts, .. } | StatementKind::ParBlock { stmts, .. } => {
    let mut v = stmts.to_vec();
    for s in &mut v { rewrite_stmt_delays(s, unit_s, tick_s); }
    *stmts = Arc::from(v);
}
```

It runs ONCE per module at elaboration and never on the hot path, so the extra
copy is irrelevant. It is also the only such pass in the tree — grep for
`&mut Statement` / `&mut Vec<Statement>` returns exactly this one function.

## 6. M0 — do this before writing any of it (~1 day, decisive)

Do NOT start from the type change. Two of the last three "obviously hot"
hypotheses in this tree measured negative *after* being fully built, and §6b
measured negative specifically because a cost was mis-attributed.

**M0a — attribute the splice clone directly.** Add an env-gated counter for
statements cloned at splice sites (`inner.clone()` and the task-inline path),
then multiply by the per-statement clone cost. The derived figure from §6 is
~0.35 us/statement, which on `cont_pre_100` (20,000 iterations x ~101 statements
cloned) predicts ~700 ms of its 3,119 ms — **~22%**. Confirm that from the
counter rather than trusting the multiplication.

**M0b — inflation probe.** Add a padding field to `Statement` so cloning it is
measurably more expensive, and check that runtime moves in proportion to the
splice count from M0a. This probe has discriminated correctly every time it has
been used in this tree; if runtime does NOT move with clone cost, the splice
clone is not the bottleneck and this whole project dies here.

**Kill criterion:** if M0b shows runtime is not sensitive to `Statement` clone
cost, stop. Expected upside if green: ~20% on suspension-heavy testbench loads.

## 7. Honest bound on the payoff

This is a **testbench/UVM lever, not a DUT lever.** On c906 the whole `process`
phase is 1,460 ms of a 2,646 ms loop, and that is dominated by one-time
memory-init `initial` blocks, not by re-splicing loops. Expect ~0 on
DUT-heavy RTL, which is also what §6b measured (c906 neutral, 400-process load
+5%).

So the benchmark that decides this is **not** c906. It needs a UVM testbench, or
failing that the `cont_*` / `many_procs` synthetics — and those synthetics should
be treated as an upper bound, since a real testbench does more non-splice work
per iteration.

## 8. Sequencing

1. **M0a + M0b** (§6). Kill or proceed.
2. Add `rc` to `xezim-parser`'s serde feature list — `Arc<[T]>` round-trips
   through bincode (verified with a standalone probe), but only with
   `serde/rc`, and `xezim-parser` declares serde WITHOUT it today. Cargo
   feature unification hides this while building the workspace (because `xezim`
   enables it) and it breaks the moment `xezim-parser` is built or tested alone.
   Do this first so the design cache is never the thing that surprises you.
3. The two field types + 7 construction sites + 27 iteration sites (mechanical).
4. The 23 `.clone()` sites (§4) — by hand, one at a time.
5. `rewrite_stmt_delays` (§5).
6. Gate: full suite, plus counter-identity on c906 AND c910, plus a design-cache
   round-trip (write then warm-start read) since the cached type changed.
7. Only then revisit the frame chain from §6b — with shared bodies, its
   `Arc::from` per splice disappears and the +5% it cost should invert.
