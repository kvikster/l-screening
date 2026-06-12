## ADDED Requirements

### Requirement: Suggested cluster bin width from surrogate behavior
The system SHALL present multiple candidate replicate RT tolerance ("cluster bin width") metrics derived from a surrogate's observed RT behavior. When a surrogate's additional data is opened, the system SHALL show all three metrics — max|RT shift| (vs expected), k·stdev of replicate RTs, and range (max−min of replicate RTs) — each alongside the replicate count, so the operator can judge reliability and decide which to use. This is advisory only; the system SHALL NOT auto-apply any of them.

#### Scenario: Surrogate additional data shows three bin-width metrics
- **WHEN** the operator opens the additional data of a surrogate cluster
- **THEN** the system shows max|RT shift|, k·stdev of replicate RTs, and the RT range, each with the replicate count

#### Scenario: Suggestion is manual, not automatic
- **WHEN** the system presents a suggested bin width
- **THEN** the configured replicate RT tolerance is unchanged until the operator enters a value into Analyzer Configuration

### Requirement: Manual entry of bin width into Analyzer Configuration
The system SHALL allow the operator to set the replicate RT tolerance in Analyzer Configuration, including a value copied from the surrogate suggestion. The entered value SHALL be used as the replicate RT tolerance on the next run.

#### Scenario: Operator copies suggestion into configuration
- **WHEN** the operator enters the suggested RT tolerance into Analyzer Configuration and re-runs screening
- **THEN** the system uses that value as the replicate RT tolerance for clustering
