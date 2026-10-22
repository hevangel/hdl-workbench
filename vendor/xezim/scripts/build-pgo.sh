#!/usr/bin/env bash
# Profile-guided release build: instrument -> train -> rebuild with the
# profile. Measured on a large SoC CoreMark run: simulation loop -15%,
# host instructions -11.7%, cycles -6.1%, output bit-exact.
#
#   usage: scripts/build-pgo.sh [training-command...]
#
# With no arguments the bundled trainer runs: the tests/perf shape designs,
# the scripts/pgo-train designs and the xezim-bench workloads, all against
# the instrumented binaries. Cheap (minutes) and always safe; measured
# about -0.5% host instructions on the SoC benchmarks and -12% on a
# loop-heavy DRAM model. A profile trained on the workload itself reaches
# -14.5% (see README), so pass a training command when you have one.
#
# A training command, when given, is run once against the instrumented
# binary instead and should resemble the workload that matters (a full
# benchmark iteration; an unrepresentative trainer can DEOPTIMIZE the
# paths you care about). Any argument that is the xezim binary path is
# substituted with the instrumented build.
# Requirements: rustup component llvm-tools (for the toolchain-matched
# llvm-profdata — the system one usually version-mismatches and the merge
# fails). The profile degrades gracefully as sources change; rebuild it
# when perf numbers matter.
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")/.."
# The profile format must match the compiler: use the ACTIVE toolchain's
# llvm-profdata (`rustup component add llvm-tools-preview`), never a system
# LLVM of a different version.
SYSROOT=$(rustc --print sysroot)
PROFDATA=$(ls "$SYSROOT"/lib/rustlib/*/bin/llvm-profdata 2>/dev/null | head -1)
[ -n "$PROFDATA" ] || { echo "llvm-profdata not found in $SYSROOT; rustup component add llvm-tools-preview" >&2; exit 2; }
# Feature set of the built binary: default none (interpreter + two-state
# path); PGO_FEATURES="--features jit" for the native-backend build.
FEATURES=${PGO_FEATURES:-}
# Separate target dir so the instrumented and optimized artifacts never
# clobber an ordinary release build.
TDIR=${PGO_TARGET_DIR:-pgo-target}
PDIR=$(mktemp -d)
trap 'rm -rf "$PDIR"' EXIT
echo "== instrumented build =="
RUSTFLAGS="-Cprofile-generate=$PDIR" ./scripts/cargo-local.sh build --release --target-dir "$TDIR" $FEATURES
if [ $# -ge 1 ]; then
  echo "== training: $* =="
  ARGS=()
  for a in "$@"; do
    case "$a" in
      */xezim|xezim) ARGS+=("$TDIR/release/xezim");;
      *) ARGS+=("$a");;
    esac
  done
  "${ARGS[@]}"
else
  echo "== training: bundled designs and workloads =="
  # The test bench is the LAST module in each of these files.
  for d in tests/perf/*.sv; do
    top=$(grep -E '^module' "$d" | tail -1 | sed -E 's/^module +([A-Za-z0-9_]+).*/\1/')
    "$TDIR/release/xezim" --simulate -s "$top" "$d" --no-cache --max-time 2000000 > /dev/null 2>&1 || {
      echo "trainer failed: $d" >&2; exit 1; }
  done
  for d in scripts/pgo-train/*.sv; do
    "$TDIR/release/xezim" --simulate -s tb "$d" --no-cache > /dev/null 2>&1 || {
      echo "trainer failed: $d" >&2; exit 1; }
  done
  "$TDIR/release/xezim-bench" --cycles 200000 > /dev/null 2>&1 || { echo "trainer failed: xezim-bench" >&2; exit 1; }
fi
"$PROFDATA" merge -o "$PDIR/merged.profdata" "$PDIR"/*.profraw
echo "== optimized build =="
RUSTFLAGS="-Cprofile-use=$PDIR/merged.profdata" ./scripts/cargo-local.sh build --release --target-dir "$TDIR" $FEATURES
echo "PGO build complete: $TDIR/release/xezim"
