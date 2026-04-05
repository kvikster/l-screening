from __future__ import annotations

from dataclasses import asdict, dataclass
from math import sqrt
from typing import Any, Dict, List, Tuple

import pandas as pd

RT_TOL = 0.1
MZ_TOL = 0.3
DEFAULT_MZ_MODE = "da"
DEFAULT_SIGNAL_TO_BLANK_MIN = 3.0
DEFAULT_CV_HIGH_MAX = 15.0
DEFAULT_CV_MODERATE_MAX = 30.0

# Maps operator_mark → canonical SampleType used throughout the pipeline.
# Reps 1 & 2 collapse to the same SampleType so they are paired in coarse_screen.
_MARK_TO_STYPE = {
    "blank_positive": "blank",
    "blank_negative": "blank",
    "sample_rep1": "sample",
    "sample_rep2": "sample",
}

# Which operator_mark values count as "replicate 1" vs "replicate 2"
_REP1_MARKS = {"sample_rep1", "blank_positive"}  # arbitrary but consistent
_REP2_MARKS = {"sample_rep2", "blank_negative"}


@dataclass(frozen=True)
class ScreeningConfig:
    replicate_rt_tol: float = RT_TOL
    replicate_mz_tol: float = MZ_TOL
    replicate_mz_mode: str = DEFAULT_MZ_MODE
    blank_rt_tol: float = RT_TOL
    blank_mz_tol: float = MZ_TOL
    blank_mz_mode: str = DEFAULT_MZ_MODE
    signal_to_blank_min: float = DEFAULT_SIGNAL_TO_BLANK_MIN
    cv_high_max: float = DEFAULT_CV_HIGH_MAX
    cv_moderate_max: float = DEFAULT_CV_MODERATE_MAX


def build_screening_config(overrides: Dict[str, Any] | None = None) -> ScreeningConfig:
    if not overrides:
        return ScreeningConfig()

    values = asdict(ScreeningConfig())
    values.update({k: v for k, v in overrides.items() if v is not None})

    replicate_mz_mode = str(values["replicate_mz_mode"]).lower().strip()
    blank_mz_mode = str(values["blank_mz_mode"]).lower().strip()
    if replicate_mz_mode not in {"da", "ppm"}:
        raise ValueError("replicate_mz_mode must be either 'da' or 'ppm'")
    if blank_mz_mode not in {"da", "ppm"}:
        raise ValueError("blank_mz_mode must be either 'da' or 'ppm'")

    for key in (
        "replicate_rt_tol",
        "replicate_mz_tol",
        "blank_rt_tol",
        "blank_mz_tol",
        "signal_to_blank_min",
        "cv_high_max",
        "cv_moderate_max",
    ):
        values[key] = float(values[key])
        if values[key] < 0:
            raise ValueError(f"{key} must be >= 0")

    if values["cv_moderate_max"] < values["cv_high_max"]:
        raise ValueError("cv_moderate_max must be greater than or equal to cv_high_max")

    values["replicate_mz_mode"] = replicate_mz_mode
    values["blank_mz_mode"] = blank_mz_mode
    return ScreeningConfig(**values)


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


def _safe_round(value: float | None, digits: int) -> float | None:
    if value is None or pd.isna(value):
        return None
    return round(float(value), digits)


def _ppm_delta(mz_a: float, mz_b: float) -> float:
    center = (float(mz_a) + float(mz_b)) / 2
    if center == 0:
        return 0.0
    return abs(float(mz_a) - float(mz_b)) / center * 1_000_000


def _mz_delta(mz_a: float, mz_b: float, mode: str) -> float:
    if mode == "ppm":
        return _ppm_delta(mz_a, mz_b)
    return abs(float(mz_a) - float(mz_b))


def _fraction_of_tol(delta: float, tolerance: float) -> float:
    if tolerance <= 0:
        return 0.0 if delta == 0 else 1.5
    return min(delta / tolerance, 1.5)


def _calc_cv_percent(values: List[float]) -> float | None:
    clean = [float(v) for v in values if v is not None and not pd.isna(v)]
    if len(clean) < 2:
        return None
    mean = sum(clean) / len(clean)
    if mean == 0:
        return None
    if len(clean) == 2:
        std = abs(clean[0] - clean[1]) / sqrt(2)
    else:
        variance = sum((x - mean) ** 2 for x in clean) / (len(clean) - 1)
        std = sqrt(variance)
    return std / mean * 100


def _classify_replicate_quality(cv_percent: float | None, config: ScreeningConfig) -> str:
    if cv_percent is None:
        return "Unknown"
    if cv_percent <= config.cv_high_max:
        return "High"
    if cv_percent <= config.cv_moderate_max:
        return "Moderate"
    return "Low"


def _replicate_confidence_score(
    rt_delta: float,
    rt_tol: float,
    mz_delta_in_mode: float,
    mz_tol: float,
    cv_percent: float | None,
    color_paired: bool,
    config: ScreeningConfig,
) -> float:
    score = 100.0
    score -= _fraction_of_tol(rt_delta, rt_tol) * 20
    score -= _fraction_of_tol(mz_delta_in_mode, mz_tol) * 25

    if cv_percent is None:
        score -= 10
    elif cv_percent <= config.cv_high_max:
        pass
    elif cv_percent <= config.cv_moderate_max:
        score -= 12
    else:
        score -= min(35, 12 + (cv_percent - config.cv_moderate_max) * 0.7)

    if not color_paired:
        score -= 5

    return round(max(0.0, min(score, 100.0)), 1)


def _final_confidence_score(
    replicate_score: float,
    has_blank_match: bool,
    signal_to_blank_ratio: float | None,
    config: ScreeningConfig,
) -> float:
    score = float(replicate_score)
    if not has_blank_match:
        score += 3
    elif signal_to_blank_ratio is None:
        score -= 10
    elif signal_to_blank_ratio >= config.signal_to_blank_min:
        score += min(5, (signal_to_blank_ratio - config.signal_to_blank_min) * 0.5)
    else:
        deficit = (config.signal_to_blank_min - signal_to_blank_ratio) / max(config.signal_to_blank_min, 1e-9)
        score -= 15 + min(30, deficit * 30)

    return round(max(0.0, min(score, 100.0)), 1)


def _pair_replicates(group: pd.DataFrame) -> Tuple[pd.DataFrame, pd.DataFrame, bool]:
    marks_present = set(group["operator_mark"].dropna())
    colour_split = bool(marks_present & _REP1_MARKS) and bool(marks_present & _REP2_MARKS)

    if colour_split:
        rep1 = group[group["operator_mark"].isin(_REP1_MARKS)]
        rep2 = group[group["operator_mark"].isin(_REP2_MARKS)]
    else:
        files = group["File"].dropna().astype(str).unique()
        if len(files) < 2:
            return pd.DataFrame(), pd.DataFrame(), False
        rep1 = group[group["File"] == files[0]]
        rep2 = group[group["File"] == files[1]]

    return rep1, rep2, colour_split


def coarse_screen(group: pd.DataFrame, config: ScreeningConfig) -> pd.DataFrame:
    """
    Pair rep1 vs rep2 rows within the same (SampleType, Polarity) group.
    When operator colours are present the split is colour-driven; otherwise
    it falls back to the legacy file-order split.
    """
    rep1, rep2, colour_split = _pair_replicates(group)
    if rep1.empty or rep2.empty:
        return pd.DataFrame()

    confirmed = []
    for _, p1 in rep1.iterrows():
        for _, p2 in rep2.iterrows():
            rt_delta = abs(float(p1["RT"]) - float(p2["RT"]))
            mz_delta_da = abs(float(p1["Base Peak"]) - float(p2["Base Peak"]))
            mz_delta_ppm = _ppm_delta(float(p1["Base Peak"]), float(p2["Base Peak"]))
            mz_delta_in_mode = _mz_delta(float(p1["Base Peak"]), float(p2["Base Peak"]), config.replicate_mz_mode)

            if rt_delta > config.replicate_rt_tol or mz_delta_in_mode > config.replicate_mz_tol:
                continue

            area_values = [float(p1["Area"]), float(p2["Area"])]
            area_mean = sum(area_values) / len(area_values)
            area_cv_pct = _calc_cv_percent(area_values)
            replicate_quality = _classify_replicate_quality(area_cv_pct, config)
            replicate_score = _replicate_confidence_score(
                rt_delta=rt_delta,
                rt_tol=config.replicate_rt_tol,
                mz_delta_in_mode=mz_delta_in_mode,
                mz_tol=config.replicate_mz_tol,
                cv_percent=area_cv_pct,
                color_paired=colour_split,
                config=config,
            )

            confirmed.append(
                {
                    "Group": f"{p1['SampleType']}_{p1['Polarity']}",
                    "RT_mean": _safe_round((float(p1["RT"]) + float(p2["RT"])) / 2, 4),
                    "MZ_mean": _safe_round((float(p1["Base Peak"]) + float(p2["Base Peak"])) / 2, 6),
                    "Area_mean": _safe_round(area_mean, 2),
                    "AreaCVPct": _safe_round(area_cv_pct, 2),
                    "ReplicateQuality": replicate_quality,
                    "ReplicateCount": 2,
                    "ReplicateConfidenceScore": replicate_score,
                    "ConfidenceScore": replicate_score,
                    "Polarity": p1["Polarity"],
                    "SampleType": p1["SampleType"],
                    "Rep1_Label": p1.get("Label"),
                    "Rep2_Label": p2.get("Label"),
                    "Rep1_Mark": p1.get("operator_mark"),
                    "Rep2_Mark": p2.get("operator_mark"),
                    "Rep1_Color": p1.get("operator_color"),
                    "Rep2_Color": p2.get("operator_color"),
                    "Confirmed": "Yes",
                    "Why": {
                        "ReplicateRT": {
                            "rep1": _safe_round(p1["RT"], 4),
                            "rep2": _safe_round(p2["RT"], 4),
                            "delta": _safe_round(rt_delta, 4),
                            "tolerance": config.replicate_rt_tol,
                            "unit": "min",
                        },
                        "ReplicateMZ": {
                            "rep1": _safe_round(p1["Base Peak"], 6),
                            "rep2": _safe_round(p2["Base Peak"], 6),
                            "delta_da": _safe_round(mz_delta_da, 6),
                            "delta_ppm": _safe_round(mz_delta_ppm, 2),
                            "tolerance": config.replicate_mz_tol,
                            "mode": config.replicate_mz_mode,
                        },
                        "ReplicateArea": {
                            "values": [_safe_round(area_values[0], 2), _safe_round(area_values[1], 2)],
                            "mean": _safe_round(area_mean, 2),
                            "cv_pct": _safe_round(area_cv_pct, 2),
                        },
                        "ReplicateQuality": replicate_quality,
                        "ReplicateConfidenceScore": replicate_score,
                        "ColorPaired": colour_split,
                        "Matches": True,
                        "ThresholdProfile": {
                            "replicate_rt_tol": config.replicate_rt_tol,
                            "replicate_mz_tol": config.replicate_mz_tol,
                            "replicate_mz_mode": config.replicate_mz_mode,
                            "blank_rt_tol": config.blank_rt_tol,
                            "blank_mz_tol": config.blank_mz_tol,
                            "blank_mz_mode": config.blank_mz_mode,
                            "signal_to_blank_min": config.signal_to_blank_min,
                        },
                    },
                }
            )

    return pd.DataFrame(confirmed)


def _blank_candidates(peak: pd.Series, blanks: pd.DataFrame, config: ScreeningConfig) -> List[Dict[str, Any]]:
    candidates: List[Dict[str, Any]] = []
    peak_rt = float(peak["RT_mean"])
    peak_mz = float(peak["MZ_mean"])

    for _, blank in blanks[blanks["Polarity"] == peak["Polarity"]].iterrows():
        rt_delta = abs(float(blank["RT_mean"]) - peak_rt)
        mz_delta_da = abs(float(blank["MZ_mean"]) - peak_mz)
        mz_delta_ppm = _ppm_delta(float(blank["MZ_mean"]), peak_mz)
        mz_delta_in_mode = _mz_delta(float(blank["MZ_mean"]), peak_mz, config.blank_mz_mode)

        if rt_delta > config.blank_rt_tol or mz_delta_in_mode > config.blank_mz_tol:
            continue

        distance = _fraction_of_tol(rt_delta, config.blank_rt_tol) + _fraction_of_tol(
            mz_delta_in_mode, config.blank_mz_tol
        )
        candidates.append(
            {
                "blank_row": blank,
                "rt_delta": rt_delta,
                "mz_delta_da": mz_delta_da,
                "mz_delta_ppm": mz_delta_ppm,
                "mz_delta_in_mode": mz_delta_in_mode,
                "distance": distance,
            }
        )

    candidates.sort(
        key=lambda item: (
            item["distance"],
            -float(item["blank_row"].get("Area_mean") or 0),
        )
    )
    return candidates


def process_peaks(
    df: pd.DataFrame, config: ScreeningConfig | Dict[str, Any] | None = None
) -> Tuple[pd.DataFrame, pd.DataFrame, List[Dict[str, Any]]]:
    config = build_screening_config(config if isinstance(config, dict) else asdict(config) if config else None)

    df = df.copy()
    df.columns = [c.strip() for c in df.columns]

    required = ["RT", "Base Peak", "Polarity", "File", "Area"]
    missing = [c for c in required if c not in df.columns]
    if missing:
        raise ValueError(f"Missing required columns: {', '.join(missing)}")

    for numeric_col in ("RT", "Base Peak", "Area"):
        df[numeric_col] = pd.to_numeric(df[numeric_col], errors="coerce")

    df = df.dropna(subset=["RT", "Base Peak", "Area"])

    # Ensure operator_mark column exists (may be absent when using pd.read_excel fallback)
    if "operator_mark" not in df.columns:
        df["operator_mark"] = None
    if "operator_color" not in df.columns:
        df["operator_color"] = None
    if "Label" not in df.columns:
        df["Label"] = None

    df["SampleType"] = df.apply(_assign_sample_type, axis=1)

    # Coarse screening — pair replicates
    coarse_rows = []
    for (_, _), grp in df.groupby(["SampleType", "Polarity"]):
        result = coarse_screen(grp, config)
        if not result.empty:
            coarse_rows.append(result)

    coarse_df = pd.concat(coarse_rows, ignore_index=True) if coarse_rows else pd.DataFrame()

    # Out-target screening (blank subtraction)
    blanks = coarse_df[coarse_df["SampleType"] == "blank"] if not coarse_df.empty else pd.DataFrame()
    samples = coarse_df[coarse_df["SampleType"] != "blank"] if not coarse_df.empty else pd.DataFrame()

    final_results = []
    for _, peak in samples.iterrows():
        candidates = _blank_candidates(peak, blanks, config) if not blanks.empty else []
        best_blank = candidates[0] if candidates else None

        signal_to_blank_ratio = None
        if best_blank:
            blank_area = float(best_blank["blank_row"].get("Area_mean") or 0)
            signal_to_blank_ratio = None if blank_area <= 0 else float(peak["Area_mean"]) / blank_area

        status = "Real Compound"
        if best_blank and signal_to_blank_ratio is not None and signal_to_blank_ratio < config.signal_to_blank_min:
            status = "Artifact"
        elif best_blank and signal_to_blank_ratio is None:
            status = "Artifact"

        row = peak.to_dict()
        row["SignalToBlankRatio"] = _safe_round(signal_to_blank_ratio, 2)
        row["ConfidenceScore"] = _final_confidence_score(
            replicate_score=float(peak["ReplicateConfidenceScore"]),
            has_blank_match=bool(best_blank),
            signal_to_blank_ratio=signal_to_blank_ratio,
            config=config,
        )
        row["Status"] = status
        row["Why"]["BlankMatch"] = bool(best_blank)
        row["Why"]["BlankCandidateCount"] = len(candidates)
        row["Why"]["SignalToBlankRatio"] = _safe_round(signal_to_blank_ratio, 2)
        row["Why"]["SignalToBlankThreshold"] = config.signal_to_blank_min
        row["Why"]["ConfidenceScore"] = row["ConfidenceScore"]
        row["Why"]["Decision"] = status
        if best_blank:
            row["Why"]["BlankDetail"] = {
                "RT": _safe_round(best_blank["blank_row"]["RT_mean"], 4),
                "MZ": _safe_round(best_blank["blank_row"]["MZ_mean"], 6),
                "Area_mean": _safe_round(best_blank["blank_row"]["Area_mean"], 2),
                "rt_delta": _safe_round(best_blank["rt_delta"], 4),
                "mz_delta_da": _safe_round(best_blank["mz_delta_da"], 6),
                "mz_delta_ppm": _safe_round(best_blank["mz_delta_ppm"], 2),
                "tolerance": {
                    "rt": config.blank_rt_tol,
                    "mz": config.blank_mz_tol,
                    "mz_mode": config.blank_mz_mode,
                },
            }
        final_results.append(row)

    # Summary
    summary = []
    for (stype, pol), grp in df.groupby(["SampleType", "Polarity"]):
        confirmed_subset = (
            coarse_df[(coarse_df["SampleType"] == stype) & (coarse_df["Polarity"] == pol)]
            if not coarse_df.empty
            else pd.DataFrame()
        )
        confirmed_count = len(confirmed_subset)

        if stype != "blank":
            sub = [r for r in final_results if r["SampleType"] == stype and r["Polarity"] == pol]
            artifacts = len([r for r in sub if r["Status"] == "Artifact"])
            real = len([r for r in sub if r["Status"] == "Real Compound"])
            mean_sb = _safe_round(
                pd.Series([r["SignalToBlankRatio"] for r in sub if r.get("SignalToBlankRatio") is not None]).mean(),
                2,
            )
        else:
            sub = []
            artifacts = 0
            real = 0
            mean_sb = None

        color_driven = df[
            (df["SampleType"] == stype) & (df["Polarity"] == pol)
        ]["operator_mark"].notna().any()

        summary.append(
            {
                "Sample": stype,
                "Polarity": pol,
                "TotalPeaks": len(grp),
                "Confirmed": confirmed_count,
                "Artifacts": artifacts,
                "RealCompounds": real,
                "ColorDriven": bool(color_driven),
                "MeanCVPct": _safe_round(confirmed_subset["AreaCVPct"].dropna().mean(), 2)
                if not confirmed_subset.empty
                else None,
                "HighQuality": int((confirmed_subset["ReplicateQuality"] == "High").sum())
                if not confirmed_subset.empty
                else 0,
                "ModerateQuality": int((confirmed_subset["ReplicateQuality"] == "Moderate").sum())
                if not confirmed_subset.empty
                else 0,
                "LowQuality": int((confirmed_subset["ReplicateQuality"] == "Low").sum())
                if not confirmed_subset.empty
                else 0,
                "MeanConfidenceScore": _safe_round(confirmed_subset["ConfidenceScore"].dropna().mean(), 1)
                if not confirmed_subset.empty
                else None,
                "MeanSignalToBlankRatio": mean_sb,
            }
        )

    return pd.DataFrame(final_results), pd.DataFrame(summary), final_results
