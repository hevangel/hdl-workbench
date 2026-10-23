#!/usr/bin/env bash
# Cross-TOOL benchmark runner: times the five benchmarks in this directory on
# one simulator and prints a table. Run it once per simulator, then compare.
#
#   ./run_crosstool.sh                       # xezim (release build)
#   ./run_crosstool.sh -s small|default|large
#   ./run_crosstool.sh -x /path/to/xezim
#   ./run_crosstool.sh -c "<compile cmd>" -r "<run cmd>"   # any other tool
#
# For another simulator, give a compile and a run command; {SRC}, {TOP} and
# {DEF} are substituted (DEF is the size-preset define, empty for default):
#
#   ./run_crosstool.sh \
#      -c '<your-compiler> -sv {DEF} {SRC}' \
#      -r '<your-simulator> {TOP}'
#
# CHECKSUM lines must be IDENTICAL across tools — they are the correctness
# comparison. Wall time is the performance comparison. A tool that prints
# FAIL has a defect (or the benchmark hit an unsupported construct); never
# compare times against a FAIL run.
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null && pwd -P)"
REPO_DIR="$(cd -- "${SCRIPT_DIR}/../.." >/dev/null && pwd -P)"
XEZIM="${REPO_DIR}/target/release/xezim"
SIZE="default"
COMPILE_CMD=""
RUN_CMD=""

while getopts "s:x:c:r:h" opt; do
  case "$opt" in
    s) SIZE="$OPTARG" ;;
    x) XEZIM="$OPTARG" ;;
    c) COMPILE_CMD="$OPTARG" ;;
    r) RUN_CMD="$OPTARG" ;;
    h) sed -n '2,25p' "$0"; exit 0 ;;
    *) exit 1 ;;
  esac
done

case "$SIZE" in
  small)   DEF="BENCH_SMALL" ;;
  large)   DEF="BENCH_LARGE" ;;
  default) DEF="" ;;
  *) echo "unknown size '$SIZE' (small|default|large)" >&2; exit 1 ;;
esac

# benchmark file : top module
BENCHES=(
  "b1_comb_mesh:bench_comb"
  "b2_pipeline:bench_pipe"
  "b3_memory:bench_mem"
  "b4_oop_tb:bench_oop"
  "b5_elab_scale:bench_elab"
)

printf "%-16s %10s %10s  %-6s  %s\n" "BENCH" "WALL_S" "WORK" "STATUS" "CHECKSUM"
printf -- "------------------------------------------------------------------------\n"

for entry in "${BENCHES[@]}"; do
  name="${entry%%:*}"
  top="${entry##*:}"
  src="${SCRIPT_DIR}/${name}.sv"
  out=$(mktemp)

  start=$(date +%s.%N)
  if [[ -n "$COMPILE_CMD" ]]; then
    # Foreign simulator: substitute and run compile, then run.
    defopt=""
    [[ -n "$DEF" ]] && defopt="+define+${DEF}"
    cc="${COMPILE_CMD//\{SRC\}/$src}"; cc="${cc//\{TOP\}/$top}"; cc="${cc//\{DEF\}/$defopt}"
    rc="${RUN_CMD//\{SRC\}/$src}";     rc="${rc//\{TOP\}/$top}";  rc="${rc//\{DEF\}/$defopt}"
    eval "$cc" >"$out" 2>&1
    eval "$rc" >>"$out" 2>&1
  else
    defopt=""
    [[ -n "$DEF" ]] && defopt="-D${DEF}"
    timeout 3600 "$XEZIM" --simulate "$src" -s "$top" $defopt \
      --max-time 100000000 >"$out" 2>&1
  fi
  end=$(date +%s.%N)

  wall=$(echo "$end - $start" | bc)
  status=$(grep -oE '^#? *BENCH [a-z0-9_]+ (PASS|FAIL)' "$out" | awk '{print $NF}' | head -1)
  csum=$(grep -oE '^#? *CHECKSUM [a-z0-9_]+ [0-9a-fx]+' "$out" | awk '{print $NF}' | head -1)
  work=$(grep -oE '^#? *WORK [a-z0-9_]+ [0-9]+' "$out" | awk '{print $NF}' | head -1)
  [[ -z "$status" ]] && status="NORUN"
  [[ -z "$csum" ]] && csum="-"
  [[ -z "$work" ]] && work="-"

  printf "%-16s %10.2f %10s  %-6s  %s\n" "$name" "$wall" "$work" "$status" "$csum"
  rm -f "$out"
done
