#!/usr/bin/env python3
"""Fail CI when Criterion benchmarks regress beyond a configured threshold."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compare two Criterion baselines by mean point estimate."
    )
    parser.add_argument("criterion_dir", type=Path)
    parser.add_argument("--baseline", default="base")
    parser.add_argument("--candidate", default="pr")
    parser.add_argument(
        "--threshold",
        type=float,
        default=float(os.environ.get("BENCH_REGRESSION_THRESHOLD", "0.05")),
        help="Allowed slowdown fraction, e.g. 0.05 allows a 5%% slowdown.",
    )
    return parser.parse_args()


def mean_estimate(path: Path) -> float:
    with path.open(encoding="utf-8") as handle:
        data = json.load(handle)
    return float(data["mean"]["point_estimate"])


def fmt_time(ns: float) -> str:
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.3f} us"
    return f"{ns:.3f} ns"


def github_error(message: str) -> None:
    escaped = message.replace("%", "%25").replace("\n", "%0A").replace("\r", "%0D")
    print(f"::error::{escaped}")


def write_step_summary(lines: list[str]) -> None:
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return
    with open(summary_path, "a", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
        handle.write("\n")


def main() -> int:
    args = parse_args()
    root = args.criterion_dir
    if not root.exists():
        github_error(f"Criterion directory does not exist: {root}")
        return 1

    baseline_files = sorted(root.glob(f"**/{args.baseline}/estimates.json"))
    if not baseline_files:
        github_error(f"No Criterion baseline estimates found for '{args.baseline}'.")
        return 1

    failures: list[str] = []
    rows: list[tuple[str, str, str, str, str]] = []

    for baseline_path in baseline_files:
        bench_id = baseline_path.parent.parent.relative_to(root)
        candidate_path = root / bench_id / args.candidate / "estimates.json"
        if not candidate_path.exists():
            failures.append(f"{bench_id}: candidate benchmark is missing")
            rows.append((str(bench_id), "missing", "missing", "missing", "fail"))
            continue

        baseline = mean_estimate(baseline_path)
        candidate = mean_estimate(candidate_path)
        if baseline <= 0:
            failures.append(f"{bench_id}: baseline estimate must be positive")
            rows.append((str(bench_id), "invalid", fmt_time(candidate), "invalid", "fail"))
            continue

        change = (candidate - baseline) / baseline
        status = "fail" if change > args.threshold else "ok"
        if status == "fail":
            failures.append(
                f"{bench_id}: {change:.2%} slower "
                f"({fmt_time(baseline)} -> {fmt_time(candidate)})"
            )
        rows.append(
            (
                str(bench_id),
                fmt_time(baseline),
                fmt_time(candidate),
                f"{change:+.2%}",
                status,
            )
        )

    candidate_files = sorted(root.glob(f"**/{args.candidate}/estimates.json"))
    baseline_ids = {path.parent.parent.relative_to(root) for path in baseline_files}
    new_ids = [
        path.parent.parent.relative_to(root)
        for path in candidate_files
        if path.parent.parent.relative_to(root) not in baseline_ids
    ]

    print(f"Benchmark regression threshold: {args.threshold:.2%}")
    for bench_id, baseline, candidate, change, status in rows:
        print(f"{status.upper():4} {bench_id}: {baseline} -> {candidate} ({change})")
    for bench_id in new_ids:
        print(f"NEW  {bench_id}: no base benchmark to compare")

    summary = [
        "### Benchmark Regression Check",
        "",
        f"Allowed slowdown: `{args.threshold:.2%}`",
        "",
        "| Benchmark | Base | PR | Change | Status |",
        "|---|---:|---:|---:|---|",
    ]
    summary.extend(
        f"| `{bench_id}` | {baseline} | {candidate} | {change} | {status} |"
        for bench_id, baseline, candidate, change, status in rows
    )
    summary.extend(f"| `{bench_id}` | n/a | new | n/a | new |" for bench_id in new_ids)
    write_step_summary(summary)

    if failures:
        github_error("Benchmark regressions detected:\n" + "\n".join(failures))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
