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
        ["RT", "number", "Retention time of the chromatographic peak", "2.345"],
        ["Base Peak", "number", "m/z of the dominant ion", "195.08"],
        ["Polarity", "string", "Ionization polarity: positive / negative", "positive"],
        ["File", "string", "File name used to identify sample and replicate", "1_pos.d"],
        ["Area", "number", "Peak area", "1250000"],
        ["Label", "string", "Optional operator label", "Caffeine"],
    ];

    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    const outputFields = [
        ["RT_mean", "Mean RT of the confirmed cluster."],
        ["MZ_mean", "Mean m/z of the confirmed cluster."],
        ["Area_mean", "Mean peak area without integer truncation."],
        ["AreaCVPct", "CV% across replicate peak areas."],
        ["ReplicateQuality", "High / Moderate / Low quality band."],
        ["SignalToBlankRatio", "S/B ratio for the matched blank peak."],
        ["ConfidenceScore", "Final 0–100 confidence score."],
        ["Status", "Real Compound or Artifact."],
        ["Why", "JSON decision trail with threshold details."],
    ];

    const params = [
        ["replicate_rt_tol", "0.1", "min", "Coarse screening"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "min", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
    ];

    const glossary = [
        ["RT", "Retention time of the analyte in the LC column."],
        ["m/z", "Mass-to-charge ratio of the ion signal."],
        ["Replicate", "Independent repeat measurement of the same sample."],
        ["Blank", "Solvent-only control used to identify background signal."],
        ["CV%", "Relative variability between replicate peak areas."],
        ["S/B ratio", "Signal-to-Blank ratio for the matched peak."],
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
