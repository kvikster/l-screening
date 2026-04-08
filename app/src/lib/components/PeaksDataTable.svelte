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

  function confidenceBar(score: any): string {
    const pct = Math.max(0, Math.min(Number(score) || 0, 100));
    return `<div class="flex items-center gap-2">
      <div class="h-2 w-20 overflow-hidden rounded-full bg-slate-100 dark:bg-slate-700">
        <div class="h-full rounded-full bg-blue-600" style="width:${pct}%"></div>
      </div>
      <span class="font-mono text-slate-700 dark:text-slate-300">${fmt(score, 1)}</span>
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
            <span class="font-mono text-slate-700 dark:text-slate-300">${fmt(row.RT_mean, 4)} × ${fmt(row.MZ_mean, 4)}</span>
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
        render: (d: any) => `<div class="${metricClass("confidence", d)}">${confidenceBar(d)}</div>`,
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
      orderFixed: grouping === "none" ? undefined : [[0, "asc"]],
      layout: {
        topStart: null,
        topEnd: null,
        bottomStart: "pageLength",
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

{#if showFilters}
<div class="mb-4 grid gap-3 rounded-2xl border border-slate-200 bg-slate-50 p-4 dark:border-slate-700 dark:bg-slate-800/70 md:grid-cols-2 xl:grid-cols-4">
  <label class="dt-filter-item">
    <span>Status</span>
    <select bind:value={statusFilter}>
      <option value="Real Compound">{getStatusLabel("Real Compound")}</option>
      <option value="all">All</option>
      <option value="Artifact">{getStatusLabel("Artifact")}</option>
    </select>
  </label>

  <label class="dt-filter-item">
    <span>{get(dictionary).replicateQuality}</span>
    <select bind:value={qualityFilter}>
      <option value="high_moderate">High + Moderate</option>
      <option value="all">All</option>
      <option value="High">{getReplicateQualityLabel("High")}</option>
      <option value="Moderate">{getReplicateQualityLabel("Moderate")}</option>
      <option value="Low">{getReplicateQualityLabel("Low")}</option>
    </select>
  </label>

  <label class="dt-filter-item">
    <span>{get(dictionary).sample}</span>
    <select bind:value={sampleFilter}>
      <option value="all">All</option>
      {#each sampleOptions as option}
        <option value={option}>{getSampleTypeLabel(option)}</option>
      {/each}
    </select>
  </label>

  <label class="dt-filter-item">
    <span>{get(dictionary).polarity}</span>
    <select bind:value={polarityFilter}>
      <option value="all">All</option>
      <option value="positive">{get(dictionary).positive}</option>
      <option value="negative">{get(dictionary).negative}</option>
    </select>
  </label>

  <label class="dt-filter-item">
    <span>{get(dictionary).confidence} ≥ {confidenceMin}</span>
    <input type="range" min="0" max="100" step="1" bind:value={confidenceMin} />
  </label>

  <label class="dt-filter-item">
    <span>CV% ≤ {cvMax}</span>
    <input type="range" min="0" max="100" step="1" bind:value={cvMax} />
  </label>

  <label class="dt-filter-item">
    <span>S/B ≥ {sbMin.toFixed(1)}</span>
    <input type="range" min="0" max="20" step="0.1" bind:value={sbMin} />
  </label>

  <label class="dt-filter-item">
    <span>Search</span>
    <input type="search" bind:value={searchQuery} placeholder="RT, m/z, статус..." />
  </label>

  <label class="dt-filter-item">
    <span>Grouping</span>
    <select bind:value={grouping}>
      <option value="sample">Sample + Polarity</option>
      <option value="status">Status</option>
      <option value="none">None</option>
    </select>
  </label>

  <button class="dt-reset-btn" onclick={resetDefaults}>Reset to defaults</button>
</div>
{/if}

<div class="peaks-dt overflow-x-auto">
  <table bind:this={tableEl} class="min-w-full text-sm"></table>
</div>

<style>
  /* ── Layout wrappers generated by DataTables ──────────────────── */
  :global(.peaks-dt .dt-layout-row) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.45rem 1rem;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  :global(.peaks-dt .dt-layout-cell) {
    display: flex;
    align-items: center;
  }

  .dt-filter-item {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.01em;
    color: #64748b;
  }

  .dt-filter-item select,
  .dt-filter-item input[type="range"],
  .dt-filter-item input[type="search"] {
    width: 100%;
  }

  .dt-filter-item select,
  .dt-filter-item input[type="search"] {
    border: 1px solid #cbd5e1;
    border-radius: 0.7rem;
    background: #fff;
    color: #0f172a;
    font-size: 0.875rem;
    padding: 0.45rem 0.6rem;
  }

  .dt-reset-btn {
    align-self: end;
    border-radius: 0.7rem;
    border: 1px solid #cbd5e1;
    background: #fff;
    color: #0f172a;
    font-size: 0.875rem;
    font-weight: 600;
    padding: 0.45rem 0.75rem;
    transition: background-color 0.15s ease;
  }

  .dt-reset-btn:hover {
    background: #e2e8f0;
  }

  :global(.dark .dt-filter-item) {
    color: #94a3b8;
  }

  :global(.dark .dt-filter-item select),
  :global(.dark .dt-filter-item input[type="search"]) {
    border-color: #475569;
    background: #1e293b;
    color: #e2e8f0;
  }

  :global(.dark .dt-reset-btn) {
    border-color: #475569;
    background: #1e293b;
    color: #e2e8f0;
  }

  :global(.dark .dt-reset-btn:hover) {
    background: #334155;
  }

  /* Page length select */
  :global(.peaks-dt .dt-length select) {
    border: 1px solid #e2e8f0;
    border-radius: 0.75rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
    color: #0f172a;
    background: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  }
  :global(.dark .peaks-dt .dt-length select) {
    background: #334155;
    border-color: #475569;
    color: #f1f5f9;
  }
  :global(.peaks-dt .dt-length label) {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #64748b;
  }
  :global(.dark .peaks-dt .dt-length label) {
    color: #94a3b8;
  }

  /* Info text */
  :global(.peaks-dt .dt-info) {
    font-size: 0.875rem;
    color: #64748b;
  }
  :global(.dark .peaks-dt .dt-info) {
    color: #94a3b8;
  }

  /* Pagination */
  :global(.peaks-dt .dt-paging) {
    display: flex;
    gap: 0.25rem;
  }
  :global(.peaks-dt .dt-paging button) {
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    padding: 0.25rem 0.625rem;
    font-size: 0.875rem;
    background: #fff;
    color: #374151;
    cursor: pointer;
    transition: background 0.15s;
  }
  :global(.peaks-dt .dt-paging button:hover:not(:disabled)) {
    background: #f1f5f9;
  }
  :global(.peaks-dt .dt-paging button.current) {
    background: #2563eb;
    color: #fff;
    border-color: #2563eb;
  }
  :global(.peaks-dt .dt-paging button:disabled) {
    opacity: 0.4;
    cursor: default;
  }
  :global(.dark .peaks-dt .dt-paging button) {
    background: #1e293b;
    border-color: #475569;
    color: #cbd5e1;
  }
  :global(.dark .peaks-dt .dt-paging button:hover:not(:disabled)) {
    background: #334155;
  }
  :global(.dark .peaks-dt .dt-paging button.current) {
    background: #2563eb;
    border-color: #2563eb;
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
    text-align: left;
    padding: 0.5rem 0.7rem;
    border-bottom: 1px solid #e2e8f0;
    white-space: nowrap;
    cursor: pointer;
    user-select: none;
  }
  :global(.dark .peaks-dt table.dataTable thead th) {
    background: #334155;
    color: #94a3b8;
    border-bottom-color: #475569;
  }
  :global(.peaks-dt table.dataTable thead th.dt-ordering-asc::after) {
    content: " ↑";
    color: #3b82f6;
  }
  :global(.peaks-dt table.dataTable thead th.dt-ordering-desc::after) {
    content: " ↓";
    color: #3b82f6;
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
    padding: 0.5rem 0.7rem;
    vertical-align: middle;
  }
  :global(.peaks-dt table.dataTable.no-footer) {
    border-bottom: none;
  }
</style>
