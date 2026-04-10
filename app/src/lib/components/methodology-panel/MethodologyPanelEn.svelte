<script lang="ts">
    import TermTooltip from "../TermTooltip.svelte";
    import MethodologyVisualizerEn from "../methodology-page/MethodologyVisualizerEn.svelte";

    const gl: Record<string, string> = {
        RT: "Retention Time is the chromatographic retention time of an analyte in the column, measured in minutes.",
        "m/z": "Mass-to-charge ratio is the central coordinate of a mass-spectrometric signal.",
        "CV%": "Coefficient of Variation measures relative variability between replicate peak areas. CV% = (sample std / mean) × 100. For n=2: std = |v₁−v₂|/√2. Lower CV means better reproducibility.",
        "S/B": "Signal-to-Blank ratio — sample peak area divided by the area of the closest confirmed blank peak.",
        blank: "A blank sample (solvent only, no analyte) is used to identify background artifacts and contamination.",
        replicate: "An independent repeat measurement of the same sample. At least 2 replicate buckets are required to confirm a peak.",
        "confidence score": "A 0–100 score starting at 100, penalised by RT/m/z deviation, CV%, and S/B ratio. See section 3b for the full formula.",
        ppm: "Parts per million — a relative m/z tolerance for high-resolution instruments.",
        Da: "Dalton — an absolute m/z tolerance in daltons.",
        "replicate bucket": "All peaks from one file or one explicit operator mark. Each file/mark forms one bucket.",
    };

    const columns = [
        { name: "RT", type: "number", desc: "Retention time of the chromatographic peak", example: "2.345" },
        { name: "Base Peak", type: "number", desc: "m/z of the dominant ion in the mass spectrum", example: "195.08" },
        { name: "Polarity", type: "string", desc: "Ionization polarity: positive / negative", example: "positive" },
        { name: "File", type: "string", desc: "Source file name used to assign replicate buckets", example: "1_pos.d" },
        { name: "Area", type: "number", desc: "Peak area proportional to analyte abundance", example: "1250000" },
        { name: "Label", type: "string", desc: "(Optional) operator label or compound name", example: "Caffeine" },
    ];

    const outputFields = [
        { name: "RT_mean", desc: "Mean RT of the confirmed cluster (across all replicates)." },
        { name: "MZ_mean", desc: "Mean m/z of the confirmed cluster." },
        { name: "Area_mean", desc: "Mean peak area (no integer truncation)." },
        { name: "AreaCVPct", desc: "CV% across replicate areas of the cluster (sample std)." },
        { name: "ReplicateQuality", desc: "High (CV% ≤ 15) / Moderate (≤ 30) / Low (> 30) — reproducibility band." },
        { name: "SignalToBlankRatio", desc: "S/B ratio for the closest confirmed blank peak." },
        { name: "ConfidenceScore", desc: "Final 0–100 confidence score after blank subtraction." },
        { name: "Status", desc: "Real Compound or Artifact classification result." },
        { name: "Why", desc: "JSON object with the full decision audit trail (RT/mz deltas, CV, S/B, thresholds)." },
    ];

    const params = [
        { name: "replicate_rt_tol", val: "0.1", unit: "min", where: "Replicate clustering" },
        { name: "replicate_mz_tol", val: "0.3", unit: "Da / ppm", where: "Replicate clustering" },
        { name: "blank_rt_tol", val: "0.1", unit: "min", where: "Blank subtraction" },
        { name: "blank_mz_tol", val: "0.3", unit: "Da / ppm", where: "Blank subtraction" },
        { name: "signal_to_blank_min", val: "3.0", unit: "ratio", where: "Artifact / Real Compound decision" },
        { name: "cv_high_max", val: "15", unit: "%", where: "ReplicateQuality = High" },
        { name: "cv_moderate_max", val: "30", unit: "%", where: "ReplicateQuality = Moderate" },
    ];
</script>

<div class="space-y-10 text-sm">
    <MethodologyVisualizerEn />

    <section id="m-input">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            1. Input data
        </h3>
        <p class="leading-relaxed text-slate-600 dark:text-slate-400">
            The system accepts an <strong>Excel file (.xlsx / .xls)</strong> with LC-MS peak data,
            automatically selects the most suitable sheet, and expects at least two
            <TermTooltip term="replicates" def={gl.replicate} /> (separate files or operator marks) plus a
            <TermTooltip term="blank" def={gl.blank} /> sample.
        </p>
    </section>

    <section id="m-columns">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            2. Required Excel columns
        </h3>
        <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
            <table class="w-full">
                <thead class="bg-slate-50 dark:bg-slate-700">
                    <tr>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Column</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Type</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Description</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Example</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                    {#each columns as col}
                        <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                            <td class="px-4 py-2.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{col.name}</td>
                            <td class="px-4 py-2.5 text-slate-500 dark:text-slate-400">{col.type}</td>
                            <td class="px-4 py-2.5 text-slate-700 dark:text-slate-300">{col.desc}</td>
                            <td class="px-4 py-2.5 font-mono text-slate-500 dark:text-slate-400">{col.example}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
        <p class="mt-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-2.5 text-amber-800 dark:border-amber-700 dark:bg-amber-950 dark:text-amber-200">
            <strong>Operator marks:</strong> cell color marks (<code class="font-mono text-xs">sample_rep1</code>, <code class="font-mono text-xs">sample_rep2</code>, <code class="font-mono text-xs">blank_positive</code>, <code class="font-mono text-xs">blank_negative</code>) take priority over the file name. Without marks, type is inferred from the filename: files containing «blank» → blank; files like <code class="font-mono text-xs">1_*.d</code>, <code class="font-mono text-xs">2_*.d</code> → <code class="font-mono text-xs">sample_1</code>, <code class="font-mono text-xs">sample_2</code>, etc.
        </p>
    </section>

    <section id="m-algo">
        <h3 class="mb-4 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            3. Algorithm
        </h3>
        <div class="space-y-3">
            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">1</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Pre-processing</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        The file is parsed, the best sheet is selected, rows missing
                        <TermTooltip term="RT" def={gl.RT} />, <TermTooltip term="m/z" def={gl["m/z"]} />
                        or Area are removed. Each row is assigned a <code class="rounded bg-slate-100 px-1 text-xs font-mono dark:bg-slate-700">SampleType</code> (blank / sample_N)
                        and a <TermTooltip term="replicate bucket" def={gl["replicate bucket"]} />.
                        Rows are then grouped by (SampleType, Polarity) — each group is processed independently.
                    </p>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">2</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Greedy replicate clustering</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        Peaks are sorted by area descending. Each peak seeds a cluster in turn.
                        From every other bucket the closest unused peak to the current
                        <strong>cluster centroid</strong> (RT, m/z averaged as members are added) is greedily selected.
                        A cluster is confirmed when it spans <strong>≥ 2 different buckets</strong>.
                    </p>
                    <div class="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="mb-1 text-[10px] font-semibold uppercase tracking-wider text-slate-400">Inclusion rule</p>
                        <p class="text-slate-700 dark:text-slate-300">|RT_candidate − RT_centroid| ≤ replicate_rt_tol</p>
                        <p class="text-slate-700 dark:text-slate-300">|mz_candidate − mz_centroid| ≤ replicate_mz_tol (<TermTooltip term="Da" def={gl.Da} /> or <TermTooltip term="ppm" def={gl.ppm} />)</p>
                        <p class="mt-2 text-[10px] text-slate-400">Ties broken by smallest distance (RT fraction + mz fraction), then largest area.</p>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">3</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Blank subtraction</p>
                    <p class="mt-1 leading-relaxed text-slate-600 dark:text-slate-400">
                        The blank undergoes the same clustering (step 2) independently. Each confirmed sample peak is then matched against confirmed
                        <TermTooltip term="blank" def={gl.blank} /> peaks within blank_rt_tol / blank_mz_tol.
                        The closest match is selected (by RT+mz distance, tie-broken by larger blank area).
                    </p>
                    <div class="mt-3 grid gap-2 sm:grid-cols-2">
                        <div class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300">
                            <strong>Artifact</strong> — blank match found and S/B &lt; signal_to_blank_min (or blank area = 0)
                        </div>
                        <div class="rounded-lg border border-green-200 bg-green-50 px-3 py-2 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300">
                            <strong>Real Compound</strong> — no blank match, or S/B ≥ threshold
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-indigo-50 text-xs font-bold text-indigo-600 dark:bg-indigo-950 dark:text-indigo-300">Σ</span>
                <div class="flex-1">
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Confidence Score — scoring formula</p>
                    <p class="mt-1 mb-3 text-slate-500 dark:text-slate-400">Starts at 100. Penalties are applied in sequence; result is clamped to [0, 100].</p>
                    <div class="space-y-1.5 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 font-mono text-xs dark:border-slate-700 dark:bg-slate-900">
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 mb-2">Stage 1 — replicates</p>
                        <p class="text-slate-700 dark:text-slate-300">RT proximity&nbsp;&nbsp;&nbsp;− (mean_RT_delta / rt_tol) × 20&nbsp;&nbsp;<span class="text-slate-400">max −20</span></p>
                        <p class="text-slate-700 dark:text-slate-300">mz proximity&nbsp;&nbsp;− (mean_mz_delta / mz_tol) × 25&nbsp;&nbsp;<span class="text-slate-400">max −25</span></p>
                        <p class="text-slate-700 dark:text-slate-300">CV% High&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;0</p>
                        <p class="text-slate-700 dark:text-slate-300">CV% Moderate&nbsp;&nbsp;−12</p>
                        <p class="text-slate-700 dark:text-slate-300">CV% Low&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;−(12 + (CV − cv_moderate_max) × 0.7),&nbsp;<span class="text-slate-400">max −35</span></p>
                        <p class="text-slate-700 dark:text-slate-300">CV% missing&nbsp;&nbsp;&nbsp;−10</p>
                        <p class="text-slate-700 dark:text-slate-300">Not colour-paired −5</p>
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-slate-400 mt-3 mb-2">Stage 2 — blank</p>
                        <p class="text-slate-700 dark:text-slate-300">No blank match&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;+3</p>
                        <p class="text-slate-700 dark:text-slate-300">S/B ≥ threshold&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;+min(5, (S/B − threshold) × 0.5)</p>
                        <p class="text-slate-700 dark:text-slate-300">S/B &lt; threshold (Artifact) −(15 + deficit × 30),&nbsp;<span class="text-slate-400">max −45</span></p>
                    </div>
                </div>
            </div>

            <div class="flex gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-blue-50 text-xs font-bold text-blue-600 dark:bg-blue-950 dark:text-blue-300">4</span>
                <div>
                    <p class="font-semibold text-slate-900 dark:text-slate-100">Summary statistics</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-400">
                        For each (SampleType, Polarity) pair the system reports Total / Confirmed / Artifacts / Real Compounds,
                        mean CV%, mean Confidence Score, and mean S/B ratio.
                    </p>
                </div>
            </div>
        </div>
    </section>

    <section id="m-output">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            4. Output fields
        </h3>
        <div class="space-y-2">
            {#each outputFields as f}
                <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-white px-4 py-3 dark:border-slate-700 dark:bg-slate-800">
                    <code class="min-w-[150px] flex-shrink-0 font-mono font-semibold text-blue-700 dark:text-blue-400">{f.name}</code>
                    <p class="text-slate-600 dark:text-slate-400">{f.desc}</p>
                </div>
            {/each}
        </div>
    </section>

    <section id="m-params">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            5. Parameters
        </h3>
        <div class="overflow-hidden rounded-xl border border-slate-200 dark:border-slate-700">
            <table class="w-full">
                <thead class="bg-slate-50 dark:bg-slate-700">
                    <tr>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Parameter</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Default</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Unit</th>
                        <th class="px-4 py-2.5 text-left font-semibold text-slate-700 dark:text-slate-300">Used in</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
                    {#each params as p}
                        <tr class="hover:bg-slate-50/60 dark:hover:bg-slate-700/40">
                            <td class="px-4 py-2.5 font-mono font-semibold text-blue-700 dark:text-blue-400">{p.name}</td>
                            <td class="px-4 py-2.5 font-mono text-slate-700 dark:text-slate-300">{p.val}</td>
                            <td class="px-4 py-2.5 text-slate-500 dark:text-slate-400">{p.unit}</td>
                            <td class="px-4 py-2.5 text-slate-600 dark:text-slate-400">{p.where}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
        <p class="mt-2 text-xs text-slate-400 dark:text-slate-500">
            RT/mz/S/B parameters are adjustable on the main form. cv_high_max and cv_moderate_max are currently fixed.
        </p>
    </section>

    <section id="m-glossary">
        <h3 class="mb-3 border-b border-slate-200 pb-2 text-base font-bold text-slate-900 dark:border-slate-700 dark:text-slate-50">
            6. Glossary
        </h3>
        <div class="grid gap-2 sm:grid-cols-2">
            {#each Object.entries(gl) as [term, def]}
                <div class="rounded-xl border border-slate-200 bg-white px-4 py-3 dark:border-slate-700 dark:bg-slate-800">
                    <p class="font-mono font-semibold text-blue-700 dark:text-blue-400">{term}</p>
                    <p class="mt-1 text-xs leading-relaxed text-slate-600 dark:text-slate-400">{def}</p>
                </div>
            {/each}
        </div>
    </section>
</div>
