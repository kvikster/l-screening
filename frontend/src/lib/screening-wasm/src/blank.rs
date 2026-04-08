use crate::config::ScreeningConfig;
use crate::math::{fraction_of_tol, mz_delta, ppm_delta, safe_round};
use crate::types::{BlankCandidate, ConfirmedRow};

/// Find all blank confirmed-rows that match a sample peak within blank tolerances.
/// Returned candidates are sorted by (distance asc, area desc).
pub fn blank_candidates(
    peak: &ConfirmedRow,
    blanks: &[ConfirmedRow],
    config: &ScreeningConfig,
) -> Vec<BlankCandidate> {
    let mut candidates: Vec<BlankCandidate> = blanks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.polarity == peak.polarity)
        .filter_map(|(idx, blank)| {
            let rt_delta = (blank.rt_mean - peak.rt_mean).abs();
            let mz_delta_da = (blank.mz_mean - peak.mz_mean).abs();
            let mz_delta_ppm = ppm_delta(blank.mz_mean, peak.mz_mean);
            let mz_delta_in_mode =
                mz_delta(blank.mz_mean, peak.mz_mean, config.blank_mz_mode_str());

            if rt_delta > config.blank_rt_tol || mz_delta_in_mode > config.blank_mz_tol {
                return None;
            }

            let distance = fraction_of_tol(rt_delta, config.blank_rt_tol)
                + fraction_of_tol(mz_delta_in_mode, config.blank_mz_tol);

            Some(BlankCandidate {
                blank_row_idx: idx,
                rt_delta,
                mz_delta_da,
                mz_delta_ppm,
                mz_delta_in_mode,
                distance,
            })
        })
        .collect();

    // Sort: closest first, then largest blank area first (as tie-break).
    candidates.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                blanks[b.blank_row_idx]
                    .area_mean
                    .partial_cmp(&blanks[a.blank_row_idx].area_mean)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    candidates
}

/// Apply blank subtraction to a mutable sample peak in place.
/// Updates `signal_to_blank_ratio`, `status`, `confidence_score`, and the `Why` audit trail.
pub fn apply_blank_result(
    peak: &mut ConfirmedRow,
    blanks: &[ConfirmedRow],
    candidates: &[BlankCandidate],
    config: &ScreeningConfig,
) {
    use crate::math::final_confidence_score;
    use serde_json::json;

    let best = candidates.first();

    let signal_to_blank_ratio = best.and_then(|cand| {
        let blank_area = blanks[cand.blank_row_idx].area_mean;
        if blank_area <= 0.0 {
            None
        } else {
            Some(peak.area_mean / blank_area)
        }
    });

    let has_blank_match = best.is_some();

    let status = if has_blank_match {
        match signal_to_blank_ratio {
            None => "Artifact",
            Some(ratio) if ratio < config.signal_to_blank_min => "Artifact",
            _ => "Real Compound",
        }
    } else {
        "Real Compound"
    };

    peak.signal_to_blank_ratio = signal_to_blank_ratio.map(|v| safe_round(v, 2));
    peak.status = status.to_string();
    peak.confidence_score = final_confidence_score(
        peak.replicate_confidence_score,
        has_blank_match,
        signal_to_blank_ratio,
        config,
    );

    // Extend Why with blank subtraction results.
    if let Some(why_obj) = peak.why.as_object_mut() {
        why_obj.insert("BlankMatch".into(), json!(has_blank_match));
        why_obj.insert("BlankCandidateCount".into(), json!(candidates.len()));
        why_obj.insert(
            "SignalToBlankRatio".into(),
            json!(signal_to_blank_ratio.map(|v| safe_round(v, 2))),
        );
        why_obj.insert("SignalToBlankThreshold".into(), json!(config.signal_to_blank_min));
        why_obj.insert("ConfidenceScore".into(), json!(peak.confidence_score));
        why_obj.insert("Decision".into(), json!(status));

        if let Some(cand) = best {
            let blank = &blanks[cand.blank_row_idx];
            why_obj.insert(
                "BlankDetail".into(),
                json!({
                    "RT": safe_round(blank.rt_mean, 4),
                    "MZ": safe_round(blank.mz_mean, 6),
                    "Area_mean": safe_round(blank.area_mean, 2),
                    "rt_delta": safe_round(cand.rt_delta, 4),
                    "mz_delta_da": safe_round(cand.mz_delta_da, 6),
                    "mz_delta_ppm": safe_round(cand.mz_delta_ppm, 2),
                    "tolerance": {
                        "rt": config.blank_rt_tol,
                        "mz": config.blank_mz_tol,
                        "mz_mode": config.blank_mz_mode_str(),
                    }
                }),
            );
        }
    }
}
