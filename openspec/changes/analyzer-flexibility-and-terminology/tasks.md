## 1. Optional Polarity & RT-only (core)

- [x] 1.1 In `types.rs`, change `Row.polarity` to `Option<String>` with `#[serde(default)]`; update `assign_sample_type` and all `.polarity` readers to handle `None`.
- [x] 1.2 In `config.rs`, add `polarity_available: bool` (default true) with `default_polarity_available()`; document it as a dataset-level flag mirroring `mz_available`.
- [x] 1.3 In `lib.rs::process_peaks_inner`, when `polarity_available=false`, use a single sentinel polarity for the `(sample_type, polarity)` group key so coarse_screen/summary paths are unchanged.
- [x] 1.4 In `screening.ts`, detect Polarity absent/empty for all rows and set `polarity_available=false`; add the flag to `ScreeningParams`.
- [x] 1.5 Confirm RT-only (`polarity_available=false` + `mz_available=false`) waives the missing-m/z penalty for all clusters and matches replicates/blanks on RT alone; add code only if a gap is found.
- [x] 1.6 Render sentinel polarity as "n/a"/blank in summary and UI rows.

## 2. Signal-to-Blank with no blank bin (core)

- [x] 2.1 Pass set/group-level blank context into `apply_blank_result` so it can tell "no blank cluster in group at all" from "blank clusters exist but none matched this peak."
- [x] 2.2 Write three distinct additive `Why` markers (e.g. `BlankState` = `no_blank_loaded` | `blank_clean_here` | `blank_subtracted`); keep `BlankMatch` unchanged; keep status = `Real Compound` for both no-blank cases.
- [x] 2.3 Ensure S/B reporting fields render the three states distinctly downstream rather than an empty value.

## 3. Input formats & column-mapping profiles (TS)

- [x] 3.1 Document accepted extensions (CSV/TSV/TXT/XLSX/XLS) in the file-add UI/help.
- [x] 3.2 Add a `ColumnProfile` type (source-header → canonical-field map) and apply it in `parseWorkbook` before building canonical rows.
- [x] 3.3 Validate required canonical fields (RT, Area) after mapping; error clearly when missing; treat omitted Base Peak/Polarity as absent → drive `mz_available`/`polarity_available`.
- [x] 3.4 Add a preset profile for the new instrument's export (its header names) and a preset dropdown in the UI to select it; surface the resolved mapping to the operator. (Visual mapping editor deferred to follow-up.)
- [x] 3.5 Clarify/confirm sample-set formation: UI lists the files composing the current set.

## 4. Surrogate-derived bin-width guidance

- [x] 4.1 Compute all three surrogate RT metrics — max|RtShift|, k·stdev of replicate RTs, range (max−min) — and attach them to the surrogate cluster's `Why`/additional data.
- [x] 4.2 In the surrogate additional-data view, show all three metrics, each with the replicate count (advisory only, read-only; no single auto-suggested value).
- [x] 4.3 Add a copy affordance next to the Analyzer Configuration RT-tolerance field; ensure a manually entered value is used as `replicate_rt_tol` on the next run.

## 5. Terminology & provenance UI (i18n + components)

- [x] 5.1 Relabel CV%/`AreaCVPct` as "RSD (CV%)" in operator-facing UI and tooltips across en/uk/ru; value unchanged.
- [x] 5.2 Relabel "Replicate"/ReplicateCount as "Replicates (chromatograms)" across en/uk/ru.
- [x] 5.3 Visually emphasize replicate count on confirmed clusters.
- [x] 5.4 Show per-replicate origin (source file/sample) from `ReplicateFiles`/`ReplicateLabels`; indicate when replicates span multiple parallel source samples (`ParallelSourceSamples`).

## 6. Tests & verification

- [x] 6.1 Add regression tests: no-polarity dataset → single group; RT-only dataset → results with no m/z penalty (`tests/test_logic.py`, `screening.test.ts`).
- [x] 6.2 Add tests for both no-blank states (`no_blank_loaded` vs `blank_clean_here`) → distinct markers, both Real Compound.
- [x] 6.3 Add test for column-mapping profile: new-instrument headers map to canonical fields; missing required field errors.
- [x] 6.4 Add test for surrogate three-metric bin-width computation (max|shift|, k·stdev, range).
- [x] 6.5 Run full Rust + TS test suites and the build; confirm green.
