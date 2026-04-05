import pandas as pd
from typing import List, Dict, Any, Tuple

RT_TOL = 0.1   # minutes
MZ_TOL = 0.3   # m/z units

# Maps operator_mark → canonical SampleType used throughout the pipeline.
# Reps 1 & 2 collapse to the same SampleType so they are paired in coarse_screen.
_MARK_TO_STYPE = {
    "blank_positive":  "blank",
    "blank_negative":  "blank",
    "sample_rep1":     "sample",
    "sample_rep2":     "sample",
}

# Which operator_mark values count as "replicate 1" vs "replicate 2"
_REP1_MARKS = {"sample_rep1", "blank_positive"}   # arbitrary but consistent
_REP2_MARKS = {"sample_rep2", "blank_negative"}


def _classify_from_filename(filename: str) -> str:
    f = str(filename).lower()
    if "blank" in f:
        return "blank"
    for i in range(1, 10):
        if f.startswith(f"{i}_") or f.startswith(f"{i}_neg"):
            return f"sample_{i}"
    return "unknown"


def _assign_sample_type(row: pd.Series) -> str:
    """Use operator_mark when present, fall back to filename heuristic."""
    mark = row.get("operator_mark")
    if mark and mark in _MARK_TO_STYPE:
        return _MARK_TO_STYPE[mark]
    return _classify_from_filename(row["File"])


def _is_rep1(row: pd.Series) -> bool:
    mark = row.get("operator_mark")
    if mark:
        return mark in _REP1_MARKS
    # filename fallback: files ending -1.d are rep1
    return str(row["File"]).endswith("-1.d")


def coarse_screen(group: pd.DataFrame, rt_tol: float = RT_TOL, mz_tol: float = MZ_TOL) -> pd.DataFrame:
    """
    Pair rep1 vs rep2 rows within the same (SampleType, Polarity) group.
    When operator colours are present the split is colour-driven; otherwise
    it falls back to the legacy file-order split.
    """
    has_marks = group["operator_mark"].notna().any()
    marks_present = set(group["operator_mark"].dropna())
    colour_split = bool(marks_present & _REP1_MARKS) and bool(marks_present & _REP2_MARKS)

    if colour_split:
        rep1 = group[group["operator_mark"].isin(_REP1_MARKS)]
        rep2 = group[group["operator_mark"].isin(_REP2_MARKS)]
    else:
        files = group["File"].unique()
        if len(files) < 2:
            return pd.DataFrame()
        rep1 = group[group["File"] == files[0]]
        rep2 = group[group["File"] == files[1]]

    if rep1.empty or rep2.empty:
        return pd.DataFrame()

    confirmed = []
    for _, p1 in rep1.iterrows():
        for _, p2 in rep2.iterrows():
            if (abs(p1["RT"] - p2["RT"]) <= rt_tol and
                    abs(p1["Base Peak"] - p2["Base Peak"]) <= mz_tol):
                confirmed.append({
                    "Group":       f"{p1['SampleType']}_{p1['Polarity']}",
                    "RT_mean":     round((p1["RT"] + p2["RT"]) / 2, 4),
                    "MZ_mean":     round((p1["Base Peak"] + p2["Base Peak"]) / 2, 2),
                    "Area_mean":   int((p1["Area"] + p2["Area"]) / 2),
                    "Polarity":    p1["Polarity"],
                    "SampleType":  p1["SampleType"],
                    "Rep1_Label":  p1["Label"],
                    "Rep2_Label":  p2["Label"],
                    "Rep1_Mark":   p1.get("operator_mark"),
                    "Rep2_Mark":   p2.get("operator_mark"),
                    "Rep1_Color":  p1.get("operator_color"),
                    "Rep2_Color":  p2.get("operator_color"),
                    "Confirmed":   "Yes",
                    "Why": {
                        "Rep1_RT": p1["RT"],  "Rep2_RT": p2["RT"],
                        "Rep1_MZ": p1["Base Peak"], "Rep2_MZ": p2["Base Peak"],
                        "ColorPaired": colour_split,
                        "Matches": True,
                    },
                })
    return pd.DataFrame(confirmed)


def process_peaks(df: pd.DataFrame) -> Tuple[pd.DataFrame, pd.DataFrame, List[Dict[str, Any]]]:
    df = df.copy()
    df.columns = [c.strip() for c in df.columns]

    required = ["RT", "Base Peak", "Polarity", "File", "Area"]
    missing = [c for c in required if c not in df.columns]
    if missing:
        raise ValueError(f"Missing required columns: {', '.join(missing)}")

    df = df.dropna(subset=["RT"])

    # Ensure operator_mark column exists (may be absent when using pd.read_excel fallback)
    if "operator_mark" not in df.columns:
        df["operator_mark"] = None

    df["SampleType"] = df.apply(_assign_sample_type, axis=1)

    # Coarse screening — pair replicates
    coarse_rows = []
    for (stype, pol), grp in df.groupby(["SampleType", "Polarity"]):
        result = coarse_screen(grp)
        if not result.empty:
            coarse_rows.append(result)

    coarse_df = pd.concat(coarse_rows, ignore_index=True) if coarse_rows else pd.DataFrame()

    # Out-target screening (blank subtraction)
    blanks  = coarse_df[coarse_df["SampleType"] == "blank"]  if not coarse_df.empty else pd.DataFrame()
    samples = coarse_df[coarse_df["SampleType"] != "blank"] if not coarse_df.empty else pd.DataFrame()

    final_results = []
    for _, peak in samples.iterrows():
        if not blanks.empty:
            match = blanks[
                (blanks["Polarity"] == peak["Polarity"]) &
                (abs(blanks["RT_mean"] - peak["RT_mean"]) <= RT_TOL) &
                (abs(blanks["MZ_mean"] - peak["MZ_mean"]) <= MZ_TOL)
            ]
        else:
            match = pd.DataFrame()

        status = "Artifact" if not match.empty else "Real Compound"
        row = peak.to_dict()
        row["Status"] = status
        row["Why"]["BlankMatch"] = not match.empty
        if not match.empty:
            row["Why"]["BlankDetail"] = {
                "RT": match.iloc[0]["RT_mean"],
                "MZ": match.iloc[0]["MZ_mean"],
            }
        final_results.append(row)

    # Summary
    summary = []
    for (stype, pol), grp in df.groupby(["SampleType", "Polarity"]):
        confirmed_count = (
            len(coarse_df[(coarse_df["SampleType"] == stype) & (coarse_df["Polarity"] == pol)])
            if not coarse_df.empty else 0
        )
        if stype != "blank":
            sub = [r for r in final_results if r["SampleType"] == stype and r["Polarity"] == pol]
            artifacts = len([r for r in sub if r["Status"] == "Artifact"])
            real      = len([r for r in sub if r["Status"] == "Real Compound"])
        else:
            artifacts = real = 0

        color_driven = df[
            (df["SampleType"] == stype) & (df["Polarity"] == pol)
        ]["operator_mark"].notna().any()

        summary.append({
            "Sample": stype, "Polarity": pol,
            "TotalPeaks": len(grp),
            "Confirmed": confirmed_count,
            "Artifacts": artifacts,
            "RealCompounds": real,
            "ColorDriven": bool(color_driven),
        })

    return pd.DataFrame(final_results), pd.DataFrame(summary), final_results
