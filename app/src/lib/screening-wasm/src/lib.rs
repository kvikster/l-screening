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
use math::{calc_cv_percent, match_metrics, replicate_confidence_score, safe_round};
use types::{ConfirmedRow, MatchMetrics, Row, SummaryRow};

// Maps operator_mark → canonical SampleType.
fn mark_to_stype(mark: &str) -> Option<&'static str> {
    match mark {
        "blank_positive" | "blank_negative" => Some("blank"),
        "sample_rep1" | "sample_rep2" => Some("sample"),
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

fn assign_sample_type(row: &Row) -> String {
    if let Some(mark) = &row.operator_mark {
        if let Some(stype) = mark_to_stype(mark) {
            return stype.to_string();
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
    let replicate_quality = crate::math::classify_replicate_quality(area_cv_pct, config).to_string();
    let rep_score = replicate_confidence_score(
        mean_or_zero(&rt_deltas),
        config.replicate_rt_tol,
        if mz_deltas_in_mode.is_empty() { None } else { Some(mean_or_zero(&mz_deltas_in_mode)) },
        config.replicate_mz_tol,
        area_cv_pct,
        parallel_source_samples.len() > 1,
        uses_mz,
        config,
    );
    let matching_mode = if uses_mz && member_rows.iter().all(|r| r.matching_mode == "RT+MZ") {
        "RT+MZ"
    } else {
        "RT"
    }
    .to_string();

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
        confidence_score: rep_score,
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
        blank_area_mean: None,
        area_difference: None,
        signal_to_blank_ratio: None,
        status: String::new(),
        why,
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

fn process_peaks_inner(rows_json: &str, config_json: &str) -> Result<String, String> {
    let config: ScreeningConfig =
        serde_json::from_str(config_json).map_err(|e| format!("Invalid config: {e}"))?;
    config.validate()?;

    let mut rows: Vec<Row> =
        serde_json::from_str(rows_json).map_err(|e| format!("Invalid rows: {e}"))?;

    // Validate required fields and assign SampleType.
    for row in &mut rows {
        row.sample_type = assign_sample_type(row);
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

    // Separate blanks and samples.
    let blanks: Vec<_> = all_confirmed
        .iter()
        .filter(|r| r.sample_type == "blank")
        .cloned()
        .collect();

    let sample_confirmed: Vec<_> = all_confirmed
        .into_iter()
        .filter(|r| r.sample_type != "blank")
        .collect();

    let mut final_results = merge_parallel_samples(sample_confirmed, &config);

    // Blank subtraction — update each sample peak in place.
    for peak in &mut final_results {
        let candidates = if blanks.is_empty() {
            vec![]
        } else {
            blank_candidates(peak, &blanks, &config)
        };
        apply_blank_result(peak, &blanks, &candidates, &config);
    }

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
