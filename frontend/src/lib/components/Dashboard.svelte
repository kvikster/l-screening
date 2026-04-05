<script lang="ts">
  import StatCard from "./StatCard.svelte";

  let allProps = $props<any>();
  let element = $derived(allProps.element || { props: allProps });
  let title = $derived(element.props?.title || "No Title");
  let summary = $derived(element.props?.summary || []);
  let peaks = $derived(element.props?.peaks || []);
  let parameters = $derived(element.props?.parameters || {});
  let metadata = $derived(element.props?.metadata || {});
  let selectedPeak = $state<any>(null);
  let statusFilter = $state("all");
  let polarityFilter = $state("all");
  let qualityFilter = $state("all");
  let sampleFilter = $state("all");
  let sortKey = $state("confidence_desc");

  let totalCompounds = $derived(summary.reduce((acc: number, s: any) => acc + (s.RealCompounds || 0), 0));
  let totalArtifacts = $derived(summary.reduce((acc: number, s: any) => acc + (s.Artifacts || 0), 0));
  let totalConfirmed = $derived(summary.reduce((acc: number, s: any) => acc + (s.Confirmed || 0), 0));
  let meanCv = $derived(
    (() => {
      const values = summary.map((s: any) => s.MeanCVPct).filter((value: any) => value !== null && value !== undefined);
      if (!values.length) return null;
      return values.reduce((acc: number, value: number) => acc + value, 0) / values.length;
    })()
  );
  let meanConfidence = $derived(
    (() => {
      const values = peaks.map((peak: any) => peak.ConfidenceScore).filter((value: any) => value !== null && value !== undefined);
      if (!values.length) return null;
      return values.reduce((acc: number, value: number) => acc + value, 0) / values.length;
    })()
  );
  let filteredPeaks = $derived.by(() => {
    let next = [...peaks];

    if (statusFilter !== "all") {
      next = next.filter((peak: any) => peak.Status === statusFilter);
    }
    if (polarityFilter !== "all") {
      next = next.filter((peak: any) => String(peak.Polarity).toLowerCase() === polarityFilter.toLowerCase());
    }
    if (qualityFilter !== "all") {
      next = next.filter((peak: any) => peak.ReplicateQuality === qualityFilter);
    }
    if (sampleFilter !== "all") {
      next = next.filter((peak: any) => peak.SampleType === sampleFilter);
    }

    next.sort((left: any, right: any) => {
      switch (sortKey) {
        case "rt_asc":
          return Number(left.RT_mean) - Number(right.RT_mean);
        case "mz_asc":
          return Number(left.MZ_mean) - Number(right.MZ_mean);
        case "cv_asc":
          return Number(left.AreaCVPct ?? Number.POSITIVE_INFINITY) - Number(right.AreaCVPct ?? Number.POSITIVE_INFINITY);
        case "sb_desc":
          return Number(right.SignalToBlankRatio ?? Number.NEGATIVE_INFINITY) - Number(left.SignalToBlankRatio ?? Number.NEGATIVE_INFINITY);
        case "confidence_desc":
        default:
          return Number(right.ConfidenceScore ?? Number.NEGATIVE_INFINITY) - Number(left.ConfidenceScore ?? Number.NEGATIVE_INFINITY);
      }
    });

    return next;
  });

  function formatNumber(value: any, digits = 0) {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    return Number(value).toLocaleString(undefined, {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });
  }

  function formatMaybe(value: any, digits = 2) {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    return Number(value).toFixed(digits);
  }

  function formatMode(mode: string | undefined) {
    return (mode || "da").toUpperCase();
  }

  function openAuditTrail(peak: any) {
    selectedPeak = peak;
  }

  function closeAuditTrail() {
    selectedPeak = null;
  }

  function handleOverlayKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      closeAuditTrail();
    }
  }

  function resetFilters() {
    statusFilter = "all";
    polarityFilter = "all";
    qualityFilter = "all";
    sampleFilter = "all";
    sortKey = "confidence_desc";
  }
</script>

<div class="mx-auto max-w-7xl space-y-8 p-8">
  <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400">LC-MS Screening</p>
      <h2 class="mt-2 text-3xl font-bold tracking-tight text-slate-950">{title}</h2>
    </div>
    <div class="rounded-2xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-600 shadow-sm">
      <span class="font-semibold text-slate-900">Threshold profile:</span>
      replicate {formatNumber(parameters.replicate_rt_tol, 2)} min / {formatNumber(parameters.replicate_mz_tol, 2)} {formatMode(parameters.replicate_mz_mode)},
      blank {formatNumber(parameters.blank_rt_tol, 2)} min / {formatNumber(parameters.blank_mz_tol, 2)} {formatMode(parameters.blank_mz_mode)},
      S/B ≥ {formatNumber(parameters.signal_to_blank_min, 1)}
      {#if metadata?.cacheHit}
        <span class="ml-2 rounded-full bg-emerald-50 px-2 py-0.5 text-xs font-medium text-emerald-700">cache hit</span>
      {/if}
    </div>
  </div>

  {#if metadata?.truncated}
    <div class="rounded-2xl border border-amber-200 bg-amber-50 px-5 py-4 text-sm text-amber-900">
      UI currently shows {metadata.shownPeaks} of {metadata.totalPeaks} screened peaks. Export contains the full screened result set.
    </div>
  {/if}

  <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-6">
    <StatCard label="Real Compounds" value={totalCompounds} />
    <StatCard label="Artifacts" value={totalArtifacts} />
    <StatCard label="Confirmed Peaks" value={totalConfirmed} />
    <StatCard label="Displayed Peaks" value={metadata?.shownPeaks ?? peaks.length} description={metadata?.totalPeaks ? `of ${metadata.totalPeaks} total` : undefined} />
    <StatCard label="Mean CV%" value={formatMaybe(meanCv, 1)} />
    <StatCard label="Mean Confidence" value={formatMaybe(meanConfidence, 1)} />
  </div>

  <div class="grid gap-6 xl:grid-cols-[1.3fr_0.7fr]">
    <section class="rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div class="border-b border-slate-100 px-6 py-5">
        <h3 class="text-lg font-semibold text-slate-900">QC Summary</h3>
        <p class="mt-1 text-sm text-slate-500">Per sample type and polarity: replicate confirmation, quality profile, confidence and blank behaviour.</p>
      </div>
      <div class="overflow-x-auto">
        <table class="min-w-full text-sm">
          <thead class="bg-slate-50 text-slate-600">
            <tr>
              <th class="px-4 py-3 text-left font-semibold">Sample</th>
              <th class="px-4 py-3 text-left font-semibold">Polarity</th>
              <th class="px-4 py-3 text-left font-semibold">Confirmed</th>
              <th class="px-4 py-3 text-left font-semibold">Real / Artifact</th>
              <th class="px-4 py-3 text-left font-semibold">Mean CV%</th>
              <th class="px-4 py-3 text-left font-semibold">Quality H/M/L</th>
              <th class="px-4 py-3 text-left font-semibold">Mean Confidence</th>
              <th class="px-4 py-3 text-left font-semibold">Mean S/B</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            {#each summary as row}
              <tr class="hover:bg-slate-50/70">
                <td class="px-4 py-3 font-semibold capitalize text-slate-900">{row.Sample}</td>
                <td class="px-4 py-3 uppercase text-slate-500">{row.Polarity}</td>
                <td class="px-4 py-3">{row.Confirmed}</td>
                <td class="px-4 py-3">{row.RealCompounds || 0} / {row.Artifacts || 0}</td>
                <td class="px-4 py-3 font-mono">{formatMaybe(row.MeanCVPct, 1)}</td>
                <td class="px-4 py-3">{row.HighQuality || 0}/{row.ModerateQuality || 0}/{row.LowQuality || 0}</td>
                <td class="px-4 py-3 font-mono">{formatMaybe(row.MeanConfidenceScore, 1)}</td>
                <td class="px-4 py-3 font-mono">{formatMaybe(row.MeanSignalToBlankRatio, 2)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>

    <section class="rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div class="border-b border-slate-100 px-6 py-5">
        <h3 class="text-lg font-semibold text-slate-900">Method Parameters</h3>
        <p class="mt-1 text-sm text-slate-500">The exact screening thresholds applied to this result set.</p>
      </div>
      <dl class="grid gap-3 px-6 py-5 text-sm">
        <div class="rounded-xl border border-slate-100 bg-slate-50 px-4 py-3">
          <dt class="text-slate-500">Replicate matching</dt>
          <dd class="mt-1 font-medium text-slate-900">{formatNumber(parameters.replicate_rt_tol, 2)} min / {formatNumber(parameters.replicate_mz_tol, 2)} {formatMode(parameters.replicate_mz_mode)}</dd>
        </div>
        <div class="rounded-xl border border-slate-100 bg-slate-50 px-4 py-3">
          <dt class="text-slate-500">Blank subtraction</dt>
          <dd class="mt-1 font-medium text-slate-900">{formatNumber(parameters.blank_rt_tol, 2)} min / {formatNumber(parameters.blank_mz_tol, 2)} {formatMode(parameters.blank_mz_mode)}</dd>
        </div>
        <div class="rounded-xl border border-slate-100 bg-slate-50 px-4 py-3">
          <dt class="text-slate-500">Signal-to-blank threshold</dt>
          <dd class="mt-1 font-medium text-slate-900">≥ {formatNumber(parameters.signal_to_blank_min, 1)}</dd>
        </div>
        <div class="rounded-xl border border-slate-100 bg-slate-50 px-4 py-3">
          <dt class="text-slate-500">Replicate quality bands</dt>
          <dd class="mt-1 font-medium text-slate-900">High ≤ {formatNumber(parameters.cv_high_max, 1)}% CV, Moderate ≤ {formatNumber(parameters.cv_moderate_max, 1)}% CV</dd>
        </div>
      </dl>
    </section>
  </div>

  <section class="rounded-2xl border border-slate-200 bg-white shadow-sm">
    <div class="border-b border-slate-100 px-6 py-5">
      <h3 class="text-lg font-semibold text-slate-900">Screened Peaks</h3>
      <p class="mt-1 text-sm text-slate-500">Peak-level QC metrics with blank subtraction and replicate audit trail.</p>
    </div>
    <div class="grid gap-3 border-b border-slate-100 bg-slate-50 px-6 py-4 md:grid-cols-6">
      <label class="space-y-1 text-sm">
        <span class="font-medium text-slate-600">Status</span>
        <select bind:value={statusFilter} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm">
          <option value="all">All</option>
          <option value="Real Compound">Real Compound</option>
          <option value="Artifact">Artifact</option>
        </select>
      </label>
      <label class="space-y-1 text-sm">
        <span class="font-medium text-slate-600">Polarity</span>
        <select bind:value={polarityFilter} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm">
          <option value="all">All</option>
          <option value="positive">Positive</option>
          <option value="negative">Negative</option>
        </select>
      </label>
      <label class="space-y-1 text-sm">
        <span class="font-medium text-slate-600">Replicate Quality</span>
        <select bind:value={qualityFilter} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm">
          <option value="all">All</option>
          <option value="High">High</option>
          <option value="Moderate">Moderate</option>
          <option value="Low">Low</option>
        </select>
      </label>
      <label class="space-y-1 text-sm">
        <span class="font-medium text-slate-600">Sample Type</span>
        <select bind:value={sampleFilter} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm">
          <option value="all">All</option>
          <option value="sample">sample</option>
          <option value="blank">blank</option>
        </select>
      </label>
      <label class="space-y-1 text-sm md:col-span-2">
        <span class="font-medium text-slate-600">Sort</span>
        <div class="flex gap-2">
          <select bind:value={sortKey} class="w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-slate-900 shadow-sm">
            <option value="confidence_desc">Confidence ↓</option>
            <option value="cv_asc">CV% ↑</option>
            <option value="sb_desc">S/B ↓</option>
            <option value="rt_asc">RT ↑</option>
            <option value="mz_asc">m/z ↑</option>
          </select>
          <button class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-100" onclick={resetFilters}>
            Reset
          </button>
        </div>
      </label>
    </div>
    <div class="flex items-center justify-between border-b border-slate-100 px-6 py-3 text-sm text-slate-500">
      <span>Showing {filteredPeaks.length} of {peaks.length} displayed peaks</span>
      <span>Default sort prioritizes highest confidence</span>
    </div>
    <div class="overflow-x-auto">
      <table class="min-w-full text-sm">
        <thead class="bg-slate-50 text-slate-600">
          <tr>
            <th class="px-4 py-3 text-left font-semibold">RT</th>
            <th class="px-4 py-3 text-left font-semibold">m/z</th>
            <th class="px-4 py-3 text-left font-semibold">Area mean</th>
            <th class="px-4 py-3 text-left font-semibold">Replicates</th>
            <th class="px-4 py-3 text-left font-semibold">CV%</th>
            <th class="px-4 py-3 text-left font-semibold">Replicate Quality</th>
            <th class="px-4 py-3 text-left font-semibold">S/B</th>
            <th class="px-4 py-3 text-left font-semibold">Confidence</th>
            <th class="px-4 py-3 text-left font-semibold">Sample</th>
            <th class="px-4 py-3 text-left font-semibold">Marks</th>
            <th class="px-4 py-3 text-left font-semibold">Status</th>
            <th class="px-4 py-3 text-left font-semibold">Audit</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100">
          {#each filteredPeaks as peak}
            <tr class="hover:bg-slate-50/70">
              <td class="px-4 py-3 font-mono text-slate-700">{formatMaybe(peak.RT_mean, 4)}</td>
              <td class="px-4 py-3 font-mono text-slate-700">{formatMaybe(peak.MZ_mean, 4)}</td>
              <td class="px-4 py-3 text-slate-900">{formatNumber(peak.Area_mean, 2)}</td>
              <td class="px-4 py-3">
                <span class="inline-flex rounded-full bg-slate-100 px-2.5 py-1 text-xs font-semibold text-slate-700">
                  n={peak.ReplicateCount || 2}
                </span>
              </td>
              <td class="px-4 py-3 font-mono text-slate-700">{formatMaybe(peak.AreaCVPct, 2)}</td>
              <td class="px-4 py-3">
                <span class={`inline-flex rounded-full px-2.5 py-1 text-xs font-semibold ${
                  peak.ReplicateQuality === "High"
                    ? "bg-emerald-100 text-emerald-800"
                    : peak.ReplicateQuality === "Moderate"
                      ? "bg-amber-100 text-amber-800"
                      : "bg-rose-100 text-rose-800"
                }`}>
                  {peak.ReplicateQuality}
                </span>
              </td>
              <td class="px-4 py-3 font-mono text-slate-700">{formatMaybe(peak.SignalToBlankRatio, 2)}</td>
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <div class="h-2 w-20 overflow-hidden rounded-full bg-slate-100">
                    <div class="h-full rounded-full bg-blue-600" style={`width: ${Math.max(0, Math.min(Number(peak.ConfidenceScore) || 0, 100))}%`}></div>
                  </div>
                  <span class="font-mono text-slate-700">{formatMaybe(peak.ConfidenceScore, 1)}</span>
                </div>
              </td>
              <td class="px-4 py-3">
                <div class="flex flex-col">
                  <span class="font-semibold capitalize text-slate-900">{peak.SampleType}</span>
                  <span class="text-xs uppercase text-slate-500">{peak.Polarity}</span>
                </div>
              </td>
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <div class="flex flex-col items-center">
                    <div
                      class="h-4 w-4 rounded-full border border-black/10 shadow-sm"
                      style={`background-color: #${peak.Rep1_Color || "e2e8f0"}`}
                      title={`Rep 1: ${peak.Rep1_Mark || "No Color"}`}
                    ></div>
                    <span class="mt-0.5 text-[10px] text-slate-400">R1</span>
                  </div>
                  <div class="flex flex-col items-center">
                    <div
                      class="h-4 w-4 rounded-full border border-black/10 shadow-sm"
                      style={`background-color: #${peak.Rep2_Color || "e2e8f0"}`}
                      title={`Rep 2: ${peak.Rep2_Mark || "No Color"}`}
                    ></div>
                    <span class="mt-0.5 text-[10px] text-slate-400">R2</span>
                  </div>
                  {#if (peak.ReplicateCount || 2) > 2}
                    <span class="rounded-full bg-slate-100 px-2 py-1 text-[10px] font-semibold text-slate-600">
                      +{(peak.ReplicateCount || 2) - 2}
                    </span>
                  {/if}
                </div>
              </td>
              <td class="px-4 py-3">
                <span class={`inline-flex rounded-full px-2.5 py-1 text-xs font-semibold ${
                  peak.Status === "Real Compound" ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"
                }`}>
                  {peak.Status}
                </span>
              </td>
              <td class="px-4 py-3">
                <button
                  class="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-100"
                  onclick={() => openAuditTrail(peak)}
                >
                  Logic detail
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>
</div>

{#if selectedPeak}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/40 p-4"
    role="button"
    tabindex="0"
    aria-label="Close audit trail"
    onclick={closeAuditTrail}
    onkeydown={handleOverlayKeydown}
  >
    <div
      class="max-h-[85vh] w-full max-w-3xl overflow-hidden rounded-3xl bg-white shadow-2xl"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => event.stopPropagation()}
    >
      <div class="flex items-start justify-between border-b border-slate-100 px-6 py-5">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.18em] text-slate-400">Audit Trail</p>
          <h3 class="mt-2 text-xl font-semibold text-slate-950">{selectedPeak.Status} • {selectedPeak.SampleType} / {selectedPeak.Polarity}</h3>
        </div>
        <button class="rounded-full border border-slate-200 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-50" onclick={closeAuditTrail}>
          Close
        </button>
      </div>
      <div class="grid gap-4 border-b border-slate-100 bg-slate-50 px-6 py-4 text-sm md:grid-cols-4">
        <div>
          <div class="text-slate-500">RT / m/z</div>
          <div class="mt-1 font-mono text-slate-900">{formatMaybe(selectedPeak.RT_mean, 4)} / {formatMaybe(selectedPeak.MZ_mean, 4)}</div>
        </div>
        <div>
          <div class="text-slate-500">Area mean</div>
          <div class="mt-1 font-mono text-slate-900">{formatNumber(selectedPeak.Area_mean, 2)}</div>
        </div>
        <div>
          <div class="text-slate-500">CV% / S/B</div>
          <div class="mt-1 font-mono text-slate-900">{formatMaybe(selectedPeak.AreaCVPct, 2)} / {formatMaybe(selectedPeak.SignalToBlankRatio, 2)}</div>
        </div>
        <div>
          <div class="text-slate-500">Confidence</div>
          <div class="mt-1 font-mono text-slate-900">{formatMaybe(selectedPeak.ConfidenceScore, 1)}</div>
        </div>
      </div>
      <div class="max-h-[50vh] overflow-auto px-6 py-5">
        <pre class="overflow-auto rounded-2xl bg-slate-950 p-4 text-xs leading-6 text-slate-100">{JSON.stringify(selectedPeak.Why, null, 2)}</pre>
      </div>
    </div>
  </div>
{/if}
