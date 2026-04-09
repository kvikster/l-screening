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
    const groups: Record<string, {
      x: number[]; y: number[]; conf: number[]; text: string[];
    }> = {
      "Real Compound": { x: [], y: [], conf: [], text: [] },
      "Artifact":      { x: [], y: [], conf: [], text: [] },
    };

    for (const p of data) {
      const area = Number(p.Area_mean);
      const cv   = Number(p.AreaCVPct);
      const conf = Number(p.ConfidenceScore);
      const rt   = Number(p.RT_mean);
      const mz   = Number(p.MZ_mean);
      const sb   = Number(p.SignalToBlankRatio);
      if (!Number.isFinite(area) || area <= 0 || !Number.isFinite(cv) || cv <= 0) continue;

      const key = p.Status === "Real Compound" ? "Real Compound" : "Artifact";
      const g = groups[key];
      g.x.push(Math.log10(area));
      g.y.push(cv);
      g.conf.push(Number.isFinite(conf) ? conf : 0);

      const polarity = p.Polarity === "positive" ? "+" : p.Polarity === "negative" ? "−" : String(p.Polarity || "?");
      g.text.push(
        `<b>${p.Status ?? "—"}</b>  ${polarity}<br>` +
        `RT: ${Number.isFinite(rt) ? rt.toFixed(2) : "—"} min<br>` +
        `m/z: ${Number.isFinite(mz) ? mz.toFixed(2) : "—"}<br>` +
        `Area: ${area.toLocaleString()}<br>` +
        `log₁₀(Area): ${Math.log10(area).toFixed(2)}<br>` +
        `CV%: ${cv.toFixed(2)}<br>` +
        `S/B: ${Number.isFinite(sb) ? sb.toFixed(2) : "—"}<br>` +
        `Confidence: ${Number.isFinite(conf) ? conf.toFixed(2) : "—"}<br>` +
        `Quality: ${p.ReplicateQuality ?? "—"}`
      );
    }

    return [
      {
        type: "scatter",
        mode: "markers",
        name: "Real Compound",
        x: groups["Real Compound"].x,
        y: groups["Real Compound"].y,
        text: groups["Real Compound"].text,
        hovertemplate: "%{text}<extra></extra>",
        marker: {
          size: 8,
          color: groups["Real Compound"].conf,
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
          opacity: 0.82,
          symbol: "circle",
          line: { color: "rgba(255,255,255,0.3)", width: 0.5 },
        },
      },
      {
        type: "scatter",
        mode: "markers",
        name: "Artifact",
        x: groups["Artifact"].x,
        y: groups["Artifact"].y,
        text: groups["Artifact"].text,
        hovertemplate: "%{text}<extra></extra>",
        marker: {
          size: 7,
          color: groups["Artifact"].conf,
          colorscale: [
            [0,    "#ef4444"],
            [0.4,  "#f59e0b"],
            [0.75, "#22c55e"],
            [1,    "#2563eb"],
          ],
          cmin: 0, cmax: 100,
          showscale: false,
          opacity: 0.45,
          symbol: "x",
          line: { color: "rgba(255,255,255,0.2)", width: 0.5 },
        },
      },
    ];
  }

  function buildShapes(dark: boolean) {
    const shapes: any[] = [];
    const cvMax = Number(parameters?.cv_max ?? parameters?.cvMax);
    if (Number.isFinite(cvMax) && cvMax > 0) {
      shapes.push({
        type: "line", x0: 0, x1: 1, xref: "paper",
        y0: cvMax, y1: cvMax,
        line: { color: dark ? "#fbbf24" : "#d97706", width: 1.5, dash: "dash" },
      });
    }
    return shapes;
  }

  function buildAnnotations(dark: boolean) {
    const ann: any[] = [];
    const cvMax = Number(parameters?.cv_max ?? parameters?.cvMax);
    if (Number.isFinite(cvMax) && cvMax > 0) {
      ann.push({
        x: 1, xref: "paper", y: cvMax,
        text: `CV% поріг: ${cvMax}`,
        showarrow: false,
        xanchor: "right",
        yanchor: "bottom",
        font: { size: 10, color: dark ? "#fbbf24" : "#d97706" },
      });
    }
    return ann;
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
      gridcolor: grid, zerolinecolor: grid, linecolor: grid,
      tickfont: { size: 10, color: label },
      titlefont: { size: 12, color: label },
    };

    return {
      xaxis: { ...axis, title: "log₁₀(Area)" },
      yaxis: { ...axis, title: "CV%" },
      paper_bgcolor: paper,
      plot_bgcolor: bg,
      margin: { l: 55, r: 20, t: 10, b: 50 },
      showlegend: false,
      font: { color: title },
      shapes: buildShapes(dark),
      annotations: buildAnnotations(dark),
    };
  }

  const config = { responsive: true, displaylogo: false };

  async function draw() {
    if (!container || !Plotly) return;
    const dark = isDark();
    const traces = buildTraces(peaks);
    const layout = buildLayout(dark);
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
        Фундаментальна аналітична залежність: слабкі сигнали завжди мають вищий CV%. Артефакти зі стабільно низьким CV% при малій інтенсивності — ознака контамінації.
      </p>
      <ul class="space-y-1.5 text-slate-500 dark:text-slate-400">
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">X</span> = log₁₀(Area)</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Y</span> = CV% реплікатів</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Колір</span> = Confidence</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">×</span> = Artifact</li>
        <li><span class="font-semibold text-amber-600 dark:text-amber-400">Жовтий</span> = CV% поріг</li>
      </ul>
    </aside>
  </div>
</div>
