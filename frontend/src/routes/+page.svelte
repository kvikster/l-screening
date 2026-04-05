<script lang="ts">
    import Dashboard from "$lib/components/Dashboard.svelte";
    import { Download, Loader2, Upload } from "lucide-svelte";

    let dashboardProps: any = $state(null);
    let loading = $state(false);
    let exporting = $state(false);
    let exportingHtml = $state(false);
    let error = $state("");
    let currentFile: File | null = $state(null);
    let screeningParams = $state({
        replicate_rt_tol: 0.1,
        replicate_mz_tol: 0.3,
        replicate_mz_mode: "da",
        blank_rt_tol: 0.1,
        blank_mz_tol: 0.3,
        blank_mz_mode: "da",
        signal_to_blank_min: 3,
    });

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
        currentFile = file;

        try {
            const formData = new FormData();
            formData.append("file", file);
            appendScreeningParams(formData);

            const response = await fetch("http://localhost:8000/api/screen", {
                method: "POST",
                body: formData,
            });

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
            const formData = new FormData();
            formData.append("file", currentFile);
            appendScreeningParams(formData);

            const response = await fetch("http://localhost:8000/api/export", {
                method: "POST",
                body: formData,
            });

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

<main class="min-h-screen bg-slate-50">
    <div class="px-8 py-12">
        {#if !dashboardProps}
            <div
                class="mx-auto grid max-w-5xl gap-8 rounded-[2rem] border border-slate-200 bg-white p-8 shadow-sm lg:grid-cols-[1.15fr_0.85fr]"
            >
                <section
                    class="rounded-3xl border-2 border-dashed border-slate-200 bg-slate-50 p-12 text-center transition-all hover:border-blue-400"
                >
                    <div
                        class="mb-6 inline-flex h-16 w-16 items-center justify-center rounded-full bg-blue-50 text-blue-600"
                    >
                        <Upload class="h-8 w-8" />
                    </div>
                    <h1 class="text-2xl font-bold text-slate-900">
                        LC-MS Screening
                    </h1>
                    <p class="mt-2 text-slate-500">
                        Upload an Excel (.xlsx) file to begin screening peaks.
                    </p>
                    <a
                        href="/methodology"
                        class="mt-3 inline-flex items-center gap-1 text-xs text-blue-500 hover:text-blue-700 hover:underline"
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
                            class="block w-full text-sm text-slate-500
                file:mr-4 file:py-2 file:px-4
                file:rounded-full file:border-0
                file:text-sm file:font-semibold
                file:bg-blue-600 file:text-white
                hover:file:bg-blue-700
                cursor-pointer"
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
                            class="mt-6 rounded-lg bg-red-50 p-3 text-sm text-red-600"
                        >
                            {error}
                        </div>
                    {/if}
                </section>

                <section
                    class="rounded-3xl border border-slate-200 bg-slate-50 p-6"
                >
                    <div class="flex items-start justify-between gap-4">
                        <div>
                            <p
                                class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400"
                            >
                                Screening Parameters
                            </p>
                            <h2
                                class="mt-2 text-xl font-semibold text-slate-900"
                            >
                                Pharma / QC controls
                            </h2>
                        </div>
                        <span
                            class="rounded-full bg-white px-3 py-1 text-xs font-medium text-slate-500 shadow-sm"
                            >Audit-ready</span
                        >
                    </div>

                    <div class="mt-6 grid gap-4 sm:grid-cols-2">
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700"
                                >Replicate RT tolerance</span
                            >
                            <input
                                bind:value={screeningParams.replicate_rt_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700"
                                >Replicate m/z tolerance</span
                            >
                            <input
                                bind:value={screeningParams.replicate_mz_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700"
                                >Replicate m/z mode</span
                            >
                            <select
                                bind:value={screeningParams.replicate_mz_mode}
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            >
                                <option value="da">Da</option>
                                <option value="ppm">ppm</option>
                            </select>
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700"
                                >Blank RT tolerance</span
                            >
                            <input
                                bind:value={screeningParams.blank_rt_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700"
                                >Blank m/z tolerance</span
                            >
                            <input
                                bind:value={screeningParams.blank_mz_tol}
                                type="number"
                                min="0"
                                step="0.01"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            />
                        </label>
                        <label class="space-y-1.5 text-sm">
                            <span class="font-medium text-slate-700"
                                >Blank m/z mode</span
                            >
                            <select
                                bind:value={screeningParams.blank_mz_mode}
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            >
                                <option value="da">Da</option>
                                <option value="ppm">ppm</option>
                            </select>
                        </label>
                        <label class="space-y-1.5 text-sm sm:col-span-2">
                            <span class="font-medium text-slate-700"
                                >Signal-to-blank minimum</span
                            >
                            <input
                                bind:value={screeningParams.signal_to_blank_min}
                                type="number"
                                min="0"
                                step="0.1"
                                class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm"
                            />
                        </label>
                    </div>

                    <div
                        class="mt-5 rounded-2xl border border-blue-100 bg-blue-50 px-4 py-3 text-sm text-blue-900"
                    >
                        Replicate matching і blank subtraction налаштовуються
                        окремо. Для high-resolution LC-MS обирай `ppm`, для
                        legacy workbook з грубим `m/z` округленням лишай `Da`.
                    </div>
                </section>
            </div>
        {:else}
            <div class="mb-8 flex items-center justify-end gap-3 px-8">
                <a
                    href="/methodology"
                    class="inline-flex items-center rounded-full border bg-white px-4 py-2 text-sm font-medium text-slate-600 shadow-sm hover:bg-slate-50"
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
                    class="inline-flex items-center rounded-full border bg-white px-4 py-2 text-sm font-medium text-slate-900 shadow-sm hover:bg-slate-50"
                    onclick={() => {
                        dashboardProps = null;
                        currentFile = null;
                    }}
                >
                    <Upload class="mr-2 h-4 w-4" /> Upload New File
                </button>
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
