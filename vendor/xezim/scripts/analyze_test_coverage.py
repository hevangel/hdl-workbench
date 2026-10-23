#!/usr/bin/env python3
"""
Coverage-Driven Test Suite Analyzer & Tiering Tool for xezim.

Uses LLVM source-based code coverage (`-C instrument-coverage`, `llvm-profdata`,
`llvm-cov`) and a cost-weighted greedy set-cover algorithm (`|new_regions| / time_sec`)
to partition xezim's integration tests into three actionable groups:

  1. Tier 1 — Fast Max-Coverage Core (`core`):
     Reaches >= 95% (configurable via `--core-target`) of total test-suite coverage
     in the minimum possible wall-clock time.

  2. Tier 2 — Corner-Case Suite (`corner`):
     Tests that cover the remaining unique 5% of compiler/simulator basic blocks
     (ranked by unique regions contributed).

  3. Tier 3 — Redundant / Subsumed Tests (`redundant`):
     Tests whose executed basic-block regions in `xezim/src/` are 100% subsumed
     by faster tests in Tier 1 + Tier 2, sorted by wall-clock time descending.

Usage examples:
  # Analyze all test modules across the suite (default granularity: module)
  python3 scripts/analyze_test_coverage.py

  # Analyze individual #[test] functions within specific test binaries
  python3 scripts/analyze_test_coverage.py --granularity test --binaries collections,scheduling,types

  # Run only the Tier 1 Fast Max-Coverage Core from a saved report
  python3 scripts/analyze_test_coverage.py --run-tier core
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

SUITE_BINARIES = [
    "classes",
    "collections",
    "gates",
    "hierarchy",
    "lrm",
    "misc",
    "perf",
    "scheduling",
    "strings",
    "types",
    "upf",
]


def find_llvm_tool(base_name: str) -> str:
    """Locate llvm-profdata or llvm-cov matching rustc's LLVM version."""
    for candidate in [
        f"{base_name}-21",
        f"{base_name}-20",
        f"{base_name}-19",
        base_name,
        f"/usr/lib/llvm-21/bin/{base_name}",
        f"/usr/lib/llvm-20/bin/{base_name}",
    ]:
        path = shutil.which(candidate)
        if path:
            return path
    raise RuntimeError(
        f"Could not find {base_name} (tried {base_name}-21, {base_name}-20, {base_name})"
    )


def build_instrumented_tests(
    crate_dir: Path, target_dir: Path, binaries: List[str]
) -> Dict[str, Path]:
    """Build test binaries with `-C instrument-coverage` and return their paths."""
    env = os.environ.copy()
    existing_flags = env.get("RUSTFLAGS", "")
    if "-C instrument-coverage" not in existing_flags:
        env["RUSTFLAGS"] = (existing_flags + " -C instrument-coverage").strip()
    env["CARGO_TARGET_DIR"] = str(target_dir)

    cmd = ["cargo", "test", "--no-run", "--message-format=json"]
    for b in binaries:
        cmd.extend(["--test", b])

    print(f"[cov] Building instrumented test binaries in {target_dir} ...")
    proc = subprocess.Popen(
        cmd,
        cwd=str(crate_dir),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    assert proc.stdout is not None
    bin_paths: Dict[str, Path] = {}
    for line in proc.stdout:
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") == "compiler-artifact" and msg.get("profile", {}).get("test"):
            target = msg.get("target", {})
            name = target.get("name")
            executable = msg.get("executable")
            if name in binaries and executable:
                bin_paths[name] = Path(executable)

    _, stderr = proc.communicate()
    if proc.returncode != 0:
        print(stderr, file=sys.stderr)
        raise RuntimeError(f"`cargo test --no-run` failed with exit code {proc.returncode}")

    missing = [b for b in binaries if b not in bin_paths]
    if missing:
        raise RuntimeError(f"Missing built test binaries for: {missing}")
    return bin_paths


def list_test_units(
    bin_name: str, bin_path: Path, granularity: str
) -> List[Tuple[str, str]]:
    """
    Return list of `(unit_id, filter_arg)` for a test binary.
    - If `granularity == 'test'`: each `#[test]` is its own unit (`filter_arg` is exact test name).
    - If `granularity == 'module'`: tests are grouped by top-level module prefix `mod_name::`
      (or exact test name for top-level tests without `::`).
    """
    env = os.environ.copy()
    env["LLVM_PROFILE_FILE"] = "/dev/null"
    out = subprocess.check_output(
        [str(bin_path), "--list", "--format", "terse"],
        text=True,
        stderr=subprocess.DEVNULL,
        env=env,
    )
    tests: List[str] = []
    for line in out.splitlines():
        line = line.strip()
        if line.endswith(": test"):
            tests.append(line[: -len(": test")])

    if granularity == "test":
        return [(f"{bin_name}::{t}", t) for t in tests]

    # Group by top-level module inside the integration test binary
    seen_modules: Dict[str, str] = {}
    for t in tests:
        if "::" in t:
            mod_prefix = t.split("::", 1)[0]
            unit_id = f"{bin_name}::{mod_prefix}"
            seen_modules.setdefault(unit_id, f"{mod_prefix}::")
        else:
            unit_id = f"{bin_name}::{t}"
            seen_modules.setdefault(unit_id, t)
    return list(seen_modules.items())


def extract_covered_regions(
    llvm_profdata: str,
    llvm_cov: str,
    bin_path: Path,
    profraw_files: List[Path],
    profdata_path: Path,
) -> Set[Tuple[str, int, int]]:
    """
    Merge `.profraw` into `.profdata` and extract the set of covered `(rel_file, line, col)`
    segments inside `xezim/src/` with execution count > 0.
    """
    if not profraw_files:
        return set()

    subprocess.run(
        [llvm_profdata, "merge", "-sparse", *[str(p) for p in profraw_files], "-o", str(profdata_path)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    export_cmd = [
        llvm_cov,
        "export",
        "-format=text",
        "-skip-expansions",
        "-skip-functions",
        "-ignore-filename-regex=(/tests/|/cargo/|/rustc/|xezim-core)",
        f"-instr-profile={profdata_path}",
        str(bin_path),
    ]
    raw = subprocess.check_output(export_cmd, text=True, stderr=subprocess.DEVNULL)
    data = json.loads(raw)

    covered: Set[Tuple[str, int, int]] = set()
    for datum in data.get("data", []):
        for fobj in datum.get("files", []):
            fname = fobj.get("filename", "")
            if "/src/" not in fname:
                continue
            rel_name = fname.split("/src/", 1)[-1]
            # Each segment is [line, col, count, has_count, is_region_entry, ...]
            for seg in fobj.get("segments", []):
                if len(seg) >= 5:
                    line, col, count, has_count, is_entry = seg[0], seg[1], seg[2], seg[3], seg[4]
                    if has_count and is_entry and count > 0:
                        covered.add((rel_name, int(line), int(col)))
    return covered


def run_one_unit(
    crate_dir: Path,
    bin_name: str,
    bin_path: Path,
    unit_id: str,
    filter_arg: str,
    granularity: str,
    work_dir: Path,
    llvm_profdata: str,
    llvm_cov: str,
) -> Tuple[str, str, str, float, Set[Tuple[str, int, int]]]:
    """Run a single test or test module, measure wall-clock time, and extract its covered regions."""
    safe_name = re.sub(r"[^A-Za-z0-9_.-]+", "__", unit_id)
    unit_dir = work_dir / safe_name
    unit_dir.mkdir(parents=True, exist_ok=True)

    env = os.environ.copy()
    # %p and %m handle child processes (e.g. CLI subprocess tests) cleanly
    env["LLVM_PROFILE_FILE"] = str(unit_dir / "cov_%p_%m.profraw")

    cmd = [str(bin_path), filter_arg]
    if granularity == "test" or not filter_arg.endswith("::"):
        cmd.append("--exact")

    t0 = time.monotonic()
    res = subprocess.run(
        cmd,
        cwd=str(crate_dir),
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    elapsed = time.monotonic() - t0

    profraws = list(unit_dir.glob("*.profraw"))
    profdata = unit_dir / "merged.profdata"
    try:
        if res.returncode != 0:
            raise RuntimeError(
                f"Test unit {unit_id} failed with exit code {res.returncode}; "
                "coverage partitioning requires every selected test to pass."
            )
        if not profraws:
            raise RuntimeError(f"Test unit {unit_id} produced no coverage profile.")
        covered = extract_covered_regions(
            llvm_profdata, llvm_cov, bin_path, profraws, profdata
        )
    finally:
        shutil.rmtree(unit_dir, ignore_errors=True)

    return (unit_id, bin_name, filter_arg, elapsed, covered)


def greedy_set_cover_partition(
    units: List[Tuple[str, str, str, float, Set[Tuple[str, int, int]]]],
    core_target_ratio: float,
) -> dict:
    """
    Partition test units using cost-weighted greedy set cover (`|new_regions| / time_sec`).
    Returns a dictionary containing `core`, `corner`, `redundant`, and summary stats.
    """
    universe: Set[Tuple[str, int, int]] = set()
    for _, _, _, _, cov in units:
        universe |= cov

    total_regions = len(universe)
    if total_regions == 0:
        raise RuntimeError("No covered regions found in `src/` — check LLVM coverage export.")

    remaining = list(range(len(units)))
    covered_so_far: Set[Tuple[str, int, int]] = set()

    core_tier: List[dict] = []
    corner_tier: List[dict] = []
    redundant_tier: List[dict] = []

    target_regions = int(total_regions * core_target_ratio)

    while remaining:
        best_idx = -1
        best_score = -1.0
        best_new_count = 0

        for idx in remaining:
            _, _, _, elapsed, cov = units[idx]
            new_cnt = len(cov - covered_so_far)
            if new_cnt == 0:
                continue
            cost = max(elapsed, 0.005)
            score = new_cnt / cost
            if score > best_score or (score == best_score and new_cnt > best_new_count):
                best_score = score
                best_idx = idx
                best_new_count = new_cnt

        if best_idx == -1:
            # All remaining units add 0 new regions!
            for idx in remaining:
                uid, bname, farg, elapsed, cov = units[idx]
                redundant_tier.append(
                    {
                        "unit_id": uid,
                        "binary": bname,
                        "filter": farg,
                        "time_sec": round(elapsed, 3),
                        "total_regions_in_unit": len(cov),
                        "new_regions": 0,
                    }
                )
            break

        remaining.remove(best_idx)
        uid, bname, farg, elapsed, cov = units[best_idx]
        was_in_core = len(covered_so_far) < target_regions
        covered_so_far |= cov
        cum_cov = len(covered_so_far) / total_regions

        entry = {
            "unit_id": uid,
            "binary": bname,
            "filter": farg,
            "time_sec": round(elapsed, 3),
            "total_regions_in_unit": len(cov),
            "new_regions": best_new_count,
            "regions_per_sec": round(best_new_count / max(elapsed, 0.001), 1),
            "cumulative_coverage_pct": round(cum_cov * 100.0, 2),
        }
        if was_in_core:
            core_tier.append(entry)
        else:
            corner_tier.append(entry)

    redundant_tier.sort(key=lambda x: x["time_sec"], reverse=True)
    corner_tier.sort(key=lambda x: x["time_sec"], reverse=True)

    total_time = sum(u[3] for u in units)
    core_time = sum(x["time_sec"] for x in core_tier)
    corner_time = sum(x["time_sec"] for x in corner_tier)
    redundant_time = sum(x["time_sec"] for x in redundant_tier)

    return {
        "summary": {
            "total_units": len(units),
            "total_covered_regions": total_regions,
            "total_cpu_time_sec": round(total_time, 2),
            "core_target_pct": round(core_target_ratio * 100.0, 1),
            "core_count": len(core_tier),
            "core_cpu_time_sec": round(core_time, 2),
            "core_coverage_pct": core_tier[-1]["cumulative_coverage_pct"] if core_tier else 0.0,
            "corner_count": len(corner_tier),
            "corner_cpu_time_sec": round(corner_time, 2),
            "redundant_count": len(redundant_tier),
            "redundant_cpu_time_sec": round(redundant_time, 2),
        },
        "core": core_tier,
        "corner": corner_tier,
        "redundant": redundant_tier,
    }


def write_markdown_report(report: dict, md_path: Path) -> None:
    s = report["summary"]
    lines = [
        "# xezim Coverage-Driven Test Suite Partitioning Report",
        "",
        "## Executive Summary",
        "",
        f"- **Total Test Units Analyzed:** {s['total_units']}",
        f"- **Total Covered Code Regions in `src/`:** {s['total_covered_regions']:,}",
        f"- **Total Cumulative Unit Time:** {s['total_cpu_time_sec']:.2f}s",
        "",
        "| Tier | Purpose | Units | Cumulative Time (s) | Time Share | Coverage Reached |",
        "|---|---|---:|---:|---:|---:|",
        f"| **Tier 1: Fast Max-Coverage Core** | High-efficiency core (`>={s['core_target_pct']}%` coverage) | {s['core_count']} | {s['core_cpu_time_sec']:.2f}s | {100.0 * s['core_cpu_time_sec'] / max(s['total_cpu_time_sec'], 0.01):.1f}% | **{s['core_coverage_pct']:.2f}%** |",
        f"| **Tier 2: Corner-Case Suite** | Remaining unique compiler/simulator branches | {s['corner_count']} | {s['corner_cpu_time_sec']:.2f}s | {100.0 * s['corner_cpu_time_sec'] / max(s['total_cpu_time_sec'], 0.01):.1f}% | **100.00%** |",
        f"| **Tier 3: 100% Subsumed / Redundant** | Zero additional basic-block coverage over Tier 1+2 | {s['redundant_count']} | {s['redundant_cpu_time_sec']:.2f}s | {100.0 * s['redundant_cpu_time_sec'] / max(s['total_cpu_time_sec'], 0.01):.1f}% | +0.00% |",
        "",
        "## Top 25 Tier 1 (Fast Max-Coverage Core) Units",
        "",
        "| Rank | Unit | Time (s) | New Regions | Regions/s | Cumulative Coverage |",
        "|---:|---|---:|---:|---:|---:|",
    ]
    for idx, item in enumerate(report["core"][:25], 1):
        lines.append(
            f"| {idx} | `{item['unit_id']}` | {item['time_sec']:.3f}s | +{item['new_regions']:,} | {item['regions_per_sec']:,.1f} | {item['cumulative_coverage_pct']:.2f}% |"
        )

    lines.extend(
        [
            "",
            "## Top 25 Slowest Tier 2 (Corner-Case Only) Units",
            "",
            "| Unit | Time (s) | Unique Corner-Case Regions Added | Total Regions Hit |",
            "|---|---:|---:|---:|",
        ]
    )
    for item in report["corner"][:25]:
        lines.append(
            f"| `{item['unit_id']}` | {item['time_sec']:.3f}s | +{item['new_regions']:,} | {item['total_regions_in_unit']:,} |"
        )

    lines.extend(
        [
            "",
            "## Top 25 Slowest Tier 3 (100% Coverage-Subsumed) Units",
            "",
            "| Unit | Time (s) | Total Regions Hit (All Subsumed) |",
            "|---|---:|---:|",
        ]
    )
    for item in report["redundant"][:25]:
        lines.append(
            f"| `{item['unit_id']}` | {item['time_sec']:.3f}s | {item['total_regions_in_unit']:,} |"
        )

    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_saved_tier(crate_dir: Path, report_path: Path, tier: str) -> None:
    """Execute only the tests in the selected tier (`core` or `corner`) using `cargo test`."""
    if not report_path.exists():
        raise RuntimeError(f"Saved coverage report not found at {report_path}. Run analysis first.")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    items = report.get(tier, [])
    if not items:
        print(f"No test units in tier '{tier}'.")
        return

    by_binary: Dict[str, List[str]] = {}
    for item in items:
        by_binary.setdefault(item["binary"], []).append(item["filter"])

    t0 = time.monotonic()
    for binary, filters in by_binary.items():
        cmd = ["cargo", "test", "--test", binary, "--", *filters]
        print(f"[tier:{tier}] Running {len(filters)} filter(s) in `--test {binary}` ...")
        subprocess.run(cmd, cwd=str(crate_dir), check=True)
    elapsed = time.monotonic() - t0
    print(f"[tier:{tier}] Completed in {elapsed:.2f}s.")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Analyze per-module/per-test code coverage and partition tests into Fast Core, Corner-Case, and Redundant tiers."
    )
    parser.add_argument(
        "--granularity",
        choices=["module", "test"],
        default="module",
        help="Collect coverage per test module (default, fast) or per individual #[test] function.",
    )
    parser.add_argument(
        "--binaries",
        default=",".join(SUITE_BINARIES),
        help="Comma-separated list of integration test binaries to analyze.",
    )
    parser.add_argument(
        "--core-target",
        type=float,
        default=0.95,
        help="Fraction of total coverage to reach in Tier 1 Fast Core (default: 0.95).",
    )
    parser.add_argument(
        "--jobs",
        type=int,
        default=8,
        help="Number of parallel workers for profiling test units (default: 8).",
    )
    parser.add_argument(
        "--run-tier",
        choices=["core", "corner"],
        help="Skip profiling and execute the specified tier from the saved report.",
    )
    args = parser.parse_args()

    crate_dir = Path(__file__).resolve().parent.parent
    target_dir = crate_dir / "target" / "cov"
    report_json = target_dir / "coverage_tiers.json"
    report_md = target_dir / "COVERAGE_STRATEGY.md"

    if args.run_tier:
        run_saved_tier(crate_dir, report_json, args.run_tier)
        return

    llvm_profdata = find_llvm_tool("llvm-profdata")
    llvm_cov = find_llvm_tool("llvm-cov")
    binaries = [b.strip() for b in args.binaries.split(",") if b.strip()]

    bin_paths = build_instrumented_tests(crate_dir, target_dir, binaries)

    all_tasks: List[Tuple[str, Path, str, str]] = []
    for b in binaries:
        for unit_id, filter_arg in list_test_units(b, bin_paths[b], args.granularity):
            all_tasks.append((b, bin_paths[b], unit_id, filter_arg))

    print(
        f"[cov] Profiling {len(all_tasks)} test units ({args.granularity} granularity) with {args.jobs} workers ..."
    )
    work_dir = target_dir / "profraw_work"
    work_dir.mkdir(parents=True, exist_ok=True)

    results: List[Tuple[str, str, str, float, Set[Tuple[str, int, int]]]] = []
    done_count = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futs = [
            pool.submit(
                run_one_unit,
                crate_dir,
                bname,
                bpath,
                uid,
                farg,
                args.granularity,
                work_dir,
                llvm_profdata,
                llvm_cov,
            )
            for (bname, bpath, uid, farg) in all_tasks
        ]
        for fut in concurrent.futures.as_completed(futs):
            res = fut.result()
            results.append(res)
            done_count += 1
            if done_count % 20 == 0 or done_count == len(all_tasks):
                print(f"[cov]   {done_count}/{len(all_tasks)} units profiled ...")

    report = greedy_set_cover_partition(results, args.core_target)
    target_dir.mkdir(parents=True, exist_ok=True)
    report_json.write_text(json.dumps(report, indent=2), encoding="utf-8")
    write_markdown_report(report, report_md)

    s = report["summary"]
    print("\n=== Coverage Partitioning Summary ===")
    print(f"Total units analyzed : {s['total_units']}")
    print(f"Total regions covered: {s['total_covered_regions']:,}")
    print(
        f"Tier 1 (Fast Core)   : {s['core_count']} units | {s['core_cpu_time_sec']:.2f}s | {s['core_coverage_pct']:.2f}% coverage"
    )
    print(
        f"Tier 2 (Corner Cases): {s['corner_count']} units | {s['corner_cpu_time_sec']:.2f}s | reaches 100.00% coverage"
    )
    print(
        f"Tier 3 (Redundant)   : {s['redundant_count']} units | {s['redundant_cpu_time_sec']:.2f}s | +0.00% new coverage"
    )
    print(f"\nSaved JSON report    : {report_json}")
    print(f"Saved Markdown report: {report_md}")


if __name__ == "__main__":
    main()
