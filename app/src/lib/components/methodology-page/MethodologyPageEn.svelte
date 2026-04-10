<!--
  @file MethodologyPageEn.svelte
  @description
  English-language methodology documentation page for the LC-MS Screening application.
  This component renders a static, self-contained reference page that describes the
  complete screening pipeline — from raw Excel input through replicate confirmation,
  blank subtraction, and audit-trail generation.

  ## Component Role

  This is one of three locale-specific methodology pages (En / Ru / Uk) dynamically
  selected by the route component at `src/routes/methodology/+page.svelte`. Each page
  is a standalone Svelte component with its own translated content; no runtime i18n
  translation keys are used because the content is predominantly long-form scientific
  prose that would be awkward to key-ify.

  ## Data Architecture

  All content is defined as static `const` arrays in the `<script>` block. Each array
  follows a tuple convention where positional elements map to specific table columns or
  UI slots in the template. This approach keeps data and presentation cleanly separated
  while avoiding the overhead of a full CMS or markdown pipeline for a single-page doc.

  ## Sections Rendered

  1. **Input data**        — expected Excel format and sheet-selection logic
  2. **Excel columns**     — required column schema (name, type, description, example)
  3. **Operator marks**    — manual color-coding scheme for row classification
  4. **Algorithm**         — four-step screening pipeline with formulas
  5. **Output fields**     — produced result fields with descriptions
  6. **Parameters**        — configurable tolerance thresholds
  7. **Glossary**          — domain term definitions
  8. **References**        — external links for further reading
-->
<script lang="ts">
    import MethodologyVisualizerEn from "./MethodologyVisualizerEn.svelte";

    // ---------------------------------------------------------------------------
    // Data Arrays — Content Tables
    // ---------------------------------------------------------------------------
    // Each array uses a tuple convention where elements are accessed by index
    // in the template (e.g., row[0], row[1]). This avoids introducing a typed
    // interface for what is essentially static documentation content.

    /**
     * Required Excel columns that the screening engine expects to find.
     *
     * Tuple structure: [column_name, data_type, description, example_value]
     *
     * - column_name  — exact header text the engine looks for during sheet parsing
     * - data_type    — JavaScript/JSON type the value is coerced to ("number" | "string")
     * - description  — human-readable explanation of what the column represents
     * - example_value — a representative value shown in the documentation table
     *
     * These columns are matched case-insensitively during sheet auto-detection.
     * If multiple sheets exist, the sheet with the highest column-match count wins.
     */
    const columns = [
        ["RT", "number", "Retention time of the chromatographic peak", "2.345"],
        ["Base Peak", "number", "m/z of the dominant ion", "195.08"],
        ["Polarity", "string", "Ionization polarity: positive / negative", "positive"],
        ["File", "string", "File name used to identify sample and replicate", "1_pos.d"],
        ["Area", "number", "Peak area", "1250000"],
        ["Label", "string", "Optional operator label", "Caffeine"],
    ];

    /**
     * Operator mark definitions — manual cell-color codes that override file-name
     * heuristics for row classification.
     *
     * Tuple structure: [mark_id, hex_color, display_label]
     *
     * - mark_id      — internal identifier used by the parsing engine to tag rows
     * - hex_color    — the exact background color the operator paints in Excel
     * - display_label — human-readable label shown in the documentation UI
     *
     * Control flow: During pre-processing, the engine reads each cell's background
     * color and matches it against these hex values. A matching row is assigned the
     * corresponding SampleType (sample / blank) and replicate index, bypassing the
     * file-name-based heuristic entirely.
     */
    const operatorMarks = [
        ["sample_rep1", "#ff00ff", "Sample, Replicate 1"],
        ["sample_rep2", "#ffff00", "Sample, Replicate 2"],
        ["blank_positive", "#00ffff", "Blank"],
        ["blank_negative", "#00ff00", "Blank"],
    ];

    /**
     * Output fields produced by the screening engine for each confirmed peak cluster.
     *
     * Tuple structure: [field_name, description]
     *
     * - field_name — exact key name in the output JSON/Excel
     * - description — what the field represents and how it is computed
     *
     * These fields constitute the final audit-ready result set. The `Why` field is
     * particularly important for regulatory compliance — it contains a JSON decision
     * trail documenting every threshold comparison that led to the final Status.
     */
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

    /**
     * Configurable tolerance parameters that control the screening algorithm's
     * matching strictness.
     *
     * Tuple structure: [param_name, default_value, unit, used_in_stage]
     *
     * - param_name    — the exact key used in the configuration object / UI slider
     * - default_value — factory default if the user does not override
     * - unit          — physical or dimensionless unit for the threshold
     * - used_in_stage — which pipeline stage consumes this parameter
     *
     * Algorithmic impact:
     *   - replicate_rt_tol / replicate_mz_tol: define the matching window for
     *     confirming that two peaks across replicates represent the same compound
     *   - blank_rt_tol / blank_mz_tol: define the matching window for finding
     *     a corresponding blank peak
     *   - signal_to_blank_min: the minimum S/B ratio below which a peak is
     *     classified as an Artifact rather than a Real Compound
     */
    const params = [
        ["replicate_rt_tol", "0.1", "min", "Coarse screening"],
        ["replicate_mz_tol", "0.3", "Da / ppm", "Coarse screening"],
        ["blank_rt_tol", "0.1", "min", "Blank subtraction"],
        ["blank_mz_tol", "0.3", "Da / ppm", "Blank subtraction"],
        ["signal_to_blank_min", "3.0", "ratio", "Artifact / Real Compound decision"],
    ];

    /**
     * Glossary of domain-specific terms used throughout the methodology page.
     *
     * Tuple structure: [term, definition]
     *
     * Rendered as a two-column card grid. Terms are displayed in monospace blue
     * to visually distinguish them as defined vocabulary.
     */
    const glossary = [
        ["RT", "Retention time of the analyte in the LC column."],
        ["m/z", "Mass-to-charge ratio of the ion signal."],
        ["Replicate", "Independent repeat measurement of the same sample."],
        ["Blank", "Solvent-only control used to identify background signal."],
        ["CV%", "Relative variability between replicate peak areas."],
        ["S/B ratio", "Signal-to-Blank ratio for the matched peak."],
        ["Confidence score", "Combined confidence metric for a screened peak."],
    ];

    /**
     * External reference links for further reading on core concepts.
     *
     * Tuple structure: [display_text, url]
     *
     * Each entry renders as an outbound link (target="_blank") with rel="noopener
     * noreferrer" for security. References provide the scientific and regulatory
     * context that underpins the screening methodology.
     */
    const refs = [
        ["Liquid chromatography–mass spectrometry (LC–MS)", "https://en.wikipedia.org/wiki/Liquid_chromatography%E2%80%93mass_spectrometry"],
        ["Mass spectrometry", "https://en.wikipedia.org/wiki/Mass_spectrometry"],
        ["Coefficient of variation", "https://en.wikipedia.org/wiki/Coefficient_of_variation"],
        ["ISO/IEC 17025", "https://en.wikipedia.org/wiki/ISO/IEC_17025"],
    ];
</script>

<!-- Set the browser tab title for this page -->
<svelte:head>
    <title>Methodology — LC-MS Screening</title>
</svelte:head>

<!--
  Main content area.
  Uses a full-height minimum viewport with a centered max-width container.
  Tailwind dark-mode classes (dark:*) ensure proper theming when the user
  toggles between light and dark modes.
-->
<main class="min-h-screen bg-slate-50 dark:bg-slate-900">
    <div class="mx-auto max-w-5xl px-6 py-12">

        <!-- ----------------------------------------------------------------->
        <!-- Navigation: Back link to the main screening dashboard            -->
        <!-- ----------------------------------------------------------------->
        <!--
          The href is conditional on the VITE_STANDALONE env var:
          - In standalone/PWA mode → relative "../" path with full page reload
          - In SvelteKit SPA mode  → "/" with client-side navigation (default)
          The data-sveltekit-reload attribute forces a full page reload only
          in standalone mode where client-side routing is unavailable.
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Page header with title and introductory summary                  -->
        <!-- ----------------------------------------------------------------->
        <header class="mb-10">
            <h1 class="text-4xl font-bold text-slate-900 dark:text-slate-50">Screening methodology</h1>
            <p class="mt-3 max-w-3xl text-lg text-slate-500 dark:text-slate-400">
                A compact but complete description of how LC-MS Screening reads Excel, confirms replicate peaks,
                performs blank subtraction, and produces an audit-ready result.
            </p>
        </header>

        <!-- ----------------------------------------------------------------->
        <!-- Table of Contents — in-page anchor navigation                    -->
        <!-- ----------------------------------------------------------------->
        <!--
          Each link targets a section `id` defined further down the page.
          The grid layout (sm:grid-cols-2) collapses to a single column on
          mobile viewports.
        -->
        <nav class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="mb-3 text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">Contents</p>
            <div class="grid gap-2 text-sm text-blue-700 dark:text-blue-400 sm:grid-cols-2">
                <a href="#input" class="hover:underline">1. Input data</a>
                <a href="#columns" class="hover:underline">2. Excel columns</a>
                <a href="#marks" class="hover:underline">3. Operator marks</a>
                <a href="#algorithm" class="hover:underline">4. Algorithm</a>
                <a href="#output" class="hover:underline">5. Output fields</a>
                <a href="#params" class="hover:underline">6. Parameters</a>
                <a href="#glossary" class="hover:underline">7. Glossary</a>
                <a href="#references" class="hover:underline">8. References</a>
            </div>
        </nav>

        <!-- ----------------------------------------------------------------->
        <!-- Interactive Visual Overview                                      -->
        <!-- ----------------------------------------------------------------->
        <MethodologyVisualizerEn />

        <!-- ----------------------------------------------------------------->
        <!-- Section 1: Input Data                                            -->
        <!-- ----------------------------------------------------------------->
        <!--
          Describes the expected input format (Excel workbook) and the
          automatic sheet-selection heuristic: when multiple sheets are present,
          the engine scores each sheet by counting how many required columns it
          contains, then picks the sheet with the highest score.
        -->
        <section id="input" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-3 text-2xl font-semibold text-slate-900 dark:text-slate-50">1. Input data</h2>
            <div class="space-y-3 text-sm leading-7 text-slate-600 dark:text-slate-400">
                <p>The application expects an Excel workbook containing LC-MS peak data. If multiple sheets exist, the sheet with the strongest required-column match is selected automatically.</p>
                <p>The typical scenario is two sample replicates plus one blank. The blank acts as a control for background, matrix effects, and laboratory artifacts.</p>
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Section 2: Required Excel Columns                                -->
        <!-- ----------------------------------------------------------------->
        <!--
          Renders the `columns` array as a four-column HTML table.
          The {#each} block iterates over the tuple array, destructuring
          each entry by index:
            row[0] → column name (monospace, blue)
            row[1] → data type
            row[2] → description
            row[3] → example value (monospace)
        -->
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

        <!-- ----------------------------------------------------------------->
        <!-- Section 3: Operator Marks                                        -->
        <!-- ----------------------------------------------------------------->
        <!--
          Renders the `operatorMarks` array as a card grid. Each card shows:
            - A colored circle swatch (background set via inline style to mark[1])
            - The mark identifier (mark[0]) in monospace
            - The display label (mark[2]) in small text

          Data flow: The Excel parsing engine reads cell background colors and
          compares them against these hex values. A match assigns the row's
          SampleType and replicate index, overriding the file-name heuristic.
        -->
        <section id="marks" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">3. Operator marks</h2>
            <p class="text-sm leading-7 text-slate-600 dark:text-slate-400">
                Manual cell colors in Excel let the operator explicitly define the role of each row. When present, these marks take precedence over file-name heuristics.
            </p>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
                {#each operatorMarks as mark}
                    <div class="flex items-start gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <!-- Color swatch: inline style sets the background to the mark's hex color -->
                        <span class="mt-1 h-4 w-4 rounded-full border border-black/10 dark:border-white/10" style={`background:${mark[1]}`}></span>
                        <div>
                            <p class="font-mono text-sm font-semibold text-slate-900 dark:text-slate-100">{mark[0]}</p>
                            <p class="text-xs text-slate-500 dark:text-slate-400">{mark[2]}</p>
                        </div>
                    </div>
                {/each}
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Section 4: Algorithm — Four-Step Screening Pipeline              -->
        <!-- ----------------------------------------------------------------->
        <!--
          This is the core methodological content. The four steps are rendered
          as sequential cards, each describing a stage of the screening pipeline:

          Step 1 — Pre-processing:
            Rows missing RT, Base Peak, or Area are discarded (data validation).
            Surviving rows receive a SampleType derived from operator marks
            (highest priority) or file-name parsing (fallback heuristic).

          Step 2 — Coarse screening (replicate clustering):
            Peaks are grouped across replicate "buckets" using a greedy matching
            algorithm. The "max one peak per bucket" rule prevents a single
            replicate from contributing multiple peaks to the same cluster.
            Confirmation requires BOTH RT and m/z to fall within their
            respective tolerance thresholds (shown in the formula block).

          Step 3 — Blank subtraction:
            Each confirmed sample cluster is matched against blank peaks of the
            same ionization polarity. The Signal-to-Blank (S/B) ratio is
            computed for the matched pair. Classification logic:
              - Artifact:      blank match found AND S/B < signal_to_blank_min
              - Real Compound: no blank match OR S/B ≥ signal_to_blank_min

          Step 4 — Summary and audit trail:
            Aggregate statistics are computed per SampleType × Polarity group.
            The `Why` JSON field captures the full decision chain (which
            thresholds were checked, what values were compared) for regulatory
            audit compliance.
        -->
        <section id="algorithm" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">4. Algorithm</h2>
            <div class="space-y-4">
                <!-- Step 1: Pre-processing — data validation and row classification -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Step 1. Pre-processing</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Rows missing RT, Base Peak, or Area are discarded. Each remaining row receives a SampleType derived from either operator marks or file-name fallback logic.</p>
                </div>

                <!-- Step 2: Coarse screening — replicate peak clustering with tolerance windows -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Step 2. Coarse screening</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Peaks are clustered across replicate buckets with a "max one peak per bucket" rule. Confirmation requires both RT and m/z to fall within the replicate thresholds.</p>
                    <!-- Mathematical formulation of the matching criteria -->
                    <div class="mt-3 rounded-xl bg-slate-50 px-4 py-3 font-mono text-xs text-slate-700 dark:bg-slate-900 dark:text-slate-300">
                        <p>|RT₁ − RT₂| ≤ replicate_rt_tol</p>
                        <p>|mz₁ − mz₂| ≤ replicate_mz_tol</p>
                    </div>
                </div>

                <!-- Step 3: Blank subtraction — artifact detection via S/B ratio -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Step 3. Blank subtraction</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">Confirmed sample peaks are matched to blank peaks of the same polarity. The matched pair is evaluated with the S/B ratio.</p>
                    <!-- Binary classification outcome cards -->
                    <div class="mt-3 grid gap-3 sm:grid-cols-2">
                        <div class="rounded-xl border border-red-200 bg-red-50 p-3 text-xs text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-300"><strong>Artifact</strong>: a blank match exists and S/B is below threshold.</div>
                        <div class="rounded-xl border border-green-200 bg-green-50 p-3 text-xs text-green-800 dark:border-green-800 dark:bg-green-950 dark:text-green-300"><strong>Real Compound</strong>: no blank match or an acceptable S/B ratio.</div>
                    </div>
                </div>

                <!-- Step 4: Summary statistics and audit trail generation -->
                <div class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">Step 4. Summary and audit trail</p>
                    <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">The application computes summary metrics per SampleType / Polarity and stores the decision logic in <code class="rounded bg-slate-100 px-1 font-mono dark:bg-slate-700">Why</code> for review and audit.</p>
                </div>
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Section 5: Output Fields                                         -->
        <!-- ----------------------------------------------------------------->
        <!--
          Renders the `outputFields` array as a vertical list of cards.
          Each card displays the field name (monospace blue) and its description.
          These are the columns the user will see in the final screened Excel output.
        -->
        <section id="output" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">5. Output fields</h2>
            <div class="space-y-3">
                {#each outputFields as field}
                    <div class="rounded-2xl border border-slate-200 bg-white px-5 py-4 shadow-sm dark:border-slate-700 dark:bg-slate-800">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{field[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{field[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Section 6: Tolerance Parameters                                  -->
        <!-- ----------------------------------------------------------------->
        <!--
          Renders the `params` array as a four-column HTML table.
          Tuple indices map to columns as follows:
            row[0] → parameter name (monospace, blue)
            row[1] → default value
            row[2] → unit of measurement
            row[3] → pipeline stage that uses this parameter
        -->
        <section id="params" class="mb-10">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">6. Tolerance parameters</h2>
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

        <!-- ----------------------------------------------------------------->
        <!-- Section 7: Glossary                                              -->
        <!-- ----------------------------------------------------------------->
        <!--
          Renders the `glossary` array as a two-column card grid.
          Each card shows the term (monospace blue) and its definition.
          This provides quick reference for domain-specific terminology
          used throughout the methodology documentation.
        -->
        <section id="glossary" class="mb-10 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">7. Glossary</h2>
            <div class="grid gap-3 sm:grid-cols-2">
                {#each glossary as item}
                    <div class="rounded-xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                        <p class="font-mono text-sm font-semibold text-blue-700 dark:text-blue-400">{item[0]}</p>
                        <p class="mt-1 text-sm text-slate-600 dark:text-slate-400">{item[1]}</p>
                    </div>
                {/each}
            </div>
        </section>

        <!-- ----------------------------------------------------------------->
        <!-- Section 8: References                                            -->
        <!-- ----------------------------------------------------------------->
        <!--
          Renders the `refs` array as a vertical list of outbound links.
          Each link opens in a new tab (target="_blank") with security
          attributes (rel="noopener noreferrer") to prevent tab-napping.
        -->
        <section id="references" class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <h2 class="mb-4 text-2xl font-semibold text-slate-900 dark:text-slate-50">8. References</h2>
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
