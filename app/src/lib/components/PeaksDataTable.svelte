<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import DataTable from "datatables.net";
  import "datatables.net-dt/css/dataTables.dataTables.css";
  import { dictionary, getReplicateQualityLabel, getStatusLabel, getSampleTypeLabel } from "$lib/i18n";

  let { peaks, parameters, onAuditClick, showFilters = true } = $props<{
    peaks: any[];
    parameters?: any;
    onAuditClick: (peak: any) => void;
    showFilters?: boolean;
  }>();

  let tableEl: HTMLTableElement;
  let dt: any = null;
  let statusFilter = $state("all");
  let qualityFilter = $state("all");
  let sampleFilter = $state("all");
  let polarityFilter = $state("all");
  let confidenceMin = $state(0);
  let cvMax = $state(100);
  let sbMin = $state(0);
  let searchQuery = $state("");
  let grouping = $state<"sample" | "status" | "none">("none");
  const collapsedGroups = new Set<string>();

  let sampleOptions = $derived(
    Array.from(new Set(peaks.map((peak: any) => String(peak.SampleType || "")).filter((value: string) => value.length > 0)))
      .map((value) => String(value))
      .sort((a: string, b: string) => a.localeCompare(b))
  );

  let activeFilterCount = $derived.by(() => {
    let n = 0;
    if (statusFilter !== "all") n++;
    if (qualityFilter !== "all") n++;
    if (sampleFilter !== "all") n++;
    if (polarityFilter !== "all") n++;
    if (confidenceMin > 0) n++;
    if (cvMax < 100) n++;
    if (sbMin > 0) n++;
    return n;
  });

  let filteredPeaks = $derived.by(() => {
    return peaks
      .filter((peak: any) => {
        if (statusFilter !== "all" && peak.Status !== statusFilter) return false;
        if (qualityFilter === "high_moderate" && !["High", "Moderate"].includes(peak.ReplicateQuality)) return false;
        if (qualityFilter !== "all" && qualityFilter !== "high_moderate" && peak.ReplicateQuality !== qualityFilter) return false;
        if (sampleFilter !== "all" && peak.SampleType !== sampleFilter) return false;
        if (polarityFilter !== "all" && String(peak.Polarity || "").toLowerCase() !== polarityFilter) return false;
        const confidence = Number(peak.ConfidenceScore);
        const cv = Number(peak.AreaCVPct);
        const sb = Number(peak.SignalToBlankRatio);

        if (Number.isFinite(confidence)) {
          if (confidence < confidenceMin) return false;
        } else if (confidenceMin > 0) {
          return false;
        }

        if (Number.isFinite(cv)) {
          if (cv > cvMax) return false;
        } else if (cvMax < 100) {
          return false;
        }

        if (Number.isFinite(sb)) {
          if (sb < sbMin) return false;
        } else if (sbMin > 0) {
          return false;
        }

        return true;
      })
      .map((peak: any) => {
        const polarity = peak.Polarity === "positive" ? "+" : peak.Polarity === "negative" ? "−" : String(peak.Polarity || "");
        const groupValue =
          grouping === "sample"
            ? `${getSampleTypeLabel(peak.SampleType)} · ${polarity}`
            : grouping === "status"
              ? `${getStatusLabel(peak.Status)}`
              : "All";
        return {
          ...peak,
          __group: groupValue,
        };
      });
  });

  function toSafeNumber(value: any, fallback: number): number {
    const next = Number(value);
    return Number.isFinite(next) ? next : fallback;
  }

  function fmt(value: any, digits = 2): string {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    return Number(value).toFixed(digits);
  }

  function metricClass(kind: "cv" | "sb" | "confidence", value: any): string {
    const number = Number(value);
    if (!Number.isFinite(number)) return "text-slate-400 dark:text-slate-500";

    if (kind === "cv") {
      const highMax = toSafeNumber(parameters?.cv_high_max, 15);
      const moderateMax = toSafeNumber(parameters?.cv_moderate_max, 30);
      if (number <= highMax) return "text-emerald-700 dark:text-emerald-300";
      if (number <= moderateMax) return "text-amber-700 dark:text-amber-300";
      return "text-rose-700 dark:text-rose-300";
    }

    if (kind === "sb") {
      return number >= sbMin ? "text-emerald-700 dark:text-emerald-300" : "text-rose-700 dark:text-rose-300";
    }

    if (number >= 85) return "text-emerald-700 dark:text-emerald-300";
    if (number >= confidenceMin) return "text-amber-700 dark:text-amber-300";
    return "text-rose-700 dark:text-rose-300";
  }

  function fmtN(value: any, digits = 0): string {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    return Number(value).toLocaleString(undefined, {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });
  }

  function qualityBadge(q: string): string {
    const cls =
      q === "High"
        ? "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
        : q === "Moderate"
          ? "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200"
          : "bg-rose-100 text-rose-800 dark:bg-rose-900 dark:text-rose-200";
    return `<span class="inline-flex rounded-full px-2.5 py-1 text-xs font-semibold ${cls}">${getReplicateQualityLabel(q) ?? "—"}</span>`;
  }

  function statusBadge(s: string): string {
    const isReal = s === "Real Compound";
    const cls = isReal
      ? "text-emerald-700 dark:text-emerald-300"
      : "text-rose-700 dark:text-rose-300";
    const title = getStatusLabel(s) ?? "—";
    const icon = isReal
      ? '<svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M20 6L9 17l-5-5"/></svg>'
      : '<svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>';
    return `<span class="inline-flex items-center justify-center rounded-full p-1 ${cls}" title="${title}" aria-label="${title}">${icon}</span>`;
  }

  function confidenceCircle(score: any): string {
    const num = Number(score);
    if (!Number.isFinite(num)) return '<span class="text-slate-400 dark:text-slate-500">—</span>';
    const pct = Math.max(0, Math.min(num, 100));
    const r = 8;
    const circumference = 2 * Math.PI * r;
    const offset = circumference * (1 - pct / 100);
    let color: string;
    if (num >= 85) color = "#10b981";
    else if (num >= confidenceMin) color = "#f59e0b";
    else color = "#ef4444";
    return `<div class="flex items-center gap-1.5">
      <svg viewBox="0 0 24 24" width="20" height="20" style="transform:rotate(-90deg);flex-shrink:0">
        <circle cx="12" cy="12" r="${r}" fill="none" stroke="rgba(100,116,139,0.18)" stroke-width="4"/>
        <circle cx="12" cy="12" r="${r}" fill="none" stroke="${color}" stroke-width="4"
          stroke-dasharray="${circumference.toFixed(2)}" stroke-dashoffset="${offset.toFixed(2)}"
          stroke-linecap="round"/>
      </svg>
      <span class="font-mono text-[11px] text-slate-700 dark:text-slate-300">${fmt(score, 2)}</span>
    </div>`;
  }

  function repDots(peak: any): string {
    const dict = get(dictionary);
    const count = peak.ReplicateCount || 2;
    let html = `<div class="flex items-center gap-2">
      <div class="flex flex-col items-center">
        <div class="h-4 w-4 rounded-full border border-black/10 shadow-sm dark:border-white/10" style="background-color:#${peak.Rep1_Color || "e2e8f0"}" title="${dict.rep1}: ${peak.Rep1_Mark || dict.noColor}"></div>
        <span class="mt-0.5 text-[10px] text-slate-400 dark:text-slate-500">R1</span>
      </div>
      <div class="flex flex-col items-center">
        <div class="h-4 w-4 rounded-full border border-black/10 shadow-sm dark:border-white/10" style="background-color:#${peak.Rep2_Color || "e2e8f0"}" title="${dict.rep2}: ${peak.Rep2_Mark || dict.noColor}"></div>
        <span class="mt-0.5 text-[10px] text-slate-400 dark:text-slate-500">R2</span>
      </div>`;
    if (count > 2) {
      html += `<span class="rounded-full bg-slate-100 px-2 py-1 text-[10px] font-semibold text-slate-600 dark:bg-slate-700 dark:text-slate-300">+${count - 2}</span>`;
    }
    html += `</div>`;
    return html;
  }

  function buildColumns() {
    const dict = get(dictionary);
    const maxArea = filteredPeaks.reduce((max: number, peak: any) => {
      const next = Number(peak.Area_mean);
      return Number.isFinite(next) ? Math.max(max, next) : max;
    }, 0);
    return [
      {
        title: "Group",
        data: "__group",
        visible: false,
        searchable: false,
      },
      {
        title: "RT (x) m/z",
        data: "RT_mean",
        render: (_d: any, _t: any, row: any) => {
          const area = Number(row.Area_mean);
          const areaPct = maxArea > 0 && Number.isFinite(area) ? Math.max(4, Math.min(100, (area / maxArea) * 100)) : 0;
          return `<div class="space-y-1">
            <span class="font-mono text-slate-700 dark:text-slate-300">${fmt(row.RT_mean, 2)} × ${fmt(row.MZ_mean, 2)}</span>
            <div class="flex items-center gap-1.5">
              <span class="font-mono text-[10px] text-blue-700 dark:text-blue-300">A ${fmtN(area, 0)}</span>
              <span class="inline-block h-1.5 w-12 overflow-hidden rounded-full bg-blue-100 dark:bg-blue-950/60">
                <span class="block h-full rounded-full bg-blue-500/80 dark:bg-blue-400/80" style="width:${areaPct}%"></span>
              </span>
            </div>
          </div>`;
        },
      },
      {
        title: dict.replicates,
        data: "ReplicateCount",
        render: (d: any) =>
          `<span class="inline-flex rounded-full bg-slate-100 px-2.5 py-1 text-xs font-semibold text-slate-700 dark:bg-slate-700 dark:text-slate-300">n=${d || 2}</span>`,
      },
      {
        title: "CV%",
        data: "AreaCVPct",
        render: (d: any) => `<span class="font-mono ${metricClass("cv", d)}">${fmt(d, 2)}</span>`,
      },
      {
        title: dict.replicateQuality,
        data: "ReplicateQuality",
        render: (d: any) => qualityBadge(d),
      },
      {
        title: "S/B",
        data: "SignalToBlankRatio",
        render: (d: any) => `<span class="font-mono ${metricClass("sb", d)}">${fmt(d, 2)}</span>`,
      },
      {
        title: dict.confidence,
        data: "ConfidenceScore",
        render: (d: any) => confidenceCircle(d),
      },
      {
        title: dict.sample,
        data: null,
        render: (_d: any, _t: any, row: any) => {
          const pol = row.Polarity === "positive" ? dict.positive : row.Polarity === "negative" ? dict.negative : row.Polarity;
          return `<div class="flex flex-col"><span class="font-semibold capitalize text-slate-900 dark:text-slate-100">${getSampleTypeLabel(row.SampleType)}</span><span class="text-xs uppercase text-slate-500 dark:text-slate-400">${pol}</span></div>`;
        },
      },
      {
        title: dict.marks,
        data: null,
        orderable: false,
        render: (_d: any, _t: any, row: any) => repDots(row),
      },
      {
        title: dict.status,
        data: "Status",
        render: (d: any) => statusBadge(d),
      },
      {
        title: dict.audit,
        data: null,
        orderable: false,
        render: (_d: any, _t: any, _row: any, meta: any) =>
          `<button class="dt-audit-btn inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-200 bg-slate-50 text-slate-700 hover:bg-slate-100 dark:border-slate-600 dark:bg-slate-700 dark:text-slate-300 dark:hover:bg-slate-600" data-row="${meta.row}" title="${dict.logicDetail}" aria-label="${dict.logicDetail}"><svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg></button>`,
      },
    ];
  }

  function initDT() {
    if (dt) {
      dt.destroy();
      tableEl.innerHTML = "";
      dt = null;
    }

    const orderBy: [number, "asc" | "desc"][] = grouping === "none" ? [[6, "desc"]] : [[0, "asc"], [6, "desc"]];

    dt = new DataTable(tableEl, {
      data: filteredPeaks,
      pageLength: 25,
      lengthMenu: [10, 25, 50, 100],
      stateSave: false,
      order: orderBy,
      layout: {
        topStart: null,
        topEnd: null,
        bottomStart: ["pageLength", "info"],
        bottomEnd: "paging",
      },
      createdRow: (row: HTMLTableRowElement, rowData: any) => {
        row.classList.toggle("dt-row-artifact", rowData.Status !== "Real Compound");
        row.classList.toggle("dt-row-low-confidence", Number(rowData.ConfidenceScore ?? -Infinity) < confidenceMin);
      },
      drawCallback: () => {
        if (!dt || grouping === "none") return;
        tableEl.tBodies.item(0)?.querySelectorAll("tr.dt-group-row").forEach((row) => row.remove());
        const rows = dt.rows({ page: "current" }).nodes();
        const pageRows = dt.rows({ page: "current" }).data().toArray();
        const counts = new Map<string, number>();

        for (const row of pageRows) {
          const key = row.__group || "—";
          counts.set(key, (counts.get(key) || 0) + 1);
        }

        let previous = "";
        let isCollapsed = false;
        for (let i = 0; i < pageRows.length; i += 1) {
          const group = pageRows[i]?.__group || "—";
          const node = rows[i] as HTMLTableRowElement;

          if (group !== previous) {
            isCollapsed = collapsedGroups.has(group);
            const header = document.createElement("tr");
            header.className = "dt-group-row";
            header.innerHTML = `<td colspan="10"><button type="button" class="dt-group-toggle" data-group="${group}" aria-expanded="${isCollapsed ? "false" : "true"}"><span class="dt-group-caret">${isCollapsed ? "▸" : "▾"}</span><span class="dt-group-label">${group}</span><span class="dt-group-count">${counts.get(group) || 0}</span></button></td>`;
            node.parentNode?.insertBefore(header, node);
            previous = group;
          }

          node.style.display = isCollapsed ? "none" : "";
        }
      },
      columns: buildColumns(),
    });
  }

  function resetDefaults() {
    statusFilter = "all";
    qualityFilter = "all";
    sampleFilter = "all";
    polarityFilter = "all";
    confidenceMin = 0;
    cvMax = 100;
    sbMin = 0;
    searchQuery = "";
    grouping = "none";
  }

  onMount(() => {
    cvMax = 100;
    sbMin = 0;
    initDT();

    tableEl.addEventListener("click", (e: MouseEvent) => {
      const toggle = (e.target as HTMLElement).closest(".dt-group-toggle") as HTMLElement | null;
      if (toggle) {
        const group = String(toggle.dataset.group || "");
        if (group) {
          if (collapsedGroups.has(group)) {
            collapsedGroups.delete(group);
          } else {
            collapsedGroups.add(group);
          }
          dt?.draw(false);
        }
        return;
      }

      const btn = (e.target as HTMLElement).closest(".dt-audit-btn") as HTMLElement | null;
      if (btn) {
        const rowIdx = Number(btn.dataset.row);
        const rowData = dt?.row(rowIdx).data();
        if (rowData) onAuditClick(rowData);
        return;
      }

      const tr = (e.target as HTMLElement).closest("tbody tr") as HTMLTableRowElement | null;
      if (!tr || tr.classList.contains("dt-group-row")) return;
      const clickedRow = dt?.row(tr).data();
      if (clickedRow) onAuditClick(clickedRow);
    });

    return () => {
      dt?.destroy();
      dt = null;
    };
  });

  $effect(() => {
    const a = filteredPeaks;
    const b = $dictionary;
    const c = grouping;
    const d = confidenceMin;
    const e = cvMax;
    const f = sbMin;
    if (dt) initDT();
  });

  $effect(() => {
    const query = searchQuery;
    dt?.search(query).draw();
  });
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="group relative z-30 mb-0 shrink-0 rounded-xl border border-slate-200 bg-white transition-all hover:rounded-b-none hover:shadow-lg dark:border-slate-700 dark:bg-slate-800/80">
    
    <!-- ROW 1 (Always Visible): Title, Search, Primary Filters -->
    <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-2.5">
      
      <!-- Informational Text -->
      <div>
        <h3 class="text-sm font-bold text-slate-900 dark:text-slate-100">{get(dictionary).screenedPeaks}</h3>
        <p class="text-[11px] text-slate-500 dark:text-slate-400">{get(dictionary).screenedPeaksDesc}</p>
      </div>

      <!-- Persistent Controls -->
      <div class="flex items-center gap-3">
        <!-- Search -->
        <div class="relative w-48 transition-all group-hover:w-56">
          <svg class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
          <input type="search" bind:value={searchQuery} placeholder="Пошук RT, m/z…" class="w-full rounded-md border border-slate-200 bg-slate-50/50 py-1.5 pl-8 pr-2 text-xs outline-none transition-colors focus:border-blue-400 dark:border-slate-600 dark:bg-slate-900/50 dark:text-white" />
        </div>
        
        <!-- Primary Status Filter (Artifact vs Compound) -->
        <div class="flex items-center gap-1 rounded-md border border-slate-200 bg-slate-50 p-0.5 dark:border-slate-600 dark:bg-slate-900/50">
          {#each [["all", "Всі"], ["Real Compound", "Компаунд"], ["Artifact", "Артефакт"]] as [val, label]}
            <button type="button" onclick={() => (statusFilter = val)} class="px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider transition-colors {statusFilter === val ? 'rounded bg-blue-600 text-white shadow-sm' : 'text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200'}">{label}</button>
          {/each}
        </div>

        <!-- Grouping -->
        <div class="flex flex-col">
          <select bind:value={grouping} class="cursor-pointer rounded-md border border-slate-200 bg-transparent py-1 pl-2 pr-6 text-xs font-medium text-slate-700 outline-none hover:bg-slate-50 dark:border-slate-600 dark:text-slate-200 dark:hover:bg-slate-700">
            <option value="none">Без групування</option>
            <option value="sample">По зразку</option>
            <option value="status">По статусу</option>
          </select>
        </div>

        <!-- Reset Indicator if hidden filters are active -->
        <button type="button" onclick={resetDefaults} class="flex h-7 items-center gap-1.5 rounded-md border border-slate-200 bg-white px-2 py-1 transition-opacity {activeFilterCount > (statusFilter!=='all'?1:0) ? 'opacity-100 hover:border-rose-300 hover:bg-rose-50' : 'pointer-events-none opacity-0'} dark:border-slate-600 dark:bg-slate-800" title="Скинути приховані фільтри">
          <svg class="h-3 w-3 text-rose-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 6 6 18M6 6l12 12"/></svg>
          <span class="text-[10px] font-bold text-rose-600 dark:text-rose-400">{activeFilterCount}</span>
        </button>

      </div>
    </div>

    <!-- ROW 2 (Hover Expansion): Advanced Filters -->
    <div class="absolute left-[-1px] right-[-1px] top-full hidden rounded-b-xl border-x border-b border-slate-200 bg-white pb-3 shadow-2xl group-hover:block dark:border-slate-700 dark:bg-slate-800/95">
      <div class="mx-4 mb-2 border-t border-slate-100 dark:border-slate-700/50"></div>
      <div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-3 px-4">
        
        <!-- Categorical Chips -->
        <div class="flex flex-1 flex-wrap items-center gap-x-6 gap-y-2">
          <div class="flex items-center gap-2">
            <span class="dt-chip-label text-xs">Клас</span>
            <div class="flex gap-1">
              {#each [["all", "Всі"], ["high_moderate", "H+M"], ["High", "H"], ["Moderate", "M"], ["Low", "L"]] as [val, label]}
                <button type="button" onclick={() => (qualityFilter = val)} class="dt-chip text-[10px] !py-0.5 {qualityFilter === val ? 'dt-chip-active' : ''}">{label}</button>
              {/each}
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span class="dt-chip-label text-xs">Ion</span>
            <div class="flex gap-1">
              {#each [["all", "Всі"], ["positive", "+"], ["negative", "−"]] as [val, label]}
                <button type="button" onclick={() => (polarityFilter = val)} class="dt-chip text-[10px] !py-0.5 {polarityFilter === val ? 'dt-chip-active' : ''}">{label}</button>
              {/each}
            </div>
          </div>
          {#if sampleOptions.length > 1}
            <div class="flex items-center gap-2">
              <span class="dt-chip-label text-xs">Зразок</span>
              <select bind:value={sampleFilter} class="max-w-[8rem] truncate rounded border border-slate-200 bg-slate-50 px-2 py-0.5 text-[10px] outline-none dark:border-slate-600 dark:bg-slate-900 dark:text-slate-200">
                <option value="all">Всі зразки</option>
                {#each sampleOptions as option}<option value={option}>{getSampleTypeLabel(option)}</option>{/each}
              </select>
            </div>
          {/if}
        </div>

        <!-- Numeric Sliders -->
        <div class="flex shrink-0 items-center gap-5 border-l border-slate-100 pl-4 dark:border-slate-700">
          <label class="flex items-center gap-1.5">
            <span class="text-[10px] font-semibold text-slate-500 dark:text-slate-400">Conf.≥</span>
            <input type="range" min="0" max="100" step="1" bind:value={confidenceMin} class="dt-slider !w-20" />
            <span class="w-6 text-[10px] font-bold text-slate-700 {confidenceMin > 0 ? 'text-blue-600 dark:text-blue-400' : 'dark:text-slate-300'}">{confidenceMin}</span>
          </label>
          <label class="flex items-center gap-1.5">
            <span class="text-[10px] font-semibold text-slate-500 dark:text-slate-400">CV%≤</span>
            <input type="range" min="0" max="100" step="1" bind:value={cvMax} class="dt-slider !w-20" />
            <span class="w-6 text-[10px] font-bold text-slate-700 {cvMax < 100 ? 'text-blue-600 dark:text-blue-400' : 'dark:text-slate-300'}">{cvMax}</span>
          </label>
          <label class="flex items-center gap-1.5">
            <span class="text-[10px] font-semibold text-slate-500 dark:text-slate-400">S/B≥</span>
            <input type="range" min="0" max="20" step="0.1" bind:value={sbMin} class="dt-slider !w-20" />
            <span class="w-6 text-[10px] font-bold text-slate-700 {sbMin > 0 ? 'text-blue-600 dark:text-blue-400' : 'dark:text-slate-300'}">{sbMin.toFixed(1)}</span>
          </label>
        </div>

      </div>
    </div>
  </div>

  <div class="peaks-dt flex flex-1 flex-col overflow-hidden">
    <table bind:this={tableEl} class="min-w-full text-sm align-middle"></table>
  </div>
</div>

<style>
  /* ── Flexbox DataTable Layout ─────────────────────────────── */
  :global(.peaks-dt .dt-container) {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0; /* Important for flex child */
  }
  :global(.peaks-dt .dt-container .dt-layout-row) {
    margin: 0 !important;
  }
  :global(.peaks-dt) {
    min-height: 0;
  }
  :global(.peaks-dt .dt-layout-table) {
    display: flex;
    align-items: stretch;
    justify-content: stretch;
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin: 0;
    padding: 0;
  }
  :global(.peaks-dt .dt-layout-table > .dt-layout-cell) {
    display: block;
    flex: 1 1 auto;
    height: 100%;
    min-height: 0;
    width: 100%;
    padding: 0;
  }

  /* ── Layout wrappers generated by DataTables ──────────────────── */
  /* Target only pagination/info rows, avoiding adding padding to the main table row itself */
  :global(.peaks-dt .dt-layout-row:not(.dt-layout-table)) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.35rem 0.25rem 0.2rem 0.25rem;
    gap: 1rem;
    flex-shrink: 0; /* Prevents pagination from squishing */
  }
  :global(.peaks-dt .dt-layout-row.dt-layout-end),
  :global(.peaks-dt .dt-layout-row:last-child) {
    margin: 0;
    padding: 0 0.35rem 0.15rem 0.35rem;
    border-top: 1px solid #e2e8f0;
    background: rgba(248, 250, 252, 0.96);
  }
  :global(.dark .peaks-dt .dt-layout-row.dt-layout-end),
  :global(.dark .peaks-dt .dt-layout-row:last-child) {
    border-top-color: #334155;
    background: rgba(15, 23, 42, 0.92);
  }
  :global(.peaks-dt .dt-layout-cell) {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  /* ── Filter: search ───────────────────────────────────────── */
  .dt-search-input {
    width: 100%;
    padding: 0.35rem 0.5rem 0.35rem 2rem;
    border: 1px solid #e2e8f0;
    border-radius: 0.625rem;
    background: #fff;
    color: #0f172a;
    font-size: 0.8rem;
    outline: none;
    transition: border-color 0.15s;
  }
  .dt-search-input:focus { border-color: #93c5fd; }

  :global(.dark .dt-search-input) {
    background: #1e293b;
    border-color: #475569;
    color: #e2e8f0;
  }

  /* ── Filter: chip label ────────────────────────────────────── */
  .dt-chip-label {
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    color: #94a3b8;
    text-transform: uppercase;
  }

  /* ── Filter: chips ─────────────────────────────────────────── */
  .dt-chip {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.6rem;
    border-radius: 9999px;
    border: 1px solid #e2e8f0;
    background: #fff;
    color: #475569;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s;
    white-space: nowrap;
  }
  .dt-chip:hover { border-color: #94a3b8; color: #0f172a; }

  .dt-chip-active {
    background: #2563eb;
    border-color: #2563eb;
    color: #fff;
  }
  .dt-chip-active-green {
    background: #059669;
    border-color: #059669;
    color: #fff;
  }
  .dt-chip-active-red {
    background: #e11d48;
    border-color: #e11d48;
    color: #fff;
  }

  :global(.dark .dt-chip) {
    background: #1e293b;
    border-color: #334155;
    color: #94a3b8;
  }
  :global(.dark .dt-chip:hover) { border-color: #64748b; color: #e2e8f0; }
  :global(.dark .dt-chip-active) { background: #2563eb; border-color: #2563eb; color: #fff; }
  :global(.dark .dt-chip-active-green) { background: #059669; border-color: #059669; color: #fff; }
  :global(.dark .dt-chip-active-red) { background: #be123c; border-color: #be123c; color: #fff; }

  /* ── Filter: sliders ───────────────────────────────────────── */
  .dt-slider-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .dt-slider-label {
    font-size: 0.7rem;
    font-weight: 600;
    color: #94a3b8;
    white-space: nowrap;
    width: 4.5rem;
    flex-shrink: 0;
  }
  .dt-slider {
    flex: 1;
    height: 3px;
    accent-color: #2563eb;
    cursor: pointer;
  }
  .dt-slider-val {
    font-size: 0.7rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    min-width: 2.2rem;
    text-align: right;
    color: #94a3b8;
    transition: color 0.15s;
  }
  .dt-slider-val-active { color: #2563eb; }

  /* Page length select */
  :global(.peaks-dt .dt-length select) {
    border: 1px solid #e2e8f0;
    border-radius: 0.375rem;
    padding: 0.1rem 1.25rem 0.1rem 0.4rem;
    font-size: 0.7rem;
    color: #475569;
    background: #fff;
    font-weight: 600;
  }
  :global(.dark .peaks-dt .dt-length select) {
    background: #1e293b;
    border-color: #334155;
    color: #cbd5e1;
  }
  :global(.peaks-dt .dt-length label) {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0;
    color: transparent;
  }

  /* Info text */
  :global(.peaks-dt .dt-info) {
    font-size: 0.7rem;
    font-weight: 500;
    color: #64748b;
    white-space: nowrap;
  }
  :global(.dark .peaks-dt .dt-info) {
    color: #94a3b8;
  }

  /* Pagination */
  :global(.peaks-dt .dt-paging) {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 0.15rem;
  }
  :global(.peaks-dt .dt-paging .dt-paging-button) {
    border: 1px solid transparent;
    border-radius: 0.375rem;
    padding: 0.15rem 0.4rem;
    font-size: 0.7rem;
    font-weight: 600;
    background: transparent;
    color: #64748b;
    cursor: pointer;
    transition: all 0.1s;
    min-width: 1.5rem;
    text-align: center;
  }
  :global(.peaks-dt .dt-paging .dt-paging-button:hover:not(.disabled)) {
    background: #f1f5f9;
    color: #0f172a;
  }
  :global(.peaks-dt .dt-paging .dt-paging-button.current) {
    background: #2563eb;
    color: #fff;
    border-color: #2563eb;
  }
  :global(.peaks-dt .dt-paging .dt-paging-button.disabled) {
    opacity: 0.3;
    cursor: default;
  }
  :global(.dark .peaks-dt .dt-paging .dt-paging-button) {
    color: #94a3b8;
  }
  :global(.dark .peaks-dt .dt-paging .dt-paging-button:hover:not(.disabled)) {
    background: #334155;
    color: #f1f5f9;
  }
  :global(.dark .peaks-dt .dt-paging .dt-paging-button.current) {
    background: #3b82f6;
    border-color: #3b82f6;
    color: #fff;
  }

  /* Table itself */
  :global(.peaks-dt table.dataTable) {
    width: 100% !important;
    border-collapse: collapse;
  }
  :global(.peaks-dt table.dataTable thead th) {
    background: #f8fafc;
    color: #475569;
    font-weight: 600;
    font-size: 0.72rem;
    text-align: left;
    padding: 0.4rem 0.6rem;
    border-bottom: 1px solid #e2e8f0;
    white-space: nowrap;
    cursor: pointer;
    user-select: none;
    vertical-align: middle;
    position: sticky;
    top: 0;
    z-index: 5;
  }
  :global(.dark .peaks-dt table.dataTable thead th) {
    background: #334155;
    color: #94a3b8;
    border-bottom-color: #475569;
  }
  :global(.peaks-dt table.dataTable thead th.dt-ordering-asc .dt-column-order:before) {
    color: #3b82f6;
    opacity: 1;
  }
  :global(.peaks-dt table.dataTable thead th.dt-ordering-desc .dt-column-order:after) {
    color: #3b82f6;
    opacity: 1;
  }
  :global(.peaks-dt .dt-scroll-body table.dataTable thead th) {
    background: transparent;
  }
  :global(.peaks-dt table.dataTable tbody tr) {
    border-bottom: 1px solid #f1f5f9;
    transition: background 0.1s;
  }
  :global(.peaks-dt table.dataTable tbody tr:hover) {
    background: rgba(248, 250, 252, 0.7);
  }

  :global(.peaks-dt table.dataTable tbody tr:nth-child(even):not(.dt-group-row):not(.dt-row-artifact)) {
    background: rgba(148, 163, 184, 0.08);
  }

  :global(.peaks-dt table.dataTable tbody tr.dt-row-artifact) {
    background: rgba(254, 242, 242, 0.7);
  }

  :global(.peaks-dt table.dataTable tbody tr.dt-row-low-confidence) {
    box-shadow: inset 3px 0 0 #f59e0b;
  }

  :global(.peaks-dt table.dataTable tbody tr.dt-group-row) {
    background: #f1f5f9;
  }

  :global(.peaks-dt table.dataTable tbody tr.dt-group-row td) {
    padding: 0.18rem 0.52rem;
  }

  :global(.peaks-dt .dt-group-toggle) {
    width: 100%;
    display: grid;
    grid-template-columns: 1.2rem minmax(0, 1fr) auto;
    align-items: center;
    gap: 0.5rem;
    border: 1px solid #dbe2ea;
    border-radius: 0.7rem;
    background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);
    color: #334155;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    padding: 0.35rem 0.55rem;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  :global(.peaks-dt .dt-group-toggle:hover) {
    border-color: #bfdbfe;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.12);
  }

  :global(.peaks-dt .dt-group-caret) {
    color: #64748b;
    font-size: 0.9rem;
    text-align: center;
  }

  :global(.peaks-dt .dt-group-label) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.peaks-dt .dt-group-count) {
    border-radius: 999px;
    border: 1px solid #c7d2fe;
    background: linear-gradient(180deg, #eff6ff 0%, #dbeafe 100%);
    color: #1e40af;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.01em;
    min-width: 1.75rem;
    text-align: center;
    padding: 0.14rem 0.5rem;
  }

  :global(.dark .peaks-dt table.dataTable tbody tr) {
    border-bottom-color: #334155;
  }
  :global(.dark .peaks-dt table.dataTable tbody tr:hover) {
    background: rgba(51, 65, 85, 0.5);
  }

  :global(.dark .peaks-dt table.dataTable tbody tr:nth-child(even):not(.dt-group-row):not(.dt-row-artifact)) {
    background: rgba(148, 163, 184, 0.06);
  }

  :global(.dark .peaks-dt table.dataTable tbody tr.dt-row-artifact) {
    background: rgba(127, 29, 29, 0.22);
  }

  :global(.dark .peaks-dt table.dataTable tbody tr.dt-group-row) {
    background: #1e293b;
  }

  :global(.dark .peaks-dt table.dataTable tbody tr.dt-group-row td) {
    padding: 0.18rem 0.52rem;
  }

  :global(.dark .peaks-dt .dt-group-toggle) {
    border-color: #475569;
    background: linear-gradient(180deg, #1e293b 0%, #0f172a 100%);
    color: #cbd5e1;
  }

  :global(.dark .peaks-dt .dt-group-toggle:hover) {
    border-color: #60a5fa;
    box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.24);
  }

  :global(.dark .peaks-dt .dt-group-caret) {
    color: #94a3b8;
  }

  :global(.dark .peaks-dt .dt-group-count) {
    border-color: #1d4ed8;
    background: linear-gradient(180deg, #1d4ed8 0%, #1e3a8a 100%);
    color: #dbeafe;
  }
  :global(.peaks-dt table.dataTable tbody td) {
    padding: 0.4rem 0.6rem;
    vertical-align: middle;
    font-size: 0.72rem;
  }
  :global(.peaks-dt table.dataTable.no-footer) {
    border-bottom: none;
  }
</style>
