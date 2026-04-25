use crate::config::ScreeningConfig;
use crate::types::MatchMetrics;

pub fn safe_round(value: f64, digits: u32) -> f64 {
    let factor = 10f64.powi(digits as i32);
    (value * factor).round() / factor
}

pub fn ppm_delta(mz_a: f64, mz_b: f64) -> f64 {
    let center = (mz_a + mz_b) / 2.0;
    if center == 0.0 {
        return 0.0;
    }
    (mz_a - mz_b).abs() / center * 1_000_000.0
}

pub fn mz_delta(mz_a: f64, mz_b: f64, mode: &str) -> f64 {
    if mode == "ppm" {
        ppm_delta(mz_a, mz_b)
    } else {
        (mz_a - mz_b).abs()
    }
}

/// Match two peaks by RT and optionally m/z.
/// Mirrors Python's `_match_metrics` helper — single source of truth for the
/// tolerance-checking logic used in clustering, parallel merge, and blank matching.
pub fn match_metrics(
    rt_a: f64,
    mz_a: Option<f64>,
    rt_b: f64,
    mz_b: Option<f64>,
    rt_tol: f64,
    mz_tol: f64,
    mz_mode: &str,
) -> MatchMetrics {
    let rt_delta = (rt_a - rt_b).abs();
    match (mz_a, mz_b) {
        (Some(ma), Some(mb)) => {
            let mz_delta_da = (ma - mb).abs();
            let mz_delta_ppm = ppm_delta(ma, mb);
            let mz_delta_in_mode = mz_delta(ma, mb, mz_mode);
            let matches = rt_delta <= rt_tol && mz_delta_in_mode <= mz_tol;
            let distance = fraction_of_tol(rt_delta, rt_tol)
                + fraction_of_tol(mz_delta_in_mode, mz_tol);
            MatchMetrics {
                matches,
                uses_mz: true,
                rt_delta,
                mz_delta_da: Some(mz_delta_da),
                mz_delta_ppm: Some(mz_delta_ppm),
                mz_delta_in_mode: Some(mz_delta_in_mode),
                distance,
            }
        }
        _ => {
            let matches = rt_delta <= rt_tol;
            let distance = fraction_of_tol(rt_delta, rt_tol);
            MatchMetrics {
                matches,
                uses_mz: false,
                rt_delta,
                mz_delta_da: None,
                mz_delta_ppm: None,
                mz_delta_in_mode: None,
                distance,
            }
        }
    }
}

pub fn fraction_of_tol(delta: f64, tolerance: f64) -> f64 {
    if tolerance <= 0.0 {
        return if delta == 0.0 { 0.0 } else { 1.5 };
    }
    (delta / tolerance).min(1.5)
}

/// Coefficient of variation in percent.
/// For n=2 uses the formula std = |v1-v2|/√2 (sample std for two observations).
pub fn calc_cv_percent(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean == 0.0 {
        return None;
    }
    let std = if values.len() == 2 {
        (values[0] - values[1]).abs() / 2f64.sqrt()
    } else {
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
            / (values.len() - 1) as f64;
        variance.sqrt()
    };
    Some(std / mean * 100.0)
}

pub fn classify_replicate_quality(cv_percent: Option<f64>, config: &ScreeningConfig) -> &'static str {
    match cv_percent {
        None => "Unknown",
        Some(cv) if cv <= config.cv_high_max => "High",
        Some(cv) if cv <= config.cv_moderate_max => "Moderate",
        _ => "Low",
    }
}

pub fn replicate_confidence_score(
    rt_delta: f64,
    rt_tol: f64,
    mz_delta_in_mode: Option<f64>,
    mz_tol: f64,
    cv_percent: Option<f64>,
    color_paired: bool,
    use_mz: bool,
    config: &ScreeningConfig,
) -> f64 {
    let mut score = 100.0_f64;
    score -= fraction_of_tol(rt_delta, rt_tol) * 20.0;
    if use_mz {
        // Contract: when use_mz=true the caller guarantees that both peaks had
        // m/z values, so mz_delta_in_mode must be Some(_).  The unwrap_or
        // fallback is a defensive guard for correctness in release builds.
        let delta = mz_delta_in_mode.unwrap_or(0.0);
        score -= fraction_of_tol(delta, mz_tol) * 25.0;
    } else if config.mz_available {
        // m/z column exists but this specific cluster pair couldn't use it.
        score -= 10.0;
    }
    // When mz_available=false the whole dataset is RT-only (GC-FID, LC-UV, …).
    // Absence of m/z is a data-format property, not a per-cluster weakness,
    // so no penalty is applied.

    match cv_percent {
        None => score -= 10.0,
        Some(cv) if cv <= config.cv_high_max => {}
        Some(cv) if cv <= config.cv_moderate_max => score -= 12.0,
        Some(cv) => score -= (12.0 + (cv - config.cv_moderate_max) * 0.7_f64).min(35.0),
    }

    if !color_paired {
        score -= 5.0;
    }

    safe_round(score.clamp(0.0, 100.0), 1)
}

pub fn final_confidence_score(
    replicate_score: f64,
    has_blank_match: bool,
    signal_to_blank_ratio: Option<f64>,
    config: &ScreeningConfig,
) -> f64 {
    let mut score = replicate_score;
    if !has_blank_match {
        score += 3.0;
    } else {
        match signal_to_blank_ratio {
            None => score -= 10.0,
            Some(ratio) if ratio >= config.signal_to_blank_min => {
                score += ((ratio - config.signal_to_blank_min) * 0.5_f64).min(5.0);
            }
            Some(ratio) => {
                let deficit = (config.signal_to_blank_min - ratio)
                    / config.signal_to_blank_min.max(1e-9);
                score -= 15.0 + (deficit * 30.0).min(30.0);
            }
        }
    }
    safe_round(score.clamp(0.0, 100.0), 1)
}
