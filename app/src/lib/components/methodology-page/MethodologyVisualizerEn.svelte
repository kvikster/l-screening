<!--
  @file MethodologyVisualizerEn.svelte
  @description
  English-language interactive visualization of the LC-MS screening pipeline.
  Two-panel layout: vertical stepper nav + viewport-fitted content card.
  Each step has two tabs: Overview (visual I/O) and Reference (detailed docs).
-->
<script lang="ts">
    import GlossaryRichText from "./GlossaryRichText.svelte";
    import MathFormula from "./MathFormula.svelte";
    import { fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { generateSampleData } from "$lib/screening";
    import { t } from "$lib/i18n";

    let { defs = {} }: { defs?: Record<string, string> } = $props();

    async function handleDownloadSample(format: "csv" | "xlsx") {
        try {
            await generateSampleData(format);
        } catch (e) {
            console.error(e);
        }
    }

    // ── Steps ────────────────────────────────────────────────────────────
    const steps = [
        {
            id: "glossary",
            title: "Glossary",
            short: "Key terms & symbols",
            tone: "slate",
            summary: "This step defines the key LC-MS terms used throughout the pipeline, including RT, m/z, Blank, Replicate, S/B, and Confidence score.",
            deepDive: "Use this as a shared vocabulary layer before moving into the pipeline itself. The same terms appear later as inline tooltips in both Overview and Reference tabs.",
            input: ["Core domain terms", "Formula symbols", "Decision abbreviations"],
            action: ["Review glossary cards", "Hover highlighted terms", "Align wording across all later steps"],
            output: ["Consistent interpretation of methodology terms"],
            formula: "Term -> Definition -> Consistent interpretation",
            formulaExplanation: "The glossary does not transform data. It standardizes the language used in later steps so formulas and decisions are interpreted consistently."
        },
        {
            id: "excel",
            title: "Data Input & Validation",
            short: "Excel reading & cleaning",
            tone: "slate",
            summary: "The workbook is read, and the system automatically selects the sheet containing the most required columns (RT, Area, m/z). All incomplete rows are discarded.",
            deepDive: "This step ensures data integrity. Rows without basic LC-MS parameters are ineligible for further calculation and are rejected before any scientific decisions are made.",
            input: ["Workbook (one or more sheets)", "Peak rows"],
            action: ["Select best-matching sheet", "Remove incomplete rows"],
            output: ["Valid rows for analysis"],
            formula: "Sheet = MaxMatch(Headers, Req)  AND  Valid = (RT>0 & Area>0)",
            formulaExplanation: "We search for the sheet where column names best match requirements. Only rows with basic numeric data are kept."
        },
        {
            id: "classify",
            title: "Role Assignment",
            short: "Sample / Blank / Rep",
            tone: "violet",
            summary: "Each row is assigned a sample type (Sample or Blank) and grouped into a specific replicate bucket. Subsequent calculations happen per polarity.",
            deepDive: "If the operator colored cells in Excel, the system trusts this color. If no color is present, it falls back to parsing the row's filename.",
            input: ["Valid rows", "Excel cell colors", "File names"],
            action: ["Read operator marks", "Apply filename logic (fallback)"],
            output: ["Grouping by (SampleType, Polarity)"],
            formula: "Role = ColorMap[CellColor] || FileNameLogic(Name)",
            formulaExplanation: "Operator colors have the highest priority. If missing, the system analyzes the text name for patterns."
        },
        {
            id: "replicates",
            title: "Replicate Clustering",
            short: "Peak confirmation",
            tone: "blue",
            summary: "Peaks from the same sample but different measurements (replicates) are grouped. We verify the same compound is present in multiple measurements.",
            deepDive: "Greedy clustering: the peak with the largest area becomes the seed. From other replicate buckets, the closest peak in (RT + m/z) is selected. If matches are found in \u2265 2 buckets, the cluster is confirmed. The algorithm also supports RT-only mode (GC-FID, LC-UV, and other non-MS instruments): if the dataset has no \u2018Base Peak\u2019 column, clustering and blank subtraction run on RT alone, and no confidence penalty is applied for missing m/z.",
            input: ["Peaks in replicate buckets"],
            action: ["Sort by descending Area", "Average centroid upon addition", "Verify tolerance windows"],
            output: ["Confirmed clusters"],
            formula: "|RT_cand \u2212 RT_cent| \u2264 Tol_RT  AND  \u0394m/z \u2264 Tol_m/z",
            formulaExplanation: "To merge peaks from different files, their retention time and mass must be nearly identical (within configured tolerances)."
        },
        {
            id: "blank",
            title: "Blank Subtraction",
            short: "Background comparison",
            tone: "cyan",
            summary: "Blanks undergo independent clustering. Each confirmed sample is matched against the nearest blank signal to assess background noise.",
            deepDive: "We look for the nearest (RT + m/z) cluster in the blank. In case of ties, the larger blank is chosen (pessimistic approach) to ensure robust background filtering.",
            input: ["Sample clusters", "Blank clusters"],
            action: ["Find nearest blank cluster (same polarity)", "Calculate S/B ratio"],
            output: ["Sample \u2194 Blank pairs with ratios"],
            formula: "Ratio (S/B) = mean(Area_sample) / mean(Area_blank)",
            formulaExplanation: "Average area in the sample divided by average area in the blank. Averaging protects against accidental spikes."
        },
        {
            id: "parallel_merge",
            title: "Parallel Sample Merge",
            short: "sample_1 ∩ sample_2",
            tone: "indigo",
            summary: "Confirmed sample clusters from different parallel samples (sample_1, sample_2 …) are merged into a single row. Each source sample already has its own blank subtraction status before merging.",
            deepDive: "The merge is weighted by replicate count. If only one sample had a blank match, blank_area_mean is aggregated only from that source. The S/B ratio and status are re-evaluated at the aggregated level, so a noisy sample does not corrupt the final conclusion.",
            input: ["Per-sample clusters with blank subtraction status"],
            action: ["Greedy clustering across samples (RT + m/z)", "Weighted averaging (area, RT, mz)", "Aggregate blank_area_mean from sources with a match", "Re-calculate S/B and confidence_score"],
            output: ["Merged ConfirmedRow with Why.BlankSubtraction.PerSource[]"],
            formula: "S/B_merged = Area_merged / blank_area_mean_weighted",
            formulaExplanation: "The aggregated blank_area_mean is computed as a weighted mean only from sources that had a blank match. Sources without a match do not dilute the denominator or mask a real signal."
        },
        {
            id: "decision",
            title: "Classification",
            short: "Artifact / Real Compound",
            tone: "green",
            summary: "If a matching blank is found and the sample signal doesn't exceed the background sufficiently (S/B threshold), it is rejected as an artifact.",
            deepDive: "This binary split is the key business outcome. It allows the chemist to focus only on real, clean compounds while discarding contamination.",
            input: ["Signal-to-Blank (S/B) ratio", "Decision threshold"],
            action: ["Compare S/B ratio against threshold"],
            output: ["Decision: Artifact or Real Compound"],
            formula: "Status = (Ratio < Threshold) ? \"Artifact\" : \"Real Compound\"",
            formulaExplanation: "If the sample signal doesn't stand out strongly enough against the blank, we classify it as an Artifact."
        },
        {
            id: "output",
            title: "Summary & Audit Trail",
            short: "Regulatory trail",
            tone: "rose",
            summary: "Every step leading to the final conclusion, along with summary statistical metrics, is saved for regulatory control.",
            deepDive: "The detailed decision log is stored as JSON. Mean RT, m/z, area, variability (CV%), and the confidence scoring bonuses/penalties are computed.",
            input: ["Confirmed clusters with status"],
            action: ["Compute CV% and averages", "Calculate confidence score", "Serialize decision trail"],
            output: ["Summary table (Audit-ready Excel)"],
            formula: "Score = 100 \u2212 \u03a3(Penalties) + Bonus",
            formulaExplanation: "A reliability rating from 0 to 100. Deductions for mass/RT shifts and replicate variance; bonus for perfect blank separation."
        }
    ];

    // ── Reference data (tables, lists) ──────────────────────────────────
    const columns = [
        { col: "RT",        type: "number", desc: "Retention time of the chromatographic peak",              ex: "2.345" },
        { col: "Base Peak", type: "number", desc: "(Optional) m/z of the dominant ion in the mass spectrum. Absence of this column activates RT-only mode (GC-FID, LC-UV).", ex: "195.08" },
        { col: "Polarity",  type: "string", desc: "Ionization polarity: positive / negative",                ex: "positive" },
        { col: "File",      type: "string", desc: "Source file name used to assign replicate buckets",       ex: "1_pos.d" },
        { col: "Area",      type: "number", desc: "Peak area proportional to analyte abundance",             ex: "1250000" },
        { col: "Label",     type: "string", desc: "(Optional) operator label or compound name",              ex: "Caffeine" },
    ];

    const operatorMarks = [
        { name: "sample_rep1",     color: "#ff00ff", label: "Sample, Replicate 1" },
        { name: "sample_rep2",     color: "#ffff00", label: "Sample, Replicate 2" },
        { name: "blank_positive",  color: "#00ffff", label: "Blank" },
        { name: "blank_negative",  color: "#00ff00", label: "Blank" },
    ];

    const outputFields = [
        { field: "RT_mean",            desc: "Mean RT of the confirmed cluster (across all replicates)." },
        { field: "MZ_mean",            desc: "Mean m/z of the confirmed cluster." },
        { field: "Area_mean",          desc: "Mean peak area (no integer truncation)." },
        { field: "AreaCVPct",          desc: "CV% across replicate areas of the cluster (sample std)." },
        { field: "ReplicateQuality",   desc: "High (CV% \u2264 15) / Moderate (\u2264 30) / Low (> 30) \u2014 reproducibility band." },
        { field: "SignalToBlankRatio", desc: "S/B ratio for the closest confirmed blank peak." },
        { field: "ConfidenceScore",    desc: "Final 0\u2013100 confidence score after blank subtraction." },
        { field: "Status",             desc: "Real Compound or Artifact classification result." },
        { field: "Why",                desc: "JSON object with the full decision audit trail (RT/mz deltas, CV, S/B, thresholds)." },
    ];

    const params = [
        { name: "replicate_rt_tol",   def: "0.1",  unit: "min",     used: "Replicate clustering" },
        { name: "replicate_mz_tol",   def: "0.3",  unit: "Da / ppm", used: "Replicate clustering" },
        { name: "blank_rt_tol",       def: "0.1",  unit: "min",     used: "Blank subtraction" },
        { name: "blank_mz_tol",       def: "0.3",  unit: "Da / ppm", used: "Blank subtraction" },
        { name: "signal_to_blank_min",   def: "3.0",  unit: "ratio",   used: "Artifact / Real Compound decision" },
        { name: "min_area_difference",    def: "—",    unit: "counts",  used: "(Optional) absolute AreaDiff floor; Artifact if area_sample−area_blank < threshold" },
        { name: "cv_high_max",        def: "15",   unit: "%",       used: "ReplicateQuality = High" },
        { name: "cv_moderate_max",    def: "30",   unit: "%",       used: "ReplicateQuality = Moderate" },
        { name: "mz_available",       def: "true", unit: "bool",    used: "Auto-detected: false for RT-only datasets (no Base Peak column)" },
    ];

    const glossary = [
        { term: "RT",               def: "Retention Time \u2014 chromatographic retention time of an analyte in the column, measured in minutes." },
        { term: "m/z",              def: "Mass-to-charge ratio \u2014 the central coordinate of a mass-spectrometric signal." },
        { term: "CV%",              def: "Coefficient of Variation \u2014 relative variability between replicate peak areas. CV% = (sample std / mean) \u00d7 100. For n=2: std = |v\u2081\u2212v\u2082|/\u221a2. Lower CV = better reproducibility." },
        { term: "S/B",              def: "Signal-to-Blank ratio \u2014 sample peak area divided by the area of the closest confirmed blank peak." },
        { term: "Blank",            def: "A blank sample (solvent only, no analyte) used to identify background artifacts and contamination." },
        { term: "Replicate",        def: "An independent repeat measurement of the same sample. At least 2 replicate buckets are required to confirm a peak." },
        { term: "Confidence score", def: "A 0\u2013100 score starting at 100, penalised by RT/m/z deviation, CV%, and S/B ratio." },
        { term: "ppm",              def: "Parts per million \u2014 a relative m/z tolerance for high-resolution instruments." },
        { term: "Da",               def: "Dalton \u2014 an absolute m/z tolerance in daltons." },
        { term: "Replicate bucket", def: "All peaks from one file or one explicit operator mark. Each file/mark forms one bucket." },
    ];

    let resolvedGlossary = $derived({
        ...Object.fromEntries(glossary.map(({ term, def }) => [term, def])),
        ...defs
    });

    let glossaryDefinitions = $derived({
        ...resolvedGlossary,
        blank: resolvedGlossary.Blank,
        blanks: resolvedGlossary.Blank,
        "blank sample": resolvedGlossary.Blank,
        replicate: resolvedGlossary.Replicate,
        replicates: resolvedGlossary.Replicate,
        "replicate bucket": resolvedGlossary["Replicate bucket"],
        "replicate buckets": resolvedGlossary["Replicate bucket"],
        "confidence score": resolvedGlossary["Confidence score"],
        "S/B ratio": resolvedGlossary["S/B"],
        "signal-to-blank": resolvedGlossary["S/B"],
        "signal-to-blank ratio": resolvedGlossary["S/B"]
    });

    const formulaMarkupById: Record<string, string> = {
        glossary: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mtext>Term</mtext><mspace width="0.4em"/><mo>→</mo><mspace width="0.4em"/><mtext>Definition</mtext><mspace width="0.4em"/><mo>→</mo><mspace width="0.4em"/><mtext>Consistent interpretation</mtext></mrow></math>',
        excel: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mi>Sheet</mi><mo>=</mo><mi>MaxMatch</mi><mo>(</mo><mi>Headers</mi><mo>,</mo><mi>Req</mi><mo>)</mo></mtd></mtr><mtr><mtd><mi>Valid</mi><mo>=</mo><mo>(</mo><mi>RT</mi><mo>&gt;</mo><mn>0</mn><mspace width="0.3em"/><mo>∧</mo><mspace width="0.3em"/><mi>Area</mi><mo>&gt;</mo><mn>0</mn><mo>)</mo></mtd></mtr></mtable></math>',
        classify: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Role</mi><mo>=</mo><mi>ColorMap</mi><mo>(</mo><mi>CellColor</mi><mo>)</mo><mspace width="0.6em"/><mo>∨</mo><mspace width="0.6em"/><mi>FileNameLogic</mi><mo>(</mo><mi>Name</mi><mo>)</mo></mrow></math>',
        replicates: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mo>|</mo><msub><mi>RT</mi><mtext>cand</mtext></msub><mo>−</mo><msub><mi>RT</mi><mtext>cent</mtext></msub><mo>|</mo><mo>≤</mo><msub><mi>Tol</mi><mtext>RT</mtext></msub></mtd></mtr><mtr><mtd><mi>Δm/z</mi><mo>≤</mo><msub><mi>Tol</mi><mtext>m/z</mtext></msub></mtd></mtr></mtable></math>',
        blank: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mtable columnalign="left"><mtr><mtd><mi>Ratio</mi><mo>(</mo><mi>S</mi><mo>/</mo><mi>B</mi><mo>)</mo><mo>=</mo></mtd></mtr><mtr><mtd><mfrac><mrow><mi>mean</mi><mo>(</mo><msub><mi>Area</mi><mtext>sample</mtext></msub><mo>)</mo></mrow><mrow><mi>mean</mi><mo>(</mo><msub><mi>Area</mi><mtext>blank</mtext></msub><mo>)</mo></mrow></mfrac></mtd></mtr></mtable></math>',
        decision: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Status</mi><mo>=</mo><mrow><mo>{</mo><mtable columnalign="left left" rowspacing="0.2em"><mtr><mtd><mtext>Artifact,</mtext></mtd><mtd><mtext>if Ratio &lt; Threshold</mtext></mtd></mtr><mtr><mtd><mtext>Real Compound,</mtext></mtd><mtd><mtext>otherwise</mtext></mtd></mtr></mtable></mrow></mrow></math>',
        output: '<math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi>Score</mi><mo>=</mo><mn>100</mn><mo>−</mo><mo>∑</mo><mo>(</mo><mi>Penalties</mi><mo>)</mo><mo>+</mo><mi>Bonus</mi></mrow></math>',
    };

    // ── State ────────────────────────────────────────────────────────────
    let current = $state(0);
    let tab: 'overview' | 'reference' = $state('overview');
    let activeStep = $derived(steps[current]);
    let activeFormulaMarkup = $derived(formulaMarkupById[activeStep.id] ?? '');
    let contentEl: HTMLDivElement | undefined = $state();

    let lastScrollTime = 0;
    const scrollThrottle = 400;
    const scrollEdgeThreshold = 2; // px tolerance for "at boundary" checks

    function handleWheel(e: WheelEvent) {
        if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;
        if (Math.abs(e.deltaY) <= 10) return;

        // If the content area is scrollable, only navigate steps at scroll boundaries
        if (contentEl) {
            const { scrollTop, scrollHeight, clientHeight } = contentEl;
            const isScrollable = scrollHeight > clientHeight + scrollEdgeThreshold;
            const atTop = scrollTop <= scrollEdgeThreshold;
            const atBottom = scrollTop + clientHeight >= scrollHeight - scrollEdgeThreshold;

            if (isScrollable) {
                if (e.deltaY > 0 && !atBottom) return;  // let content scroll down
                if (e.deltaY < 0 && !atTop) return;     // let content scroll up
            }
        }

        const now = Date.now();
        if (now - lastScrollTime < scrollThrottle) return;

        if (e.deltaY > 0 && current < steps.length - 1) {
            current++;
            lastScrollTime = now;
            e.preventDefault();
            if (contentEl) contentEl.scrollTop = 0;
        } else if (e.deltaY < 0 && current > 0) {
            current--;
            lastScrollTime = now;
            e.preventDefault();
            if (contentEl) contentEl.scrollTop = 0;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
            if (current < steps.length - 1) { current++; e.preventDefault(); scrollContentToTop(); }
        } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
            if (current > 0) { current--; e.preventDefault(); scrollContentToTop(); }
        }
    }

    function scrollContentToTop() {
        if (contentEl) contentEl.scrollTop = 0;
    }

    const toneBadge: Record<string, string> = {
        slate:  "border-slate-300 bg-slate-50 text-slate-700 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-300",
        violet: "border-violet-300 bg-violet-50 text-violet-700 dark:border-violet-700 dark:bg-violet-900/30 dark:text-violet-300",
        blue:   "border-blue-300 bg-blue-50 text-blue-700 dark:border-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
        cyan:   "border-cyan-300 bg-cyan-50 text-cyan-700 dark:border-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-300",
        green:  "border-emerald-300 bg-emerald-50 text-emerald-700 dark:border-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
        rose:   "border-rose-300 bg-rose-50 text-rose-700 dark:border-rose-700 dark:bg-rose-900/30 dark:text-rose-300",
    };

    const toneDot: Record<string, string> = {
        slate:  "border-slate-400 bg-slate-400",
        violet: "border-violet-500 bg-violet-500",
        blue:   "border-blue-500 bg-blue-500",
        cyan:   "border-cyan-500 bg-cyan-500",
        green:  "border-emerald-500 bg-emerald-500",
        rose:   "border-rose-500 bg-rose-500",
    };
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    onwheel={handleWheel}
    onkeydown={handleKeydown}
    role="region"
    aria-label="Methodology visualizer"
    tabindex="0"
    class="flex h-[calc(100dvh-10rem)] min-h-[520px] max-h-[800px] overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-xl shadow-slate-200/40 focus:outline-none dark:border-slate-700/60 dark:bg-slate-900/80 dark:shadow-none"
>
    <!-- ================================================================= -->
    <!-- LEFT: Vertical Stepper Nav                                        -->
    <!-- ================================================================= -->
    <nav class="flex w-52 shrink-0 flex-col border-r border-slate-100 bg-slate-50/60 dark:border-slate-800 dark:bg-slate-900/60">
        <div class="px-4 pt-5 pb-2">
            <h2 class="text-[10px] font-bold uppercase tracking-[0.15em] text-slate-400 dark:text-slate-500">Pipeline</h2>
        </div>

        <div class="flex-1 overflow-y-auto px-4 pb-4">
            <div class="relative">
                <div class="absolute left-[11px] top-[14px] bottom-[14px] w-px bg-slate-200 dark:bg-slate-700"></div>
                <div
                    class="absolute left-[11px] top-[14px] w-px bg-blue-500 transition-all duration-500 ease-out"
                    style="height: {steps.length > 1 ? (current / (steps.length - 1)) * 100 : 0}%"
                ></div>

                {#each steps as st, i}
                    <button
                        onclick={() => (current = i)}
                        class="group relative flex w-full items-start gap-3 py-2.5 text-left transition-colors"
                    >
                        <div class="relative z-10 flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded-full border-2 transition-all duration-300
                            {current === i
                                ? `${toneDot[st.tone] ?? 'border-blue-500 bg-blue-500'} text-white shadow-md scale-110`
                                : i < current
                                    ? 'border-blue-400 bg-blue-50 text-blue-600 dark:border-blue-500 dark:bg-blue-900/40 dark:text-blue-400'
                                    : 'border-slate-300 bg-white text-slate-400 group-hover:border-slate-400 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-500'
                            }">
                            <span class="text-[9px] font-bold">{i}</span>
                        </div>
                        <div class="min-w-0 pt-px">
                            <p class="truncate text-[11px] font-semibold leading-tight transition-colors
                                {current === i ? 'text-slate-900 dark:text-white' : 'text-slate-500 dark:text-slate-400 group-hover:text-slate-700 dark:group-hover:text-slate-300'}">
                                <GlossaryRichText text={st.title} definitions={glossaryDefinitions} />
                            </p>
                            <p class="mt-0.5 truncate text-[10px] text-slate-400 dark:text-slate-500">{st.short}</p>
                        </div>
                    </button>
                {/each}
            </div>
        </div>

        <div class="shrink-0 border-t border-slate-100 px-4 py-2.5 dark:border-slate-800">
            <p class="text-center text-[10px] text-slate-400 dark:text-slate-500">
                <kbd class="rounded border border-slate-200 bg-white px-1 py-0.5 font-mono text-[9px] dark:border-slate-700 dark:bg-slate-800">&uarr;</kbd>
                <kbd class="rounded border border-slate-200 bg-white px-1 py-0.5 font-mono text-[9px] dark:border-slate-700 dark:bg-slate-800">&darr;</kbd>
                or scroll
            </p>
        </div>
    </nav>

    <!-- ================================================================= -->
    <!-- RIGHT: Content Panel                                              -->
    <!-- ================================================================= -->
    <div class="flex flex-1 flex-col min-w-0">
        <!-- Progress bar -->
        <div class="h-1 shrink-0 bg-slate-100 dark:bg-slate-800">
            <div class="h-full bg-gradient-to-r from-blue-500 to-indigo-500 transition-all duration-500 ease-out" style="width: {steps.length > 1 ? (current / (steps.length - 1)) * 100 : 0}%"></div>
        </div>

        <!-- Tab switcher -->
        <div class="shrink-0 flex items-center gap-1 border-b border-slate-100 px-5 pt-3 pb-0 dark:border-slate-800">
            <button
                onclick={() => (tab = 'overview')}
                class="rounded-t-lg px-3 py-1.5 text-xs font-semibold transition-colors
                    {tab === 'overview'
                        ? 'border-b-2 border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300'}"
            >
                Overview
            </button>
            <button
                onclick={() => (tab = 'reference')}
                class="rounded-t-lg px-3 py-1.5 text-xs font-semibold transition-colors
                    {tab === 'reference'
                        ? 'border-b-2 border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300'}"
            >
                Reference
            </button>
        </div>

        <!-- Content area -->
        <div bind:this={contentEl} class="flex-1 overflow-y-auto p-5 lg:p-6">
            {#key `${current}-${tab}`}
            <div in:fade={{ duration: 200, easing: cubicOut }}>

                <!-- Header (shared) -->
                <div class="mb-3 flex items-center gap-3">
                    <span class="inline-flex items-center rounded-full border px-2.5 py-0.5 font-mono text-[10px] font-bold uppercase tracking-widest {toneBadge[activeStep.tone] ?? toneBadge.slate}">
                        {activeStep.id}
                    </span>
                    <h3 class="text-lg font-extrabold tracking-tight text-slate-900 lg:text-xl dark:text-white">
                        <GlossaryRichText text={activeStep.title} definitions={glossaryDefinitions} />
                    </h3>
                </div>

                <!-- ====================================================== -->
                <!-- TAB: Overview                                           -->
                <!-- ====================================================== -->
                {#if tab === 'overview'}
                    <p class="mb-4 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
                        <GlossaryRichText text={activeStep.summary} definitions={glossaryDefinitions} />
                    </p>

                    <!-- I/O Grid -->
                    <div class="mb-4 grid grid-cols-3 gap-2.5">
                        <div class="rounded-xl border border-slate-100 bg-slate-50/60 p-3 dark:border-slate-800 dark:bg-slate-800/30">
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">Input</p>
                            <ul class="space-y-1">
                                {#each activeStep.input as item}
                                    <li class="flex items-start gap-1.5 text-[11px] leading-snug text-slate-600 dark:text-slate-300">
                                        <span class="mt-[5px] h-1 w-1 shrink-0 rounded-full bg-slate-300 dark:bg-slate-600"></span>
                                        <GlossaryRichText text={item} definitions={glossaryDefinitions} />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                        <div class="rounded-xl border-l-2 border-blue-400 bg-blue-50/40 p-3 dark:bg-blue-900/10">
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-blue-500 dark:text-blue-400">Process</p>
                            <ul class="space-y-1">
                                {#each activeStep.action as item}
                                    <li class="flex items-start gap-1.5 text-[11px] font-medium leading-snug text-slate-700 dark:text-slate-200">
                                        <svg class="mt-0.5 h-3 w-3 shrink-0 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"><path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"></path></svg>
                                        <GlossaryRichText text={item} definitions={glossaryDefinitions} />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                        <div class="rounded-xl border border-emerald-100 bg-emerald-50/40 p-3 dark:border-emerald-900/30 dark:bg-emerald-900/10">
                            <p class="mb-1.5 text-[9px] font-bold uppercase tracking-widest text-emerald-500 dark:text-emerald-400">Output</p>
                            <ul class="space-y-1">
                                {#each activeStep.output as item}
                                    <li class="flex items-start gap-1.5 text-[11px] leading-snug text-slate-600 dark:text-slate-300">
                                        <svg class="mt-0.5 h-3 w-3 shrink-0 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"></path></svg>
                                        <GlossaryRichText text={item} definitions={glossaryDefinitions} />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    </div>

                    <!-- Detail + Math -->
                    <div class="grid gap-2.5 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
                        <div class="rounded-xl border border-slate-100 bg-slate-50/50 p-3.5 dark:border-slate-800 dark:bg-slate-800/20">
                            <h4 class="mb-1 flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                                <svg class="h-3.5 w-3.5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                                Detail
                            </h4>
                            <p class="text-[11px] leading-[1.35rem] text-slate-600 dark:text-slate-400"><GlossaryRichText text={activeStep.deepDive} definitions={glossaryDefinitions} /></p>
                        </div>
                        <div class="rounded-xl border border-indigo-100 bg-indigo-50/30 p-4 dark:border-indigo-900/30 dark:bg-indigo-900/10">
                            <h4 class="mb-1.5 text-[10px] font-bold uppercase tracking-wider text-indigo-600 dark:text-indigo-400">Math & Logic</h4>
                            <div class="mb-1.5">
                                <MathFormula markup={activeFormulaMarkup} label={activeStep.formula} />
                            </div>
                            <p class="text-xs leading-5 text-slate-600 dark:text-slate-400"><GlossaryRichText text={activeStep.formulaExplanation} definitions={glossaryDefinitions} /></p>
                        </div>
                    </div>

                <!-- ====================================================== -->
                <!-- TAB: Reference                                          -->
                <!-- ====================================================== -->
                {:else}
                    <div class="space-y-5 text-xs leading-relaxed text-slate-600 dark:text-slate-400">

                        {#if activeStep.id === 'glossary'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Shared terminology</h4>
                                <p><GlossaryRichText text="This glossary is step 0 because the later pipeline uses these same terms in summaries, rules, formulas, and audit output." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div class="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                                {#each glossary as g}
                                    <div class="rounded-lg border border-slate-100 bg-slate-50/50 p-2.5 dark:border-slate-800 dark:bg-slate-800/20">
                                        <span class="font-mono font-semibold text-blue-700 dark:text-blue-400">{g.term}</span>
                                        <p class="mt-0.5 text-[10px] leading-snug text-slate-500 dark:text-slate-400"><GlossaryRichText text={g.def} definitions={glossaryDefinitions} /></p>
                                    </div>
                                {/each}
                            </div>

                        <!-- Step 1: Input & Validation -->
                        {:else if activeStep.id === 'excel'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Input data</h4>
                                <p><GlossaryRichText text="The system accepts Excel (.xlsx / .xls) or CSV/TSV/TXT files with LC-MS peak data, automatically selects the most suitable sheet (for Excel) or parses the first sheet (for CSV), and expects at least two replicates (separate files or operator marks) plus a blank sample." definitions={glossaryDefinitions} /></p>
                                <div class="mt-3 flex gap-2">
                                    <button onclick={() => handleDownloadSample("csv")} class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700">
                                        {t("downloadSampleCsv")}
                                    </button>
                                    <button onclick={() => handleDownloadSample("xlsx")} class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700">
                                        {t("downloadSampleXlsx")}
                                    </button>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Required Excel columns</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Column</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Type</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Description</th>
                                                <th class="px-3 py-2 text-left font-semibold text-slate-700 dark:text-slate-300">Example</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each columns as c}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{c.col}</td>
                                                    <td class="px-3 py-1.5 text-slate-500 dark:text-slate-400">{c.type}</td>
                                                    <td class="px-3 py-1.5"><GlossaryRichText text={c.desc} definitions={glossaryDefinitions} /></td>
                                                    <td class="px-3 py-1.5 font-mono text-slate-500 dark:text-slate-400">{c.ex}</td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 2: Role Assignment -->
                        {:else if activeStep.id === 'classify'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Operator marks</h4>
                                <p class="mb-3"><GlossaryRichText text='Cell color marks take priority over the file name. Without marks, type is inferred from the filename: files containing "blank" become blank; files like 1_*.d, 2_*.d become sample_1, sample_2, etc.' definitions={glossaryDefinitions} /></p>
                                <div class="grid grid-cols-2 gap-2">
                                    {#each operatorMarks as m}
                                        <div class="flex items-center gap-2.5 rounded-lg border border-slate-200 bg-slate-50 p-2.5 dark:border-slate-700 dark:bg-slate-800">
                                            <span class="h-4 w-4 shrink-0 rounded-full border border-black/10 dark:border-white/10" style="background:{m.color}"></span>
                                            <div>
                                                <span class="font-mono font-semibold text-slate-900 dark:text-slate-100">{m.name}</span>
                                                <span class="ml-1.5 text-slate-400">{m.label}</span>
                                            </div>
                                        </div>
                                    {/each}
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Grouping</h4>
                                <p><GlossaryRichText text="After assignment, rows are grouped by (SampleType, Polarity). Each group is processed independently through clustering and blank subtraction." definitions={glossaryDefinitions} /></p>
                            </div>

                        <!-- Step 3: Replicate Clustering -->
                        {:else if activeStep.id === 'replicates'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Greedy replicate clustering</h4>
                                <p><GlossaryRichText text="Peaks are sorted by area descending. Each peak seeds a cluster in turn. From every other bucket the closest unused peak to the current cluster centroid (RT, m/z averaged as members are added) is greedily selected. A cluster is confirmed when it spans 2 or more different buckets." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Inclusion rule</h4>
                                <div class="rounded-lg border border-blue-100 bg-blue-50/50 p-3 font-mono text-[11px] dark:border-blue-900/40 dark:bg-blue-900/10">
                                    <p>|RT_candidate &minus; RT_centroid| &le; replicate_rt_tol</p>
                                    <p>|mz_candidate &minus; mz_centroid| &le; replicate_mz_tol (Da or ppm)</p>
                                </div>
                                <p class="mt-2"><GlossaryRichText text="Ties are broken by smallest distance (RT fraction + mz fraction), then largest area." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Parameters</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Parameter</th>
                                                <th class="px-3 py-2 text-left font-semibold">Default</th>
                                                <th class="px-3 py-2 text-left font-semibold">Unit</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params.filter(p => p.used === 'Replicate clustering') as p}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                                                    <td class="px-3 py-1.5">{p.def}</td>
                                                    <td class="px-3 py-1.5 text-slate-500">{p.unit}</td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 4: Blank Subtraction -->
                        {:else if activeStep.id === 'blank'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Blank subtraction</h4>
                                <p><GlossaryRichText text="The blank undergoes the same clustering independently. Each confirmed sample peak is then matched against confirmed blank peaks within blank_rt_tol / blank_mz_tol. The closest match is selected by RT+m/z distance, tie-broken by larger blank area." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Classification rules</h4>
                                <div class="space-y-1.5">
                                    <div class="flex items-start gap-2 rounded-lg border border-rose-100 bg-rose-50/50 p-2.5 dark:border-rose-900/30 dark:bg-rose-900/10">
                                        <span class="mt-0.5 font-mono font-bold text-rose-600 dark:text-rose-400">Artifact</span>
                                        <span><GlossaryRichText text={"blank match found and S/B < signal_to_blank_min (or blank area = 0)"} definitions={glossaryDefinitions} /></span>
                                    </div>
                                    <div class="flex items-start gap-2 rounded-lg border border-emerald-100 bg-emerald-50/50 p-2.5 dark:border-emerald-900/30 dark:bg-emerald-900/10">
                                        <span class="mt-0.5 font-mono font-bold text-emerald-600 dark:text-emerald-400">Real Compound</span>
                                        <span><GlossaryRichText text={"no blank match, or S/B >= threshold"} definitions={glossaryDefinitions} /></span>
                                    </div>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Parameters</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Parameter</th>
                                                <th class="px-3 py-2 text-left font-semibold">Default</th>
                                                <th class="px-3 py-2 text-left font-semibold">Unit</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params.filter(p => p.used.includes('Blank') || p.used.includes('Artifact')) as p}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                                                    <td class="px-3 py-1.5">{p.def}</td>
                                                    <td class="px-3 py-1.5 text-slate-500">{p.unit}</td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 5: Classification / Confidence Score -->
                        {:else if activeStep.id === 'decision'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Confidence Score formula</h4>
                                <p><GlossaryRichText text="Starts at 100. Penalties are applied in sequence; result is clamped to [0, 100]." definitions={glossaryDefinitions} /></p>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Stage 1 &mdash; Replicates</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Factor</th>
                                                <th class="px-3 py-2 text-left font-semibold">Penalty</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            <tr><td class="px-3 py-1.5">RT proximity</td><td class="px-3 py-1.5 font-mono">&minus;(mean_RT_delta / rt_tol) &times; 20, max &minus;20</td></tr>
                                            <tr><td class="px-3 py-1.5">m/z proximity</td><td class="px-3 py-1.5 font-mono">&minus;(mean_mz_delta / mz_tol) &times; 25, max &minus;25</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% High</td><td class="px-3 py-1.5 font-mono">0</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% Moderate</td><td class="px-3 py-1.5 font-mono">&minus;12</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% Low</td><td class="px-3 py-1.5 font-mono">&minus;(12 + (CV &minus; cv_moderate_max) &times; 0.7), max &minus;35</td></tr>
                                            <tr><td class="px-3 py-1.5">CV% missing</td><td class="px-3 py-1.5 font-mono">&minus;10</td></tr>
                                            <tr><td class="px-3 py-1.5">Not colour-paired</td><td class="px-3 py-1.5 font-mono">&minus;5</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Stage 2 &mdash; Blank</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Condition</th>
                                                <th class="px-3 py-2 text-left font-semibold">Adjustment</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            <tr><td class="px-3 py-1.5">No blank match</td><td class="px-3 py-1.5 font-mono text-emerald-600 dark:text-emerald-400">+3</td></tr>
                                            <tr><td class="px-3 py-1.5">S/B &ge; threshold</td><td class="px-3 py-1.5 font-mono text-emerald-600 dark:text-emerald-400">+min(5, (S/B &minus; threshold) &times; 0.5)</td></tr>
                                            <tr><td class="px-3 py-1.5">S/B &lt; threshold (Artifact)</td><td class="px-3 py-1.5 font-mono text-rose-600 dark:text-rose-400">&minus;(15 + deficit &times; 30), max &minus;45</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                        <!-- Step 6: Output & Audit -->
                        {:else if activeStep.id === 'output'}
                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">Output fields</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Field</th>
                                                <th class="px-3 py-2 text-left font-semibold">Description</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each outputFields as f}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400 whitespace-nowrap">{f.field}</td>
                                                    <td class="px-3 py-1.5"><GlossaryRichText text={f.desc} definitions={glossaryDefinitions} /></td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div>
                                <h4 class="mb-2 text-sm font-bold text-slate-800 dark:text-slate-200">All parameters</h4>
                                <div class="overflow-x-auto rounded-lg border border-slate-200 dark:border-slate-700">
                                    <table class="w-full text-[11px]">
                                        <thead class="bg-slate-50 dark:bg-slate-800">
                                            <tr>
                                                <th class="px-3 py-2 text-left font-semibold">Parameter</th>
                                                <th class="px-3 py-2 text-left font-semibold">Default</th>
                                                <th class="px-3 py-2 text-left font-semibold">Unit</th>
                                                <th class="px-3 py-2 text-left font-semibold">Used in</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                                            {#each params as p}
                                                <tr>
                                                    <td class="px-3 py-1.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                                                    <td class="px-3 py-1.5">{p.def}</td>
                                                    <td class="px-3 py-1.5 text-slate-500">{p.unit}</td>
                                                    <td class="px-3 py-1.5"><GlossaryRichText text={p.used} definitions={glossaryDefinitions} /></td>
                                                </tr>
                                            {/each}
                                        </tbody>
                                    </table>
                                </div>
                                <p class="mt-2 text-[10px] text-slate-400"><GlossaryRichText text="RT/m/z/S/B parameters are adjustable on the main form. cv_high_max and cv_moderate_max are currently fixed." definitions={glossaryDefinitions} /></p>
                            </div>
                        {/if}
                    </div>
                {/if}

            </div>
            {/key}
        </div>

        <!-- Bottom navigation -->
        <div class="shrink-0 flex items-center justify-between border-t border-slate-100 px-5 py-2.5 dark:border-slate-800">
            <button
                onclick={() => current > 0 && current--}
                class="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold text-slate-500 transition-colors hover:bg-slate-100 hover:text-slate-900 disabled:pointer-events-none disabled:opacity-30 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-white"
                disabled={current === 0}
            >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><path d="M15 18L9 12L15 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                Back
            </button>

            <div class="flex items-center gap-1.5">
                {#each steps as _, i}
                    <button
                        onclick={() => (current = i)}
                        class="h-1.5 rounded-full transition-all duration-300
                            {current === i ? 'w-5 bg-blue-500' : 'w-1.5 bg-slate-300 hover:bg-slate-400 dark:bg-slate-600 dark:hover:bg-slate-500'}"
                        aria-label="Go to step {i}"
                    ></button>
                {/each}
            </div>

            <button
                onclick={() => current < steps.length - 1 && current++}
                class="flex items-center gap-1.5 rounded-lg bg-slate-900 px-4 py-1.5 text-xs font-semibold text-white shadow-sm transition-transform hover:scale-105 disabled:pointer-events-none disabled:opacity-30 dark:bg-white dark:text-slate-900"
                disabled={current === steps.length - 1}
            >
                Next
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><path d="M9 18L15 12L9 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </button>
        </div>
    </div>
</div>
