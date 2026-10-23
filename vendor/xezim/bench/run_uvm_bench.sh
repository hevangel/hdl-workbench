#!/usr/bin/env bash
# UVM procedural-throughput benchmark (B6).
#
# WHY THIS EXISTS. Every other benchmark in bench/ is DUT-shaped: its time goes
# to `settle` and `edges`, i.e. the compiled combinational/edge paths. The UVM
# examples are the opposite — measured on this tree they spend ~98% of the
# simulation loop in `process`, with settle_calls=0, entry_evals=0, insns~0 and
# no edge waiters at all. That is the AST-interpreted procedural path: task
# inlining, blocking begin/end flattening, continuation capture on every
# `#delay`. Nothing else here measures it, which is why two earlier attempts
# at that path had to be judged on synthetics.
#
# So: this is the benchmark to use for procedural/testbench work, and NOT for
# DUT work. On a UVM example `settle` is zero; on c910 `process` is one-time
# memory init. They measure disjoint halves of the simulator.
#
# Usage:
#   ./bench/run_uvm_bench.sh                          # profile one binary
#   ./bench/run_uvm_bench.sh -a OLD -b NEW            # paired A/B, interleaved
#   ./bench/run_uvm_bench.sh -r 5 -t hello_world      # reps / subset
#   ./bench/run_uvm_bench.sh -l                       # list benchmarks
#
# Env: UVM_HOME (default /home/bondan/agent/repo/UVM/1.2), XEZIM (default
# ./target/release/xezim).
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null && pwd -P)"
UVM_HOME="${UVM_HOME:-/home/bondan/agent/repo/UVM/1.2}"
UVM_SRC="$UVM_HOME/src"
EX="$UVM_HOME/examples/simple"
XEZIM_A="${XEZIM:-$SCRIPT_DIR/../target/release/xezim}"
XEZIM_B=""
REPS=3
ONLY=""
LIST_ONLY=0

# Examples verified to run clean (0 UVM_FATAL, no elaboration error) on this
# tree, ordered by how much procedural work they do. `sim_ns` is the simulated
# end time, which is FIXED per example — that is what makes the comparison a
# fixed-work one. Excluded deliberately: factory, registers/*,
# sequence/basic_read_write_sequence and tlm1/bidir all fatal or fail to
# elaborate here, and GettingVerilatorStartedWithUVM ends at t=0 with a [SEQ]
# fatal, so none of them can carry a timing number.
#   name                       sim_ns
BENCHES=(
  "hello_world                 1000"
  "tlm1/hierarchy              1000"
  "tlm1/producer_consumer      1000"
  "tlm1/fifo                   5000"
  "interfaces                   100"
  "phases/basic                 500"
  "objections                    60"
)

usage() { sed -n '2,26p' "$0"; exit "${1:-0}"; }
while [ $# -gt 0 ]; do
  case "$1" in
    -a) XEZIM_A="$2"; shift 2 ;;
    -b) XEZIM_B="$2"; shift 2 ;;
    -r) REPS="$2"; shift 2 ;;
    -t) ONLY="$2"; shift 2 ;;
    -l) LIST_ONLY=1; shift ;;
    -h|--help) usage 0 ;;
    *) echo "unknown option: $1" >&2; usage 1 ;;
  esac
done

if [ "$LIST_ONLY" = 1 ]; then
  printf '%s\n' "${BENCHES[@]}" | awk '{printf "  %-28s sim_ns=%s\n", $1, $2}'
  exit 0
fi

for b in "$XEZIM_A" ${XEZIM_B:+"$XEZIM_B"}; do
  [ -x "$b" ] || { echo "not executable: $b" >&2; exit 1; }
done
[ -r "$UVM_SRC/uvm_pkg.sv" ] || { echo "no uvm_pkg.sv under $UVM_SRC (set UVM_HOME)" >&2; exit 1; }

# One run. Echoes: sim_ms process_ms settle_ms edges_ms waiter_iters end_time fatals
run_one() {
  local bin="$1" dir="$2" out
  out=$(cd "$dir" && XEZIM_PROFILE_TIMING=1 timeout 600 "$bin" --sv2017 \
        -D UVM_NO_DPI -D UVM_REPORT_DISABLE_FILE_LINE -D UVM_ENABLE_DEPRECATED_API \
        "+incdir+$UVM_SRC" "+incdir+$dir" \
        "$UVM_SRC/uvm_pkg.sv" $(ls "$dir"/*.sv | tr '\n' ' ') 2>&1)
  echo "$(grep -oE 'simulation_loop=[0-9.]+' <<<"$out" | grep -oE '[0-9.]+' | tail -1)" \
       "$(grep -oE 'process=[0-9.]+ms' <<<"$out" | grep -oE '[0-9.]+' | tail -1)" \
       "$(grep -oE 'settle=[0-9.]+ms' <<<"$out" | grep -oE '[0-9.]+' | tail -1)" \
       "$(grep -oE 'edges=[0-9.]+ms' <<<"$out" | grep -oE '[0-9.]+' | tail -1)" \
       "$(grep -oE 'waiter_iters=[0-9]+' <<<"$out" | grep -oE '[0-9]+' | tail -1)" \
       "$(grep -oE 'finished at time [0-9]+' <<<"$out" | grep -oE '[0-9]+' | tail -1)" \
       "$(grep -cE 'UVM_FATAL @|^Simulation error|^Error:' <<<"$out")"
}

median() { tr ' ' '\n' <<<"$*" | grep -E '^[0-9.]+$' | sort -g | awk '{a[NR]=$1} END{if(NR)print a[int((NR+1)/2)]}'; }

HOST="$(hostname -s 2>/dev/null || echo host)"
CSV="bench_uvm_${HOST}.csv"
echo "bench,binary,rep,sim_ms,process_ms,settle_ms,edges_ms,waiter_iters,end_time,fatals" > "$CSV"

if [ -n "$XEZIM_B" ]; then
  echo "UVM procedural benchmark — paired A/B, $REPS reps, interleaved"
  echo "  A=$XEZIM_A"
  echo "  B=$XEZIM_B"
  echo
  printf "%-26s %11s %11s %9s  %s\n" "bench" "A sim_ms" "B sim_ms" "delta" "verdict"
else
  echo "UVM procedural benchmark — $REPS reps"
  echo "  binary=$XEZIM_A"
  echo
  printf "%-26s %10s %10s %8s %8s %9s\n" "bench" "sim_ms" "process" "settle" "edges" "wait_it"
fi

fail=0
for entry in "${BENCHES[@]}"; do
  name="${entry%% *}"; expect_ns="${entry##* }"
  [ -n "$ONLY" ] && [[ "$name" != *"$ONLY"* ]] && continue
  dir="$EX/$name"
  [ -d "$dir" ] || { echo "  SKIP $name (missing $dir)"; continue; }

  a_times=""; b_times=""; a_ref=""; b_ref=""
  for r in $(seq "$REPS"); do
    # Interleave A and B within each rep: this host drifts several percent
    # between runs and sequential A-then-B manufactures fake results.
    read -r sim proc set edg wi et ft <<<"$(run_one "$XEZIM_A" "$dir")"
    echo "$name,A,$r,$sim,$proc,$set,$edg,$wi,$et,$ft" >> "$CSV"
    a_times="$a_times $sim"; a_ref="$et/$ft"
    if [ -n "$XEZIM_B" ]; then
      read -r bsim bproc bset bedg bwi bet bft <<<"$(run_one "$XEZIM_B" "$dir")"
      echo "$name,B,$r,$bsim,$bproc,$bset,$bedg,$bwi,$bet,$bft" >> "$CSV"
      b_times="$b_times $bsim"; b_ref="$bet/$bft"
    fi
  done

  am=$(median "$a_times")
  if [ -n "$XEZIM_B" ]; then
    bm=$(median "$b_times")
    # Correctness gate FIRST: a timing delta means nothing if the two binaries
    # did not do the same work. End time and fatal count must match.
    if [ "$a_ref" != "$b_ref" ]; then
      verdict="DIVERGED (A end/fatals=$a_ref B=$b_ref)"; fail=1
    elif [ "${a_ref#*/}" != "0" ]; then
      verdict="FATAL in both ($a_ref) — not a valid measurement"; fail=1
    else
      verdict="ok"
    fi
    d=$(awk -v a="$am" -v b="$bm" 'BEGIN{if(a>0)printf "%+.1f%%",(b-a)/a*100; else print "n/a"}')
    printf "%-26s %11s %11s %9s  %s\n" "$name" "$am" "$bm" "$d" "$verdict"
  else
    read -r sim proc set edg wi et ft <<<"$(run_one "$XEZIM_A" "$dir")"
    [ "$ft" != "0" ] && { echo "  WARNING $name: $ft fatal(s) — excluded from the list?"; fail=1; }
    [ "$et" != "$expect_ns" ] && echo "  WARNING $name: end time $et != expected $expect_ns"
    printf "%-26s %10s %10s %8s %8s %9s\n" "$name" "$am" "$proc" "$set" "$edg" "$wi"
  fi
done

echo
echo "wrote $CSV"
cat <<'EOF'

Reading this:
  * `process` is the number that matters here — it is the AST-interpreted
    procedural path, and on these examples it is ~98% of the loop.
  * `settle`/`edges` are ~0 by construction. If they are NOT, the example grew a
    clocked DUT and stopped being a pure procedural benchmark.
  * A large part of `process` is one-time UVM build/connect, which does not
    amortize on these short runs. Treat it as a fixed-work throughput number,
    not as steady-state stimulus cost.
  * In A/B mode, `verdict` gates on end time AND fatal count matching. A timing
    delta with `DIVERGED` is meaningless.
EOF
exit $fail
