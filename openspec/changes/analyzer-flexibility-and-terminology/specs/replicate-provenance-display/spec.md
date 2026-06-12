## ADDED Requirements

### Requirement: Highlight replicate count and origin
The system SHALL visually highlight, for each confirmed cluster, the number of replicates and the origin of each replicate — the source file and/or sample each replicate came from. The provenance SHALL be derived from the cluster's per-replicate file and label data already produced by screening.

#### Scenario: Replicate count is highlighted
- **WHEN** the operator views a confirmed cluster
- **THEN** the replicate count is visually emphasized so it stands out from other metrics

#### Scenario: Replicate origin is shown
- **WHEN** the operator inspects a confirmed cluster's replicates
- **THEN** the system shows, per replicate, the source file and/or sample it originated from

#### Scenario: Parallel-sample provenance is distinguished
- **WHEN** a cluster was merged from parallel samples (multiple source samples)
- **THEN** the system indicates that the replicates span more than one source sample and which samples those are
