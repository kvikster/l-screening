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
    const real: number[] = [];
    const art: number[] = [];

    for (const p of data) {
      const conf = Number(p.ConfidenceScore);
      if (!Number.isFinite(conf)) continue;
      if (p.Status === "Real Compound") real.push(conf);
      else art.push(conf);
    }

    const traces: any[] = [];

    if (art.length) {
      traces.push({
        type: "histogram",
        name: "Artifact",
        x: art,
        nbinsx: 25,
        opacity: 0.65,
        marker: { color: "#f43f5e" },
        hovertemplate: "Confidence: %{x:.1f}<br>Count: %{y}<extra>Artifact</extra>",
      });
    }

    if (real.length) {
      traces.push({
        type: "histogram",
        name: "Real Compound",
        x: real,
        nbinsx: 25,
        opacity: 0.72,
        marker: { color: "#10b981" },
        hovertemplate: "Confidence: %{x:.1f}<br>Count: %{y}<extra>Real Compound</extra>",
      });
    }

    return traces;
  }

  function buildAnnotations(data: any[], dark: boolean) {
    const real: number[] = [];
    const art: number[] = [];
    for (const p of data) {
      const conf = Number(p.ConfidenceScore);
      if (!Number.isFinite(conf)) continue;
      if (p.Status === "Real Compound") real.push(conf);
      else art.push(conf);
    }
    const mean = (arr: number[]) => arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : null;
    const annotations = [];
    const label = dark ? "#e2e8f0" : "#0f172a";

    const mReal = mean(real);
    const mArt  = mean(art);

    if (mReal !== null) {
      annotations.push({
        x: mReal, y: 0, yref: "paper",
        text: `μ Real = ${mReal.toFixed(1)}`,
        showarrow: false,
        yanchor: "bottom",
        font: { size: 10, color: "#10b981" },
        bgcolor: "rgba(0,0,0,0)",
        xanchor: "left",
        ay: 20,
      });
    }
    if (mArt !== null) {
      annotations.push({
        x: mArt, y: 0, yref: "paper",
        text: `μ Art = ${mArt.toFixed(1)}`,
        showarrow: false,
        yanchor: "bottom",
        font: { size: 10, color: "#f43f5e" },
        bgcolor: "rgba(0,0,0,0)",
        xanchor: "right",
        ay: 20,
      });
    }
    return annotations;
  }

  function buildShapes(data: any[], dark: boolean) {
    const real: number[] = [];
    const art: number[] = [];
    for (const p of data) {
      const conf = Number(p.ConfidenceScore);
      if (!Number.isFinite(conf)) continue;
      if (p.Status === "Real Compound") real.push(conf);
      else art.push(conf);
    }
    const mean = (arr: number[]) => arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : null;
    const shapes = [];

    const mReal = mean(real);
    const mArt  = mean(art);

    if (mReal !== null) {
      shapes.push({
        type: "line", x0: mReal, x1: mReal, y0: 0, y1: 1, yref: "paper",
        line: { color: "#10b981", width: 1.5, dash: "dot" },
      });
    }
    if (mArt !== null) {
      shapes.push({
        type: "line", x0: mArt, x1: mArt, y0: 0, y1: 1, yref: "paper",
        line: { color: "#f43f5e", width: 1.5, dash: "dot" },
      });
    }

    // Threshold from parameters if present
    const thresh = Number(parameters?.confidence_min ?? parameters?.confidenceMin);
    if (Number.isFinite(thresh) && thresh > 0) {
      shapes.push({
        type: "line", x0: thresh, x1: thresh, y0: 0, y1: 1, yref: "paper",
        line: { color: dark ? "#fbbf24" : "#d97706", width: 1.5, dash: "dash" },
      });
    }

    return shapes;
  }

  function isDark() {
    return document.documentElement.classList.contains("dark");
  }

  function buildLayout(data: any[], dark: boolean) {
    const paper = dark ? "#1e293b" : "#f8fafc";
    const bg    = dark ? "#0f172a" : "#ffffff";
    const grid  = dark ? "#334155" : "#e2e8f0";
    const label = dark ? "#94a3b8" : "#475569";
    const title = dark ? "#e2e8f0" : "#0f172a";
    const legBg = dark ? "rgba(15,23,42,0.8)" : "rgba(255,255,255,0.85)";

    const axis = {
      gridcolor: grid, zerolinecolor: grid, linecolor: grid,
      tickfont: { size: 10, color: label },
      titlefont: { size: 12, color: label },
    };

    return {
      barmode: "overlay",
      xaxis: { ...axis, title: "Confidence Score", range: [0, 100] },
      yaxis: { ...axis, title: "Кількість піків" },
      paper_bgcolor: paper,
      plot_bgcolor: bg,
      margin: { l: 55, r: 20, t: 10, b: 50 },
      showlegend: false,
      font: { color: title },
      shapes: buildShapes(data, dark),
      annotations: buildAnnotations(data, dark),
    };
  }

  const config = { responsive: true, displaylogo: false };

  async function draw() {
    if (!container || !Plotly) return;
    const dark = isDark();
    const traces = buildTraces(peaks);
    const layout = buildLayout(peaks, dark);
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
      Plotly = mod.default ?? mod;
      loading = false;
      await draw();
      observer = new MutationObserver(() => { if (rendered) draw(); });
      observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });

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
    <div bind:this={container} class="h-[540px] min-w-0 flex-1 rounded-xl"></div>
    <aside class="w-48 shrink-0 self-start rounded-xl border border-slate-200 bg-slate-50 p-4 text-xs dark:border-slate-700 dark:bg-slate-900">
      <p class="mb-3 text-slate-600 dark:text-slate-300">
        Розподіл Confidence Score окремо для Real та Artifact. Чим менше перекриття — тим чіткіше алгоритм розділяє класи. Зона перетину є зоною неоднозначності.
      </p>
      <ul class="space-y-1.5 text-slate-500 dark:text-slate-400">
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">X</span> = Confidence Score (0–100)</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Y</span> = кількість піків</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Пунктир</span> = середнє групи</li>
        <li><span class="font-semibold text-amber-600 dark:text-amber-400">Жовтий</span> = поріг параметрів</li>
      </ul>
      <div class="mt-3 space-y-1 border-t border-slate-200 pt-3 dark:border-slate-700">
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-4 rounded-sm bg-emerald-500"></span><span class="text-slate-500 dark:text-slate-400">Real Compound</span></div>
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-4 rounded-sm bg-rose-500"></span><span class="text-slate-500 dark:text-slate-400">Artifact</span></div>
      </div>
    </aside>
  </div>
</div>
