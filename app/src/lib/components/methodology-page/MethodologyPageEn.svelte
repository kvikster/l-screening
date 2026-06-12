<!--
  @file MethodologyPageEn.svelte
  @description
  English-language methodology documentation page for the LC-MS Screening application.
-->
<script lang="ts">
    import MethodologyVisualizerEn from "./MethodologyVisualizerEn.svelte";
    import GlossaryTooltip from "./GlossaryTooltip.svelte";

    // ---------------------------------------------------------------------------
    // Data Arrays — Content Tables
    // ---------------------------------------------------------------------------
    const columns = [
        ["RT", "number", "Retention time of the chromatographic peak (required)", "2.345"],
        ["Base Peak", "number", "m/z of the dominant ion — OPTIONAL. Absent ⇒ RT-only run (e.g. GC-FID, LC-UV); the missing-m/z confidence penalty is waived.", "195.08"],
        ["Polarity", "string", "Ionization polarity — OPTIONAL. Absent/empty for all rows ⇒ single \"n/a\" group, no per-polarity split.", "positive"],
        ["File", "string", "File name used to identify sample and replicate (required)", "1_pos.d"],
        ["Area", "number", "Peak area (required)", "1250000"],
        ["Label", "string", "Optional operator label", "Caffeine"],
    ];

    // Accepted input file formats. Delimited text (CSV/TSV/TXT) is parsed as a
    // table; XLSX/XLS additionally carry cell fill colors used as operator marks.
    // A new instrument with different headers is onboarded via a column-mapping
    // profile that maps its source headers onto these canonical fields.
    const inputFormats = [
        ["CSV / TSV / TXT", "Delimited text. No cell colors — operator marks must be in operator_mark / operator_color columns."],
        ["XLSX / XLS", "Excel workbook. Cell fill colors are read as operator marks."],
        ["Column-mapping profile", "Maps a new instrument's headers (e.g. RetentionTime → RT, PeakArea → Area) onto canonical fields; can declare Base Peak / Polarity absent."],
    ];

    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    const outputFields = [
        ["RT_mean", "Mean RT of the confirmed cluster."],
        ["MZ_mean", "Mean m/z of the confirmed cluster (null on RT-only data)."],
        ["Area_mean", "Mean peak area without integer truncation."],
        ["AreaCVPct", "RSD (relative standard deviation) of replicate peak areas — identical to the quantity historically labeled CV%. Shown in the UI as \"RSD (CV%)\"."],
        ["ReplicateCount", "Number of replicates in the cluster. 1 replicate = 1 chromatogram."],
        ["ReplicateQuality", "High / Moderate / Low quality band (from RSD)."],
        ["SignalToBlankRatio", "S/B ratio for the matched blank peak; null when there is no blank to subtract."],
        ["Why.BlankState", "blank_subtracted (blank matched & subtracted) · blank_clean_here (a blank exists but is clean at this RT) · no_blank_loaded (no blank in the set at all). The last two stay Real Compound."],
        ["Why.BinWidthSuggestion", "On surrogate rows: advisory cluster bin-width metrics from the surrogate's RT spread — MaxAbsRtShift, KStdevRt (k·σ), RangeRt — each with the replicate count."],
        ["ConfidenceScore", "Final 0–100 confidence score."],
        ["Status", "Real Compound or Artifact."],
        ["Why", "JSON decision trail with threshold details."],
    ];

    const params = [
        ["replicate_rt_tol", "0.1", "min", "Coarse screening (the surrogate \"bin width\" suggestion guides this)"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "min", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
        ["mz_available", "true", "flag", "Auto-set false when no Base Peak column ⇒ RT-only; waives missing-m/z penalty"],
        ["polarity_available", "true", "flag", "Auto-set false when no Polarity for any row ⇒ single \"n/a\" group"],
    ];

    const glossary = [
        ["RT", "Retention time of the analyte in the LC column."],
        ["m/z", "Mass-to-charge ratio of the ion signal."],
        ["Replicate", "Independent repeat measurement of the same sample. 1 replicate = 1 chromatogram."],
        ["Blank", "Solvent-only control used to identify background signal."],
        ["RSD (CV%)", "Relative standard deviation of replicate peak areas (σ/mean·100). RSD and CV% are the same quantity; the UI uses RSD."],
        ["S/B ratio", "Signal-to-Blank ratio. When no blank peak exists, the result is reported as 'no blank detected' rather than empty."],
        ["Cluster bin width", "The replicate RT tolerance. A surrogate's observed RT spread suggests a value (advisory) to enter in Analyzer Configuration."],
        ["RT-only mode", "Running with no m/z (and optionally no polarity) — matching replicates and blanks on retention time alone."],
        ["Confidence score", "Combined confidence metric for a screened peak."],
    ];

    const glossaryMap = Object.fromEntries(glossary);

    const refs = [
        ["Liquid chromatography–mass spectrometry (LC–MS)", "https://en.wikipedia.org/wiki/Liquid_chromatography%E2%80%93mass_spectrometry"],
        ["Mass spectrometry", "https://en.wikipedia.org/wiki/Mass_spectrometry"],
        ["Coefficient of variation", "https://en.wikipedia.org/wiki/Coefficient_of_variation"],
        ["ISO/IEC 17025", "https://en.wikipedia.org/wiki/ISO/IEC_17025"],
    ];
</script>

<svelte:head>
    <title>Methodology — LC-MS Screening</title>
</svelte:head>

<main class="min-h-screen bg-slate-50 dark:bg-slate-900">
    <div class="mx-auto max-w-5xl px-6 py-12">

        <div class="mb-8">
            <a
                href={import.meta.env.VITE_STANDALONE ? "../" : "/"}
                data-sveltekit-reload={import.meta.env.VITE_STANDALONE ? "" : undefined}
                class="inline-flex items-center gap-2 text-sm text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
            >
                <span>←</span>
                <span>Back</span>
            </a>
        </div>

        <header class="mb-10">
            <h1 class="text-4xl font-bold text-slate-900 dark:text-slate-50">Screening methodology</h1>
            <p class="mt-3 max-w-3xl text-lg text-slate-500 dark:text-slate-400">
                A compact but complete description of how LC-MS Screening reads Excel, confirms replicate peaks,
                performs blank subtraction, and produces an audit-ready result.
            </p>
        </header>

        <nav class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="mb-3 text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">Contents</p>
            <div class="grid gap-2 text-sm text-blue-700 dark:text-blue-400 sm:grid-cols-2">
                <a href="#input" class="hover:underline">1. Input data</a>
                <a href="#flexibility" class="hover:underline">⓿ Analyzer flexibility & terminology (v0.9.0)</a>
                <a href="#columns" class="hover:underline">2. Excel columns</a>
                <a href="#marks" class="hover:underline">3. Operator marks</a>
                <a href="#algorithm" class="hover:underline">4. Algorithm (Interactive Guide)</a>
                <a href="#output" class="hover:underline">5. Output fields</a>
                <a href="#params" class="hover:underline">6. Parameters</a>
                <a href="#glossary" class="hover:underline">7. Glossary</a>
                <a href="#references" class="hover:underline">8. References</a>
            </div>
        </nav>

        <!-- Interactive Visual Overview -->
        <div id="algorithm">
            <MethodologyVisualizerEn defs={glossaryMap} />
        </div>

        <!-- Section 1: Input Data -->
        <section id="input" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-3 text-2xl font-semibold text-slate-900 dark:text-slate-50">1. Input data</h2>
            <div class="space-y-3 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <p>The application expects an Excel workbook containing LC-MS peak data. If multiple sheets exist, the sheet with the strongest required-column match is selected automatically.</p>
                <p>The typical scenario is two <GlossaryTooltip term="Replicate" definition={glossaryMap["Replicate"]} /> measurements plus one <GlossaryTooltip term="Blank" definition={glossaryMap["Blank"]} />. The blank acts as a control for background, matrix effects, and laboratory artifacts.</p>
            </div>
        </section>

        <!-- Section: Analyzer flexibility & terminology (v0.9.0) -->
        <section id="flexibility" class="mb-10 rounded-2xl border border-blue-200 bg-blue-50/40 p-6 shadow-sm dark:border-blue-900/60 dark:bg-blue-950/20">
            <h2 class="mb-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">Analyzer flexibility & terminology <span class="align-middle text-xs font-bold text-blue-700 dark:text-blue-400">v0.9.0</span></h2>
            <p class="mb-5 text-sm leading-7 text-slate-600 dark:text-slate-400">This release makes the pipeline tolerant of real-world inputs — other instruments, missing dimensions, and clean blanks — and clarifies on-screen terminology.</p>

            <div class="space-y-5 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">1 · Input formats & instrument profiles</h3>
                    <p>Accepted file types: <strong>CSV, TSV, TXT, XLSX, XLS</strong>. Delimited text is read as a table; Excel additionally carries cell-fill colors as operator marks. To onboard an instrument whose export uses different headers, pick an <strong>instrument profile</strong> in the import panel — it maps that instrument's headers (e.g. <span class="font-mono">RetentionTime → RT</span>, <span class="font-mono">PeakArea → Area</span>) onto the canonical fields and may declare <span class="font-mono">Base Peak</span> / <span class="font-mono">Polarity</span> absent. A sample set is assembled in the workspace from one or more files, each contributing its rows.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">2 · Optional Polarity / Base Peak (RT-only)</h3>
                    <p>The algorithm now runs without <span class="font-mono">Polarity</span> (all rows form one <span class="font-mono">n/a</span> group instead of splitting per-polarity), and without both <span class="font-mono">Polarity</span> and <span class="font-mono">Base Peak</span> (RT-only — replicates and blanks match on retention time alone). When m/z is absent for the whole dataset the missing-m/z confidence penalty is waived, because the absence is a data-format property, not a matching weakness.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">3 · Signal-to-Blank with no blank bin</h3>
                    <p>When a sample peak has no matching blank cluster it is no longer left empty. Three explicit states are reported in <span class="font-mono">Why.BlankState</span>: <span class="font-mono">blank_subtracted</span> (a blank matched and was subtracted), <span class="font-mono">blank_clean_here</span> (a blank exists but is clean at this RT), and <span class="font-mono">no_blank_loaded</span> (no blank in the set at all — e.g. the blank was so clean the operator detected nothing). The latter two stay <strong>Real Compound</strong> — a clean blank reads as a strong result, not a missing one.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">4 · Surrogate-suggested cluster bin width</h3>
                    <p>A surrogate's observed RT behavior suggests a replicate RT tolerance (the cluster "bin width"). Opening a surrogate's audit shows <span class="font-mono">Why.BinWidthSuggestion</span> with three advisory metrics — <span class="font-mono">MaxAbsRtShift</span>, <span class="font-mono">KStdevRt</span> (k·σ, k=3) and <span class="font-mono">RangeRt</span> — each with the replicate count. Nothing is auto-applied; the operator enters a value into Analyzer Configuration manually.</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">5 · Terminology: RSD = CV%, Replicate = chromatogram</h3>
                    <p><strong>CV% is RSD</strong> (relative standard deviation, σ/mean·100) — the same quantity; the UI now labels it <span class="font-mono">RSD (CV%)</span> with the value unchanged. <strong>1 replicate = 1 chromatogram</strong>, shown as "Replicates (chromatograms)".</p>
                </div>
                <div>
                    <h3 class="font-semibold text-slate-900 dark:text-slate-100">6 · Replicate provenance highlighting</h3>
                    <p>The replicate count is visually emphasized; hovering shows the origin of each replicate — the source files and, when a cluster was merged across parallel samples, which samples it spans (an amber ring flags mixed provenance).</p>
                </div>
            </div>

            <div class="mt-6 overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Accepted format</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Notes</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                        {#each inputFormats as row}
                            <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                                <td class="px-4 py-3 font-mono font-semibold text-blue-700 dark:text-blue-400">{row[0]}</td>
                                <td class="px-4 py-3 text-slate-600 dark:text-slate-400">{row[1]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- Section 2: Required Excel Columns -->
        <section id="columns" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">2. Required Excel columns</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Column</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Type</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Description</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Example</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                        {#each columns as row}
                            <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                                <td class="px-4 py-3 font-mono font-semibold text-blue-700 dark:text-blue-400">{row[0]}</td>
                                <td class="px-4 py-3 text-slate-500 dark:text-slate-400">{row[1]}</td>
                                <td class="px-4 py-3 text-slate-700 dark:text-slate-300">{row[2]}</td>
                                <td class="px-4 py-3 font-mono text-slate-500 dark:text-slate-400">{row[3]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- Section 3: Operator Marks -->
        <section id="marks" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">3. Operator marks</h2>
            <p class="text-sm leading-7 text-slate-600 dark:text-slate-400">
                Manual cell colors in Excel let the operator explicitly define the role of each row. When present, these marks take precedence over file-name heuristics.
            </p>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
                {#each operatorMarks as mark}
                    <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <span class="mt-1 h-4 w-4 rounded-full border border-black/10 dark:border-white/10" style={`background:${mark[1]}`}></span>
                        <div>
                            <p class="font-mono text-sm font-semibold text-slate-900 dark:text-slate-100">{mark[0]}</p>
                            <p class="text-xs text-slate-500 dark:text-slate-400">{mark[2]}</p>
                        </div>
                    </div>
                {/each}
            </div>
        </section>

        <!-- Section 5: Output Fields -->
        <section id="output" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">4. Output fields</h2>
            <div class="space-y-3">
                {#each outputFields as field}
                    <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{field[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{field[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- Section 6: Tolerance Parameters -->
        <section id="params" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">5. Tolerance parameters</h2>
            <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <table class="w-full text-sm">
                    <thead class="bg-slate-50 dark:bg-slate-700">
                        <tr>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Parameter</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Default</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Unit</th>
                            <th class="px-4 py-3 text-left font-semibold text-slate-700 dark:text-slate-300">Used in</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                        {#each params as row}
                            <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                                <td class="px-4 py-3 font-mono font-semibold text-blue-700 dark:text-blue-400">{row[0]}</td>
                                <td class="px-4 py-3 text-slate-700 dark:text-slate-300">{row[1]}</td>
                                <td class="px-4 py-3 text-slate-500 dark:text-slate-400">{row[2]}</td>
                                <td class="px-4 py-3 text-slate-600 dark:text-slate-400">{row[3]}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- Section 7: Glossary -->
        <section id="glossary" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">6. Glossary</h2>
            <div class="grid gap-3 sm:grid-cols-2">
                {#each glossary as item}
                    <div class="rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{item[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{item[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- Section 8: References -->
        <section id="references" class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">7. References</h2>
            <p class="mb-4 text-sm leading-7 text-slate-600 dark:text-slate-400">Core terminology and regulatory context that support this methodology.</p>
            <div class="space-y-3">
                {#each refs as ref}
                    <a href={ref[1]} target="_blank" rel="noopener noreferrer" class="block rounded-xl border border-slate-200 px-4 py-3 text-sm text-blue-700 hover:bg-slate-50 dark:border-slate-700 dark:text-blue-400 dark:hover:bg-slate-900">
                        {ref[0]}
                    </a>
                {/each}
            </div>
        </section>
    </div>
</main>
