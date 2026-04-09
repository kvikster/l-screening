<script lang="ts">
    import { onMount } from "svelte";
    import {
        dictionary,
        getSampleTypeLabel,
        getStatusLabel,
        locale,
        localeLabels,
        setLocale,
        t,
    } from "$lib/i18n";
    import { theme, cycleTheme, themeIcons } from "$lib/theme";
    import Dashboard from "$lib/components/Dashboard.svelte";
    import {
        screenFile,
        exportToXlsx,
        isServerMode,
        setServerMode,
    } from "$lib/screening";
    import type { ScreeningResult } from "$lib/screening";
    import {
        CircleHelp,
        Loader2,
        Upload,
        Server,
    } from "lucide-svelte";
    import MethodologyPanel from "$lib/components/MethodologyPanel.svelte";

    let showMethodology = $state(false);
    let dict = $derived($dictionary);
    let currentTheme = $derived($theme);
    let currentLocale = $derived($locale);
    let themeLabels = $derived({ auto: dict.themeAuto, light: dict.themeLight, dark: dict.themeDark });

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
    let syncBlankWithReplicate = $state(true);
    let showAdvancedParams = $state(false);

    $effect(() => {
        if (!syncBlankWithReplicate) return;
        screeningParams.blank_rt_tol = screeningParams.replicate_rt_tol;
        screeningParams.blank_mz_tol = screeningParams.replicate_mz_tol;
        screeningParams.blank_mz_mode = screeningParams.replicate_mz_mode;
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
            error = t("serverModeNeedsNetwork");
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
                    throw new Error(t("offlineProcessWithWasm"));
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
                    throw new Error(err.detail || t("serverError"));
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
                t("processFileFailed");
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
                    throw new Error(t("offlineExportWithWasm"));
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
                    throw new Error(err.detail || t("exportFailed"));
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
            error = (typeof e === "string" ? e : e.message) || t("exportFailed");
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
                            `<tr><td>${p.RT_mean}</td><td>${p.MZ_mean}</td><td>${p.Area_mean}</td><td>${p.Polarity === "positive" ? dict.positive : p.Polarity === "negative" ? dict.negative : p.Polarity}</td><td>${getSampleTypeLabel(p.SampleType)}</td><td>${getStatusLabel(p.Status)}</td><td>${p.ConfidenceScore}</td><td>${p.AreaCVPct ?? ""}</td><td>${p.SignalToBlankRatio ?? ""}</td></tr>`,
                    )
                    .join("");
                const summaryHtml = summary
                    .map(
                        (s: any) =>
                            `<tr><td>${getSampleTypeLabel(s.Sample)}</td><td>${s.Polarity === "positive" ? dict.positive : s.Polarity === "negative" ? dict.negative : s.Polarity}</td><td>${s.TotalPeaks}</td><td>${s.Confirmed}</td><td>${s.RealCompounds}</td><td>${s.Artifacts}</td></tr>`,
                    )
                    .join("");

                const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${stem} — ${t("appName")}</title>
<style>body{font-family:system-ui,sans-serif;padding:2rem}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:.4rem .6rem;font-size:.8rem}th{background:#1e40af;color:#fff}tr:nth-child(even){background:#f8fafc}</style>
</head><body>
<h1>${t("appName")} — ${stem}</h1>
<h2>${t("htmlSummary")}</h2><table><tr><th>${t("htmlSample")}</th><th>${t("polarity")}</th><th>${t("htmlTotal")}</th><th>${t("htmlConfirmed")}</th><th>${t("htmlReal")}</th><th>${t("htmlArtifact")}</th></tr>${summaryHtml}</table>
<h2>${t("htmlScreenedPeaks")}</h2><table><tr><th>RT</th><th>m/z</th><th>Area</th><th>${t("polarity")}</th><th>${t("htmlSample")}</th><th>${t("htmlStatus")}</th><th>${t("htmlConfidence")}</th><th>CV%</th><th>S/B</th></tr>${peaksHtml}</table>
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
                    throw new Error(t("offlineHtmlWithWasm"));
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
                    throw new Error(err.detail || t("exportHtmlFailed"));
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
                (typeof e === "string" ? e : e.message) || t("exportHtmlFailed");
        } finally {
            exportingHtml = false;
        }
    }
</script>

<svelte:head>
    <title>{dict.appName}</title>
</svelte:head>

{#if updateReady}
    <div
        class="fixed inset-x-0 top-0 z-50 flex items-center justify-between gap-4 bg-blue-600 px-6 py-3 text-sm text-white shadow-md"
    >
        <span>{dict.updateReady}</span>
        <button
            class="rounded-full bg-white px-4 py-1.5 text-xs font-semibold text-blue-700 hover:bg-blue-50"
            onclick={() => window.location.reload()}
        >
            {dict.reload}
        </button>
    </div>
{/if}

<svelte:window onkeydown={(e) => { if (e.key === "Escape" && showMethodology) showMethodology = false; }} />

<main
    class="min-h-screen bg-slate-50 dark:bg-slate-900"
    class:pt-12={updateReady}
>
    <div class="px-6 py-8">
        {#if !dashboardProps}
            <div
                class="mx-auto grid max-w-5xl gap-5 rounded-[1.75rem] border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-700 dark:bg-slate-900 lg:grid-cols-[1.05fr_0.95fr]"
            >
                <section
                    class="rounded-[1.5rem] border-2 border-dashed border-slate-200 bg-slate-50 p-7 text-center transition-all hover:border-blue-400 dark:border-slate-700 dark:bg-slate-800"
                >
                    <img
                        src="./icons/icon-128.png"
                        alt={dict.appName}
                        class="mx-auto mb-4 h-16 w-16 rounded-2xl shadow-sm"
                    />
                    <h1
                        class="text-xl font-bold text-slate-900 dark:text-slate-50"
                    >
                        {dict.appName}
                    </h1>
                    <div class="mt-3 flex items-center justify-center gap-2">
                        <span
                            class={[
                                "rounded-full px-3 py-1 text-xs font-medium",
                                onlineStatus
                                    ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
                                    : "bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300",
                            ].join(" ")}
                        >
                            {onlineStatus ? dict.online : dict.offline}
                        </span>
                        <span class="h-4 w-px bg-slate-200 dark:bg-slate-700" aria-hidden="true"></span>
                        <label class="flex cursor-pointer items-center gap-1 text-[11px] text-slate-500 dark:text-slate-400">
                            <svg class="h-3.5 w-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                <circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                            </svg>
                            <select
                                class="bg-transparent outline-none text-[11px]"
                                value={currentLocale}
                                onchange={(e) => setLocale((e.currentTarget as HTMLSelectElement).value)}
                                aria-label={dict.languageLabel}
                            >
                                {#each Object.entries(localeLabels) as [value, label]}
                                    <option {value}>{label}</option>
                                {/each}
                            </select>
                        </label>
                        <button
                            type="button"
                            onclick={cycleTheme}
                            title={themeLabels[currentTheme]}
                            aria-label={themeLabels[currentTheme]}
                            class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-500 transition-colors hover:bg-slate-50 dark:border-slate-600 dark:bg-slate-700 dark:text-slate-300 dark:hover:bg-slate-600"
                        >
                            <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                <path d={themeIcons[currentTheme]}/>
                            </svg>
                        </button>
                    </div>
                    <p class="mt-2 text-sm text-slate-500 dark:text-slate-400">
                        {dict.uploadIntro}
                    </p>
                    <button
                        class="mt-2 inline-flex items-center gap-1 text-xs text-blue-500 hover:text-blue-700 hover:underline dark:text-blue-400 dark:hover:text-blue-300"
                        onclick={() => (showMethodology = true)}
                        type="button"
                    >
                        <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10" /><path d="M12 16v-4m0-4h.01" />
                        </svg>
                        {dict.methodologyToggle}
                    </button>

                    <label class="mt-5 block">
                        <span class="sr-only">{dict.chooseFile}</span>
                        <input
                            type="file"
                            accept=".xlsx,.xls"
                            class="block w-full cursor-pointer text-xs text-slate-500 dark:text-slate-400 file:mr-3 file:rounded-full file:border-0 file:bg-blue-600 file:px-3.5 file:py-2 file:text-xs file:font-semibold file:text-white hover:file:bg-blue-700"
                            onchange={handleUpload}
                            disabled={loading}
                        />
                    </label>

                    {#if loading}
                        <div
                            class="mt-4 flex items-center justify-center text-sm text-blue-600"
                        >
                            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                            <span>{dict.processingData}</span>
                        </div>
                    {/if}

                    {#if error}
                        <div
                            class="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-950 dark:text-red-400"
                        >
                            {error}
                        </div>
                    {/if}
                </section>

                <section
                    class="rounded-[1.5rem] border border-slate-200 bg-slate-50 p-5 dark:border-slate-700 dark:bg-slate-800"
                >
                    <div class="flex items-start justify-between gap-4">
                        <div>
                            <p
                                class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500"
                            >
                                {dict.screeningParameters}
                            </p>
                            <div class="mt-2 flex items-center gap-2">
                                <h2
                                    class="text-xl font-semibold text-slate-900 dark:text-slate-50"
                                >
                                    {dict.qcControls}
                                </h2>
                                <div class="group relative">
                                    <button
                                        type="button"
                                        aria-label={dict.settingHintTitle}
                                        class="inline-flex h-7 w-7 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-500 shadow-sm transition-colors hover:border-blue-300 hover:text-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-300 dark:hover:border-blue-500 dark:hover:text-blue-300"
                                    >
                                        <CircleHelp class="h-4 w-4" />
                                    </button>
                                    <div
                                        class="pointer-events-none absolute left-0 top-full z-20 mt-2 hidden w-[23rem] rounded-2xl border border-blue-200 bg-white p-4 text-sm text-slate-700 shadow-2xl group-hover:block group-focus-within:block dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200"
                                    >
                                        <p class="font-medium text-slate-900 dark:text-slate-50">
                                            {dict.qcControlsHint}
                                        </p>
                                        <p class="mt-2 text-xs leading-5 text-slate-600 dark:text-slate-300">
                                            {dict.qcControlsDefaultsHint}
                                        </p>
                                        <div class="mt-3 grid gap-2 text-[11px] leading-4">
                                            <div class="inline-flex items-center gap-2 rounded-xl bg-slate-50 px-2.5 py-2 dark:bg-slate-800">
                                                <span class="rounded-full bg-emerald-50 px-1.5 py-0.5 font-semibold text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">
                                                    {dict.recommended}
                                                </span>
                                                <span>{dict.qcLegendRecommended}</span>
                                            </div>
                                            <div class="inline-flex items-center gap-2 rounded-xl bg-slate-50 px-2.5 py-2 dark:bg-slate-800">
                                                <span class="rounded-full bg-slate-100 px-1.5 py-0.5 font-semibold text-slate-700 dark:bg-slate-700 dark:text-slate-300">
                                                    {dict.advanced}
                                                </span>
                                                <span>{dict.qcLegendAdvanced}</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex flex-col items-end gap-1.5">
                            <button
                                onclick={toggleServerMode}
                                title={serverMode
                                    ? dict.serverModeTitleOn
                                    : dict.serverModeTitleOff}
                                class={[
                                    "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium transition-colors",
                                    serverMode
                                        ? "bg-amber-100 text-amber-700 hover:bg-amber-200 dark:bg-amber-900 dark:text-amber-300 dark:hover:bg-amber-800"
                                        : "bg-emerald-50 text-emerald-700 hover:bg-emerald-100 dark:bg-emerald-950 dark:text-emerald-300 dark:hover:bg-emerald-900",
                                ].join(" ")}
                            >
                                <Server class="h-3 w-3" />
                                {serverMode ? dict.serverMode : dict.wasmMode}
                            </button>
                        </div>
                    </div>

                    <div class="mt-4 space-y-3">
                        <label class="space-y-1 text-sm">
                            <div class="flex items-center justify-between gap-3">
                                <span class="font-medium text-slate-700 dark:text-slate-300">{dict.signalToBlankMinimum}</span>
                                <span class="rounded-full bg-emerald-50 px-2 py-0.5 text-[11px] font-semibold text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">{dict.recommended}</span>
                            </div>
                            <input
                                bind:value={screeningParams.signal_to_blank_min}
                                type="number"
                                min="0"
                                step="0.1"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm dark:border-slate-700 dark:bg-slate-700 dark:text-slate-100"
                            />
                        </label>

                        <div class="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-700 dark:bg-slate-900">
                            <label class="inline-flex items-center gap-2 text-sm text-slate-700 dark:text-slate-300">
                                <input bind:checked={syncBlankWithReplicate} type="checkbox" class="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500 dark:border-slate-600" />
                                <span>{dict.syncReplicateBlank}</span>
                            </label>
                            <button
                                type="button"
                                class="rounded-lg border border-slate-200 px-2.5 py-1 text-xs font-medium text-slate-600 hover:bg-slate-50 dark:border-slate-600 dark:text-slate-300 dark:hover:bg-slate-800"
                                onclick={() => (showAdvancedParams = !showAdvancedParams)}
                            >
                                {showAdvancedParams ? dict.hideAdvancedParams : dict.showAdvancedParams}
                            </button>
                        </div>

                        <div class="rounded-xl border border-slate-200 bg-white p-3 dark:border-slate-700 dark:bg-slate-900">
                            <div class="grid grid-cols-[1fr_1fr_1fr] items-center gap-2 text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">
                                <div>{dict.parameterColumn}</div>
                                <div>{dict.replicateColumn}</div>
                                <div>{dict.blankColumn}</div>
                            </div>

                            <div class="mt-2 grid grid-cols-[1fr_1fr_1fr] items-center gap-2">
                                <div class="text-sm font-medium text-slate-700 dark:text-slate-300">{dict.rtToleranceShort}</div>
                                <input
                                    bind:value={screeningParams.replicate_rt_tol}
                                    type="number"
                                    min="0"
                                    step="0.01"
                                    class="w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-sm text-slate-900 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                                />
                                {#if syncBlankWithReplicate}
                                    <div class="rounded-lg border border-dashed border-slate-300 bg-slate-50 px-2.5 py-1.5 text-sm text-slate-500 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-400">{dict.synced}</div>
                                {:else}
                                    <input
                                        bind:value={screeningParams.blank_rt_tol}
                                        type="number"
                                        min="0"
                                        step="0.01"
                                        class="w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-sm text-slate-900 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                                    />
                                {/if}
                            </div>

                            <div class="mt-2 grid grid-cols-[1fr_1fr_1fr] items-center gap-2">
                                <div class="text-sm font-medium text-slate-700 dark:text-slate-300">{dict.mzToleranceShort}</div>
                                <input
                                    bind:value={screeningParams.replicate_mz_tol}
                                    type="number"
                                    min="0"
                                    step="0.01"
                                    class="w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-sm text-slate-900 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                                />
                                {#if syncBlankWithReplicate}
                                    <div class="rounded-lg border border-dashed border-slate-300 bg-slate-50 px-2.5 py-1.5 text-sm text-slate-500 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-400">{dict.synced}</div>
                                {:else}
                                    <input
                                        bind:value={screeningParams.blank_mz_tol}
                                        type="number"
                                        min="0"
                                        step="0.01"
                                        class="w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-sm text-slate-900 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                                    />
                                {/if}
                            </div>

                            {#if showAdvancedParams}
                                <div class="mt-3 border-t border-slate-100 pt-3 dark:border-slate-700">
                                    <div class="mb-2 text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">{dict.advanced}</div>
                                    <div class="grid grid-cols-[1fr_1fr_1fr] items-center gap-2">
                                        <div class="text-sm font-medium text-slate-700 dark:text-slate-300">{dict.mzModeShort}</div>
                                        <select
                                            bind:value={screeningParams.replicate_mz_mode}
                                            class="w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-sm text-slate-900 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                                        >
                                            <option value="da">Da</option>
                                            <option value="ppm">ppm</option>
                                        </select>
                                        {#if syncBlankWithReplicate}
                                            <div class="rounded-lg border border-dashed border-slate-300 bg-slate-50 px-2.5 py-1.5 text-sm text-slate-500 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-400">{dict.synced}</div>
                                        {:else}
                                            <select
                                                bind:value={screeningParams.blank_mz_mode}
                                                class="w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-sm text-slate-900 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                                            >
                                                <option value="da">Da</option>
                                                <option value="ppm">ppm</option>
                                            </select>
                                        {/if}
                                    </div>
                                </div>
                            {/if}
                        </div>
                    </div>
                </section>
            </div>

        {:else}
            <Dashboard
                title={dashboardProps.title}
                summary={dashboardProps.summary}
                peaks={dashboardProps.peaks}
                parameters={dashboardProps.parameters}
                metadata={dashboardProps.metadata}
                onlineStatus={onlineStatus}
                exporting={exporting}
                exportingHtml={exportingHtml}
                onExportXlsx={handleExport}
                onExportHtml={handleExportHtml}
                onUploadNewFile={() => {
                    dashboardProps = null;
                    cachedResult = null;
                    currentFile = null;
                }}
            />
        {/if}
    </div>
</main>

{#if showMethodology}
  <div
    class="fixed inset-0 z-50 flex flex-col bg-white dark:bg-slate-900"
    role="dialog"
    aria-modal="true"
    aria-label={dict.methodologyToggle}
  >
    <div class="flex shrink-0 items-center justify-between border-b border-slate-200 px-6 py-4 dark:border-slate-700">
      <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500">
        {dict.methodologyToggle}
      </p>
      <button
        type="button"
        onclick={() => (showMethodology = false)}
        class="flex h-8 w-8 items-center justify-center rounded-full text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700 dark:text-slate-500 dark:hover:bg-slate-800 dark:hover:text-slate-200"
        aria-label={dict.close ?? "Close"}
      >
        <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 6 6 18M6 6l12 12"/>
        </svg>
      </button>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-8">
      <div class="mx-auto max-w-4xl">
        <MethodologyPanel />
      </div>
    </div>
  </div>
{/if}

<style>
    :global(body) {
        font-family:
            "Inter",
            system-ui,
            -apple-system,
            sans-serif;
    }
</style>
