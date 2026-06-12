## ADDED Requirements

### Requirement: Accepted input file formats
The system SHALL accept peak data in CSV, TSV, TXT, and XLSX (and legacy XLS) formats. Delimited text files (CSV/TSV/TXT) SHALL be parsed as tabular data; XLSX/XLS SHALL additionally be parsed for cell fill colors used as operator marks. The system SHALL select a parser by file extension and SHALL document the accepted extensions to the operator.

#### Scenario: CSV file is accepted
- **WHEN** the operator adds a file ending in `.csv`, `.tsv`, or `.txt`
- **THEN** the system parses it as delimited tabular peak data and includes its rows in the set

#### Scenario: XLSX file with color marks is accepted
- **WHEN** the operator adds a file ending in `.xlsx` or `.xls`
- **THEN** the system parses both the tabular data and the cell fill colors as operator marks

#### Scenario: Unknown extension falls back gracefully
- **WHEN** the operator adds a file whose extension matches no specific parser
- **THEN** the system attempts the delimited-text parser rather than rejecting the file outright

### Requirement: Per-instrument column-mapping profile
The system SHALL support onboarding a new instrument's export by mapping that instrument's column headers to the canonical fields (RT, Base Peak, Area, Polarity, File, Label, operator_mark). A column-mapping profile SHALL define, per source column, which canonical field it supplies; unmapped canonical fields that are optional (e.g. Base Peak, Polarity) SHALL be treated as absent rather than causing a parse failure.

#### Scenario: New instrument columns are mapped to canonical fields
- **WHEN** the operator selects a column-mapping profile for a new instrument and adds a file from that instrument
- **THEN** the system reads each canonical field from the mapped source column and produces canonical rows

#### Scenario: Required field missing from mapping is reported
- **WHEN** a column-mapping profile does not map a required canonical field (RT or Area)
- **THEN** the system reports a clear error identifying the missing required field

#### Scenario: Optional field absent from mapping is tolerated
- **WHEN** a column-mapping profile omits Base Peak and/or Polarity
- **THEN** the system parses successfully and marks those dimensions as unavailable for downstream processing

### Requirement: Sample-set formation in the UI
The system SHALL allow an operator to assemble a sample set from multiple input files in the interface, where each file contributes its rows to the combined set used for screening. The UI SHALL show which files compose the current set.

#### Scenario: Multiple files compose one set
- **WHEN** the operator adds two or more files to the workspace
- **THEN** the system combines their rows into one screening set and lists the contributing files
