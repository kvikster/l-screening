<script lang="ts">
  import { base } from "$app/paths";
  import { onMount } from "svelte";
  import {
    dictionary,
    locale,
    localeLabels,
    setLocale,
  } from "$lib/i18n";
  import { theme, cycleTheme, themeIcons } from "$lib/theme";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import MethodologyPanel from "$lib/components/MethodologyPanel.svelte";
  import LabSpaceWorkspace from "$lib/components/LabSpaceWorkspace.svelte";
  import {
    downloadHtmlReport,
    exportSnapshotToXlsx,
    renderOfflineHtmlReport,
  } from "$lib/screening";
  import type { StoredReport } from "$lib/labspace/types";

  let showMethodology = $state(false);
  let dict = $derived($dictionary);
  let currentTheme = $derived($theme);
  let currentLocale = $derived($locale);
  let themeLabels = $derived({ auto: dict.themeAuto, light: dict.themeLight, dark: dict.themeDark });
  let onlineStatus = $state(typeof navigator === "undefined" ? true : navigator.onLine);
  let updateReady = $state(false);
  let activeReport = $state<StoredReport | null>(null);
  let exporting = $state(false);
  let exportingHtml = $state(false);
  const currentVersion = "v0.8.0";
  const releaseNotesHref = `${base}/releases/v0.8.0.html`;

  onMount(() => {
    const syncOnlineStatus = () => {
      onlineStatus = typeof navigator === "undefined" ? true : navigator.onLine;
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
      navigator.serviceWorker?.removeEventListener("message", handleSwMessage);
    };
  });

  async function handleExportXlsx() {
    if (!activeReport) return;
    exporting = true;
    try {
      await exportSnapshotToXlsx(activeReport.resultSnapshot, `${activeReport.title}.xlsx`);
    } finally {
      exporting = false;
    }
  }

  async function handleExportHtml() {
    if (!activeReport) return;
    exportingHtml = true;
    try {
      const html = renderOfflineHtmlReport(activeReport.resultSnapshot, activeReport.title);
      downloadHtmlReport(html, `${activeReport.title}.html`);
    } finally {
      exportingHtml = false;
    }
  }
</script>

<svelte:head>
  <title>{dict.appName}</title>
</svelte:head>

{#if updateReady}
  <div class="fixed inset-x-0 top-0 z-50 flex items-center justify-between gap-4 bg-blue-600 px-6 py-3 text-sm text-white shadow-md">
    <span>{dict.updateReady}</span>
    <button class="rounded-full bg-white px-4 py-1.5 text-xs font-semibold text-blue-700 hover:bg-blue-50" onclick={() => window.location.reload()}>
      {dict.reload}
    </button>
  </div>
{/if}

<svelte:window onkeydown={(event) => { if (event.key === "Escape" && showMethodology) showMethodology = false; }} />

<main class="min-h-screen bg-slate-50 dark:bg-slate-900" class:pt-12={updateReady}>
  <div class="px-4 pb-10 pt-4">
    <div class="mx-auto mb-5 flex max-w-7xl flex-wrap items-center justify-between gap-4 rounded-[1.75rem] border border-slate-200 bg-white px-5 py-4 shadow-sm dark:border-slate-700 dark:bg-slate-900">
      <div class="flex items-center gap-4">
        <img src="./icons/icon-128.png" alt={dict.appName} class="h-14 w-14 rounded-2xl shadow-sm" />
        <div>
          <div class="flex items-center gap-2">
            <h1 class="text-xl font-semibold text-slate-900 dark:text-slate-50">{dict.appName}</h1>
            <span class="rounded-full bg-slate-100 px-2.5 py-1 text-[11px] font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300">LabSpace</span>
          </div>
          <p class="mt-1 text-sm text-slate-500 dark:text-slate-400">Singleton workspace for blank, surrogate, analysis datasets, analyzers, and report history.</p>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <span class={[
          "rounded-full px-3 py-1 text-xs font-medium",
          onlineStatus
            ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
            : "bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300",
        ].join(" ")}>
          {onlineStatus ? dict.online : dict.offline}
        </span>
        <label class="flex cursor-pointer items-center gap-1 rounded-full border border-slate-200 bg-white px-2 py-1 text-[11px] text-slate-500 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-400">
          <select class="bg-transparent outline-none text-[11px]" value={currentLocale} onchange={(event) => setLocale((event.currentTarget as HTMLSelectElement).value)} aria-label={dict.languageLabel}>
            {#each Object.entries(localeLabels) as [value, label]}
              <option {value}>{label}</option>
            {/each}
          </select>
        </label>
        <button type="button" onclick={cycleTheme} title={themeLabels[currentTheme]} aria-label={themeLabels[currentTheme]} class="flex h-8 w-8 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-500 hover:bg-slate-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d={themeIcons[currentTheme]} />
          </svg>
        </button>
        <button type="button" onclick={() => (showMethodology = true)} class="rounded-full border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700">
          {dict.methodologyToggle}
        </button>
        <a href={releaseNotesHref} target="_blank" rel="noreferrer" class="rounded-full border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-600 hover:border-blue-300 hover:text-blue-700 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-300 dark:hover:border-blue-500 dark:hover:text-blue-300">
          {dict.releaseBaseline.replace("{version}", currentVersion)}
        </a>
      </div>
    </div>

    {#if activeReport}
      <Dashboard
        title={activeReport.resultSnapshot.dashboardProps.title}
        summary={activeReport.resultSnapshot.dashboardProps.summary}
        peaks={activeReport.resultSnapshot.dashboardProps.peaks}
        parameters={activeReport.resultSnapshot.dashboardProps.parameters}
        metadata={activeReport.resultSnapshot.dashboardProps.metadata}
        onlineStatus={onlineStatus}
        exporting={exporting}
        exportingHtml={exportingHtml}
        onExportXlsx={handleExportXlsx}
        onExportHtml={handleExportHtml}
        onUploadNewFile={() => {
          activeReport = null;
        }}
      />
    {:else}
      <LabSpaceWorkspace onOpenReport={(report) => { activeReport = report; }} />
    {/if}
  </div>
</main>

{#if showMethodology}
  <div class="fixed inset-0 z-50 flex flex-col bg-white dark:bg-slate-900" role="dialog" aria-modal="true" aria-label={dict.methodologyToggle}>
    <div class="flex shrink-0 items-center justify-between border-b border-slate-200 px-6 py-4 dark:border-slate-700">
      <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400 dark:text-slate-500">{dict.methodologyToggle}</p>
      <button type="button" onclick={() => (showMethodology = false)} class="flex h-8 w-8 items-center justify-center rounded-full text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700 dark:text-slate-500 dark:hover:bg-slate-800 dark:hover:text-slate-200" aria-label={dict.close ?? "Close"}>
        <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto px-4 py-6 sm:px-6 sm:py-8">
      <div class="mx-auto w-full max-w-[68rem]">
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
