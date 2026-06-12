## Why

The screening algorithm and its UI were built around a single Excel/CSV layout from one instrument that always provides Polarity, Base Peak (m/z), and a blank sample. Real lab workflows break these assumptions: a new instrument exports a different column layout, some methods have no Polarity or no m/z at all, and a clean run produces no blank peaks. At the same time several on-screen terms (CV%, "Replicate", cluster bin "width") are ambiguous to operators and don't reflect information the algorithm already has (e.g. surrogate RT behavior). This change makes the pipeline tolerant of these real-world inputs and makes the terminology and surrogate guidance self-explanatory.

## What Changes

- **Input formats & set formation**: Document and extend the file-format registry (currently CSV/TSV/TXT + XLSX in `screening.ts`) so a new instrument export can be onboarded via a column-mapping profile, and clarify how a sample set is assembled in the UI from multiple files.
- **Surrogate-informed cluster bin width**: When a surrogate's additional data is opened, surface its observed RT-shift behavior as a *suggested* replicate RT tolerance ("bin width"), which the operator can copy manually into Analyzer Configuration.
- **Optional Polarity / Base Peak**: Allow the algorithm to run when Polarity is absent, and when both Polarity and Base Peak are absent (RT-only). Polarity becomes optional rather than a required column.
- **Signal-to-Blank with no blank bin**: When no blank peak exists at all (blank was clean), compute/report S/B meaningfully (effectively "no blank detected") instead of silently leaving it empty, so a clean blank reads as a strong, not missing, result.
- **CV% ↔ RSD terminology**: Label CV% as RSD (relative standard deviation) where the two are the same quantity, so the displayed term matches lab vocabulary.
- **Replicate = chromatogram clarification**: Where "Replicate" appears, clarify in parentheses that 1 replicate = 1 chromatogram.
- **Highlight replicate provenance**: Visually highlight the replicate count and the origin of each replicate (which file/sample it came from) in the UI.

## Capabilities

### New Capabilities
- `input-formats`: Accepted input file formats (CSV/TSV/TXT/XLSX), per-instrument column-mapping profiles for onboarding new instruments, and how a sample set is formed from multiple files in the UI.
- `optional-dimensions`: Running the screening algorithm when Polarity and/or Base Peak (m/z) columns are absent, and how confidence/grouping degrade gracefully.
- `blank-subtraction-reporting`: Signal-to-Blank computation and reporting, including the case where no blank bin exists because the blank was clean.
- `surrogate-bin-guidance`: Deriving a suggested replicate RT tolerance ("cluster bin width") from observed surrogate RT behavior and exposing it in the surrogate's additional-data view for manual entry into Analyzer Configuration.
- `metric-terminology`: Consistent operator-facing terminology — CV% surfaced as RSD where equivalent, and "Replicate (= 1 chromatogram)" labeling.
- `replicate-provenance-display`: UI highlighting of replicate count and replicate origin (source file/sample) for each confirmed cluster.

### Modified Capabilities
<!-- None: no existing specs in openspec/specs/. All behavior above is captured as new capabilities. -->

## Impact

- **Rust/WASM core** (`app/src/lib/screening-wasm/src/`): `types.rs` (`Row.polarity` → optional), `config.rs` (`mz_available`, new RT-tolerance guidance fields), `blank.rs` (no-blank S/B reporting), `cluster.rs`/`lib.rs` (grouping without polarity/mz).
- **TS pipeline** (`app/src/lib/screening.ts`): format registry / column-mapping profiles, `ScreeningParams` flags.
- **UI** (`app/src/lib/components/*`, methodology pages, Analyzer Configuration panel): RSD/replicate labels, surrogate additional-data view, replicate-provenance highlighting, sample-set formation flow.
- **i18n** (`app/src/lib/i18n/messages/{en,uk,ru}.ts`): new/renamed strings for RSD, replicate-as-chromatogram, surrogate bin-width hint.
- **Tests** (`tests/test_logic.py`, `app/src/lib/screening.test.ts`): RT-only / no-polarity / no-blank regression cases.
