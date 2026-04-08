<script lang="ts">
    import { onMount } from "svelte";
    import Dashboard from "$lib/components/Dashboard.svelte";
    import {
        screenFile,
        exportToXlsx,
        isServerMode,
        setServerMode,
    } from "$lib/screening";
    import type { ScreeningResult } from "$lib/screening";
    import { Download, Loader2, Upload, Server } from "lucide-svelte";
    import { base } from "$app/paths";

    let dashboardProps: any = $state(null);
    let loading = $state(false);
    let exporting = $state(false);
    let exportingHtml = $state(false);
    let error = $state("");
    let currentFile: File | null = $state(null);
    let cachedResult: ScreeningResult | null = $state(null);
    let serverMode = $state(isServerMode());
    let screeningParams = $state({
        replicate_rt_tol: 0.1,
        replicate_mz_tol: 0.3,
        replicate_mz_mode: "da",
        blank_rt_tol: 0.1,
        blank_mz_tol: 0.3,
        blank_mz_mode: "da",
        signal_to_blank_min: 3,
    });

    const isOnline = () =>
        typeof navigator === "undefined" ? true : navigator.onLine;

    let onlineStatus = $state(isOnline());
    let updateReady = $state(false);

    onMount(() => {
        const syncOnlineStatus = () => {
            onlineStatus = isOnline();
        };

        const handleSwMessage = (event: MessageEvent) => {
            if (event.data?.type === "SW_UPDATED") {
                updateReady = true;
            }
        };

        navigator.serviceWorker?.addEventListener("message", handleSwMessage);

        window.addEventListener("online", syncOnlineStatus);
        window.addEventListener("offline", syncOnlineStatus);

        return () => {
            window.removeEventListener("online", syncOnlineStatus);
            window.removeEventListener("offline", syncOnlineStatus);
            navigator.serviceWorker?.removeEventListener(
                "message",
                handleSwMessage,
            );
        };
    });

    function toggleServerMode() {
        if (!serverMode && !isOnline()) {
            error =
                "Server mode requires a network connection. Stay in WASM mode for full offline use.";
            return;
        }
        serverMode = !serverMode;
        setServerMode(serverMode);
        if (!serverMode) {
            error = "";
        }
    }

    function appendScreeningParams(formData: FormData) {
        for (const [key, value] of Object.entries(screeningParams)) {
            formData.append(key, String(value));
        }
    }

    async function handleUpload(event: Event) {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (!file) return;

        loading = true;
        error = "";
        dashboardProps = null;
        cachedResult = null;
        currentFile = file;

        try {
            if (!serverMode) {
                // ── WASM path (default) ──────────────────────────────────
                const result = await screenFile(file, screeningParams);
                cachedResult = result;
                dashboardProps = result.dashboardProps;
                if (result.dashboardProps.parameters) {
                    screeningParams = {
                        ...screeningParams,
                        ...result.dashboardProps.parameters,
                    };
                }
            } else {
                // ── Server path (fallback) ───────────────────────────────
                if (!isOnline()) {
                    throw new Error(
                        "You are offline. Disable Server mode to process files with local WASM.",
                    );
                }
                const formData = new FormData();
                formData.append("file", file);
                appendScreeningParams(formData);

                const response = await fetch(
                    "http://localhost:8000/api/screen",
                    {
                        method: "POST",
                        body: formData,
                    },
                );
                if (!response.ok) {
                    const err = await response
                        .json()
                        .catch(() => ({ detail: response.statusText }));
                    throw new Error(err.detail || "Server error");
                }
                const spec = await response.json();
                const root = spec.elements?.[spec.root];
                dashboardProps = root?.props ?? spec;
                if (dashboardProps?.parameters) {
                    screeningParams = {
                        ...screeningParams,
                        ...dashboardProps.parameters,
                    };
                }
            }
        } catch (e: any) {
            error =
                (typeof e === "string" ? e : e.message) ||
                "Failed to process file";
            console.error(e);
        } finally {
            loading = false;
        }
    }

    async function handleExport() {
        if (!currentFile) return;
        exporting = true;
        try {
            if (!serverMode && cachedResult) {
                await exportToXlsx(
                    cachedResult,
                    screeningParams,
                    currentFile.name,
                );
            } else {
                if (!isOnline()) {
                    throw new Error(
                        "You are offline. Disable Server mode to export with local WASM.",
                    );
                }
                const formData = new FormData();
                formData.append("file", currentFile);
                appendScreeningParams(formData);
                const response = await fetch(
                    "http://localhost:8000/api/export",
                    {
                        method: "POST",
                        body: formData,
                    },
                );
                if (!response.ok) {
                    const err = await response
                        .json()
                        .catch(() => ({ detail: response.statusText }));
                    throw new Error(err.detail || "Export failed");
                }
                const blob = await response.blob();
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                const stem = currentFile.name.replace(/\.[^.]+$/, "");
                a.href = url;
                a.download = `${stem}_screened.xlsx`;
                a.click();
                URL.revokeObjectURL(url);
            }
        } catch (e: any) {
            error = (typeof e === "string" ? e : e.message) || "Export failed";
        } finally {
            exporting = false;
        }
    }

    async function handleExportHtml() {
        if (!currentFile) return;
        exportingHtml = true;
        try {
            if (!serverMode) {
                const result = await screenFile(currentFile, screeningParams);
                const peaks = result.allPeaks ?? [];
                const summary = result.allSummary ?? [];
                const stem = currentFile.name.replace(/\.[^.]+$/, "");

                const peaksHtml = peaks
                    .map(
                        (p: any) =>
                            `<tr><td>${p.RT_mean}</td><td>${p.MZ_mean}</td><td>${p.Area_mean}</td><td>${p.Polarity}</td><td>${p.SampleType}</td><td>${p.Status}</td><td>${p.ConfidenceScore}</td><td>${p.AreaCVPct ?? ""}</td><td>${p.SignalToBlankRatio ?? ""}</td></tr>`,
                    )
                    .join("");
                const summaryHtml = summary
                    .map(
                        (s: any) =>
                            `<tr><td>${s.Sample}</td><td>${s.Polarity}</td><td>${s.TotalPeaks}</td><td>${s.Confirmed}</td><td>${s.RealCompounds}</td><td>${s.Artifacts}</td></tr>`,
                    )
                    .join("");

                const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${stem} — LC-MS Screening</title>
<style>body{font-family:system-ui,sans-serif;padding:2rem}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:.4rem .6rem;font-size:.8rem}th{background:#1e40af;color:#fff}tr:nth-child(even){background:#f8fafc}</style>
</head><body>
<h1>LC-MS Screening — ${stem}</h1>
<h2>Summary</h2><table><tr><th>Sample</th><th>Polarity</th><th>Total</th><th>Confirmed</th><th>Real</th><th>Artifact</th></tr>${summaryHtml}</table>
<h2>Screened Peaks</h2><table><tr><th>RT</th><th>m/z</th><th>Area</th><th>Polarity</th><th>Sample</th><th>Status</th><th>Confidence</th><th>CV%</th><th>S/B</th></tr>${peaksHtml}</table>
</body></html>`;

                const blob = new Blob([html], { type: "text/html" });
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                a.href = url;
                a.download = `${stem}_screened_offline.html`;
                a.click();
                URL.revokeObjectURL(url);
            } else {
                if (!isOnline()) {
                    throw new Error(
                        "You are offline. Disable Server mode to export HTML with local WASM.",
                    );
                }
                const formData = new FormData();
                formData.append("file", currentFile);
                appendScreeningParams(formData);
                const response = await fetch(
                    "http://localhost:8000/api/export/html",
                    {
                        method: "POST",
                        body: formData,
                    },
                );
                if (!response.ok) {
                    const err = await response
                        .json()
                        .catch(() => ({ detail: response.statusText }));
                    throw new Error(err.detail || "HTML export failed");
                }
                const blob = await response.blob();
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                const stem = currentFile.name.replace(/\.[^.]+$/, "");
                a.href = url;
                a.download = `${stem}_screened_offline.html`;
                a.click();
                URL.revokeObjectURL(url);
            }
        } catch (e: any) {
            error =
                (typeof e === "string" ? e : e.message) || "HTML export failed";
        } finally {
            exportingHtml = false;
        }
    }
</script>

<svelte:head>
    <title>LC-MS Screening</title>
</svelte:head>

{#if updateReady}
    <div
        class="fixed inset-x-0 top-0 z-50 flex items-center justify-between gap-4 bg-blue-600 px-6 py-3 text-sm text-white shadow-md"
    >
        <span>Нова версія доступна.</span>
        <button
            class="rounded-full bg-white px-4 py-1.5 text-xs font-semibold text-blue-700 hover:bg-blue-50"
            onclick={() => window.location.reload()}
        >
            Оновити
        </button>
    </div>
{/if}

<main class="min-h-screen bg-slate-50 dark:bg-slate-900" class:pt-12={updateReady}>
    <div class="px-8 py-12">
        {#if !dashboardProps}
            <div
                class="mx-auto grid max-w-5xl gap-8 rounded-[2rem] border border-slate-200 bg-white p-8 shadow-sm dark:border-slate-700 dark:bg-slate-900 lg:grid-cols-[1.15fr_0.85fr]"
            >
                <section
                    class="rounded-3xl border-2 border-dashed border-slate-200 bg-slate-50 p-12 text-center transition-all hover:border-blue-400 dark:border-slate-700 dark:bg-slate-800"
                >
                    <div
                        class="mb-6 inline-flex h-16 w-16 items-center justify-center rounded-full bg-blue-50 text-blue-600 dark:bg-blue-950"
                    >
                        <Upload class="h-8 w-8" />
                    </div>
                    <h1 class="text-2xl font-bold text-slate-900 dark:text-slate-50">
                        LC-MS Screening
                    </h1>
                    <div class="mt-4 flex items-center justify-center gap-2">
                        <span
                            class={[
                                "rounded-full px-3 py-1 text-xs font-medium",
                                onlineStatus
                                    ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
                                    : "bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300",
                            ].join(" ")}
                        >
                            {onlineStatus ? "Online" : "Offline"}
                        </span>
                    </div>
                    <p class="mt-3 text-slate-500 dark:text-slate-400">
                        Upload an Excel (.xlsx) file to begin screening peaks.
                    </p>
                    <a
                        href={import.meta.env.VITE_STANDALONE
                            ? "./methodology/"
                            : `${base}/methodology`}
                        data-sveltekit-reload={import.meta.env.VITE_STANDALONE
                            ? ""
                            : undefined}
                        class="mt-3 inline-flex items-center gap-1 text-xs text-blue-500 hover:text-blue-700 hover:underline dark:text-blue-400 dark:hover:text-blue-300"
                    >
                        <svg
                            class="h-3.5 w-3.5"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <circle cx="12" cy="12" r="10" /><path
                                d="M12 16v-4m0-4h.01"
                            />
                        </svg>
                        Про методологію скрінінгу
                    </a>

                    <label class="mt-8 block">
                        <span class="sr-only">Choose file</span>
                        <input
                            type="file"
                            accept=".xlsx,.xls"
                            class="block w-full cursor-pointer text-sm text-slate-500 dark:text-slate-400 file:mr-4 file:rounded-full file:border-0 file:bg-blue-600 file:px-4 file:py-2 file:text-sm file:font-semibold file:text-white hover:file:bg-blue-700"
                            onchange={handleUpload}
                            disabled={loading}
                        />
                    </label>

                    {#if loading}
                        <div
                            class="mt-6 flex items-center justify-center text-blue-600"
                        >
                            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
                            <span>Processing data...</span>
                        </div>
                    {/if}

                    {#if error}
                        <div
                            class="mt-6 rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-950 dark:text-red-400"
                        >
                            {error}
                        </div>
                    {/if}
                </section>

                <section
                    class="rounded-3xl border border-slate-200 bg-slate-50 p-6 dark:border-slate-700 dark:bg-slate-800"
                >
                    <div class="flex items-start justify-between gap-4">
                        <div>
                            <p
                                class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500"
                            >
                                Screening Parameters
                            </p>
                            <h2
                                class="mt-2 text-xl font-semibold text-slate-900 dark:text-slate-50"
                            >
                                Pharma / QC controls
                            </h2>
                        </div>
                        <div class="flex flex-col items-end gap-1.5">
                            <span
                                class="rounded-full bg-white px-3 py-1 text-xs font-medium text-slate-500 shadow-sm dark:bg-slate-700 dark:text-slate-400"
                                >Audit-ready</span
                            >
                            <button
                                onclick={toggleServerMode}
                                title={serverMode
                                    ? "Server mode ON — click to switch to WASM"
                                    : "WASM mode — click to use Python server"}
                                class={[
                                    "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium transition-colors",
                                    serverMode
                                        ? "bg-amber-100 text-amber-700 hover:bg-amber-200 dark:bg-amber-900 dark:text-amber-300 dark:hover:bg-amber-800"
                                        : "bg-emerald-50 text-emerald-700 hover:bg-emerald-100 dark:bg-emerald-950 dark:text-emerald-300 dark:hover:bg-emerald-900",
                                ].join(" ")}
                            >
                                <Server class="h-3 w-3" />
                                {serverMode ? "Server mode" : "WASM"}
                            </button>
                        </div>
                    </div>

                    <div class="mt-6 grid gap-4 sm:grid-cols-2">
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Replicate RT tolerance</span
                            >
                            <input
                                bind:value={screeningParams.replicate_rt_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Replicate m/z tolerance</span
                            >
                            <input
                                bind:value={screeningParams.replicate_mz_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Replicate m/z mode</span
                            >
                            <select
                                bind:value={screeningParams.replicate_mz_mode}
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            >
                                <option value="da">Da</option>
                                <option value="ppm">ppm</option>
                            </select>
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Blank RT tolerance</span
                            >
                            <input
                                bind:value={screeningParams.blank_rt_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Blank m/z tolerance</span
                            >
                            <input
                                bind:value={screeningParams.blank_mz_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Blank m/z mode</span
                            >
                            <select
                                bind:value={screeningParams.blank_mz_mode}
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            >
                                <option value="da">Da</option>
                                <option value="ppm">ppm</option>
                            </select>
                        </label>
                        <label class="space-y-1.5 text-sm sm:col-span-2">
                            <span class="font-medium text-slate-700 dark:text-slate-300"
                                >Signal-to-blank minimum</span
                            >
                            <input
                                bind:value={screeningParams.signal_to_blank_min}
                                type="number"
                                min="0"
                                step="0.1"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            />
                        </label>
                    </div>

                    <div
                        class="mt-5 rounded-2xl border border-blue-100 bg-blue-50 px-4 py-3 text-sm text-blue-900 dark:border-blue-800 dark:bg-blue-950 dark:text-blue-200"
                    >
                        Replicate matching і blank subtraction налаштовуються
                        окремо. Для high-resolution LC-MS обирай `ppm`, для
                        legacy workbook з грубим `m/z` округленням лишай `Da`.
                    </div>
                </section>
            </div>
        {:else}
            <div class="mb-8 flex items-center justify-between gap-3 px-8">
                <div class="flex items-center gap-2">
                    <span
                        class={[
                            "rounded-full px-3 py-1 text-xs font-medium",
                            onlineStatus
                                ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
                                : "bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300",
                        ].join(" ")}
                    >
                        {onlineStatus ? "Online" : "Offline"}
                    </span>
                </div>
                <div class="flex items-center gap-3">
                    <a
                        href={import.meta.env.VITE_STANDALONE
                            ? "./methodology/"
                            : `${base}/methodology`}
                        data-sveltekit-reload={import.meta.env.VITE_STANDALONE
                            ? ""
                            : undefined}
                        class="inline-flex items-center rounded-full border bg-white px-4 py-2 text-sm font-medium text-slate-600 shadow-sm hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
                    >
                        <svg
                            class="mr-2 h-4 w-4"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <circle cx="12" cy="12" r="10" /><path
                                d="M12 16v-4m0-4h.01"
                            />
                        </svg>
                        Методологія
                    </a>
                    <button
                        class="inline-flex items-center rounded-full bg-green-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-green-700 disabled:opacity-50"
                        onclick={handleExport}
                        disabled={exporting}
                    >
                        {#if exporting}
                            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                            Експорт...
                        {:else}
                            <Download class="mr-2 h-4 w-4" />
                            Експорт .xlsx
                        {/if}
                    </button>
                    <button
                        class="inline-flex items-center rounded-full bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
                        onclick={handleExportHtml}
                        disabled={exportingHtml}
                    >
                        {#if exportingHtml}
                            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                            Експорт...
                        {:else}
                            <Download class="mr-2 h-4 w-4" />
                            Offline HTML
                        {/if}
                    </button>
                    <button
                        class="inline-flex items-center rounded-full border bg-white px-4 py-2 text-sm font-medium text-slate-900 shadow-sm hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100 dark:hover:bg-slate-700"
                        onclick={() => {
                            dashboardProps = null;
                            cachedResult = null;
                            currentFile = null;
                        }}
                    >
                        <Upload class="mr-2 h-4 w-4" /> Upload New File
                    </button>
                </div>
            </div>

            <Dashboard
                title={dashboardProps.title}
                summary={dashboardProps.summary}
                peaks={dashboardProps.peaks}
                parameters={dashboardProps.parameters}
                metadata={dashboardProps.metadata}
            />
        {/if}
    </div>
</main>

<style>
    :global(body) {
        font-family:
            "Inter",
            system-ui,
            -apple-system,
            sans-serif;
    }
</style>
