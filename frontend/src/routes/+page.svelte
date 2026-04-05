<script lang="ts">
  import Dashboard from "$lib/components/Dashboard.svelte";
  import { Upload, Loader2, Download } from "lucide-svelte";

  let dashboardProps: any = $state(null);
  let loading = $state(false);
  let exporting = $state(false);
  let error = $state("");
  let currentFile: File | null = $state(null);

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

      const response = await fetch("http://localhost:8000/api/screen", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const err = await response.json().catch(() => ({ detail: response.statusText }));
        throw new Error(err.detail || "Server error");
      }

      const spec = await response.json();
      const root = spec.elements?.[spec.root];
      dashboardProps = root?.props ?? spec;
    } catch (e: any) {
      error = (typeof e === "string" ? e : e.message) || "Failed to process file";
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

      const response = await fetch("http://localhost:8000/api/export", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const err = await response.json().catch(() => ({ detail: response.statusText }));
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
</script>

<svelte:head>
  <title>LC-MS Screening</title>
</svelte:head>

<main class="min-h-screen bg-slate-50">
  <div class="px-8 py-12">
    {#if !dashboardProps}
      <div class="max-w-md mx-auto rounded-3xl border-2 border-dashed border-slate-200 bg-white p-12 text-center transition-all hover:border-blue-400">
        <div class="mb-6 inline-flex h-16 w-16 items-center justify-center rounded-full bg-blue-50 text-blue-600">
          <Upload class="h-8 w-8" />
        </div>
        <h1 class="text-2xl font-bold text-slate-900">LC-MS Screening</h1>
        <p class="mt-2 text-slate-500">Upload an Excel (.xlsx) file to begin screening peaks.</p>
        <a
          href="/methodology"
          class="mt-3 inline-flex items-center gap-1 text-xs text-blue-500 hover:text-blue-700 hover:underline"
        >
          <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><path d="M12 16v-4m0-4h.01"/>
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
          <div class="mt-6 flex items-center justify-center text-blue-600">
            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
            <span>Processing data...</span>
          </div>
        {/if}

        {#if error}
          <div class="mt-6 rounded-lg bg-red-50 p-3 text-sm text-red-600">
            {error}
          </div>
        {/if}
      </div>
    {:else}
      <div class="mb-8 flex items-center justify-end gap-3 px-8">
        <a
          href="/methodology"
          class="inline-flex items-center rounded-full bg-white px-4 py-2 text-sm font-medium text-slate-600 shadow-sm border hover:bg-slate-50"
        >
          <svg class="mr-2 h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><path d="M12 16v-4m0-4h.01"/>
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
          class="inline-flex items-center rounded-full bg-white px-4 py-2 text-sm font-medium text-slate-900 shadow-sm border hover:bg-slate-50"
          onclick={() => { dashboardProps = null; currentFile = null; }}
        >
          <Upload class="mr-2 h-4 w-4" /> Upload New File
        </button>
      </div>

      <Dashboard title={dashboardProps.title} summary={dashboardProps.summary} peaks={dashboardProps.peaks} />
    {/if}
  </div>
</main>

<style>
  :global(body) {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
  }
</style>
