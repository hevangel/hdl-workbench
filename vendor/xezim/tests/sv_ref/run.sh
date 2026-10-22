#!/usr/bin/env bash
# Run the sv_ref testcases on the reference simulator and/or xezim.
#
#   ./run.sh ref             # reference (golden) run
#   ./run.sh xezim           # xezim run
#   ./run.sh both            # run both and compare the verdicts
#
# Each testcase prints "TEST <name>: N checks, M errors -> PASS|FAIL".
# The `ref` and `both` modes need the reference toolchain's library, compile
# and simulate commands in $REF_LIB, $REF_COMP and $REF_SIM, so no vendor
# tool name is baked into this script or its logs.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="${HERE}/out"
XEZIM="${XEZIM:-${HERE}/../../target/release/xezim}"
REF_LIB="${REF_LIB:-}"
REF_COMP="${REF_COMP:-}"
REF_SIM="${REF_SIM:-}"
MODE="${1:-both}"
if [[ ( "$MODE" == ref || "$MODE" == both ) && ( -z "$REF_LIB" || -z "$REF_COMP" || -z "$REF_SIM" ) ]]; then
  echo "run.sh: set REF_LIB, REF_COMP and REF_SIM to the reference toolchain's library, compile and simulate commands (or run './run.sh xezim')" >&2
  exit 2
fi

TESTS=(
  packed_port_elem_select
  packed_port_formal_width
  packed_port_loop_select
  pct_m_hier_scope
  param_static_park
)

mkdir -p "$OUT"

run_ref() {
  local t=$1
  ( cd "$OUT" || exit 1
    rm -rf "work_$t"
    "$REF_LIB"  "work_$t"                                    >/dev/null 2>&1
    "$REF_COMP" -sv -work "work_$t" -quiet "${HERE}/${t}.sv" >"${t}.ref.log" 2>&1 || return 1
    "$REF_SIM"  -c -work "work_$t" "$t" -do "run -all; quit -f" >>"${t}.ref.log" 2>&1
  )
}

run_xezim() {
  local t=$1
  ( cd "$OUT" || exit 1
    # A bounded run so a hang-regression (the param_static_park class of
    # bug spins forever) yields a clean <no verdict>/FAIL instead of wedging
    # the whole harness.
    timeout 60 "$XEZIM" "${HERE}/${t}.sv" -s "$t" >"${t}.xezim.log" 2>&1
  )
}

verdict() {  # $1 = logfile
  grep -hoE 'TEST [a-z_0-9]+: [0-9]+ checks, [0-9]+ errors -> (PASS|FAIL)' "$1" 2>/dev/null | tail -1
}

status=0
printf '%-28s %-14s %s\n' "TESTCASE" "REFERENCE" "XEZIM"
printf '%-28s %-14s %s\n' "--------" "---------" "-----"
for t in "${TESTS[@]}"; do
  q="-"; x="-"
  if [[ "$MODE" == ref || "$MODE" == both ]]; then
    run_ref "$t"; q="$(verdict "${OUT}/${t}.ref.log")"; q="${q:-<no verdict>}"
  fi
  if [[ "$MODE" == xezim || "$MODE" == both ]]; then
    run_xezim "$t"; x="$(verdict "${OUT}/${t}.xezim.log")"; x="${x:-<no verdict>}"
  fi
  printf '%-28s %-14s %s\n' "$t" "${q##*-> }" "${x##*-> }"
  [[ "$q" == *FAIL* || "$q" == *"<no verdict>"* ]] && status=1
  [[ "$MODE" != ref && ( "$x" == *FAIL* || "$x" == *"<no verdict>"* ) ]] && status=1
done
echo
echo "logs: ${OUT}/<test>.{ref,xezim}.log"
exit $status
