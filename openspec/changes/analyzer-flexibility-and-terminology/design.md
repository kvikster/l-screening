## Context

The pipeline is: file → `screening.ts` parse (SheetJS) → canonical `Row` JSON → Rust/WASM core (`coarse_screen` → blank subtraction → `merge_parallel_samples`) → `ConfirmedRow[]` → Svelte UI. Current hard assumptions:

- `Row.polarity` is a required `String` (`types.rs`); `process_peaks_inner` groups on `(sample_type, polarity)`.
- `Row.base_peak` is already `Option<f64>`; `config.mz_available` already exists to waive the missing-m/z penalty (`config.rs`).
- `blank.rs::apply_blank_result`: when no blank match, `signal_to_blank_ratio = None`, `blank_area_mean = None`, status = `Real Compound` — but the "no blank at all" case is not labeled distinctly from "blank existed but no ratio".
- Format registry (`PARSERS` in `screening.ts`) keys only on extension; columns are read by fixed canonical header names in `parseWorkbook`.
- Surrogates (`SurrogateSpec`) already compute `SurrogateRtShift`; nothing feeds that back to `replicate_rt_tol`.
- UI labels CV% (`AreaCVPct`) and "ReplicateCount" literally.

Most of these items are small, additive changes plus UI/i18n labeling. Two require core changes (optional polarity grouping; explicit no-blank reporting).

## Goals / Non-Goals

**Goals:**
- Make polarity optional and support RT-only datasets without breaking existing per-polarity behavior.
- Report a clean (absent) blank as an explicit, positive outcome instead of an empty field.
- Add a column-mapping profile layer so a new instrument can be onboarded without code changes to header names.
- Surface surrogate RT spread as an advisory replicate-RT-tolerance suggestion in the surrogate's additional-data view.
- Relabel CV%→RSD and Replicate→"(chromatograms)", and highlight replicate count/origin — UI/i18n only, no numeric changes.

**Non-Goals:**
- Auto-applying the surrogate-derived bin width (advisory only).
- Changing how CV%/RSD is computed (label-only).
- Quantitative IS response-factor work (already tracked separately).
- A general visual column-mapping editor UI beyond selecting/declaring a profile (can be a follow-up).

## Decisions

**1. Polarity becomes optional via a dataset-level flag, mirroring `mz_available`.**
Make `Row.polarity` deserialize as `Option<String>` (default `None`). Add `polarity_available: bool` to `ScreeningConfig` (default true), set by `screening.ts` when the Polarity column is absent/empty for all rows. When false, group key uses a single sentinel polarity (e.g. `""`/`"n/a"`) so `coarse_screen`/summary code paths stay structurally identical. *Alternative considered:* infer per-row — rejected; a half-missing polarity column is ambiguous and a dataset-level flag matches the existing `mz_available` pattern.

**2. RT-only = `polarity_available=false` AND `mz_available=false`.** No new mode enum; the two existing flags compose. The missing-m/z penalty waiver already keys on `mz_available`, so RT-only confidence handling is already covered once polarity is optional.

**3. No-blank reporting is a distinct audit state, not a new status.** Keep status = `Real Compound`, but in `apply_blank_result` set an explicit `Why.BlankPresent=false` / `Why.NoBlankDetected=true` marker (today only `BlankMatch=false` is written, shared with "blank cluster existed but didn't match this peak"). Distinguish "no blank cluster anywhere in this group" (clean blank) from "blank clusters exist but none matched this peak." UI reads this marker to render "no blank detected" vs an empty S/B cell. *Alternative:* a new status string — rejected; would ripple through summary counts and downstream filters.

**4. Column-mapping profile is a TS-layer concern.** Add an optional `ColumnProfile` (source-header → canonical-field map) applied in `parseWorkbook` before building canonical rows; the WASM core keeps receiving canonical `Row` JSON unchanged. A profile may declare Base Peak / Polarity as absent → drives the `mz_available` / `polarity_available` flags. *Alternative:* push mapping into Rust — rejected; parsing/extraction already lives in TS and SheetJS gives us headers there.

**5. Surrogate bin-width suggestion is computed where surrogate stats already are and shown read-only.** Expose observed RT spread (e.g. max|RtShift| or stdev of replicate RTs) in the surrogate cluster's `Why`/additional-data, plus a suggested tolerance (e.g. spread rounded up). UI shows it next to the Analyzer Configuration RT-tolerance field with a copy affordance; no auto-write.

**6. Terminology/highlight changes are i18n + component-only.** RSD label (with "CV%" as parenthetical equivalent), "Replicates (chromatograms)", and emphasized replicate count + per-replicate origin all read from existing `ConfirmedRow` fields (`ReplicateFiles`, `ReplicateLabels`, `ParallelSourceSamples`). No core/data changes.

## Risks / Trade-offs

- **Sentinel-polarity grouping leaks into output** → Mitigation: render the sentinel as "n/a"/blank in UI and summary; add a regression test asserting single-group output for no-polarity input.
- **Half-populated Polarity column** (some rows have it, some don't) → Mitigation: treat dataset as polarity-available and let blank/empty rows fall into their own group as today; document that `polarity_available=false` requires the column fully absent/empty.
- **No-blank vs failed-match conflation** in existing consumers reading `BlankMatch` → Mitigation: keep `BlankMatch` as-is and add the new marker additively; update only the components that need the distinction.
- **Column-mapping mis-mapping produces silently wrong canonical fields** → Mitigation: validate required fields (RT, Area) present after mapping and error clearly; surface the resolved mapping to the operator.
- **Surrogate spread is a poor tolerance estimate with few replicates** → Mitigation: advisory-only and shown with the replicate count so the operator judges reliability.

## Migration Plan

1. Land core changes behind the new flags (`polarity_available`), defaulting to current behavior; existing datasets (polarity present) are unaffected.
2. Add no-blank audit marker additively; existing exports gain a field, none are removed.
3. Add `ColumnProfile` as opt-in; with no profile, parsing is unchanged.
4. Ship UI/i18n relabels last. Rollback = revert UI/i18n commit independently of core, since core changes are backward-compatible and inert without the new inputs.

## Resolved Decisions (from operator)

- **Surrogate bin width — show all three metrics.** The surrogate additional-data view presents max|RT shift|, k·stdev of replicate RTs, and range (max−min), each with the replicate count. No single auto-suggested value; the operator reads all three and decides what to enter into Analyzer Configuration.
- **Instrument profile — preset dropdown.** Column-mapping profiles are predefined presets selectable from a dropdown. Onboarding a new instrument = adding a preset in code. A full visual mapping editor is deferred to a follow-up.
- **No-blank — distinguish both cases.** Report "no blank loaded in the set at all" separately from "blank exists but is clean at this RT." This requires set-level context (whether any blank cluster exists in the group) passed into `apply_blank_result`, plus two distinct `Why` markers.

## Open Questions

- None outstanding for this change.
