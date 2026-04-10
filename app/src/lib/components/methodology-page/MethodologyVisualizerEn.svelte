<!--
  @file MethodologyVisualizer.svelte
  @description
  Interactive visual overview of the LC-MS screening pipeline.
  Contains three sections:
    1. Flow diagram — horizontal step cards connected by arrows
    2. Interactive walkthrough — step-by-step stepper with input/action/output details
    3. Decision logic cards — condition → result rules for Artifact / Real Compound

  Uses Svelte 5 runes ($state, $derived) and native Tailwind dark-mode classes.
  No external runtime dependencies (Mermaid, D3, etc.) — pure HTML/SVG/CSS.
-->
<script lang="ts">
    // ---------------------------------------------------------------------------
    // Flow Steps — Top horizontal diagram
    // ---------------------------------------------------------------------------
    const flowSteps = [
        {
            id: "excel",
            title: "Excel input",
            short: "Workbook with LC-MS peaks",
            tone: "slate"
        },
        {
            id: "validate",
            title: "Validate rows",
            short: "Remove incomplete rows",
            tone: "amber"
        },
        {
            id: "classify",
            title: "Assign roles",
            short: "Sample / Blank / Replicate",
            tone: "violet"
        },
        {
            id: "replicates",
            title: "Confirm in replicates",
            short: "Check RT and m/z match",
            tone: "blue"
        },
        {
            id: "blank",
            title: "Compare with blank",
            short: "Find background signal",
            tone: "cyan"
        },
        {
            id: "decision",
            title: "Final decision",
            short: "Artifact or Real Compound",
            tone: "green"
        },
        {
            id: "output",
            title: "Audit-ready output",
            short: "Metrics + Why trail",
            tone: "rose"
        }
    ];

    // ---------------------------------------------------------------------------
    // Walkthrough Steps — Interactive stepper content
    // ---------------------------------------------------------------------------
    const walkthroughSteps = [
        {
            title: "1. Input Excel data",
            summary:
                "The application loads the Excel workbook and automatically selects the sheet that best matches the required columns.",
            input: ["Workbook", "One or more sheets"],
            action: ["Detect required headers", "Choose the best matching sheet"],
            output: ["Structured LC-MS peak rows"],
            highlight: "The user does not need to manually pick the correct sheet in a typical case."
        },
        {
            title: "2. Validate rows",
            summary:
                "Rows without RT, Base Peak, or Area are removed because they cannot be screened reliably.",
            input: ["Peak rows from Excel"],
            action: ["Check RT", "Check Base Peak", "Check Area"],
            output: ["Only usable rows remain"],
            highlight: "This step removes incomplete data before any scientific decision is made."
        },
        {
            title: "3. Assign row roles",
            summary:
                "Each row is classified as sample or blank. Operator color marks have priority over file-name heuristics.",
            input: ["Valid rows", "Excel cell colors", "File names"],
            action: ["Read operator mark", "Fallback to file name logic if needed"],
            output: ["Sample / Blank / Replicate assignment"],
            highlight: "Explicit operator marks override assumptions from file naming."
        },
        {
            title: "4. Confirm peaks across replicates",
            summary:
                "A signal becomes more trustworthy when it appears in more than one replicate within configured RT and m/z tolerance.",
            input: ["Rows grouped by polarity and replicate"],
            action: ["Compare RT", "Compare m/z", "Apply one-peak-per-bucket rule"],
            output: ["Confirmed replicate cluster"],
            highlight: "The system checks that the same compound is seen repeatedly, not just once."
        },
        {
            title: "5. Compare with blank",
            summary:
                "Each confirmed sample peak is compared against blank peaks of the same polarity to detect background or contamination.",
            input: ["Confirmed sample cluster", "Blank peaks"],
            action: ["Find blank match", "Calculate signal-to-blank ratio"],
            output: ["Blank-associated or clean signal"],
            highlight: "This is the main protection against false positives caused by background signal."
        },
        {
            title: "6. Final decision",
            summary:
                "If the blank signal is too strong relative to the sample signal, the result is marked as Artifact. Otherwise it is a Real Compound.",
            input: ["Signal-to-Blank ratio", "Decision threshold"],
            action: ["Compare ratio against threshold"],
            output: ["Artifact or Real Compound"],
            highlight: "This is the business outcome that most non-technical users care about."
        },
        {
            title: "7. Audit-ready output",
            summary:
                "The application computes final metrics and stores the decision explanation in the Why field for review and audit.",
            input: ["Confirmed decision"],
            action: ["Compute means", "Compute CV%", "Write Why trail"],
            output: ["Final result table"],
            highlight: "Every important result can be explained later."
        }
    ];

    // ---------------------------------------------------------------------------
    // Decision Cards — Condition → Result rules
    // ---------------------------------------------------------------------------
    const decisionCards = [
        {
            title: "Missing required peak values",
            condition: "RT, Base Peak, or Area is missing",
            result: "Discard row",
            kind: "bad"
        },
        {
            title: "Operator mark is present",
            condition: "Excel color code matches a known operator mark",
            result: "Use explicit role from mark",
            kind: "info"
        },
        {
            title: "No operator mark",
            condition: "No known color mark is present",
            result: "Use file-name fallback logic",
            kind: "info"
        },
        {
            title: "Replicate confirmation failed",
            condition: "RT or m/z is outside replicate tolerance",
            result: "Peak is not confirmed",
            kind: "bad"
        },
        {
            title: "Blank match found and ratio too low",
            condition: "Blank match exists and S/B is below threshold",
            result: "Artifact",
            kind: "bad"
        },
        {
            title: "No blank problem",
            condition: "No blank match or acceptable S/B ratio",
            result: "Real Compound",
            kind: "good"
        }
    ];

    // ---------------------------------------------------------------------------
    // Typical Example — Concrete walkthrough of one peak
    // ---------------------------------------------------------------------------
    const typicalExample = {
        samplePeak: {
            rt: 2.35,
            mz: 195.08,
            area: 1250000,
            polarity: "positive",
            file: "1_pos.d"
        },
        replicateMatch: {
            rt: 2.36,
            mz: 195.09,
            area: 1180000,
            polarity: "positive",
            file: "2_pos.d"
        },
        blankPeak: {
            rt: 2.34,
            mz: 195.07,
            area: 85000,
            polarity: "positive",
            file: "blank_pos.d"
        },
        results: {
            rtMean: 2.355,
            mzMean: 195.085,
            areaMean: 1215000,
            cvPct: 4.1,
            signalToBlank: 14.3,
            status: "Real Compound" as const
        }
    };

    // ---------------------------------------------------------------------------
    // Component State (Svelte 5 runes)
    // ---------------------------------------------------------------------------
    let current = $state(0);
    let step = $derived(walkthroughSteps[current]);

    // ---------------------------------------------------------------------------
    // Navigation helpers
    // ---------------------------------------------------------------------------
    function next() {
        if (current < walkthroughSteps.length - 1) current += 1;
    }

    function prev() {
        if (current > 0) current -= 1;
    }

    function jumpTo(index: number) {
        current = index;
    }

    // ---------------------------------------------------------------------------
    // Style helpers
    // ---------------------------------------------------------------------------
    function toneClasses(tone: string) {
        switch (tone) {
            case "amber":
                return "border-amber-200 bg-amber-50 text-amber-900 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-200";
            case "violet":
                return "border-violet-200 bg-violet-50 text-violet-900 dark:border-violet-900 dark:bg-violet-950 dark:text-violet-200";
            case "blue":
                return "border-blue-200 bg-blue-50 text-blue-900 dark:border-blue-900 dark:bg-blue-950 dark:text-blue-200";
            case "cyan":
                return "border-cyan-200 bg-cyan-50 text-cyan-900 dark:border-cyan-900 dark:bg-cyan-950 dark:text-cyan-200";
            case "green":
                return "border-green-200 bg-green-50 text-green-900 dark:border-green-900 dark:bg-green-950 dark:text-green-200";
            case "rose":
                return "border-rose-200 bg-rose-50 text-rose-900 dark:border-rose-900 dark:bg-rose-950 dark:text-rose-200";
            default:
                return "border-slate-200 bg-slate-50 text-slate-900 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200";
        }
    }

    function cardClasses(kind: string) {
        switch (kind) {
            case "good":
                return "border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950";
            case "bad":
                return "border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950";
            default:
                return "border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950";
        }
    }

    function badgeClasses(kind: string) {
        switch (kind) {
            case "good":
                return "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200";
            case "bad":
                return "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200";
            default:
                return "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200";
        }
    }
</script>

<!-- ======================================================================= -->
<!-- In one sentence                                                         -->
<!-- ======================================================================= -->
<section class="mb-8 rounded-3xl border border-blue-200 bg-gradient-to-r from-blue-50 to-indigo-50 p-6 shadow-sm dark:border-blue-900 dark:from-blue-950 dark:to-indigo-950">
    <div class="flex items-start gap-4">
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-blue-100 text-blue-600 dark:bg-blue-900/60 dark:text-blue-300">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 16v-4" />
                <path d="M12 8h.01" />
            </svg>
        </div>
        <div>
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-blue-500 dark:text-blue-400">
                In one sentence
            </p>
            <p class="mt-2 text-base leading-8 text-blue-900 dark:text-blue-100">
                The system reads raw LC-MS peaks from Excel, validates them, confirms each signal across replicates,
                compares it against the blank, and labels the result as <strong>Real Compound</strong> or
                <strong>Artifact</strong> — with a full audit trail explaining why.
            </p>
        </div>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Flow Diagram — Horizontal step cards connected by arrows                -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-6">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Simple visual overview
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            How the screening works
        </h2>
        <p class="mt-3 max-w-3xl text-sm leading-7 text-slate-600 dark:text-slate-400">
            The system takes raw LC-MS peaks from Excel, removes incomplete rows, confirms signals across replicates,
            checks them against the blank, and then produces a final result with an explanation.
        </p>
    </div>

    <div class="overflow-x-auto pb-2">
        <div class="flex min-w-[980px] items-stretch gap-3">
            {#each flowSteps as item, i}
                <div class="flex w-[170px] shrink-0 flex-col rounded-2xl border p-4 {toneClasses(item.tone)}">
                    <div class="mb-3 flex items-center justify-between">
                        <span class="text-xs font-semibold uppercase tracking-wide opacity-70">
                            Step {i + 1}
                        </span>
                        <span class="rounded-full bg-white/70 px-2 py-0.5 text-[10px] font-semibold dark:bg-black/20">
                            {item.id}
                        </span>
                    </div>

                    <p class="text-sm font-semibold">{item.title}</p>
                    <p class="mt-2 text-xs leading-6 opacity-80">{item.short}</p>
                </div>

                {#if i < flowSteps.length - 1}
                    <div class="flex shrink-0 items-center justify-center px-1 text-slate-300 dark:text-slate-600">
                        <svg width="32" height="24" viewBox="0 0 32 24" fill="none" aria-hidden="true">
                            <path d="M2 12H26" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
                            <path d="M20 6L26 12L20 18" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                    </div>
                {/if}
            {/each}
        </div>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Interactive Walkthrough — Step-by-step stepper                          -->
<!-- ======================================================================= -->
<section class="mb-10">
    <div class="mb-4">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Interactive walkthrough
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Step-by-step explanation
        </h2>
    </div>

    <!-- Progress bar -->
    <div class="mb-6 h-2 overflow-hidden rounded-full bg-slate-200 dark:bg-slate-800">
        <div
            class="h-full rounded-full bg-blue-600 transition-all duration-300"
            style="width:{((current + 1) / walkthroughSteps.length) * 100}%"
        ></div>
    </div>

    <div class="grid gap-6 lg:grid-cols-[1.15fr_0.85fr]">
        <!-- Main content panel -->
        <section class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <div class="flex flex-wrap items-center gap-3">
                <span class="rounded-full bg-blue-100 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-blue-800 dark:bg-blue-950 dark:text-blue-200">
                    Current step
                </span>
                <p class="text-sm font-semibold text-slate-500 dark:text-slate-400">
                    {step.title}
                </p>
            </div>

            <p class="mt-5 text-base leading-8 text-slate-700 dark:text-slate-300">
                {step.summary}
            </p>

            <div class="mt-6 rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                <p class="text-xs font-semibold uppercase tracking-[0.15em] text-slate-400 dark:text-slate-500">
                    Why this matters
                </p>
                <p class="mt-2 text-sm leading-7 text-slate-600 dark:text-slate-400">
                    {step.highlight}
                </p>
            </div>

            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <!-- Input column -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Input
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.input as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Action column -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        What the system does
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.action as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Output column -->
                <div class="rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-900">
                    <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                        Output
                    </p>
                    <div class="mt-3 space-y-2">
                        {#each step.output as item}
                            <div class="rounded-xl bg-white px-3 py-2 text-sm text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                                {item}
                            </div>
                        {/each}
                    </div>
                </div>
            </div>
        </section>

        <!-- Sidebar navigation -->
        <aside class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
                Navigate steps
            </p>

            <div class="mt-4 space-y-3">
                {#each walkthroughSteps as s, i}
                    <button
                        class="w-full rounded-2xl border px-4 py-3 text-left transition {i === current
                            ? 'border-blue-500 bg-blue-50 text-blue-900 dark:border-blue-500 dark:bg-blue-950 dark:text-blue-200'
                            : 'border-slate-200 bg-slate-50 text-slate-600 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-400'}"
                        onclick={() => jumpTo(i)}
                    >
                        <span class="block text-sm font-semibold">{s.title}</span>
                    </button>
                {/each}
            </div>

            <div class="mt-6 flex gap-3">
                <button
                    onclick={prev}
                    disabled={current === 0}
                    class="rounded-2xl border border-slate-300 px-4 py-2 text-sm text-slate-700 disabled:opacity-40 dark:border-slate-600 dark:text-slate-300"
                >
                    Previous
                </button>

                <button
                    onclick={next}
                    disabled={current === walkthroughSteps.length - 1}
                    class="rounded-2xl bg-blue-600 px-4 py-2 text-sm text-white disabled:opacity-40"
                >
                    Next
                </button>
            </div>
        </aside>
    </div>
</section>

<!-- ======================================================================= -->
<!-- Decision Logic Cards                                                    -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Decision logic
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            Why a result becomes Artifact or Real Compound
        </h2>
    </div>

    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {#each decisionCards as card}
            <div class="rounded-2xl border p-4 {cardClasses(card.kind)}">
                <div class="flex items-center justify-between gap-3">
                    <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">
                        {card.title}
                    </p>
                    <span class="rounded-full px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wide {badgeClasses(card.kind)}">
                        {card.kind}
                    </span>
                </div>

                <div class="mt-4 space-y-3">
                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Condition
                        </p>
                        <p class="mt-1 text-sm leading-7 text-slate-700 dark:text-slate-300">
                            {card.condition}
                        </p>
                    </div>

                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-slate-400 dark:text-slate-500">
                            Result
                        </p>
                        <p class="mt-1 text-sm font-semibold text-slate-900 dark:text-slate-100">
                            {card.result}
                        </p>
                    </div>
                </div>
            </div>
        {/each}
    </div>
</section>

<!-- ======================================================================= -->
<!-- Typical Example — Concrete walkthrough of one peak                      -->
<!-- ======================================================================= -->
<section class="mb-10 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-800">
    <div class="mb-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400 dark:text-slate-500">
            Typical example
        </p>
        <h2 class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-50">
            One peak, from raw signal to final verdict
        </h2>
        <p class="mt-2 max-w-3xl text-sm leading-7 text-slate-600 dark:text-slate-400">
            Below is a concrete example showing how one LC-MS peak travels through the pipeline —
            from the initial sample measurement, through replicate confirmation and blank comparison,
            to the final classification.
        </p>
    </div>

    <div class="grid gap-4 lg:grid-cols-3">
        <!-- Sample peak -->
        <div class="rounded-2xl border border-violet-200 bg-violet-50 p-5 dark:border-violet-900 dark:bg-violet-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-violet-200 text-xs font-bold text-violet-800 dark:bg-violet-800 dark:text-violet-100">S</span>
                <p class="text-sm font-semibold text-violet-900 dark:text-violet-200">Sample peak</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.rt} min</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Area</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.samplePeak.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">File</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.samplePeak.file}</span>
                </div>
            </div>
        </div>

        <!-- Replicate match -->
        <div class="rounded-2xl border border-blue-200 bg-blue-50 p-5 dark:border-blue-900 dark:bg-blue-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-blue-200 text-xs font-bold text-blue-800 dark:bg-blue-800 dark:text-blue-100">R</span>
                <p class="text-sm font-semibold text-blue-900 dark:text-blue-200">Replicate match</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.rt} min</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Area</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.replicateMatch.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">File</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.replicateMatch.file}</span>
                </div>
            </div>
            <div class="mt-3 rounded-xl border border-blue-200 bg-blue-100/60 px-3 py-2 text-xs text-blue-700 dark:border-blue-800 dark:bg-blue-900/40 dark:text-blue-300">
                ΔRT = 0.01 min ✓ &nbsp;·&nbsp; Δm/z = 0.01 Da ✓
            </div>
        </div>

        <!-- Blank peak -->
        <div class="rounded-2xl border border-cyan-200 bg-cyan-50 p-5 dark:border-cyan-900 dark:bg-cyan-950">
            <div class="mb-3 flex items-center gap-2">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-cyan-200 text-xs font-bold text-cyan-800 dark:bg-cyan-800 dark:text-cyan-100">B</span>
                <p class="text-sm font-semibold text-cyan-900 dark:text-cyan-200">Blank peak</p>
            </div>
            <div class="space-y-2 text-sm">
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">RT</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.rt} min</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">m/z</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.mz}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">Area</span>
                    <span class="font-mono font-semibold text-slate-800 dark:text-slate-200">{typicalExample.blankPeak.area.toLocaleString()}</span>
                </div>
                <div class="flex justify-between rounded-xl bg-white/70 px-3 py-2 dark:bg-black/20">
                    <span class="text-slate-500 dark:text-slate-400">File</span>
                    <span class="font-mono text-xs text-slate-600 dark:text-slate-300">{typicalExample.blankPeak.file}</span>
                </div>
            </div>
            <div class="mt-3 rounded-xl border border-cyan-200 bg-cyan-100/60 px-3 py-2 text-xs text-cyan-700 dark:border-cyan-800 dark:bg-cyan-900/40 dark:text-cyan-300">
                Blank match found · S/B = {typicalExample.results.signalToBlank} ≥ 3.0 ✓
            </div>
        </div>
    </div>

    <!-- Final verdict -->
    <div class="mt-5 rounded-2xl border border-green-200 bg-green-50 p-5 dark:border-green-900 dark:bg-green-950">
        <div class="flex flex-wrap items-center gap-4">
            <span class="rounded-full bg-green-100 px-3 py-1 text-xs font-bold uppercase tracking-wide text-green-800 dark:bg-green-900/60 dark:text-green-200">
                {typicalExample.results.status}
            </span>
            <div class="flex flex-wrap gap-4 text-sm text-green-800 dark:text-green-300">
                <span>RT<sub>mean</sub> = {typicalExample.results.rtMean}</span>
                <span>m/z<sub>mean</sub> = {typicalExample.results.mzMean}</span>
                <span>Area<sub>mean</sub> = {typicalExample.results.areaMean.toLocaleString()}</span>
                <span>CV% = {typicalExample.results.cvPct}</span>
                <span>S/B = {typicalExample.results.signalToBlank}</span>
            </div>
        </div>
    </div>
</section>
