use serde::Deserialize;

pub const RT_TOL: f64 = 0.1;
pub const MZ_TOL: f64 = 0.3;
pub const DEFAULT_SIGNAL_TO_BLANK_MIN: f64 = 3.0;
pub const DEFAULT_CV_HIGH_MAX: f64 = 15.0;
pub const DEFAULT_CV_MODERATE_MAX: f64 = 30.0;

#[derive(Debug, Clone, Deserialize)]
pub struct ScreeningConfig {
    #[serde(default = "default_rt_tol")]
    pub replicate_rt_tol: f64,
    #[serde(default = "default_mz_tol")]
    pub replicate_mz_tol: f64,
    #[serde(default = "default_mz_mode")]
    pub replicate_mz_mode: String,
    #[serde(default = "default_rt_tol")]
    pub blank_rt_tol: f64,
    #[serde(default = "default_mz_tol")]
    pub blank_mz_tol: f64,
    #[serde(default = "default_mz_mode")]
    pub blank_mz_mode: String,
    #[serde(default = "default_signal_to_blank_min")]
    pub signal_to_blank_min: f64,
    #[serde(default = "default_cv_high_max")]
    pub cv_high_max: f64,
    #[serde(default = "default_cv_moderate_max")]
    pub cv_moderate_max: f64,
}

fn default_rt_tol() -> f64 { RT_TOL }
fn default_mz_tol() -> f64 { MZ_TOL }
fn default_mz_mode() -> String { "da".to_string() }
fn default_signal_to_blank_min() -> f64 { DEFAULT_SIGNAL_TO_BLANK_MIN }
fn default_cv_high_max() -> f64 { DEFAULT_CV_HIGH_MAX }
fn default_cv_moderate_max() -> f64 { DEFAULT_CV_MODERATE_MAX }

impl ScreeningConfig {
    pub fn validate(&self) -> Result<(), String> {
        let rep_mode = self.replicate_mz_mode.to_lowercase();
        let blank_mode = self.blank_mz_mode.to_lowercase();
        if rep_mode != "da" && rep_mode != "ppm" {
            return Err("replicate_mz_mode must be 'da' or 'ppm'".to_string());
        }
        if blank_mode != "da" && blank_mode != "ppm" {
            return Err("blank_mz_mode must be 'da' or 'ppm'".to_string());
        }
        for (name, val) in [
            ("replicate_rt_tol", self.replicate_rt_tol),
            ("replicate_mz_tol", self.replicate_mz_tol),
            ("blank_rt_tol", self.blank_rt_tol),
            ("blank_mz_tol", self.blank_mz_tol),
            ("signal_to_blank_min", self.signal_to_blank_min),
            ("cv_high_max", self.cv_high_max),
            ("cv_moderate_max", self.cv_moderate_max),
        ] {
            if val < 0.0 {
                return Err(format!("{name} must be >= 0"));
            }
        }
        if self.cv_moderate_max < self.cv_high_max {
            return Err("cv_moderate_max must be >= cv_high_max".to_string());
        }
        Ok(())
    }

    pub fn replicate_mz_mode_str(&self) -> &str {
        &self.replicate_mz_mode
    }

    pub fn blank_mz_mode_str(&self) -> &str {
        &self.blank_mz_mode
    }
}

impl Default for ScreeningConfig {
    fn default() -> Self {
        ScreeningConfig {
            replicate_rt_tol: RT_TOL,
            replicate_mz_tol: MZ_TOL,
            replicate_mz_mode: "da".to_string(),
            blank_rt_tol: RT_TOL,
            blank_mz_tol: MZ_TOL,
            blank_mz_mode: "da".to_string(),
            signal_to_blank_min: DEFAULT_SIGNAL_TO_BLANK_MIN,
            cv_high_max: DEFAULT_CV_HIGH_MAX,
            cv_moderate_max: DEFAULT_CV_MODERATE_MAX,
        }
    }
}
