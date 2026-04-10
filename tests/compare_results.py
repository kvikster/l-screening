#!/usr/bin/env python3
"""Compare Python and Rust WASM screening results."""
from __future__ import annotations

import json
import math
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

# Key fields to compare for each peak result
PEAK_COMPARE_FIELDS = [
    "Group",
    "RT_mean",
    "MZ_mean",
    "Area_mean",
    "AreaCVPct",
    "ReplicateQuality",
    "ReplicateCount",
    "ReplicateConfidenceScore",
    "ConfidenceScore",
    "Polarity",
    "SampleType",
    "MatchingMode",
    "ParallelMatch",
    "ParallelSampleCount",
    "BlankAreaMean",
    "AreaDifference",
    "SignalToBlankRatio",
    "Status",
    "Confirmed",
]

SUMMARY_COMPARE_FIELDS = [
    "Sample",
    "Polarity",
    "TotalPeaks",
    "Confirmed",
    "Artifacts",
    "RealCompounds",
    "ColorDriven",
    "MeanCVPct",
    "HighQuality",
    "ModerateQuality",
    "LowQuality",
    "MeanConfidenceScore",
    "MeanSignalToBlankRatio",
]


def approx_equal(a, b, rel_tol=1e-4, abs_tol=1e-4) -> bool:
    """Compare two values with tolerance for floats.

    abs_tol=1e-4 accounts for rounding differences in values rounded to 4 decimals
    (e.g. RT_mean rounded to 4 digits can differ by 0.0001 between Python and Rust).
    """
    if a is None and b is None:
        return True
    if a is None or b is None:
        return False
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        if math.isnan(a) and math.isnan(b):
            return True
        return math.isclose(float(a), float(b), rel_tol=rel_tol, abs_tol=abs_tol)
    if isinstance(a, bool) and isinstance(b, bool):
        return a == b
    if isinstance(a, bool) or isinstance(b, bool):
        # Handle bool vs non-bool
        return str(a).lower() == str(b).lower()
    return str(a) == str(b)


def compare_peaks(py_peaks: list, rs_peaks: list) -> list[dict]:
    """Compare peak results and return list of differences."""
    diffs = []

    if len(py_peaks) != len(rs_peaks):
        diffs.append({
            "type": "count_mismatch",
            "python_count": len(py_peaks),
            "wasm_count": len(rs_peaks),
        })
        return diffs

    # Sort both by (SampleType, RT_mean) for alignment
    def sort_key(p):
        return (
            str(p.get("SampleType") or ""),
            float(p.get("RT_mean") or 0),
            float(p.get("MZ_mean") or 0),
        )

    py_sorted = sorted(py_peaks, key=sort_key)
    rs_sorted = sorted(rs_peaks, key=sort_key)

    for i, (py, rs) in enumerate(zip(py_sorted, rs_sorted)):
        for field in PEAK_COMPARE_FIELDS:
            py_val = py.get(field)
            rs_val = rs.get(field)

            # Normalize: treat "None" string as None
            if isinstance(py_val, str) and py_val.lower() == "none":
                py_val = None
            if isinstance(rs_val, str) and rs_val.lower() == "none":
                rs_val = None

            if not approx_equal(py_val, rs_val):
                diffs.append({
                    "type": "peak_field_mismatch",
                    "index": i,
                    "field": field,
                    "python_value": py_val,
                    "wasm_value": rs_val,
                    "python_rt": py.get("RT_mean"),
                    "wasm_rt": rs.get("RT_mean"),
                })

    return diffs


def compare_summaries(py_summaries: list, rs_summaries: list) -> list[dict]:
    """Compare summary results and return list of differences."""
    diffs = []

    if len(py_summaries) != len(rs_summaries):
        diffs.append({
            "type": "summary_count_mismatch",
            "python_count": len(py_summaries),
            "wasm_count": len(rs_summaries),
        })
        return diffs

    def sort_key(s):
        return (str(s.get("Sample") or ""), str(s.get("Polarity") or ""))

    py_sorted = sorted(py_summaries, key=sort_key)
    rs_sorted = sorted(rs_summaries, key=sort_key)

    for i, (py, rs) in enumerate(zip(py_sorted, rs_sorted)):
        for field in SUMMARY_COMPARE_FIELDS:
            py_val = py.get(field)
            rs_val = rs.get(field)

            if isinstance(py_val, str) and py_val.lower() == "none":
                py_val = None
            if isinstance(rs_val, str) and rs_val.lower() == "none":
                rs_val = None

            if not approx_equal(py_val, rs_val):
                diffs.append({
                    "type": "summary_field_mismatch",
                    "index": i,
                    "field": field,
                    "python_value": py_val,
                    "wasm_value": rs_val,
                    "sample": py.get("Sample"),
                    "polarity": py.get("Polarity"),
                })

    return diffs


def main():
    py_path = ROOT / "tests" / "_python_output.json"
    rs_path = ROOT / "tests" / "_wasm_output.json"

    if not py_path.exists():
        print(f"ERROR: Python output not found: {py_path}")
        print("Run: python tests/run_python.py")
        sys.exit(1)

    if not rs_path.exists():
        print(f"ERROR: WASM output not found: {rs_path}")
        print("Run: node tests/run_wasm.mjs")
        sys.exit(1)

    py_data = json.loads(py_path.read_text("utf-8"))
    rs_data = json.loads(rs_path.read_text("utf-8"))

    py_peaks = py_data.get("results", [])
    rs_peaks = rs_data.get("results", [])
    py_summaries = py_data.get("summary", [])
    rs_summaries = rs_data.get("summary", [])

    print("=" * 70)
    print("COMPARISON: Python vs Rust WASM Screening Results")
    print("=" * 70)
    print()
    print(f"Python peaks: {len(py_peaks)}")
    print(f"WASM peaks:   {len(rs_peaks)}")
    print(f"Python summary rows: {len(py_summaries)}")
    print(f"WASM summary rows:   {len(rs_summaries)}")
    print()

    # Compare peaks
    peak_diffs = compare_peaks(py_peaks, rs_peaks)
    print(f"Peak differences: {len(peak_diffs)}")

    if peak_diffs:
        print()
        for d in peak_diffs[:50]:  # Show first 50 diffs
            if d["type"] == "count_mismatch":
                print(f"  ⚠ PEAK COUNT: Python={d['python_count']}, WASM={d['wasm_count']}")
            else:
                print(
                    f"  ⚠ Peak #{d['index']} field '{d['field']}': "
                    f"Python={d['python_value']!r} vs WASM={d['wasm_value']!r} "
                    f"(RT={d.get('python_rt')})"
                )
        if len(peak_diffs) > 50:
            print(f"  ... and {len(peak_diffs) - 50} more differences")

    # Compare summaries
    summary_diffs = compare_summaries(py_summaries, rs_summaries)
    print(f"\nSummary differences: {len(summary_diffs)}")

    if summary_diffs:
        print()
        for d in summary_diffs:
            if d["type"] == "summary_count_mismatch":
                print(f"  ⚠ SUMMARY COUNT: Python={d['python_count']}, WASM={d['wasm_count']}")
            else:
                print(
                    f"  ⚠ Summary #{d['index']} ({d.get('sample')}/{d.get('polarity')}) "
                    f"field '{d['field']}': Python={d['python_value']!r} vs WASM={d['wasm_value']!r}"
                )

    # Final verdict
    print()
    print("=" * 70)
    total_diffs = len(peak_diffs) + len(summary_diffs)
    if total_diffs == 0:
        print("✅ PASS: Python and Rust WASM produce IDENTICAL results!")
    else:
        print(f"❌ FAIL: {total_diffs} total differences found")
        print(f"   Peak diffs: {len(peak_diffs)}")
        print(f"   Summary diffs: {len(summary_diffs)}")
    print("=" * 70)

    return 0 if total_diffs == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
