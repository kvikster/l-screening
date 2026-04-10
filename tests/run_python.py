#!/usr/bin/env python3
"""Run the Python screening algorithm on Sample EL_1.xlsx and save results."""
from __future__ import annotations

import importlib.util
import json
import math
import pathlib
import sys

import pandas as pd

ROOT = pathlib.Path(__file__).resolve().parents[1]

# ── Import logic.py as a standalone module ──────────────────────────────────
LOGIC_PATH = ROOT / "server/python-version/logic.py"
SPEC = importlib.util.spec_from_file_location("screening_logic", LOGIC_PATH)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

# ── Import colors.py ────────────────────────────────────────────────────────
COLORS_PATH = ROOT / "server/python-version/colors.py"
COLORS_SPEC = importlib.util.spec_from_file_location("colors", COLORS_PATH)
assert COLORS_SPEC and COLORS_SPEC.loader
COLORS_MODULE = importlib.util.module_from_spec(COLORS_SPEC)
sys.modules[COLORS_SPEC.name] = COLORS_MODULE
COLORS_SPEC.loader.exec_module(COLORS_MODULE)


def select_best_sheet(content: bytes) -> str:
    sheets_dict = pd.read_excel(pd.io.common.BytesIO(content), sheet_name=None)
    required = ["RT", "Polarity", "File", "Area"]
    best_name = max(
        sheets_dict,
        key=lambda n: len(
            [c for c in required if c in [str(x).strip() for x in sheets_dict[n].columns]]
        ),
    )
    return best_name


def df_to_wasm_json(df: pd.DataFrame) -> str:
    """Convert a DataFrame to the JSON format expected by the Rust WASM."""
    rows = []
    for _, row in df.iterrows():
        obj = {
            "RT": float(row["RT"]),
            "Area": float(row["Area"]),
            "Polarity": str(row["Polarity"]),
            "File": str(row["File"]),
        }
        # Optional fields
        bp = row.get("Base Peak")
        if bp is not None and not (isinstance(bp, float) and math.isnan(bp)):
            obj["Base Peak"] = float(bp)
        else:
            obj["Base Peak"] = None

        label = row.get("Label")
        if label is not None and not (isinstance(label, float) and math.isnan(label)):
            obj["Label"] = str(label)
        else:
            obj["Label"] = None

        op_mark = row.get("operator_mark")
        if op_mark is not None and not (isinstance(op_mark, float) and math.isnan(op_mark)):
            obj["operator_mark"] = str(op_mark)
        else:
            obj["operator_mark"] = None

        op_color = row.get("operator_color")
        if op_color is not None and not (isinstance(op_color, float) and math.isnan(op_color)):
            obj["operator_color"] = str(op_color)
        else:
            obj["operator_color"] = None

        rows.append(obj)
    return json.dumps(rows, ensure_ascii=False)


def make_json_safe(obj):
    """Recursively convert NaN/Inf to None for JSON serialization."""
    if isinstance(obj, dict):
        return {k: make_json_safe(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [make_json_safe(v) for v in obj]
    if isinstance(obj, float):
        if math.isnan(obj) or math.isinf(obj):
            return None
        return obj
    try:
        if pd.isna(obj):
            return None
    except TypeError:
        pass
    if hasattr(obj, "item") and callable(obj.item):
        try:
            return make_json_safe(obj.item())
        except (ValueError, TypeError):
            return obj
    return obj


def main():
    xlsx_path = ROOT / "Sample EL_1.xlsx"
    print(f"Reading: {xlsx_path}")

    content = xlsx_path.read_bytes()
    best_sheet = select_best_sheet(content)
    print(f"Best sheet: {best_sheet}")

    # Read with operator colors (same as the Python server does)
    df = COLORS_MODULE.read_with_operator_colors(content, best_sheet)
    print(f"Rows read: {len(df)}")

    # Prepare JSON input for WASM
    rows_json = df_to_wasm_json(df)
    config_json = json.dumps({})  # default config

    # Save WASM input
    wasm_input_path = ROOT / "tests" / "_wasm_input.json"
    wasm_input_path.write_text(
        json.dumps({"rows": json.loads(rows_json), "config": {}}),
        encoding="utf-8",
    )
    print(f"WASM input saved to: {wasm_input_path}")

    # Run Python algorithm
    results_df, summary_df, results_list = MODULE.process_peaks(df)

    # Save Python results
    python_results = {
        "results": make_json_safe(results_list),
        "summary": make_json_safe(summary_df.to_dict(orient="records")),
    }
    python_output_path = ROOT / "tests" / "_python_output.json"
    python_output_path.write_text(
        json.dumps(python_results, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    print(f"Python output saved to: {python_output_path}")
    print(f"Python results: {len(results_list)} peaks")
    print(f"Python summary: {len(summary_df)} rows")


if __name__ == "__main__":
    main()
