<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  let { peaks, parameters } = $props<{
    peaks: any[];
    parameters?: any;
  }>();

  let container: HTMLDivElement;
  let loading = $state(true);
  let error = $state<string | null>(null);
  let Plotly: any = null;
  let rendered = false;

  // ── Data ─────────────────────────────────────────────────────────────────

  function buildTraces(data: any[]) {
    const groups: Record<string, { x: number[]; y: number[]; z: number[]; text: string[]; sizes: number[] }> = {
      "Real Compound": { x: [], y: [], z: [], text: [], sizes: [] },
      "Artifact":      { x: [], y: [], z: [], text: [], sizes: [] },
    };

    for (const p of data) {
      const cv   = Number(p.AreaCVPct);
      const sb   = Number(p.SignalToBlankRatio);
      const conf = Number(p.ConfidenceScore);
      const area = Number(p.Area_mean);
      if (!Number.isFinite(cv) || !Number.isFinite(sb) || !Number.isFinite(conf)) continue;

      const key = p.Status === "Real Compound" ? "Real Compound" : "Artifact";
      const g = groups[key];
      g.x.push(cv);
      g.y.push(sb);
      g.z.push(conf);

      const logArea = Number.isFinite(area) && area > 0 ? Math.log10(area) : 3;
      g.sizes.push(Math.max(4, Math.min(16, logArea * 1.4)));

      g.text.push(
        `<b>${p.Status ?? "—"}</b><br>` +
        `RT: ${Number(p.RT_mean).toFixed(2)}<br>` +
        `m/z: ${Number(p.MZ_mean).toFixed(2)}<br>` +
        `CV%: ${cv.toFixed(2)}<br>` +
        `S/B: ${sb.toFixed(2)}<br>` +
        `Confidence: ${conf.toFixed(2)}<br>` +
        `Quality: ${p.ReplicateQuality ?? "—"}<br>` +
        `Area: ${Number.isFinite(area) ? area.toLocaleString() : "—"}`
      );
    }

    return [
      {
        type: "scatter3d", mode: "markers", name: "Real Compound",
        x: groups["Real Compound"].x,
        y: groups["Real Compound"].y,
        z: groups["Real Compound"].z,
        text: groups["Real Compound"].text,
        hovertemplate: "%{text}<extra></extra>",
        marker: {
          size: groups["Real Compound"].sizes,
          color: "#10b981",
          opacity: 0.85,
          line: { color: "rgba(255,255,255,0.3)", width: 0.5 },
        },
      },
      {
        type: "scatter3d", mode: "markers", name: "Artifact",
        x: groups["Artifact"].x,
        y: groups["Artifact"].y,
        z: groups["Artifact"].z,
        text: groups["Artifact"].text,
        hovertemplate: "%{text}<extra></extra>",
        marker: {
          size: groups["Artifact"].sizes,
          symbol: "cross",
          color: "#f43f5e",
          opacity: 0.75,
          line: { color: "rgba(255,255,255,0.3)", width: 0.5 },
        },
      },
    ];
  }

  function isDark() {
    return document.documentElement.classList.contains("dark");
  }

  function buildLayout(dark: boolean) {
    const bg      = dark ? "#0f172a" : "#ffffff";
    const paper   = dark ? "#1e293b" : "#f8fafc";
    const grid    = dark ? "#334155" : "#e2e8f0";
    const label   = dark ? "#94a3b8" : "#475569";
    const title   = dark ? "#e2e8f0" : "#0f172a";
    const legBg   = dark ? "rgba(15,23,42,0.8)" : "rgba(255,255,255,0.85)";

    const axis = {
      gridcolor: grid, backgroundcolor: paper,
      showbackground: true,
      tickfont: { size: 10, color: label },
      titlefont: { size: 12, color: label },
      linecolor: grid, zerolinecolor: grid,
    };

    return {
      scene: {
        xaxis: { ...axis, title: "CV%" },
        yaxis: { ...axis, title: "S/B" },
        zaxis: { ...axis, title: "Confidence" },
        bgcolor: bg,
        aspectmode: "cube",
        camera: { eye: { x: 1.7, y: 1.7, z: 0.85 } },
      },
      paper_bgcolor: paper,
      plot_bgcolor: bg,
      margin: { l: 0, r: 0, t: 10, b: 0 },
      showlegend: false,
      font: { color: title },
    };
  }

  const config = {
    responsive: true,
    displaylogo: false,
    modeBarButtonsToRemove: ["resetCameraLastSave3d"],
  };

  async function draw() {
    if (!container || !Plotly) return;
    const traces = buildTraces(peaks);
    const layout = buildLayout(isDark());
    if (rendered) {
      Plotly.react(container, traces, layout, config);
    } else {
      Plotly.newPlot(container, traces, layout, config);
      rendered = true;
    }
  }

  let observer: MutationObserver | null = null;
  let resizeObserver: ResizeObserver | null = null;

  onMount(async () => {
    try {
      const mod = await import("plotly.js-dist-min");
      // UMD bundle: real Plotly object is on .default
      Plotly = mod.default ?? mod;
      loading = false;
      await draw();

      observer = new MutationObserver(() => {
        if (rendered) draw();
      });
      observer.observe(document.documentElement, {
        attributes: true, attributeFilter: ["class"],
      });

      resizeObserver = new ResizeObserver(() => {
        if (Plotly && container && rendered) Plotly.Plots.resize(container);
      });
      resizeObserver.observe(container);
    } catch (e: any) {
      error = String(e?.message ?? e);
      loading = false;
    }
  });

  onDestroy(() => {
    observer?.disconnect();
    resizeObserver?.disconnect();
    try { if (Plotly && container) Plotly.purge(container); } catch {}
  });

  // Redraw when peaks change (after initial mount)
  $effect(() => {
    void peaks;
    if (Plotly && !loading) draw();
  });
</script>

<div class="w-full">
  {#if loading}
    <div class="flex h-96 items-center justify-center gap-2 text-sm text-slate-400 dark:text-slate-500">
      <svg class="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"
          stroke-dasharray="40" stroke-dashoffset="15"/>
      </svg>
      Завантаження Plotly…
    </div>
  {:else if error}
    <div class="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700 dark:border-rose-800 dark:bg-rose-950 dark:text-rose-300">
      <strong>Помилка:</strong> {error}
    </div>
  {/if}

  <div class="flex gap-4" class:hidden={loading || !!error}>
    <div bind:this={container} class="h-[620px] min-w-0 flex-1 rounded-xl"></div>
    <aside class="w-48 shrink-0 self-start rounded-xl border border-slate-200 bg-slate-50 p-4 text-xs dark:border-slate-700 dark:bg-slate-900">
      <p class="mb-3 text-slate-600 dark:text-slate-300">
        Просторовий розподіл компаундів у трьох вимірах якості. Дозволяє одразу побачити, де Real та Artifact розходяться і наскільки.
      </p>
      <ul class="space-y-1.5 text-slate-500 dark:text-slate-400">
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">X</span> = CV% — відтворюваність</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Y</span> = S/B — специфічність</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Z</span> = Confidence Score</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Розмір</span> = log₁₀(Area)</li>
      </ul>
      <div class="mt-3 space-y-1 border-t border-slate-200 pt-3 dark:border-slate-700">
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-2.5 rounded-full bg-emerald-500"></span><span class="text-slate-500 dark:text-slate-400">Real Compound</span></div>
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-2.5 rounded-full bg-rose-500"></span><span class="text-slate-500 dark:text-slate-400">Artifact</span></div>
      </div>
    </aside>
  </div>
</div>
