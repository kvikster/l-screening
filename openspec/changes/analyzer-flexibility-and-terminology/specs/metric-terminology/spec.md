## ADDED Requirements

### Requirement: CV% surfaced as RSD
Where the displayed CV% is the relative standard deviation (standard deviation divided by mean, expressed as a percent), the system SHALL label it as RSD (or "RSD (CV%)") in operator-facing UI so the term matches lab vocabulary. The underlying computed value SHALL be unchanged; only the label and any tooltip/help text change.

#### Scenario: Area variability labeled as RSD
- **WHEN** the operator views the area variability metric for a confirmed cluster
- **THEN** the metric is labeled RSD (relative standard deviation), with CV% noted as the equivalent term, and the numeric value is identical to the prior CV% value

### Requirement: Replicate clarified as one chromatogram
Wherever "Replicate" or "ReplicateCount" appears in operator-facing UI, the system SHALL clarify that one replicate corresponds to one chromatogram, e.g. "Replicates (chromatograms)".

#### Scenario: Replicate count shows chromatogram meaning
- **WHEN** the operator views the replicate count for a confirmed cluster
- **THEN** the label clarifies that 1 replicate = 1 chromatogram
