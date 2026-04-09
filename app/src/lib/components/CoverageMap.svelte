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
    // Background density contour (all points)
    const allRt: number[] = [];
    const allMz: number[] = [];

    const groups: Record<string, {
      rt: number[]; mz: number[]; area: number[]; conf: number[]; text: string[];
    }> = {
      "Real Compound": { rt: [], mz: [], area: [], conf: [], text: [] },
      "Artifact":      { rt: [], mz: [], area: [], conf: [], text: [] },
    };

    for (const p of data) {
      const rt   = Number(p.RT_mean);
      const mz   = Number(p.MZ_mean);
      const area = Number(p.Area_mean);
      const conf = Number(p.ConfidenceScore);
      const cv   = Number(p.AreaCVPct);
      const sb   = Number(p.SignalToBlankRatio);
      if (!Number.isFinite(rt) || !Number.isFinite(mz)) continue;

      allRt.push(rt);
      allMz.push(mz);

      const key = p.Status === "Real Compound" ? "Real Compound" : "Artifact";
      const g = groups[key];
      g.rt.push(rt);
      g.mz.push(mz);
      g.area.push(Number.isFinite(area) && area > 0 ? Math.log10(area) : 3);
      g.conf.push(Number.isFinite(conf) ? conf : 0);

      const polarity = p.Polarity === "positive" ? "+" : p.Polarity === "negative" ? "−" : String(p.Polarity || "?");
      g.text.push(
        `<b>${p.Status ?? "—"}</b>  ${polarity}<br>` +
        `RT: ${rt.toFixed(2)} min<br>` +
        `m/z: ${mz.toFixed(2)}<br>` +
        `Area: ${Number.isFinite(area) ? area.toLocaleString() : "—"}<br>` +
        `CV%: ${Number.isFinite(cv) ? cv.toFixed(2) : "—"}<br>` +
        `S/B: ${Number.isFinite(sb) ? sb.toFixed(2) : "—"}<br>` +
        `Confidence: ${Number.isFinite(conf) ? conf.toFixed(2) : "—"}<br>` +
        `Quality: ${p.ReplicateQuality ?? "—"}`
      );
    }

    const traces: any[] = [];

    // Density contour background
    if (allRt.length >= 3) {
      traces.push({
        type: "histogram2dcontour",
        x: allRt,
        y: allMz,
        name: "Density",
        showlegend: false,
        ncontours: 10,
        colorscale: [
          [0,   "rgba(0,0,0,0)"],
          [0.3, "rgba(99,102,241,0.08)"],
          [0.7, "rgba(99,102,241,0.18)"],
          [1,   "rgba(99,102,241,0.32)"],
        ],
        reversescale: false,
        showscale: false,
        contours: { coloring: "fill", showlines: true, size: 0 },
        line: { width: 0.5, color: "rgba(99,102,241,0.35)" },
        hoverinfo: "skip",
      });
    }

    // Real Compound scatter
    const real = groups["Real Compound"];
    if (real.rt.length) {
      traces.push({
        type: "scatter",
        mode: "markers",
        name: "Real Compound",
        x: real.rt,
        y: real.mz,
        text: real.text,
        hovertemplate: "%{text}<extra></extra>",
        marker: {
          size: real.area.map(a => Math.max(6, Math.min(18, a * 2.2))),
          color: real.conf,
          colorscale: [
            [0,    "#ef4444"],
            [0.4,  "#f59e0b"],
            [0.75, "#22c55e"],
            [1,    "#2563eb"],
          ],
          cmin: 0, cmax: 100,
          showscale: true,
          colorbar: {
            title: { text: "Conf", side: "right" },
            thickness: 12,
            len: 0.5,
            y: 0.5,
            tickfont: { size: 9 },
          },
          opacity: 0.85,
          symbol: "circle",
          line: { color: "rgba(255,255,255,0.4)", width: 0.8 },
        },
      });
    }

    // Artifact scatter
    const art = groups["Artifact"];
    if (art.rt.length) {
      traces.push({
        type: "scatter",
        mode: "markers",
        name: "Artifact",
        x: art.rt,
        y: art.mz,
        text: art.text,
        hovertemplate: "%{text}<extra></extra>",
        marker: {
          size: art.area.map(a => Math.max(5, Math.min(14, a * 1.8))),
          color: art.conf,
          colorscale: [
            [0,   "#ef4444"],
            [0.4, "#f59e0b"],
            [0.75,"#22c55e"],
            [1,   "#2563eb"],
          ],
          cmin: 0, cmax: 100,
          showscale: false,
          opacity: 0.45,
          symbol: "x",
          line: { color: "rgba(255,255,255,0.3)", width: 0.5 },
        },
      });
    }

    return traces;
  }

  function isDark() {
    return document.documentElement.classList.contains("dark");
  }

  function buildLayout(dark: boolean) {
    const paper = dark ? "#1e293b" : "#f8fafc";
    const bg    = dark ? "#0f172a" : "#ffffff";
    const grid  = dark ? "#334155" : "#e2e8f0";
    const label = dark ? "#94a3b8" : "#475569";
    const title = dark ? "#e2e8f0" : "#0f172a";
    const legBg = dark ? "rgba(15,23,42,0.8)" : "rgba(255,255,255,0.85)";

    const axis = {
      gridcolor: grid,
      zerolinecolor: grid,
      linecolor: grid,
      tickfont: { size: 10, color: label },
      titlefont: { size: 12, color: label },
    };

    return {
      xaxis: { ...axis, title: "RT (min)" },
      yaxis: { ...axis, title: "m/z" },
      paper_bgcolor: paper,
      plot_bgcolor: bg,
      margin: { l: 60, r: 20, t: 10, b: 50 },
      showlegend: false,
      font: { color: title },
    };
  }

  const config = {
    responsive: true,
    displaylogo: false,
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
      Plotly = mod.default ?? mod;
      loading = false;
      await draw();

      observer = new MutationObserver(() => { if (rendered) draw(); });
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
        Покриття аналітичного простору RT × m/z. Контури густоти показують, де компаунди кластеризуються — допомагає виявити перевантажені зони та прогалини.
      </p>
      <ul class="space-y-1.5 text-slate-500 dark:text-slate-400">
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">X</span> = RT (хв)</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Y</span> = m/z</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Колір</span> = Confidence</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Розмір</span> = log(Area)</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Фон</span> = щільність піків</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">×</span> = Artifact</li>
      </ul>
    </aside>
  </div>
</div>
