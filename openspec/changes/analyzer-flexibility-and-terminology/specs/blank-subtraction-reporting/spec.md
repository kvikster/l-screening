## ADDED Requirements

### Requirement: Signal-to-Blank when no blank bin exists
When a sample peak has no matching blank cluster, the system SHALL report this as an explicit outcome rather than an empty/missing value, and SHALL distinguish two cases: (a) no blank was loaded in the set at all (no blank cluster exists in the group), and (b) a blank exists in the set but is clean at this RT (blank clusters exist but none match this peak). In both cases the peak SHALL be classified as a Real Compound (nothing to subtract); the audit trail SHALL record which of the two cases applies, and both SHALL be distinct from the case where a blank match existed but the ratio was not computed.

#### Scenario: No blank loaded in the set
- **WHEN** a sample peak's group contains no blank cluster at all
- **THEN** the system marks the peak as Real Compound and records "no blank loaded in set" in the audit trail

#### Scenario: Blank present but clean at this RT
- **WHEN** the group contains blank clusters but none match this sample peak within blank tolerances
- **THEN** the system marks the peak as Real Compound and records "blank present but clean at this RT" in the audit trail

#### Scenario: Reporting distinguishes the cases
- **WHEN** the result is displayed or exported
- **THEN** "no blank loaded", "blank clean here", and "blank subtracted" peaks are each presented distinctly

#### Scenario: Present blank still drives subtraction
- **WHEN** a sample peak matches a blank cluster
- **THEN** the system computes the Signal-to-Blank ratio and area difference and classifies using the configured threshold as before
