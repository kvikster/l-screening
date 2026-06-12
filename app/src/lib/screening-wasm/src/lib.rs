mod blank;
mod cluster;
mod config;
mod math;
mod types;

use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use blank::{apply_blank_result, blank_candidates};
use cluster::coarse_screen;
use config::ScreeningConfig;
use math::{calc_cv_percent, final_confidence_score, match_metrics, replicate_confidence_score, resolve_thresholds, safe_round};
use types::{ConfirmedRow, MatchMetrics, Row, SummaryRow};

// Maps operator_mark → canonical SampleType.
// Checks config.mark_aliases first, then falls back to built-in defaults.
fn mark_to_stype(mark: &str, config: &ScreeningConfig) -> Option<String> {
    if let Some(stype) = config.mark_aliases.get(mark) {
        return Some(stype.clone());
    }
    match mark {
        "blank_positive" | "blank_negative" => Some("blank".to_string()),
        "sample_rep1" | "sample_rep2" => Some("sample".to_string()),
        "surrogate" | "surrogate_positive" | "surrogate_negative" => Some("surrogate".to_string()),
        _ => None,
    }
}

fn classify_from_filename(filename: &str) -> String {
    let f = filename.to_lowercase();
    if f.contains("blank") {
        return "blank".to_string();
    }
    for i in 1..10 {
        let prefix = format!("{}_", i);
        let neg_prefix = format!("{}_neg", i);
        if f.starts_with(&prefix) || f.starts_with(&neg_prefix) {
            return format!("sample_{}", i);
        }
    }
    "unknown".to_string()
}

fn assign_sample_type(row: &Row, config: &ScreeningConfig) -> String {
    if let Some(mark) = &row.operator_mark {
        if let Some(stype) = mark_to_stype(mark, config) {
            return stype;
        }
    }
    classify_from_filename(&row.file)
}

fn sample_family_key(sample_type: &str) -> String {
    if sample_type == "blank" {
        "blank".to_string()
    } else if sample_type.starts_with("sample") {
        "sample".to_string()
    } else {
        sample_type.to_string()
    }
}

#[derive(Clone)]
struct ParallelMember {
    #[allow(dead_code)]
    bucket_name: String,
    bucket_idx: usize,
    row_idx: usize,
    distance: f64,
}

fn weighted_mean(values: &[(Option<f64>, usize)]) -> Option<f64> {
    let mut weighted_total = 0.0;
    let mut total_weight = 0usize;
    for (value, weight) in values {
        if let Some(v) = value {
            if *weight > 0 {
                weighted_total += v * *weight as f64;
                total_weight += *weight;
            }
        }
    }
    if total_weight == 0 {
        None
    } else {
        Some(weighted_total / total_weight as f64)
    }
}

/// Extracts per-replicate area values from a coarse-level ConfirmedRow's `Why.ReplicateArea.values`.
/// NOTE: This reads from `"ReplicateArea"`, not `"Area"` (used by parallel merge).
/// It is safe to call only on coarse-level rows; parallel-merged rows would fall through to the fallback.
fn replicate_area_values(row: &ConfirmedRow) -> Vec<f64> {
    if let Some(values) = row
        .why
        .get("ReplicateArea")
        .and_then(|v| v.get("values"))
        .and_then(|v| v.as_array())
    {
        let parsed: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    vec![row.area_mean; row.replicate_count.max(1)]
}

fn final_centroid(members: &[ParallelMember], rows: &[ConfirmedRow]) -> (f64, Option<f64>) {
    let n = members.len() as f64;
    let rt = members.iter().map(|m| rows[m.row_idx].rt_mean).sum::<f64>() / n;
    let mz_values: Vec<f64> = members.iter().filter_map(|m| rows[m.row_idx].mz_mean).collect();
    let mz = if mz_values.is_empty() {
        None
    } else {
        Some(mz_values.iter().sum::<f64>() / mz_values.len() as f64)
    };
    (rt, mz)
}

/// Thin wrapper around `match_metrics` for `ConfirmedRow` vs centroid.
fn match_final_to_centroid(
    centroid_rt: f64,
    centroid_mz: Option<f64>,
    candidate: &ConfirmedRow,
    config: &ScreeningConfig,
) -> MatchMetrics {
    match_metrics(
        centroid_rt,
        centroid_mz,
        candidate.rt_mean,
        candidate.mz_mean,
        config.replicate_rt_tol,
        config.replicate_mz_tol,
        config.replicate_mz_mode_str(),
    )
}

fn choose_parallel_members(
    seed_row_idx: usize,
    seed_bucket_idx: usize,
    buckets: &[(String, Vec<usize>)],
    rows: &[ConfirmedRow],
    used: &HashSet<(usize, usize)>,
    config: &ScreeningConfig,
) -> Vec<ParallelMember> {
    let mut members = vec![ParallelMember {
        bucket_name: buckets[seed_bucket_idx].0.clone(),
        bucket_idx: seed_bucket_idx,
        row_idx: seed_row_idx,
        distance: 0.0,
    }];

    let (mut centroid_rt, mut centroid_mz) = final_centroid(&members, rows);

    for (bucket_idx, (bucket_name, bucket_indices)) in buckets.iter().enumerate() {
        if bucket_idx == seed_bucket_idx {
            continue;
        }

        let mut best: Option<ParallelMember> = None;
        let mut best_area = f64::NEG_INFINITY;

        for &row_idx in bucket_indices {
            if used.contains(&(bucket_idx, row_idx)) {
                continue;
            }
            let cm =
                match_final_to_centroid(centroid_rt, centroid_mz, &rows[row_idx], config);
            if !cm.matches {
                continue;
            }
            let distance = cm.distance;
            let area = rows[row_idx].area_mean;
            let better = match &best {
                None => true,
                Some(b) => distance < b.distance || (distance == b.distance && area > best_area),
            };
            if better {
                best = Some(ParallelMember {
                    bucket_name: bucket_name.clone(),
                    bucket_idx,
                    row_idx,
                    distance,
                });
                best_area = area;
            }
        }

        if let Some(member) = best {
            members.push(member);
            let (crt, cmz) = final_centroid(&members, rows);
            centroid_rt = crt;
            centroid_mz = cmz;
        }
    }

    members
}

fn merge_parallel_cluster(
    members: &[ParallelMember],
    rows: &[ConfirmedRow],
    family: &str,
    polarity: &str,
    config: &ScreeningConfig,
) -> ConfirmedRow {
    let member_rows: Vec<&ConfirmedRow> = members.iter().map(|m| &rows[m.row_idx]).collect();
    let weighted_rt_inputs: Vec<(Option<f64>, usize)> = member_rows
        .iter()
        .map(|r| (Some(r.rt_mean), r.replicate_count))
        .collect();
    let weighted_mz_inputs: Vec<(Option<f64>, usize)> = member_rows
        .iter()
        .map(|r| (r.mz_mean, r.replicate_count))
        .collect();
    let weighted_area_inputs: Vec<(Option<f64>, usize)> = member_rows
        .iter()
        .map(|r| (Some(r.area_mean), r.replicate_count))
        .collect();
    let rt_mean = weighted_mean(&weighted_rt_inputs).unwrap_or(0.0);
    let mz_mean = weighted_mean(&weighted_mz_inputs);
    let area_mean = weighted_mean(&weighted_area_inputs).unwrap_or(0.0);
    let replicate_area_values: Vec<f64> = member_rows
        .iter()
        .flat_map(|r| replicate_area_values(r))
        .collect();
    let area_cv_pct = calc_cv_percent(&replicate_area_values);
    let parallel_source_samples: Vec<String> = {
        let mut vals: Vec<String> = member_rows.iter().map(|r| r.sample_type.clone()).collect();
        vals.sort();
        vals.dedup();
        vals
    };
    let total_replicates = member_rows.iter().map(|r| r.replicate_count).sum::<usize>();

    let mut rt_deltas = Vec::new();
    let mut mz_deltas_da = Vec::new();
    let mut mz_deltas_ppm = Vec::new();
    let mut mz_deltas_in_mode = Vec::new();
    // Pairwise distances: use match_final_to_centroid with row[i] as the reference
    // point. The delta computation is symmetric so this is equivalent to a generic
    // pairwise distance, despite the function's centroid-oriented naming.
    for i in 0..member_rows.len() {
        for j in (i + 1)..member_rows.len() {
            let cm = match_final_to_centroid(
                member_rows[i].rt_mean,
                member_rows[i].mz_mean,
                member_rows[j],
                config,
            );
            rt_deltas.push(cm.rt_delta);
            if cm.uses_mz {
                if let Some(v) = cm.mz_delta_da { mz_deltas_da.push(v); }
                if let Some(v) = cm.mz_delta_ppm { mz_deltas_ppm.push(v); }
                if let Some(v) = cm.mz_delta_in_mode { mz_deltas_in_mode.push(v); }
            }
        }
    }
    let mean_or_zero = |v: &[f64]| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
    let max_or_none = |v: &[f64]| if v.is_empty() { None } else { Some(v.iter().cloned().fold(0.0_f64, f64::max)) };
    let uses_mz = !mz_deltas_in_mode.is_empty();
    let merge_label = member_rows.first().and_then(|r| r.rep1_label.as_deref());
    let thresholds = resolve_thresholds(merge_label, config);
    let replicate_quality = crate::math::classify_replicate_quality(
        area_cv_pct,
        thresholds.cv_high_max,
        thresholds.cv_moderate_max,
    )
    .to_string();
    let rep_score = replicate_confidence_score(
        mean_or_zero(&rt_deltas),
        config.replicate_rt_tol,
        if mz_deltas_in_mode.is_empty() { None } else { Some(mean_or_zero(&mz_deltas_in_mode)) },
        config.replicate_mz_tol,
        area_cv_pct,
        parallel_source_samples.len() > 1,
        uses_mz,
        config.mz_available,
        thresholds.cv_high_max,
        thresholds.cv_moderate_max,
    );
    let matching_mode = if uses_mz && member_rows.iter().all(|r| r.matching_mode == "RT+MZ") {
        "RT+MZ"
    } else {
        "RT"
    }
    .to_string();

    // ── Aggregate per-source blank-subtraction results ────────────────────
    // Each member already went through `apply_blank_result` before merge, so
    // its blank_area_mean / signal_to_blank_ratio / status reflect a real,
    // per-sample blank match. We aggregate by replicate-count weighting over
    // sources that actually had a blank match; status is re-derived from the
    // aggregated S/B ratio so the merged row stays consistent with the
    // configured threshold.
    let blank_area_inputs: Vec<(Option<f64>, usize)> = member_rows
        .iter()
        .map(|r| (r.blank_area_mean, r.replicate_count))
        .collect();
    let agg_blank_area = weighted_mean(&blank_area_inputs);
    let sources_with_blank: usize = member_rows
        .iter()
        .filter(|r| r.blank_area_mean.is_some())
        .count();
    let has_blank_match = sources_with_blank > 0;
    let agg_signal_to_blank_ratio = match agg_blank_area {
        Some(b) if b > 0.0 => Some(area_mean / b),
        _ => None,
    };
    let agg_area_difference = agg_blank_area.map(|b| area_mean - b);
    let agg_status = if !has_blank_match {
        "Real Compound".to_string()
    } else {
        match agg_signal_to_blank_ratio {
            None => "Artifact".to_string(),
            Some(r) if r < thresholds.signal_to_blank_min => "Artifact".to_string(),
            _ => "Real Compound".to_string(),
        }
    };
    let agg_confidence_score = final_confidence_score(
        rep_score,
        has_blank_match,
        agg_signal_to_blank_ratio,
        thresholds.signal_to_blank_min,
    );
    // Aggregate the three-state blank outcome across sources. If any source had a
    // blank match the merged row is subtracted; otherwise it is "clean here" when
    // a blank was loaded for at least one source, or "no blank loaded" when none was.
    let agg_blank_state = if has_blank_match {
        "blank_subtracted"
    } else if member_rows.iter().any(|r| {
        r.why
            .get("BlankState")
            .and_then(|v| v.as_str())
            .map_or(false, |s| s == "blank_clean_here")
    }) {
        "blank_clean_here"
    } else {
        "no_blank_loaded"
    };
    let total_blank_candidate_count: u64 = member_rows
        .iter()
        .map(|r| {
            r.why
                .get("BlankCandidateCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        })
        .sum();
    let best_source_blank_detail = member_rows
        .iter()
        .filter(|r| r.blank_area_mean.is_some())
        .min_by(|a, b| {
            let da = a
                .why
                .get("BlankDetail")
                .and_then(|v| v.get("rt_delta"))
                .and_then(|v| v.as_f64())
                .unwrap_or(f64::INFINITY);
            let db = b
                .why
                .get("BlankDetail")
                .and_then(|v| v.get("rt_delta"))
                .and_then(|v| v.as_f64())
                .unwrap_or(f64::INFINITY);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|r| r.why.get("BlankDetail").cloned());
    let per_source_blank: Vec<Value> = member_rows
        .iter()
        .map(|r| {
            json!({
                "SampleType": r.sample_type.clone(),
                "BlankAreaMean": r.blank_area_mean,
                "AreaDifference": r.area_difference,
                "SignalToBlankRatio": r.signal_to_blank_ratio,
                "Status": r.status.clone(),
                "ConfidenceScore": r.confidence_score,
                "ReplicateCount": r.replicate_count,
                "BlankDetail": r.why.get("BlankDetail").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();

    let replicate_files: Vec<Option<String>> = member_rows.iter().flat_map(|r| r.replicate_files.clone()).collect();
    let replicate_labels: Vec<Option<String>> = member_rows.iter().flat_map(|r| r.replicate_labels.clone()).collect();
    let replicate_marks: Vec<Option<String>> = member_rows.iter().flat_map(|r| r.replicate_marks.clone()).collect();
    let replicate_colors: Vec<Option<String>> = member_rows.iter().flat_map(|r| r.replicate_colors.clone()).collect();
    let rep1_label = replicate_labels.iter().flatten().next().cloned();
    let rep2_label = replicate_labels.iter().flatten().nth(1).cloned();
    let rep1_mark = replicate_marks.iter().flatten().next().cloned();
    let rep2_mark = replicate_marks.iter().flatten().nth(1).cloned();
    let rep1_color = replicate_colors.iter().flatten().next().cloned();
    let rep2_color = replicate_colors.iter().flatten().nth(1).cloned();

    let why = json!({
        "ParallelMerge": {
            "Strategy": "greedy_cluster_union",
            "SourceSamples": parallel_source_samples.clone(),
            "ParallelSampleCount": parallel_source_samples.len(),
            "MatchedAcrossSamples": parallel_source_samples.len() > 1,
            "MatchingMode": matching_mode.clone(),
            "RT": {
                "mean": safe_round(rt_mean, 4),
                "max_delta": safe_round(rt_deltas.iter().cloned().fold(0.0_f64, f64::max), 4),
                "tolerance": config.replicate_rt_tol,
            },
            "MZ": {
                "mean": mz_mean.map(|v| safe_round(v, 6)),
                "max_delta_da": max_or_none(&mz_deltas_da).map(|v| safe_round(v, 6)),
                "max_delta_ppm": max_or_none(&mz_deltas_ppm).map(|v| safe_round(v, 2)),
                "tolerance": config.replicate_mz_tol,
                "mode": config.replicate_mz_mode_str(),
                "used": uses_mz,
            },
            "Area": {
                "values": replicate_area_values.iter().map(|&v| safe_round(v, 2)).collect::<Vec<_>>(),
                "mean": safe_round(area_mean, 2),
            },
            "SourceRows": member_rows.iter().map(|r| {
                json!({
                    "SampleType": r.sample_type.clone(),
                    "RT_mean": safe_round(r.rt_mean, 4),
                    "MZ_mean": r.mz_mean.map(|v| safe_round(v, 6)),
                    "Area_mean": safe_round(r.area_mean, 2),
                    "ReplicateCount": r.replicate_count,
                    "MatchingMode": r.matching_mode.clone(),
                })
            }).collect::<Vec<Value>>(),
            "SourceWhy": member_rows.iter().map(|r| r.why.clone()).collect::<Vec<Value>>(),
        },
        "ReplicateConfidenceScore": rep_score,
        "BlankSubtraction": {
            "Policy": "per_source_then_aggregate",
            "TotalSources": member_rows.len(),
            "SourcesWithBlankMatch": sources_with_blank,
            "AggregatedBlankAreaMean": agg_blank_area.map(|v| safe_round(v, 2)),
            "AggregatedSignalToBlankRatio": agg_signal_to_blank_ratio.map(|v| safe_round(v, 2)),
            "AggregatedAreaDifference": agg_area_difference.map(|v| safe_round(v, 2)),
            "Threshold": config.signal_to_blank_min,
            "Decision": agg_status.clone(),
            "BlankState": agg_blank_state,
            "PerSource": per_source_blank,
        },
        "BlankMatch": has_blank_match,
        "BlankState": agg_blank_state,
        "BlankCandidateCount": total_blank_candidate_count,
        "BlankAreaMean": agg_blank_area.map(|v| safe_round(v, 2)),
        "AreaDifference": agg_area_difference.map(|v| safe_round(v, 2)),
        "SignalToBlankRatio": agg_signal_to_blank_ratio.map(|v| safe_round(v, 2)),
        "SignalToBlankThreshold": config.signal_to_blank_min,
        "ConfidenceScore": agg_confidence_score,
        "Decision": agg_status.clone(),
        "BlankDetail": best_source_blank_detail.unwrap_or(Value::Null),
        "ThresholdProfile": {
            "replicate_rt_tol": config.replicate_rt_tol,
            "replicate_mz_tol": config.replicate_mz_tol,
            "replicate_mz_mode": config.replicate_mz_mode_str(),
            "blank_rt_tol": config.blank_rt_tol,
            "blank_mz_tol": config.blank_mz_tol,
            "blank_mz_mode": config.blank_mz_mode_str(),
            "signal_to_blank_min": config.signal_to_blank_min,
        },
    });

    ConfirmedRow {
        group: format!("{}_{}", family, polarity),
        rt_mean: safe_round(rt_mean, 4),
        mz_mean: mz_mean.map(|v| safe_round(v, 6)),
        area_mean: safe_round(area_mean, 2),
        area_cv_pct: area_cv_pct.map(|v| safe_round(v, 2)),
        replicate_quality,
        replicate_count: total_replicates,
        replicate_confidence_score: rep_score,
        confidence_score: agg_confidence_score,
        polarity: polarity.to_string(),
        sample_type: family.to_string(),
        rep1_label,
        rep2_label,
        rep1_mark,
        rep2_mark,
        rep1_color,
        rep2_color,
        replicate_files,
        replicate_labels,
        replicate_marks,
        replicate_colors,
        confirmed: "Yes".to_string(),
        matching_mode,
        parallel_match: parallel_source_samples.len() > 1,
        parallel_sample_count: parallel_source_samples.len(),
        parallel_source_samples,
        blank_area_mean: agg_blank_area.map(|v| safe_round(v, 2)),
        area_difference: agg_area_difference.map(|v| safe_round(v, 2)),
        signal_to_blank_ratio: agg_signal_to_blank_ratio.map(|v| safe_round(v, 2)),
        status: agg_status,
        is_surrogate: false,
        surrogate_recovery_pct: None,
        surrogate_rt_shift: None,
        surrogate_pass: None,
        why,
    }
}

/// Extract per-replicate RT values for a coarse-level ConfirmedRow from
/// `Why.ReplicateMembers[].rt`, falling back to `Why.ReplicateRT.mean` / rt_mean.
fn replicate_rt_values(row: &ConfirmedRow) -> Vec<f64> {
    if let Some(members) = row.why.get("ReplicateMembers").and_then(|v| v.as_array()) {
        let parsed: Vec<f64> = members
            .iter()
            .filter_map(|m| m.get("rt").and_then(|v| v.as_f64()))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    vec![row.rt_mean; row.replicate_count.max(1)]
}

/// Suggested cluster bin-width ("replicate RT tolerance") metrics derived from a
/// surrogate's observed RT behavior. All three are advisory; the operator chooses
/// which to enter into Analyzer Configuration. `k` is the multiplier used for the
/// stdev-based suggestion.
fn surrogate_bin_width_metrics(rts: &[f64], expected_rt: Option<f64>, k: f64) -> Value {
    let n = rts.len();
    let max_abs_shift = expected_rt.map(|exp| {
        rts.iter()
            .map(|&rt| (rt - exp).abs())
            .fold(0.0_f64, f64::max)
    });
    let range = if n >= 1 {
        let max = rts.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = rts.iter().cloned().fold(f64::INFINITY, f64::min);
        max - min
    } else {
        0.0
    };
    let stdev = if n >= 2 {
        let mean = rts.iter().sum::<f64>() / n as f64;
        let var = rts.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        var.sqrt()
    } else {
        0.0
    };
    json!({
        "ReplicateCount": n,
        "MaxAbsRtShift": max_abs_shift.map(|v| safe_round(v, 4)),
        "StdevRt": safe_round(stdev, 4),
        "StdevMultiplier": k,
        "KStdevRt": safe_round(k * stdev, 4),
        "RangeRt": safe_round(range, 4),
        "Unit": "min",
        "Advisory": true,
    })
}

/// Validate surrogate rows against SurrogateSpec entries in config.
/// Matches each surrogate ConfirmedRow to the closest configured spec by RT,
/// then populates surrogate_recovery_pct, surrogate_rt_shift, surrogate_pass, and
/// extends Why with a SurrogateValidation block.
fn surrogate_validation_pass(surrogates: &mut [ConfirmedRow], config: &ScreeningConfig) {
    let default_rt_window = config.replicate_rt_tol * 2.0;

    // Multiplier for the stdev-based bin-width suggestion (≈ ±k·σ coverage).
    const BIN_WIDTH_K: f64 = 3.0;

    for row in surrogates.iter_mut() {
        row.is_surrogate = true;

        let replicate_rts = replicate_rt_values(row);

        let best_spec = config
            .surrogates
            .iter()
            .min_by(|a, b| {
                let da = (row.rt_mean - a.expected_rt).abs();
                let db = (row.rt_mean - b.expected_rt).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(spec) = best_spec {
            let rt_shift = row.rt_mean - spec.expected_rt;
            let recovery_pct = if spec.expected_area > 0.0 {
                row.area_mean / spec.expected_area * 100.0
            } else {
                0.0
            };
            let rt_window = spec.rt_window.unwrap_or(default_rt_window);
            let min_pct = spec.recovery_min_pct.unwrap_or(70.0);
            let max_pct = spec.recovery_max_pct.unwrap_or(130.0);
            let pass =
                recovery_pct >= min_pct && recovery_pct <= max_pct && rt_shift.abs() <= rt_window;

            row.surrogate_recovery_pct = Some(safe_round(recovery_pct, 1));
            row.surrogate_rt_shift = Some(safe_round(rt_shift, 4));
            row.surrogate_pass = Some(pass);
            row.status = if pass {
                "Surrogate OK".to_string()
            } else {
                "Surrogate Failed".to_string()
            };

            let is_response_factor = spec.expected_concentration.filter(|&c| c > 0.0).map(|c| {
                safe_round(row.area_mean / c, 4)
            });

            let bin_width = surrogate_bin_width_metrics(
                &replicate_rts,
                Some(spec.expected_rt),
                BIN_WIDTH_K,
            );

            if let Some(why_obj) = row.why.as_object_mut() {
                why_obj.insert("BinWidthSuggestion".into(), bin_width.clone());
                why_obj.insert(
                    "SurrogateValidation".into(),
                    json!({
                        "BinWidthSuggestion": bin_width,
                        "MatchedSpec": spec.name,
                        "ExpectedRT": spec.expected_rt,
                        "ExpectedMZ": spec.expected_mz,
                        "ObservedRT": safe_round(row.rt_mean, 4),
                        "RTShift": safe_round(rt_shift, 4),
                        "RTWindow": rt_window,
                        "ExpectedArea": spec.expected_area,
                        "ObservedArea": safe_round(row.area_mean, 2),
                        "RecoveryPct": safe_round(recovery_pct, 1),
                        "RecoveryMin": min_pct,
                        "RecoveryMax": max_pct,
                        "Pass": pass,
                        "ExpectedConcentration": spec.expected_concentration,
                        "ISResponseFactor": is_response_factor,
                    }),
                );
            }
        } else {
            // Row is a surrogate but no spec matched within its RT window.
            // Bin-width spread metrics still apply (range/stdev need no spec).
            let bin_width = surrogate_bin_width_metrics(&replicate_rts, None, BIN_WIDTH_K);
            if let Some(why_obj) = row.why.as_object_mut() {
                why_obj.insert("BinWidthSuggestion".into(), bin_width);
            }
            row.status = "Surrogate".to_string();
        }
    }
}

fn merge_parallel_samples(samples: Vec<ConfirmedRow>, config: &ScreeningConfig) -> Vec<ConfirmedRow> {
    let mut grouped: HashMap<(String, String), Vec<ConfirmedRow>> = HashMap::new();
    for row in samples {
        grouped
            .entry((sample_family_key(&row.sample_type), row.polarity.clone()))
            .or_default()
            .push(row);
    }

    let mut results = Vec::new();
    for ((family, polarity), rows) in grouped {
        let mut bucket_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut bucket_order: Vec<String> = Vec::new();
        for (idx, row) in rows.iter().enumerate() {
            if !bucket_map.contains_key(&row.sample_type) {
                bucket_order.push(row.sample_type.clone());
            }
            bucket_map.entry(row.sample_type.clone()).or_default().push(idx);
        }
        let buckets: Vec<(String, Vec<usize>)> = bucket_order
            .into_iter()
            .map(|name| {
                let indices = bucket_map.remove(&name).unwrap_or_default();
                (name, indices)
            })
            .collect();

        let rows_ref = &rows;
        let mut seeds: Vec<(usize, usize, f64)> = buckets
            .iter()
            .enumerate()
            .flat_map(|(bi, (_, indices))| indices.iter().map(move |&ri| (bi, ri, rows_ref[ri].area_mean)))
            .collect();
        seeds.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    rows_ref[a.1].rt_mean
                        .partial_cmp(&rows_ref[b.1].rt_mean)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    rows_ref[a.1].mz_mean
                        .unwrap_or(0.0)
                        .partial_cmp(&rows_ref[b.1].mz_mean.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut used: HashSet<(usize, usize)> = HashSet::new();
        for (bucket_idx, row_idx, _) in &seeds {
            if used.contains(&(*bucket_idx, *row_idx)) {
                continue;
            }
            let members = choose_parallel_members(*row_idx, *bucket_idx, &buckets, &rows, &used, config);
            if members.len() < 2 {
                continue;
            }
            for m in &members {
                used.insert((m.bucket_idx, m.row_idx));
            }
            results.push(merge_parallel_cluster(&members, &rows, &family, &polarity, config));
        }

        for (bucket_idx, row_idx, _) in &seeds {
            if used.contains(&(*bucket_idx, *row_idx)) {
                continue;
            }
            used.insert((*bucket_idx, *row_idx));
            let singleton = vec![ParallelMember {
                bucket_name: buckets[*bucket_idx].0.clone(),
                bucket_idx: *bucket_idx,
                row_idx: *row_idx,
                distance: 0.0,
            }];
            results.push(merge_parallel_cluster(&singleton, &rows, &family, &polarity, config));
        }
    }

    results.sort_by(|a, b| {
        a.sample_type
            .cmp(&b.sample_type)
            .then(a.polarity.cmp(&b.polarity))
            .then(a.rt_mean.partial_cmp(&b.rt_mean).unwrap_or(std::cmp::Ordering::Equal))
    });
    results
}

/// Main WASM entry point.
/// `rows_json`: JSON array of row objects (from SheetJS parsing in JS).
/// `config_json`: JSON object with optional screening parameters.
/// Returns a JSON string `{ "results": [...], "summary": [...] }`.
#[wasm_bindgen]
pub fn process_peaks(rows_json: &str, config_json: &str) -> String {
    match process_peaks_inner(rows_json, config_json) {
        Ok(result) => result,
        Err(e) => {
            serde_json::json!({ "error": e }).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn run(rows: &str, config: &str) -> Value {
        let out = process_peaks_inner(rows, config).expect("process ok");
        serde_json::from_str(&out).unwrap()
    }

    // Two replicates per sample, no Polarity column → single "n/a" group.
    #[test]
    fn no_polarity_collapses_to_single_group() {
        let rows = r#"[
            {"RT":1.00,"Base Peak":100.0,"Area":100.0,"File":"1_a","operator_mark":"sample_rep1"},
            {"RT":1.01,"Base Peak":100.0,"Area":110.0,"File":"1_b","operator_mark":"sample_rep2"}
        ]"#;
        let cfg = r#"{"polarity_available":false}"#;
        let v = run(rows, cfg);
        let summary = v["summary"].as_array().unwrap();
        // All sample rows share one polarity group.
        let pols: std::collections::HashSet<&str> = summary
            .iter()
            .map(|s| s["Polarity"].as_str().unwrap())
            .collect();
        assert_eq!(pols, ["n/a"].into_iter().collect());
        let results = v["results"].as_array().unwrap();
        assert!(results.iter().all(|r| r["Polarity"] == "n/a"));
    }

    // No Polarity and no Base Peak → RT-only run with the missing-m/z penalty waived.
    #[test]
    fn rt_only_waives_mz_penalty() {
        let rows = r#"[
            {"RT":1.00,"Area":100.0,"File":"1_a","operator_mark":"sample_rep1"},
            {"RT":1.01,"Area":110.0,"File":"1_b","operator_mark":"sample_rep2"}
        ]"#;
        let cfg = r#"{"polarity_available":false,"mz_available":false}"#;
        let v = run(rows, cfg);
        let results = v["results"].as_array().unwrap();
        let cluster = results.iter().find(|r| r["ReplicateCount"] == 2).unwrap();
        // No m/z: confidence should not carry the −10 missing-m/z penalty, so a
        // tight RT match stays at/above 95.
        assert!(cluster["ReplicateConfidenceScore"].as_f64().unwrap() >= 95.0);
    }

    // No blank rows anywhere → BlankState = no_blank_loaded, still Real Compound.
    #[test]
    fn no_blank_loaded_state() {
        let rows = r#"[
            {"RT":1.00,"Base Peak":100.0,"Area":100.0,"Polarity":"+","File":"1_a","operator_mark":"sample_rep1"},
            {"RT":1.01,"Base Peak":100.0,"Area":110.0,"Polarity":"+","File":"1_b","operator_mark":"sample_rep2"}
        ]"#;
        let v = run(rows, "{}");
        let results = v["results"].as_array().unwrap();
        let cluster = results.iter().find(|r| r["ReplicateCount"] == 2).unwrap();
        assert_eq!(cluster["Why"]["BlankState"], "no_blank_loaded");
        assert_eq!(cluster["Status"], "Real Compound");
    }

    // Blank exists but at a different RT → BlankState = blank_clean_here.
    #[test]
    fn blank_clean_here_state() {
        let rows = r#"[
            {"RT":1.00,"Base Peak":100.0,"Area":100.0,"Polarity":"+","File":"1_a","operator_mark":"sample_rep1"},
            {"RT":1.01,"Base Peak":100.0,"Area":110.0,"Polarity":"+","File":"1_b","operator_mark":"sample_rep2"},
            {"RT":8.00,"Base Peak":300.0,"Area":50.0,"Polarity":"+","File":"blank_a","operator_mark":"blank_positive"},
            {"RT":8.01,"Base Peak":300.0,"Area":55.0,"Polarity":"+","File":"blank_b","operator_mark":"blank_negative"}
        ]"#;
        let v = run(rows, "{}");
        let results = v["results"].as_array().unwrap();
        let cluster = results
            .iter()
            .find(|r| r["SampleType"] == "sample" && r["ReplicateCount"] == 2)
            .unwrap();
        assert_eq!(cluster["Why"]["BlankState"], "blank_clean_here");
        assert_eq!(cluster["Status"], "Real Compound");
    }

    #[test]
    fn bin_width_metrics_cover_three_measures() {
        let rts = vec![1.00, 1.04, 1.10];
        let m = surrogate_bin_width_metrics(&rts, Some(1.00), 3.0);
        // range = 0.10; max|shift| vs 1.00 = 0.10; stdev > 0.
        assert!((m["RangeRt"].as_f64().unwrap() - 0.10).abs() < 1e-6);
        assert!((m["MaxAbsRtShift"].as_f64().unwrap() - 0.10).abs() < 1e-6);
        assert!(m["StdevRt"].as_f64().unwrap() > 0.0);
        assert!(m["KStdevRt"].as_f64().unwrap() > m["StdevRt"].as_f64().unwrap());
        assert_eq!(m["ReplicateCount"], 3);
    }
}

fn process_peaks_inner(rows_json: &str, config_json: &str) -> Result<String, String> {
    let config: ScreeningConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid config: {e}"))?;
    config.validate()?;

    let mut rows: Vec<Row> =
        serde_json::from_str(rows_json).map_err(|e| format!("Invalid rows: {e}"))?;

    // Validate required fields and assign SampleType.
    // Normalize polarity to the sentinel "n/a" when the dataset has no polarity
    // dimension (config.polarity_available == false) or an individual cell is
    // empty, so those rows group together instead of forming a split group.
    for row in &mut rows {
        row.sample_type = assign_sample_type(row, &config);
        if !config.polarity_available || row.polarity.trim().is_empty() {
            row.polarity = "n/a".to_string();
        }
    }

    // Group row indices by (SampleType, Polarity).
    let mut groups: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, row) in rows.iter().enumerate() {
        groups
            .entry((row.sample_type.clone(), row.polarity.clone()))
            .or_default()
            .push(i);
    }

    // Coarse screen — replicate pairing and clustering.
    let mut all_confirmed = Vec::new();
    for (_, group_indices) in &groups {
        let confirmed = coarse_screen(group_indices, &rows, &config);
        all_confirmed.extend(confirmed);
    }

    // Separate blanks, surrogates, and samples.
    let blanks: Vec<_> = all_confirmed
        .iter()
        .filter(|r| r.sample_type == "blank")
        .cloned()
        .collect();

    let mut surrogate_confirmed: Vec<_> = all_confirmed
        .iter()
        .filter(|r| r.sample_type == "surrogate")
        .cloned()
        .collect();

    let mut sample_confirmed: Vec<_> = all_confirmed
        .into_iter()
        .filter(|r| r.sample_type != "blank" && r.sample_type != "surrogate")
        .collect();

    // Surrogate validation: match each surrogate row to a SurrogateSpec and
    // compute RT shift, recovery %, and pass/fail.
    surrogate_validation_pass(&mut surrogate_confirmed, &config);

    // Per-sample blank subtraction BEFORE parallel merge: each source sample-row
    // is matched against blanks individually so that artifact/real-compound
    // status is decided at the level of an actual measurement, not on an
    // already-averaged row that may mask sample-specific contamination.
    let any_blank_loaded = !blanks.is_empty();
    for peak in &mut sample_confirmed {
        let candidates = if blanks.is_empty() {
            vec![]
        } else {
            blank_candidates(peak, &blanks, &config)
        };
        apply_blank_result(peak, &blanks, &candidates, &config, any_blank_loaded);
    }

    // Parallel merge across samples — aggregates per-source blank results.
    let mut final_results = merge_parallel_samples(sample_confirmed, &config);

    // Surrogates are appended to output after validation (not merged with samples).
    final_results.extend(surrogate_confirmed);

    // Build summary per (SampleType, Polarity).
    let mut summaries: Vec<SummaryRow> = Vec::new();

    // Collect all unique (family, polarity) pairs from original rows.
    let mut family_groups: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, row) in rows.iter().enumerate() {
        family_groups
            .entry((sample_family_key(&row.sample_type), row.polarity.clone()))
            .or_default()
            .push(i);
    }
    let mut group_keys: Vec<(String, String)> = family_groups.keys().cloned().collect();
    group_keys.sort();

    for (stype, pol) in group_keys {
        let group_indices = &family_groups[&(stype.clone(), pol.clone())];

        // Confirmed rows for this group (from blanks + final results).
        let confirmed_blanks: Vec<_> = blanks
            .iter()
            .filter(|r| r.sample_type == stype && r.polarity == pol)
            .collect();
        let confirmed_samples: Vec<_> = final_results
            .iter()
            .filter(|r| r.sample_type == stype && r.polarity == pol)
            .collect();
        let confirmed_all: Vec<_> = if stype == "blank" {
            confirmed_blanks.clone()
        } else {
            confirmed_samples.clone()
        };

        let confirmed_count = confirmed_all.len();

        let (artifacts, real, mean_sb) = if stype != "blank" {
            let art = confirmed_samples
                .iter()
                .filter(|r| r.status == "Artifact")
                .count();
            let real = confirmed_samples
                .iter()
                .filter(|r| r.status == "Real Compound")
                .count();
            let sb_values: Vec<f64> = confirmed_samples
                .iter()
                .filter_map(|r| r.signal_to_blank_ratio)
                .collect();
            let mean_sb = if sb_values.is_empty() {
                None
            } else {
                Some(safe_round(
                    sb_values.iter().sum::<f64>() / sb_values.len() as f64,
                    2,
                ))
            };
            (art, real, mean_sb)
        } else {
            (0, 0, None)
        };

        let color_driven = group_indices
            .iter()
            .any(|&i| rows[i].operator_mark.is_some());

        let cv_values: Vec<f64> = confirmed_all
            .iter()
            .filter_map(|r| r.area_cv_pct)
            .collect();
        let mean_cv_pct = if cv_values.is_empty() {
            None
        } else {
            Some(safe_round(
                cv_values.iter().sum::<f64>() / cv_values.len() as f64,
                2,
            ))
        };

        let high_q = confirmed_all
            .iter()
            .filter(|r| r.replicate_quality == "High")
            .count();
        let mod_q = confirmed_all
            .iter()
            .filter(|r| r.replicate_quality == "Moderate")
            .count();
        let low_q = confirmed_all
            .iter()
            .filter(|r| r.replicate_quality == "Low")
            .count();

        let conf_scores: Vec<f64> = confirmed_all
            .iter()
            .map(|r| r.confidence_score)
            .collect();
        let mean_confidence = if conf_scores.is_empty() {
            None
        } else {
            Some(safe_round(
                conf_scores.iter().sum::<f64>() / conf_scores.len() as f64,
                1,
            ))
        };

        summaries.push(SummaryRow {
            sample: stype.clone(),
            polarity: pol.clone(),
            total_peaks: group_indices.len(),
            confirmed: confirmed_count,
            artifacts,
            real_compounds: real,
            color_driven,
            mean_cv_pct,
            high_quality: high_q,
            moderate_quality: mod_q,
            low_quality: low_q,
            mean_confidence_score: mean_confidence,
            mean_signal_to_blank_ratio: mean_sb,
        });
    }

    let output = serde_json::json!({
        "results": final_results,
        "summary": summaries,
    });

    Ok(output.to_string())
}
