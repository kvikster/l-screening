## ADDED Requirements

### Requirement: Run without Polarity
The system SHALL run the screening algorithm when the Polarity column is absent or empty for all rows. When Polarity is unavailable, the system SHALL treat all rows as a single polarity group rather than splitting per-polarity, and SHALL NOT fail parsing or processing for the missing column.

#### Scenario: Dataset has no Polarity column
- **WHEN** the input set contains no Polarity values
- **THEN** the system processes all peaks in one combined polarity group and produces confirmed clusters without error

#### Scenario: Polarity present is unchanged
- **WHEN** the input set contains Polarity values
- **THEN** the system continues to group and report per-polarity as before

### Requirement: Run without Polarity and Base Peak (RT-only)
The system SHALL run when both Polarity and Base Peak (m/z) are absent, matching replicates and blanks on retention time alone. When m/z is unavailable for the whole dataset, the confidence penalty that normally applies to a missing m/z match SHALL be waived, because the absence is a data-format property and not a matching weakness.

#### Scenario: RT-only dataset is screened
- **WHEN** the input set has neither Polarity nor Base Peak values
- **THEN** the system matches replicates and blanks by RT within the configured RT tolerance and produces results

#### Scenario: Missing-m/z penalty waived for RT-only data
- **WHEN** the dataset is RT-only (m/z unavailable for all rows)
- **THEN** the system does not apply the missing-m/z confidence penalty to any cluster

#### Scenario: Mixed availability still penalizes per-row gaps
- **WHEN** the dataset has m/z available but an individual cluster lacks an m/z match
- **THEN** the system still applies the per-cluster missing-m/z penalty as before
