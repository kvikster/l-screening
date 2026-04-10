use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};

use crate::config::ScreeningConfig;
use crate::math::{
    calc_cv_percent, classify_replicate_quality, match_metrics,
    replicate_confidence_score, safe_round,
};
use crate::types::{ClusterMember, ConfirmedRow, Row};

// Operator mark constants — mirror Python's _REP1_MARKS / _REP2_MARKS.
const REP1_MARKS: &[&str] = &["sample_rep1", "blank_positive"];
const REP2_MARKS: &[&str] = &["sample_rep2", "blank_negative"];

fn is_rep1(mark: &str) -> bool {
    REP1_MARKS.contains(&mark)
}
fn is_rep2(mark: &str) -> bool {
    REP2_MARKS.contains(&mark)
}

/// Determine bucket key for a row.
fn replicate_bucket_key(row: &Row, idx: usize, use_marks: bool) -> String {
    if use_marks {
        if let Some(m) = &row.operator_mark {
            if !m.is_empty() {
                return m.clone();
            }
        }
    }
    // Fall back to filename or row index as unique key.
    if row.file.is_empty() {
        idx.to_string()
    } else {
        row.file.clone()
    }
}

/// Split a group of row indices into replicate buckets.
/// Returns `(buckets, colour_split)` where each bucket is `(name, Vec<usize>)`.
pub fn split_replicate_buckets(
    group_indices: &[usize],
    rows: &[Row],
) -> (Vec<(String, Vec<usize>)>, bool) {
    let marks_present: HashSet<&str> = group_indices
        .iter()
        .filter_map(|&i| rows[i].operator_mark.as_deref())
        .collect();

    let has_rep1 = marks_present.iter().any(|m| is_rep1(m));
    let has_rep2 = marks_present.iter().any(|m| is_rep2(m));
    let colour_split = has_rep1 && has_rep2;

    let unique_files: HashSet<&str> = group_indices
        .iter()
        .filter(|&&i| !rows[i].file.is_empty())
        .map(|&i| rows[i].file.as_str())
        .collect();

    let use_marks = colour_split && unique_files.len() <= 2;

    let mut bucket_map: HashMap<String, Vec<usize>> = HashMap::new();
    // Preserve insertion order via a separate key list.
    let mut key_order: Vec<String> = Vec::new();

    for &i in group_indices {
        let key = replicate_bucket_key(&rows[i], i, use_marks);
        if !bucket_map.contains_key(&key) {
            key_order.push(key.clone());
        }
        bucket_map.entry(key).or_default().push(i);
    }

    let buckets = key_order
        .into_iter()
        .map(|k| {
            let v = bucket_map.remove(&k).unwrap();
            (k, v)
        })
        .collect();

    (buckets, colour_split)
}

/// Centroid (RT, mz) of a set of rows.
fn cluster_centroid(members: &[ClusterMember], rows: &[Row]) -> (f64, Option<f64>) {
    let n = members.len() as f64;
    let rt = members.iter().map(|m| rows[m.row_idx].rt).sum::<f64>() / n;
    let mz_values: Vec<f64> = members.iter().filter_map(|m| rows[m.row_idx].base_peak).collect();
    let mz = if mz_values.is_empty() {
        None
    } else {
        Some(mz_values.iter().sum::<f64>() / mz_values.len() as f64)
    };
    (rt, mz)
}

/// Check whether a candidate row matches a centroid within tolerances.
/// Returns `(matches, rt_delta, mz_delta_da, mz_delta_ppm, mz_delta_in_mode, distance)`.
fn match_to_centroid(
    centroid_rt: f64,
    centroid_mz: Option<f64>,
    candidate: &Row,
    config: &ScreeningConfig,
) -> (bool, f64, f64, f64, f64, f64) {
    let mm = match_metrics(
        centroid_rt,
        centroid_mz,
        candidate.rt,
        candidate.base_peak,
        config.replicate_rt_tol,
        config.replicate_mz_tol,
        config.replicate_mz_mode_str(),
    );
    (
        mm.matches,
        mm.rt_delta,
        mm.mz_delta_da.unwrap_or(0.0),
        mm.mz_delta_ppm.unwrap_or(0.0),
        mm.mz_delta_in_mode.unwrap_or(0.0),
        mm.distance,
    )
}

/// Greedily pick the best candidate from each other bucket and build the cluster.
pub fn choose_cluster_members(
    seed_row_idx: usize,
    seed_bucket_idx: usize,
    buckets: &[(String, Vec<usize>)],
    rows: &[Row],
    used: &HashSet<(usize, usize)>,
    config: &ScreeningConfig,
) -> Vec<ClusterMember> {
    let mut members = vec![ClusterMember {
        bucket_name: buckets[seed_bucket_idx].0.clone(),
        bucket_idx: seed_bucket_idx,
        row_idx: seed_row_idx,
        distance: 0.0,
    }];

    let (mut centroid_rt, mut centroid_mz) = cluster_centroid(&members, rows);

    for (bucket_idx, (bucket_name, bucket_indices)) in buckets.iter().enumerate() {
        if bucket_idx == seed_bucket_idx {
            continue;
        }

        let mut best: Option<ClusterMember> = None;
        let mut best_area = f64::NEG_INFINITY;

        for &row_idx in bucket_indices {
            if used.contains(&(bucket_idx, row_idx)) {
                continue;
            }
            let (matches, _, _, _, _, distance) =
                match_to_centroid(centroid_rt, centroid_mz, &rows[row_idx], config);
            if !matches {
                continue;
            }
            let area = rows[row_idx].area;
            let better = match &best {
                None => true,
                Some(b) => distance < b.distance || (distance == b.distance && area > best_area),
            };
            if better {
                best = Some(ClusterMember {
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
            let (crt, cmz) = cluster_centroid(&members, rows);
            centroid_rt = crt;
            centroid_mz = cmz;
        }
    }

    members
}

/// Compute pairwise RT and m/z metrics across all member pairs.
struct PairwiseMetrics {
    max_rt_delta: f64,
    max_mz_delta_da: Option<f64>,
    max_mz_delta_ppm: Option<f64>,
    mean_rt_delta: f64,
    mean_mz_delta_in_mode: Option<f64>,
    uses_mz: bool,
}

fn pairwise_cluster_metrics(members: &[ClusterMember], rows: &[Row], config: &ScreeningConfig) -> PairwiseMetrics {
    let mut rt_deltas = Vec::new();
    let mut mz_deltas_da = Vec::new();
    let mut mz_deltas_ppm = Vec::new();
    let mut mz_deltas_in_mode = Vec::new();

    for i in 0..members.len() {
        for j in (i + 1)..members.len() {
            let a = &rows[members[i].row_idx];
            let b = &rows[members[j].row_idx];
            let mm = match_metrics(
                a.rt, a.base_peak,
                b.rt, b.base_peak,
                config.replicate_rt_tol,
                config.replicate_mz_tol,
                config.replicate_mz_mode_str(),
            );
            rt_deltas.push(mm.rt_delta);
            if mm.uses_mz {
                mz_deltas_da.push(mm.mz_delta_da.unwrap_or(0.0));
                mz_deltas_ppm.push(mm.mz_delta_ppm.unwrap_or(0.0));
                mz_deltas_in_mode.push(mm.mz_delta_in_mode.unwrap_or(0.0));
            }
        }
    }

    let mean_or_zero = |v: &[f64]| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
    let max_or_zero = |v: &[f64]| v.iter().cloned().fold(0.0_f64, f64::max);

    PairwiseMetrics {
        uses_mz: !mz_deltas_in_mode.is_empty(),
        max_rt_delta: max_or_zero(&rt_deltas),
        max_mz_delta_da: if mz_deltas_da.is_empty() { None } else { Some(max_or_zero(&mz_deltas_da)) },
        max_mz_delta_ppm: if mz_deltas_ppm.is_empty() { None } else { Some(max_or_zero(&mz_deltas_ppm)) },
        mean_rt_delta: mean_or_zero(&rt_deltas),
        mean_mz_delta_in_mode: if mz_deltas_in_mode.is_empty() { None } else { Some(mean_or_zero(&mz_deltas_in_mode)) },
    }
}

/// Convert a fully assembled cluster into a `ConfirmedRow`.
pub fn cluster_to_confirmed_row(
    members: &[ClusterMember],
    rows: &[Row],
    colour_split: bool,
    config: &ScreeningConfig,
) -> ConfirmedRow {
    let member_rows: Vec<&Row> = members.iter().map(|m| &rows[m.row_idx]).collect();
    let area_values: Vec<f64> = member_rows.iter().map(|r| r.area).collect();
    let rt_values: Vec<f64> = member_rows.iter().map(|r| r.rt).collect();
    let mz_values: Vec<f64> = member_rows.iter().filter_map(|r| r.base_peak).collect();

    let area_mean = area_values.iter().sum::<f64>() / area_values.len() as f64;
    let rt_mean = rt_values.iter().sum::<f64>() / rt_values.len() as f64;
    let mz_mean = if mz_values.is_empty() {
        None
    } else {
        Some(mz_values.iter().sum::<f64>() / mz_values.len() as f64)
    };

    let area_cv_pct = calc_cv_percent(&area_values);
    let replicate_quality = classify_replicate_quality(area_cv_pct, config).to_string();
    let pm = pairwise_cluster_metrics(members, rows, config);

    let rep_score = replicate_confidence_score(
        pm.mean_rt_delta,
        config.replicate_rt_tol,
        pm.mean_mz_delta_in_mode,
        config.replicate_mz_tol,
        area_cv_pct,
        colour_split,
        pm.uses_mz,
        config,
    );
    let matching_mode = if pm.uses_mz { "RT+MZ" } else { "RT" }.to_string();

    let first = member_rows[0];
    let second = member_rows.get(1).copied();

    // Build Why.ReplicateMembers
    let replicate_members: Vec<Value> = members
        .iter()
        .zip(member_rows.iter())
        .map(|(m, r)| {
            json!({
                "bucket": m.bucket_name,
                "file": r.file,
                "label": r.label,
                "operator_mark": r.operator_mark,
                "operator_color": r.operator_color,
                "rt": safe_round(r.rt, 4),
                "mz": r.base_peak.map(|v| safe_round(v, 6)),
                "area": safe_round(r.area, 2),
            })
        })
        .collect();

    let why = json!({
        "ReplicateStrategy": "greedy_cluster_with_singleton_carryover",
        "ReplicateBuckets": members.iter().map(|m| &m.bucket_name).collect::<Vec<_>>(),
        "ReplicateCount": members.len(),
        "ReplicateMembers": replicate_members,
        "ReplicateRT": {
            "mean": safe_round(rt_mean, 4),
            "max_delta": safe_round(pm.max_rt_delta, 4),
            "tolerance": config.replicate_rt_tol,
            "unit": "min",
        },
        "ReplicateMZ": {
            "mean": mz_mean.map(|v| safe_round(v, 6)),
            "max_delta_da": pm.max_mz_delta_da.map(|v| safe_round(v, 6)),
            "max_delta_ppm": pm.max_mz_delta_ppm.map(|v| safe_round(v, 2)),
            "tolerance": config.replicate_mz_tol,
            "mode": config.replicate_mz_mode_str(),
            "used": pm.uses_mz,
        },
        "ReplicateArea": {
            "values": area_values.iter().map(|&v| safe_round(v, 2)).collect::<Vec<_>>(),
            "mean": safe_round(area_mean, 2),
            "cv_pct": area_cv_pct.map(|v| safe_round(v, 2)),
        },
        "ReplicateQuality": replicate_quality.clone(),
        "ReplicateConfidenceScore": rep_score,
        "ColorPaired": colour_split,
        "Matches": members.len() > 1,
        "MatchingMode": matching_mode.clone(),
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
        group: format!("{}_{}", first.sample_type, first.polarity),
        rt_mean: safe_round(rt_mean, 4),
        mz_mean: mz_mean.map(|v| safe_round(v, 6)),
        area_mean: safe_round(area_mean, 2),
        area_cv_pct: area_cv_pct.map(|v| safe_round(v, 2)),
        replicate_quality,
        replicate_count: members.len(),
        replicate_confidence_score: rep_score,
        confidence_score: rep_score, // updated after blank subtraction
        polarity: first.polarity.clone(),
        sample_type: first.sample_type.clone(),
        rep1_label: first.label.clone(),
        rep2_label: second.and_then(|r| r.label.clone()),
        rep1_mark: first.operator_mark.clone(),
        rep2_mark: second.and_then(|r| r.operator_mark.clone()),
        rep1_color: first.operator_color.clone(),
        rep2_color: second.and_then(|r| r.operator_color.clone()),
        replicate_files: member_rows.iter().map(|r| Some(r.file.clone())).collect(),
        replicate_labels: member_rows.iter().map(|r| r.label.clone()).collect(),
        replicate_marks: member_rows.iter().map(|r| r.operator_mark.clone()).collect(),
        replicate_colors: member_rows.iter().map(|r| r.operator_color.clone()).collect(),
        confirmed: "Yes".to_string(),
        matching_mode,
        parallel_match: false,
        parallel_sample_count: 1,
        parallel_source_samples: vec![first.sample_type.clone()],
        blank_area_mean: None,
        area_difference: None,
        signal_to_blank_ratio: None,
        status: String::new(), // filled by blank subtraction
        why,
    }
}

/// Greedy cluster screening for a single (SampleType, Polarity) group.
pub fn coarse_screen(
    group_indices: &[usize],
    rows: &[Row],
    config: &ScreeningConfig,
) -> Vec<ConfirmedRow> {
    let (buckets, colour_split) = split_replicate_buckets(group_indices, rows);
    if buckets.is_empty() {
        return vec![];
    }

    // Build all seeds with (bucket_idx, row_idx, area) and sort by area desc.
    let mut seeds: Vec<(usize, usize, f64)> = buckets
        .iter()
        .enumerate()
        .flat_map(|(bi, (_, indices))| {
            indices.iter().map(move |&ri| (bi, ri, rows[ri].area))
        })
        .collect();
    seeds.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });

    let mut used: HashSet<(usize, usize)> = HashSet::new();
    let mut confirmed = Vec::new();

    for (bucket_idx, row_idx, _) in &seeds {
        if used.contains(&(*bucket_idx, *row_idx)) {
            continue;
        }
        let members =
            choose_cluster_members(*row_idx, *bucket_idx, &buckets, rows, &used, config);
        if members.len() < 2 {
            continue;
        }
        for m in &members {
            used.insert((m.bucket_idx, m.row_idx));
        }
        confirmed.push(cluster_to_confirmed_row(&members, rows, colour_split, config));
    }

    for (bucket_idx, row_idx, _) in &seeds {
        if used.contains(&(*bucket_idx, *row_idx)) {
            continue;
        }
        used.insert((*bucket_idx, *row_idx));
        let singleton = vec![ClusterMember {
            bucket_name: buckets[*bucket_idx].0.clone(),
            bucket_idx: *bucket_idx,
            row_idx: *row_idx,
            distance: 0.0,
        }];
        confirmed.push(cluster_to_confirmed_row(&singleton, rows, colour_split, config));
    }

    confirmed
}
