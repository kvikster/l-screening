use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Input row parsed from the Excel sheet (after SheetJS + color extraction in JS).
#[derive(Debug, Clone, Deserialize)]
pub struct Row {
    #[serde(rename = "RT")]
    pub rt: f64,
    #[serde(rename = "Base Peak")]
    pub base_peak: f64,
    #[serde(rename = "Area")]
    pub area: f64,
    #[serde(rename = "Polarity")]
    pub polarity: String,
    #[serde(rename = "File")]
    pub file: String,
    #[serde(rename = "Label", default)]
    pub label: Option<String>,
    #[serde(rename = "operator_mark", default)]
    pub operator_mark: Option<String>,
    #[serde(rename = "operator_color", default)]
    pub operator_color: Option<String>,
    /// Assigned by assign_sample_type — not present in the raw Excel data.
    #[serde(skip_deserializing, default)]
    pub sample_type: String,
}

/// A confirmed peak cluster — one row per matched replicate group.
#[derive(Debug, Clone, Serialize)]
pub struct ConfirmedRow {
    #[serde(rename = "Group")]
    pub group: String,
    #[serde(rename = "RT_mean")]
    pub rt_mean: f64,
    #[serde(rename = "MZ_mean")]
    pub mz_mean: f64,
    #[serde(rename = "Area_mean")]
    pub area_mean: f64,
    #[serde(rename = "AreaCVPct")]
    pub area_cv_pct: Option<f64>,
    #[serde(rename = "ReplicateQuality")]
    pub replicate_quality: String,
    #[serde(rename = "ReplicateCount")]
    pub replicate_count: usize,
    #[serde(rename = "ReplicateConfidenceScore")]
    pub replicate_confidence_score: f64,
    /// Updated after blank subtraction.
    #[serde(rename = "ConfidenceScore")]
    pub confidence_score: f64,
    #[serde(rename = "Polarity")]
    pub polarity: String,
    #[serde(rename = "SampleType")]
    pub sample_type: String,
    #[serde(rename = "Rep1_Label")]
    pub rep1_label: Option<String>,
    #[serde(rename = "Rep2_Label")]
    pub rep2_label: Option<String>,
    #[serde(rename = "Rep1_Mark")]
    pub rep1_mark: Option<String>,
    #[serde(rename = "Rep2_Mark")]
    pub rep2_mark: Option<String>,
    #[serde(rename = "Rep1_Color")]
    pub rep1_color: Option<String>,
    #[serde(rename = "Rep2_Color")]
    pub rep2_color: Option<String>,
    #[serde(rename = "ReplicateFiles")]
    pub replicate_files: Vec<Option<String>>,
    #[serde(rename = "ReplicateLabels")]
    pub replicate_labels: Vec<Option<String>>,
    #[serde(rename = "ReplicateMarks")]
    pub replicate_marks: Vec<Option<String>>,
    #[serde(rename = "ReplicateColors")]
    pub replicate_colors: Vec<Option<String>>,
    #[serde(rename = "Confirmed")]
    pub confirmed: String,
    /// Filled by blank subtraction step.
    #[serde(rename = "SignalToBlankRatio")]
    pub signal_to_blank_ratio: Option<f64>,
    #[serde(rename = "Status")]
    pub status: String,
    /// Full audit trail object.
    #[serde(rename = "Why")]
    pub why: Value,
}

/// Per-(SampleType, Polarity) summary statistics.
#[derive(Debug, Serialize)]
pub struct SummaryRow {
    #[serde(rename = "Sample")]
    pub sample: String,
    #[serde(rename = "Polarity")]
    pub polarity: String,
    #[serde(rename = "TotalPeaks")]
    pub total_peaks: usize,
    #[serde(rename = "Confirmed")]
    pub confirmed: usize,
    #[serde(rename = "Artifacts")]
    pub artifacts: usize,
    #[serde(rename = "RealCompounds")]
    pub real_compounds: usize,
    #[serde(rename = "ColorDriven")]
    pub color_driven: bool,
    #[serde(rename = "MeanCVPct")]
    pub mean_cv_pct: Option<f64>,
    #[serde(rename = "HighQuality")]
    pub high_quality: usize,
    #[serde(rename = "ModerateQuality")]
    pub moderate_quality: usize,
    #[serde(rename = "LowQuality")]
    pub low_quality: usize,
    #[serde(rename = "MeanConfidenceScore")]
    pub mean_confidence_score: Option<f64>,
    #[serde(rename = "MeanSignalToBlankRatio")]
    pub mean_signal_to_blank_ratio: Option<f64>,
}

/// Internal member record used during cluster assembly.
#[derive(Debug, Clone)]
pub struct ClusterMember {
    pub bucket_name: String,
    pub bucket_idx: usize,
    pub row_idx: usize,
    pub distance: f64,
}

/// Internal blank candidate record.
#[derive(Debug)]
pub struct BlankCandidate {
    pub blank_row_idx: usize,
    pub rt_delta: f64,
    pub mz_delta_da: f64,
    pub mz_delta_ppm: f64,
    #[allow(dead_code)]
    pub mz_delta_in_mode: f64,
    pub distance: f64,
}
