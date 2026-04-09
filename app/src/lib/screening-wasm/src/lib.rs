mod blank;
mod cluster;
mod config;
mod math;
mod types;

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use blank::{apply_blank_result, blank_candidates};
use cluster::coarse_screen;
use config::ScreeningConfig;
use math::safe_round;
use types::{Row, SummaryRow};

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

    let mut final_results: Vec<_> = all_confirmed
        .into_iter()
        .filter(|r| r.sample_type != "blank")
        .collect();

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

    // Collect all unique (stype, polarity) pairs from original rows.
    let mut group_keys: Vec<(String, String)> = groups.keys().cloned().collect();
    group_keys.sort();

    for (stype, pol) in group_keys {
        let group_indices = &groups[&(stype.clone(), pol.clone())];

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
