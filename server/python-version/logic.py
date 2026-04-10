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

# Maps operator_mark → canonical SampleType used throughout the raw-stage pipeline.
# Reps 1 & 2 collapse to the same SampleType so they are paired in coarse_screen.
_MARK_TO_STYPE = {
    "blank_positive": "blank",
    "blank_negative": "blank",
    "sample_rep1": "sample",
    "sample_rep2": "sample",
}

# Which operator_mark values count as "replicate 1" vs "replicate 2"
_REP1_MARKS = {"sample_rep1", "blank_positive"}
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
    mark = row.get("operator_mark")
    if mark and mark in _MARK_TO_STYPE:
        return _MARK_TO_STYPE[mark]
    return _classify_from_filename(row["File"])


def _parallel_sample_family(sample_type: str | None) -> str:
    value = str(sample_type or "unknown")
    if value == "blank":
        return "blank"
    if value.startswith("sample"):
        return "sample"
    return value


def _safe_round(value: float | None, digits: int) -> float | None:
    if value is None or pd.isna(value):
        return None
    return round(float(value), digits)


def _optional_float(value: Any) -> float | None:
    if value is None:
        return None
    try:
        if pd.isna(value):
            return None
    except TypeError:
        pass
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


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
    mz_delta_in_mode: float | None,
    mz_tol: float,
    cv_percent: float | None,
    color_paired: bool,
    use_mz: bool,
    config: ScreeningConfig,
) -> float:
    score = 100.0
    score -= _fraction_of_tol(rt_delta, rt_tol) * 20
    if use_mz and mz_delta_in_mode is not None:
        score -= _fraction_of_tol(mz_delta_in_mode, mz_tol) * 25
    else:
        score -= 10

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


def _match_metrics(
    rt_a: float,
    mz_a: float | None,
    rt_b: float,
    mz_b: float | None,
    rt_tol: float,
    mz_tol: float,
    mz_mode: str,
) -> Dict[str, Any]:
    rt_delta = abs(float(rt_a) - float(rt_b))
    uses_mz = mz_a is not None and mz_b is not None
    if uses_mz:
        mz_delta_da = abs(float(mz_a) - float(mz_b))
        mz_delta_ppm = _ppm_delta(float(mz_a), float(mz_b))
        mz_delta_in_mode = _mz_delta(float(mz_a), float(mz_b), mz_mode)
        matches = rt_delta <= rt_tol and mz_delta_in_mode <= mz_tol
        distance = _fraction_of_tol(rt_delta, rt_tol) + _fraction_of_tol(mz_delta_in_mode, mz_tol)
    else:
        mz_delta_da = None
        mz_delta_ppm = None
        mz_delta_in_mode = None
        matches = rt_delta <= rt_tol
        distance = _fraction_of_tol(rt_delta, rt_tol)

    return {
        "matches": matches,
        "uses_mz": uses_mz,
        "rt_delta": rt_delta,
        "mz_delta_da": mz_delta_da,
        "mz_delta_ppm": mz_delta_ppm,
        "mz_delta_in_mode": mz_delta_in_mode,
        "distance": distance,
    }


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


def _replicate_bucket_key(row: pd.Series, use_marks: bool) -> str:
    if use_marks:
        return str(row.get("operator_mark") or row.get("File") or "unassigned")
    return str(row.get("File") or row.name)


def _split_replicate_buckets(group: pd.DataFrame) -> Tuple[List[Tuple[str, pd.DataFrame]], bool]:
    marks_present = set(group["operator_mark"].dropna())
    colour_split = bool(marks_present & _REP1_MARKS) and bool(marks_present & _REP2_MARKS)
    unique_files = group["File"].dropna().astype(str).nunique()
    use_marks = colour_split and unique_files <= 2

    prepared = group.copy()
    prepared["_replicate_bucket"] = prepared.apply(lambda row: _replicate_bucket_key(row, use_marks), axis=1)
    buckets = []
    for bucket_name, bucket_df in prepared.groupby("_replicate_bucket", sort=False):
        buckets.append((str(bucket_name), bucket_df.drop(columns=["_replicate_bucket"])))
    return buckets, use_marks


def _candidate_distance(row_a: pd.Series, row_b: pd.Series, config: ScreeningConfig) -> Dict[str, Any]:
    return _match_metrics(
        rt_a=float(row_a["RT"]),
        mz_a=_optional_float(row_a.get("Base Peak")),
        rt_b=float(row_b["RT"]),
        mz_b=_optional_float(row_b.get("Base Peak")),
        rt_tol=config.replicate_rt_tol,
        mz_tol=config.replicate_mz_tol,
        mz_mode=config.replicate_mz_mode,
    )


def _cluster_centroid(rows: List[pd.Series]) -> Dict[str, float | None]:
    mz_values = [_optional_float(row.get("Base Peak")) for row in rows]
    clean_mz = [value for value in mz_values if value is not None]
    return {
        "RT": sum(float(row["RT"]) for row in rows) / len(rows),
        "Base Peak": sum(clean_mz) / len(clean_mz) if clean_mz else None,
    }


def _cluster_match_to_centroid(
    centroid: Dict[str, float | None], candidate: pd.Series, config: ScreeningConfig
) -> Dict[str, Any]:
    return _match_metrics(
        rt_a=float(candidate["RT"]),
        mz_a=_optional_float(candidate.get("Base Peak")),
        rt_b=float(centroid["RT"]),
        mz_b=_optional_float(centroid.get("Base Peak")),
        rt_tol=config.replicate_rt_tol,
        mz_tol=config.replicate_mz_tol,
        mz_mode=config.replicate_mz_mode,
    )


def _choose_cluster_members(
    seed_row: pd.Series,
    seed_bucket_idx: int,
    buckets: List[Tuple[str, pd.DataFrame]],
    used_indices: set[tuple[int, int]],
    config: ScreeningConfig,
) -> List[Dict[str, Any]]:
    members = [
        {
            "bucket_name": buckets[seed_bucket_idx][0],
            "bucket_index": seed_bucket_idx,
            "row_index": int(seed_row.name),
            "row": seed_row,
            "distance": 0.0,
        }
    ]
    centroid = _cluster_centroid([seed_row])

    for bucket_idx, (bucket_name, bucket_df) in enumerate(buckets):
        if bucket_idx == seed_bucket_idx:
            continue

        best_candidate: Dict[str, Any] | None = None
        for row_index, candidate in bucket_df.iterrows():
            index_key = (bucket_idx, int(row_index))
            if index_key in used_indices:
                continue
            metrics = _cluster_match_to_centroid(centroid, candidate, config)
            if not metrics["matches"]:
                continue
            contender = {
                "bucket_name": bucket_name,
                "bucket_index": bucket_idx,
                "row_index": int(row_index),
                "row": candidate,
                "distance": metrics["distance"],
                "metrics": metrics,
            }
            if (
                best_candidate is None
                or contender["distance"] < best_candidate["distance"]
                or (
                    contender["distance"] == best_candidate["distance"]
                    and float(candidate["Area"]) > float(best_candidate["row"]["Area"])
                )
            ):
                best_candidate = contender

        if best_candidate is not None:
            members.append(best_candidate)
            centroid = _cluster_centroid([member["row"] for member in members])

    return members


def _pairwise_cluster_metrics(rows: List[pd.Series], config: ScreeningConfig) -> Dict[str, Any]:
    rt_deltas: List[float] = []
    mz_deltas_da: List[float] = []
    mz_deltas_ppm: List[float] = []
    mz_deltas_in_mode: List[float] = []

    for left in range(len(rows)):
        for right in range(left + 1, len(rows)):
            metrics = _candidate_distance(rows[left], rows[right], config)
            rt_deltas.append(metrics["rt_delta"])
            if metrics["uses_mz"]:
                mz_deltas_da.append(float(metrics["mz_delta_da"]))
                mz_deltas_ppm.append(float(metrics["mz_delta_ppm"]))
                mz_deltas_in_mode.append(float(metrics["mz_delta_in_mode"]))

    return {
        "uses_mz": bool(mz_deltas_in_mode),
        "max_rt_delta": max(rt_deltas, default=0.0),
        "max_mz_delta_da": max(mz_deltas_da, default=None),
        "max_mz_delta_ppm": max(mz_deltas_ppm, default=None),
        "max_mz_delta_in_mode": max(mz_deltas_in_mode, default=None),
        "mean_rt_delta": sum(rt_deltas) / len(rt_deltas) if rt_deltas else 0.0,
        "mean_mz_delta_in_mode": sum(mz_deltas_in_mode) / len(mz_deltas_in_mode) if mz_deltas_in_mode else None,
    }


def _compact_list(values: List[Any]) -> List[Any]:
    return [value for value in values if value is not None and not (isinstance(value, float) and pd.isna(value))]


def _weighted_mean(values: List[float | None], weights: List[int]) -> float | None:
    weighted_total = 0.0
    total_weight = 0
    for value, weight in zip(values, weights):
        numeric_value = _optional_float(value)
        numeric_weight = int(weight)
        if numeric_value is None or numeric_weight <= 0:
            continue
        weighted_total += numeric_value * numeric_weight
        total_weight += numeric_weight
    if total_weight == 0:
        return None
    return weighted_total / total_weight


def _replicate_area_values(row: Dict[str, Any]) -> List[float]:
    """Extract per-replicate area values from a coarse-level row's Why.ReplicateArea.values.

    NOTE: This reads from ``"ReplicateArea"``, not ``"Area"`` (used by parallel merge).
    It is safe to call only on coarse-level rows; parallel-merged rows fall through
    to the ``[Area_mean] * ReplicateCount`` fallback.
    """
    values = row.get("Why", {}).get("ReplicateArea", {}).get("values", [])
    if not isinstance(values, list):
        return []
    return [float(value) for value in _compact_list(values)]


def _cluster_to_confirmed_row(
    members: List[Dict[str, Any]],
    colour_split: bool,
    config: ScreeningConfig,
) -> Dict[str, Any]:
    rows = [member["row"] for member in members]
    area_values = [float(row["Area"]) for row in rows]
    rt_values = [float(row["RT"]) for row in rows]
    mz_values = _compact_list([_optional_float(row.get("Base Peak")) for row in rows])
    area_mean = sum(area_values) / len(area_values)
    area_cv_pct = _calc_cv_percent(area_values)
    replicate_quality = _classify_replicate_quality(area_cv_pct, config)
    pairwise_metrics = _pairwise_cluster_metrics(rows, config)
    matching_mode = "RT+MZ" if pairwise_metrics["uses_mz"] else "RT"
    replicate_score = _replicate_confidence_score(
        rt_delta=pairwise_metrics["mean_rt_delta"],
        rt_tol=config.replicate_rt_tol,
        mz_delta_in_mode=pairwise_metrics["mean_mz_delta_in_mode"],
        mz_tol=config.replicate_mz_tol,
        cv_percent=area_cv_pct,
        color_paired=colour_split,
        use_mz=pairwise_metrics["uses_mz"],
        config=config,
    )

    first_row = rows[0]
    second_row = rows[1] if len(rows) > 1 else None
    replicate_labels = [row.get("Label") for row in rows]
    replicate_marks = [row.get("operator_mark") for row in rows]
    replicate_colors = [row.get("operator_color") for row in rows]
    replicate_files = [row.get("File") for row in rows]

    return {
        "Group": f"{first_row['SampleType']}_{first_row['Polarity']}",
        "RT_mean": _safe_round(sum(rt_values) / len(rt_values), 4),
        "MZ_mean": _safe_round(sum(mz_values) / len(mz_values), 6) if mz_values else None,
        "Area_mean": _safe_round(area_mean, 2),
        "AreaCVPct": _safe_round(area_cv_pct, 2),
        "ReplicateQuality": replicate_quality,
        "ReplicateCount": len(rows),
        "ReplicateConfidenceScore": replicate_score,
        "ConfidenceScore": replicate_score,
        "Polarity": first_row["Polarity"],
        "SampleType": first_row["SampleType"],
        "Rep1_Label": first_row.get("Label"),
        "Rep2_Label": second_row.get("Label") if second_row is not None else None,
        "Rep1_Mark": first_row.get("operator_mark"),
        "Rep2_Mark": second_row.get("operator_mark") if second_row is not None else None,
        "Rep1_Color": first_row.get("operator_color"),
        "Rep2_Color": second_row.get("operator_color") if second_row is not None else None,
        "ReplicateFiles": replicate_files,
        "ReplicateLabels": replicate_labels,
        "ReplicateMarks": replicate_marks,
        "ReplicateColors": replicate_colors,
        "Confirmed": "Yes",
        "MatchingMode": matching_mode,
        "ParallelMatch": False,
        "ParallelSampleCount": 1,
        "ParallelSourceSamples": [str(first_row["SampleType"])],
        "BlankAreaMean": None,
        "AreaDifference": None,
        "Why": {
            "ReplicateStrategy": "greedy_cluster_with_singleton_carryover",
            "ReplicateBuckets": [member["bucket_name"] for member in members],
            "ReplicateCount": len(rows),
            "ReplicateMembers": [
                {
                    "bucket": member["bucket_name"],
                    "file": row.get("File"),
                    "label": row.get("Label"),
                    "operator_mark": row.get("operator_mark"),
                    "operator_color": row.get("operator_color"),
                    "rt": _safe_round(row.get("RT"), 4),
                    "mz": _safe_round(_optional_float(row.get("Base Peak")), 6),
                    "area": _safe_round(row.get("Area"), 2),
                }
                for member, row in [(member, member["row"]) for member in members]
            ],
            "ReplicateRT": {
                "mean": _safe_round(sum(rt_values) / len(rt_values), 4),
                "max_delta": _safe_round(pairwise_metrics["max_rt_delta"], 4),
                "tolerance": config.replicate_rt_tol,
                "unit": "min",
            },
            "ReplicateMZ": {
                "mean": _safe_round(sum(mz_values) / len(mz_values), 6) if mz_values else None,
                "max_delta_da": _safe_round(pairwise_metrics["max_mz_delta_da"], 6),
                "max_delta_ppm": _safe_round(pairwise_metrics["max_mz_delta_ppm"], 2),
                "tolerance": config.replicate_mz_tol,
                "mode": config.replicate_mz_mode,
                "used": pairwise_metrics["uses_mz"],
            },
            "ReplicateArea": {
                "values": [_safe_round(value, 2) for value in area_values],
                "mean": _safe_round(area_mean, 2),
                "cv_pct": _safe_round(area_cv_pct, 2),
            },
            "ReplicateQuality": replicate_quality,
            "ReplicateConfidenceScore": replicate_score,
            "ColorPaired": colour_split,
            "Matches": len(rows) > 1,
            "MatchingMode": matching_mode,
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


def coarse_screen(group: pd.DataFrame, config: ScreeningConfig) -> pd.DataFrame:
    buckets, colour_split = _split_replicate_buckets(group)
    if not buckets:
        return pd.DataFrame()

    used_indices: set[tuple[int, int]] = set()
    seed_candidates: List[Dict[str, Any]] = []
    for bucket_idx, (_, bucket_df) in enumerate(buckets):
        for row_index, row in bucket_df.iterrows():
            seed_candidates.append(
                {
                    "bucket_idx": bucket_idx,
                    "row_index": int(row_index),
                    "row": row,
                    "area": float(row["Area"]),
                }
            )

    seed_candidates.sort(
        key=lambda item: (
            -item["area"],
            float(item["row"]["RT"]),
            _optional_float(item["row"].get("Base Peak")) or 0.0,
        )
    )

    confirmed = []
    for seed in seed_candidates:
        seed_key = (seed["bucket_idx"], seed["row_index"])
        if seed_key in used_indices:
            continue

        members = _choose_cluster_members(
            seed_row=seed["row"],
            seed_bucket_idx=seed["bucket_idx"],
            buckets=buckets,
            used_indices=used_indices,
            config=config,
        )
        if len(members) < 2:
            continue

        for member in members:
            used_indices.add((member["bucket_index"], member["row_index"]))
        confirmed.append(_cluster_to_confirmed_row(members, colour_split, config))

    for seed in seed_candidates:
        seed_key = (seed["bucket_idx"], seed["row_index"])
        if seed_key in used_indices:
            continue
        used_indices.add(seed_key)
        confirmed.append(
            _cluster_to_confirmed_row(
                [
                    {
                        "bucket_name": buckets[seed["bucket_idx"]][0],
                        "bucket_index": seed["bucket_idx"],
                        "row_index": seed["row_index"],
                        "row": seed["row"],
                        "distance": 0.0,
                    }
                ],
                colour_split,
                config,
            )
        )

    return pd.DataFrame(confirmed)


def _blank_candidates(peak: pd.Series | Dict[str, Any], blanks: pd.DataFrame, config: ScreeningConfig) -> List[Dict[str, Any]]:
    candidates: List[Dict[str, Any]] = []
    peak_rt = float(peak["RT_mean"])
    peak_mz = _optional_float(peak.get("MZ_mean"))

    for _, blank in blanks[blanks["Polarity"] == peak["Polarity"]].iterrows():
        metrics = _match_metrics(
            rt_a=float(blank["RT_mean"]),
            mz_a=_optional_float(blank.get("MZ_mean")),
            rt_b=peak_rt,
            mz_b=peak_mz,
            rt_tol=config.blank_rt_tol,
            mz_tol=config.blank_mz_tol,
            mz_mode=config.blank_mz_mode,
        )

        if not metrics["matches"]:
            continue

        candidates.append(
            {
                "blank_row": blank,
                **metrics,
            }
        )

    candidates.sort(
        key=lambda item: (
            item["distance"],
            -float(item["blank_row"].get("Area_mean") or 0),
        )
    )
    return candidates


def _final_row_match_to_centroid(
    centroid: Dict[str, float | None], candidate: Dict[str, Any], config: ScreeningConfig
) -> Dict[str, Any]:
    return _match_metrics(
        rt_a=float(candidate["RT_mean"]),
        mz_a=_optional_float(candidate.get("MZ_mean")),
        rt_b=float(centroid["RT_mean"]),
        mz_b=_optional_float(centroid.get("MZ_mean")),
        rt_tol=config.replicate_rt_tol,
        mz_tol=config.replicate_mz_tol,
        mz_mode=config.replicate_mz_mode,
    )


def _final_centroid(rows: List[Dict[str, Any]]) -> Dict[str, float | None]:
    mz_values = _compact_list([_optional_float(row.get("MZ_mean")) for row in rows])
    return {
        "RT_mean": sum(float(row["RT_mean"]) for row in rows) / len(rows),
        "MZ_mean": sum(mz_values) / len(mz_values) if mz_values else None,
    }


def _choose_parallel_members(
    seed_row: Dict[str, Any],
    seed_row_idx: int,
    seed_bucket_idx: int,
    buckets: List[Tuple[str, List[Dict[str, Any]]]],
    used_indices: set[tuple[int, int]],
    config: ScreeningConfig,
) -> List[Dict[str, Any]]:
    members = [
        {
            "bucket_name": buckets[seed_bucket_idx][0],
            "bucket_index": seed_bucket_idx,
            "row_index": seed_row_idx,
            "row": seed_row,
            "distance": 0.0,
        }
    ]
    centroid = _final_centroid([seed_row])

    for bucket_idx, (bucket_name, bucket_rows) in enumerate(buckets):
        if bucket_idx == seed_bucket_idx:
            continue

        best_candidate: Dict[str, Any] | None = None
        for row_index, candidate in enumerate(bucket_rows):
            index_key = (bucket_idx, row_index)
            if index_key in used_indices:
                continue
            metrics = _final_row_match_to_centroid(centroid, candidate, config)
            if not metrics["matches"]:
                continue
            contender = {
                "bucket_name": bucket_name,
                "bucket_index": bucket_idx,
                "row_index": row_index,
                "row": candidate,
                "distance": metrics["distance"],
                "metrics": metrics,
            }
            if (
                best_candidate is None
                or contender["distance"] < best_candidate["distance"]
                or (
                    contender["distance"] == best_candidate["distance"]
                    and float(candidate["Area_mean"]) > float(best_candidate["row"]["Area_mean"])
                )
            ):
                best_candidate = contender

        if best_candidate is not None:
            members.append(best_candidate)
            centroid = _final_centroid([member["row"] for member in members])

    return members


def _merge_parallel_cluster(
    members: List[Dict[str, Any]],
    family: str,
    polarity: str,
    config: ScreeningConfig,
) -> Dict[str, Any]:
    rows = [member["row"] for member in members]
    replicate_counts = [int(row.get("ReplicateCount") or 0) for row in rows]
    rt_values = [float(row["RT_mean"]) for row in rows]
    mz_row_values = [_optional_float(row.get("MZ_mean")) for row in rows]
    source_area_means = [float(row["Area_mean"]) for row in rows]
    replicate_area_values = [
        area
        for row in rows
        for area in (
            _replicate_area_values(row)
            or [float(row["Area_mean"])] * max(int(row.get("ReplicateCount") or 0), 1)
        )
    ]
    rt_mean = _weighted_mean(rt_values, replicate_counts) or 0.0
    mz_mean = _weighted_mean(mz_row_values, replicate_counts)
    area_mean = _weighted_mean(source_area_means, replicate_counts) or 0.0
    area_cv_pct = _calc_cv_percent(replicate_area_values)
    total_replicates = int(sum(replicate_counts))
    source_sample_types = sorted({str(row.get("SampleType") or family) for row in rows})
    replicate_files = [item for row in rows for item in row.get("ReplicateFiles", [])]
    replicate_labels = [item for row in rows for item in row.get("ReplicateLabels", [])]
    replicate_marks = [item for row in rows for item in row.get("ReplicateMarks", [])]
    replicate_colors = [item for row in rows for item in row.get("ReplicateColors", [])]
    pairwise_metrics = {
        "uses_mz": False,
        "max_rt_delta": 0.0,
        "max_mz_delta_da": None,
        "max_mz_delta_ppm": None,
        "mean_rt_delta": 0.0,
        "mean_mz_delta_in_mode": None,
    }
    if len(rows) > 1:
        rt_deltas: List[float] = []
        mz_deltas_da: List[float] = []
        mz_deltas_ppm: List[float] = []
        mz_deltas_in_mode: List[float] = []
        for left in range(len(rows)):
            for right in range(left + 1, len(rows)):
                metrics = _match_metrics(
                    rt_a=float(rows[left]["RT_mean"]),
                    mz_a=_optional_float(rows[left].get("MZ_mean")),
                    rt_b=float(rows[right]["RT_mean"]),
                    mz_b=_optional_float(rows[right].get("MZ_mean")),
                    rt_tol=config.replicate_rt_tol,
                    mz_tol=config.replicate_mz_tol,
                    mz_mode=config.replicate_mz_mode,
                )
                rt_deltas.append(metrics["rt_delta"])
                if metrics["uses_mz"]:
                    mz_deltas_da.append(float(metrics["mz_delta_da"]))
                    mz_deltas_ppm.append(float(metrics["mz_delta_ppm"]))
                    mz_deltas_in_mode.append(float(metrics["mz_delta_in_mode"]))
        pairwise_metrics = {
            "uses_mz": bool(mz_deltas_in_mode),
            "max_rt_delta": max(rt_deltas, default=0.0),
            "max_mz_delta_da": max(mz_deltas_da, default=None),
            "max_mz_delta_ppm": max(mz_deltas_ppm, default=None),
            "mean_rt_delta": sum(rt_deltas) / len(rt_deltas) if rt_deltas else 0.0,
            "mean_mz_delta_in_mode": sum(mz_deltas_in_mode) / len(mz_deltas_in_mode) if mz_deltas_in_mode else None,
        }

    matching_mode = "RT+MZ" if pairwise_metrics["uses_mz"] and all(row.get("MatchingMode") == "RT+MZ" for row in rows) else "RT"
    replicate_quality = _classify_replicate_quality(area_cv_pct, config)
    replicate_score = _replicate_confidence_score(
        rt_delta=pairwise_metrics["mean_rt_delta"],
        rt_tol=config.replicate_rt_tol,
        mz_delta_in_mode=pairwise_metrics["mean_mz_delta_in_mode"],
        mz_tol=config.replicate_mz_tol,
        cv_percent=area_cv_pct,
        color_paired=len(source_sample_types) > 1,
        use_mz=pairwise_metrics["uses_mz"],
        config=config,
    )

    all_labels = replicate_labels + [row.get("Rep1_Label") for row in rows] + [row.get("Rep2_Label") for row in rows]
    all_marks = replicate_marks + [row.get("Rep1_Mark") for row in rows] + [row.get("Rep2_Mark") for row in rows]
    all_colors = replicate_colors + [row.get("Rep1_Color") for row in rows] + [row.get("Rep2_Color") for row in rows]
    compact_labels = _compact_list(all_labels)
    compact_marks = _compact_list(all_marks)
    compact_colors = _compact_list(all_colors)

    return {
        "Group": f"{family}_{polarity}",
        "RT_mean": _safe_round(rt_mean, 4),
        "MZ_mean": _safe_round(mz_mean, 6),
        "Area_mean": _safe_round(area_mean, 2),
        "AreaCVPct": _safe_round(area_cv_pct, 2),
        "ReplicateQuality": replicate_quality,
        "ReplicateCount": total_replicates,
        "ReplicateConfidenceScore": replicate_score,
        "ConfidenceScore": replicate_score,
        "Polarity": polarity,
        "SampleType": family,
        "Rep1_Label": compact_labels[0] if len(compact_labels) > 0 else None,
        "Rep2_Label": compact_labels[1] if len(compact_labels) > 1 else None,
        "Rep1_Mark": compact_marks[0] if len(compact_marks) > 0 else None,
        "Rep2_Mark": compact_marks[1] if len(compact_marks) > 1 else None,
        "Rep1_Color": compact_colors[0] if len(compact_colors) > 0 else None,
        "Rep2_Color": compact_colors[1] if len(compact_colors) > 1 else None,
        "ReplicateFiles": replicate_files,
        "ReplicateLabels": replicate_labels,
        "ReplicateMarks": replicate_marks,
        "ReplicateColors": replicate_colors,
        "Confirmed": "Yes",
        "MatchingMode": matching_mode,
        "ParallelMatch": len(source_sample_types) > 1,
        "ParallelSampleCount": len(source_sample_types),
        "ParallelSourceSamples": source_sample_types,
        "BlankAreaMean": None,
        "AreaDifference": None,
        "Why": {
            "ParallelMerge": {
                "Strategy": "greedy_cluster_union",
                "SourceSamples": source_sample_types,
                "ParallelSampleCount": len(source_sample_types),
                "MatchedAcrossSamples": len(source_sample_types) > 1,
                "MatchingMode": matching_mode,
                "RT": {
                    "mean": _safe_round(rt_mean, 4),
                    "max_delta": _safe_round(pairwise_metrics["max_rt_delta"], 4),
                    "tolerance": config.replicate_rt_tol,
                },
                "MZ": {
                    "mean": _safe_round(mz_mean, 6),
                    "max_delta_da": _safe_round(pairwise_metrics["max_mz_delta_da"], 6),
                    "max_delta_ppm": _safe_round(pairwise_metrics["max_mz_delta_ppm"], 2),
                    "tolerance": config.replicate_mz_tol,
                    "mode": config.replicate_mz_mode,
                    "used": pairwise_metrics["uses_mz"],
                },
                "Area": {
                    "values": [_safe_round(value, 2) for value in replicate_area_values],
                    "mean": _safe_round(area_mean, 2),
                },
                "SourceRows": [
                    {
                        "SampleType": row.get("SampleType"),
                        "RT_mean": _safe_round(row.get("RT_mean"), 4),
                        "MZ_mean": _safe_round(_optional_float(row.get("MZ_mean")), 6),
                        "Area_mean": _safe_round(row.get("Area_mean"), 2),
                        "ReplicateCount": row.get("ReplicateCount"),
                        "MatchingMode": row.get("MatchingMode"),
                    }
                    for row in rows
                ],
                "SourceWhy": [row.get("Why") for row in rows],
            },
            "ReplicateConfidenceScore": replicate_score,
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


def _parallel_merge_samples(samples: List[Dict[str, Any]], config: ScreeningConfig) -> List[Dict[str, Any]]:
    grouped: Dict[Tuple[str, str], List[Dict[str, Any]]] = {}
    for row in samples:
        family = _parallel_sample_family(row.get("SampleType"))
        key = (family, str(row.get("Polarity")))
        grouped.setdefault(key, []).append(row)

    merged: List[Dict[str, Any]] = []
    for (family, polarity), rows in grouped.items():
        bucket_map: Dict[str, List[Dict[str, Any]]] = {}
        bucket_order: List[str] = []
        for row in rows:
            bucket_name = str(row.get("SampleType") or family)
            if bucket_name not in bucket_map:
                bucket_order.append(bucket_name)
                bucket_map[bucket_name] = []
            bucket_map[bucket_name].append(row)

        buckets = [(bucket_name, bucket_map[bucket_name]) for bucket_name in bucket_order]

        seed_candidates: List[Dict[str, Any]] = []
        for bucket_idx, (_, bucket_rows) in enumerate(buckets):
            for row_index, row in enumerate(bucket_rows):
                seed_candidates.append(
                    {
                        "bucket_idx": bucket_idx,
                        "row_index": row_index,
                        "row": row,
                        "area": float(row["Area_mean"]),
                    }
                )

        seed_candidates.sort(
            key=lambda item: (
                -item["area"],
                float(item["row"]["RT_mean"]),
                _optional_float(item["row"].get("MZ_mean")) or 0.0,
            )
        )

        used_indices: set[tuple[int, int]] = set()
        for seed in seed_candidates:
            seed_key = (seed["bucket_idx"], seed["row_index"])
            if seed_key in used_indices:
                continue
            members = _choose_parallel_members(
                seed_row=seed["row"],
                seed_row_idx=seed["row_index"],
                seed_bucket_idx=seed["bucket_idx"],
                buckets=buckets,
                used_indices=used_indices,
                config=config,
            )
            if len(members) < 2:
                continue
            for member in members:
                used_indices.add((member["bucket_index"], member["row_index"]))
            merged.append(_merge_parallel_cluster(members, family, polarity, config))

        for seed in seed_candidates:
            seed_key = (seed["bucket_idx"], seed["row_index"])
            if seed_key in used_indices:
                continue
            used_indices.add(seed_key)
            merged.append(
                _merge_parallel_cluster(
                    [
                        {
                            "bucket_name": buckets[seed["bucket_idx"]][0],
                            "bucket_index": seed["bucket_idx"],
                            "row_index": seed["row_index"],
                            "row": seed["row"],
                            "distance": 0.0,
                        }
                    ],
                    family,
                    polarity,
                    config,
                )
            )

    merged.sort(
        key=lambda row: (
            str(row.get("SampleType") or ""),
            str(row.get("Polarity") or ""),
            float(row.get("RT_mean") or 0),
            _optional_float(row.get("MZ_mean")) or 0.0,
            -float(row.get("Area_mean") or 0),
        )
    )
    return merged


def process_peaks(
    df: pd.DataFrame, config: ScreeningConfig | Dict[str, Any] | None = None
) -> Tuple[pd.DataFrame, pd.DataFrame, List[Dict[str, Any]]]:
    config = build_screening_config(config if isinstance(config, dict) else asdict(config) if config else None)

    df = df.copy()
    df.columns = [c.strip() for c in df.columns]

    required = ["RT", "Polarity", "File", "Area"]
    missing = [c for c in required if c not in df.columns]
    if missing:
        raise ValueError(f"Missing required columns: {', '.join(missing)}")

    if "Base Peak" not in df.columns:
        df["Base Peak"] = None

    for numeric_col in ("RT", "Area", "Base Peak"):
        df[numeric_col] = pd.to_numeric(df[numeric_col], errors="coerce")

    df = df.dropna(subset=["RT", "Area"])

    if "operator_mark" not in df.columns:
        df["operator_mark"] = None
    if "operator_color" not in df.columns:
        df["operator_color"] = None
    if "Label" not in df.columns:
        df["Label"] = None

    df["SampleType"] = df.apply(_assign_sample_type, axis=1)
    df["SummarySampleType"] = df["SampleType"].apply(_parallel_sample_family)

    coarse_rows = []
    for (_, _), grp in df.groupby(["SampleType", "Polarity"]):
        result = coarse_screen(grp, config)
        if not result.empty:
            coarse_rows.append(result)

    coarse_df = pd.concat(coarse_rows, ignore_index=True) if coarse_rows else pd.DataFrame()

    blanks = coarse_df[coarse_df["SampleType"] == "blank"].copy() if not coarse_df.empty else pd.DataFrame()
    sample_rows = (
        coarse_df[coarse_df["SampleType"] != "blank"].to_dict(orient="records") if not coarse_df.empty else []
    )

    merged_samples = _parallel_merge_samples(sample_rows, config)

    final_results: List[Dict[str, Any]] = []
    for peak in merged_samples:
        candidates = _blank_candidates(peak, blanks, config) if not blanks.empty else []
        best_blank = candidates[0] if candidates else None

        signal_to_blank_ratio = None
        blank_area_mean = None
        area_difference = None
        if best_blank:
            blank_area_mean = float(best_blank["blank_row"].get("Area_mean") or 0)
            area_difference = float(peak["Area_mean"]) - blank_area_mean
            signal_to_blank_ratio = None if blank_area_mean <= 0 else float(peak["Area_mean"]) / blank_area_mean

        status = "Real Compound"
        if best_blank and signal_to_blank_ratio is not None and signal_to_blank_ratio < config.signal_to_blank_min:
            status = "Artifact"
        elif best_blank and signal_to_blank_ratio is None:
            status = "Artifact"

        row = dict(peak)
        row["SignalToBlankRatio"] = _safe_round(signal_to_blank_ratio, 2)
        row["BlankAreaMean"] = _safe_round(blank_area_mean, 2)
        row["AreaDifference"] = _safe_round(area_difference, 2)
        row["ConfidenceScore"] = _final_confidence_score(
            replicate_score=float(peak["ReplicateConfidenceScore"]),
            has_blank_match=bool(best_blank),
            signal_to_blank_ratio=signal_to_blank_ratio,
            config=config,
        )
        row["Status"] = status
        row.setdefault("Why", {})
        row["Why"]["BlankMatch"] = bool(best_blank)
        row["Why"]["BlankCandidateCount"] = len(candidates)
        row["Why"]["SignalToBlankRatio"] = _safe_round(signal_to_blank_ratio, 2)
        row["Why"]["SignalToBlankThreshold"] = config.signal_to_blank_min
        row["Why"]["BlankAreaMean"] = _safe_round(blank_area_mean, 2)
        row["Why"]["AreaDifference"] = _safe_round(area_difference, 2)
        row["Why"]["ConfidenceScore"] = row["ConfidenceScore"]
        row["Why"]["Decision"] = status
        if best_blank:
            row["Why"]["BlankDetail"] = {
                "RT": _safe_round(best_blank["blank_row"]["RT_mean"], 4),
                "MZ": _safe_round(_optional_float(best_blank["blank_row"].get("MZ_mean")), 6),
                "Area_mean": _safe_round(best_blank["blank_row"].get("Area_mean"), 2),
                "rt_delta": _safe_round(best_blank["rt_delta"], 4),
                "mz_delta_da": _safe_round(best_blank["mz_delta_da"], 6),
                "mz_delta_ppm": _safe_round(best_blank["mz_delta_ppm"], 2),
                "matching_mode": "RT+MZ" if best_blank["uses_mz"] else "RT",
                "tolerance": {
                    "rt": config.blank_rt_tol,
                    "mz": config.blank_mz_tol,
                    "mz_mode": config.blank_mz_mode,
                },
            }
        final_results.append(row)

    summary = []
    for (stype, pol), grp in df.groupby(["SummarySampleType", "Polarity"]):
        if stype == "blank" and not blanks.empty:
            confirmed_subset = blanks[(blanks["SampleType"] == stype) & (blanks["Polarity"] == pol)]
            sub: List[Dict[str, Any]] = []
        else:
            sub = [r for r in final_results if r["SampleType"] == stype and r["Polarity"] == pol]
            confirmed_subset = pd.DataFrame(sub) if sub else pd.DataFrame()

        confirmed_count = len(confirmed_subset)

        if stype != "blank":
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

        color_driven = df[(df["SummarySampleType"] == stype) & (df["Polarity"] == pol)]["operator_mark"].notna().any()

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
                if not confirmed_subset.empty and "AreaCVPct" in confirmed_subset
                else None,
                "HighQuality": int((confirmed_subset["ReplicateQuality"] == "High").sum())
                if not confirmed_subset.empty and "ReplicateQuality" in confirmed_subset
                else 0,
                "ModerateQuality": int((confirmed_subset["ReplicateQuality"] == "Moderate").sum())
                if not confirmed_subset.empty and "ReplicateQuality" in confirmed_subset
                else 0,
                "LowQuality": int((confirmed_subset["ReplicateQuality"] == "Low").sum())
                if not confirmed_subset.empty and "ReplicateQuality" in confirmed_subset
                else 0,
                "MeanConfidenceScore": _safe_round(confirmed_subset["ConfidenceScore"].dropna().mean(), 1)
                if not confirmed_subset.empty and "ConfidenceScore" in confirmed_subset
                else None,
                "MeanSignalToBlankRatio": mean_sb,
            }
        )

    return pd.DataFrame(final_results), pd.DataFrame(summary), final_results
