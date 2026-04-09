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
    // Split by polarity so we get separate legend entries
    const groups: Record<string, {
      x: number[]; y: number[]; z: number[];
      text: string[]; sizes: number[]; colors: number[];
    }> = {};

    for (const p of data) {
      const rt   = Number(p.RT_mean);
      const mz   = Number(p.MZ_mean);
      const area = Number(p.Area_mean);
      const conf = Number(p.ConfidenceScore);
      const cv   = Number(p.AreaCVPct);
      if (!Number.isFinite(rt) || !Number.isFinite(mz) || !Number.isFinite(area) || area <= 0) continue;

      const polarity = p.Polarity === "positive" ? "+" : p.Polarity === "negative" ? "−" : String(p.Polarity || "?");
      const status   = p.Status === "Real Compound" ? "Real" : "Artifact";
      const key      = `${polarity} · ${status}`;

      if (!groups[key]) groups[key] = { x: [], y: [], z: [], text: [], sizes: [], colors: [] };
      const g = groups[key];

      const logArea = Math.log10(area);
      g.x.push(rt);
      g.y.push(mz);
      g.z.push(logArea);
      g.colors.push(Number.isFinite(conf) ? conf : 0);

      // Size from CV: low CV → bigger dot, missing CV → medium
      const cvSize = Number.isFinite(cv) && cv > 0
        ? Math.max(3, Math.min(14, 14 - cv * 0.18))
        : 6;
      g.sizes.push(cvSize);

      g.text.push(
        `<b>${p.Status ?? "—"}</b>  ${polarity}<br>` +
        `RT: ${rt.toFixed(2)} min<br>` +
        `m/z: ${mz.toFixed(2)}<br>` +
        `Area: ${area.toLocaleString()}<br>` +
        `log₁₀(Area): ${logArea.toFixed(2)}<br>` +
        `CV%: ${Number.isFinite(cv) ? cv.toFixed(2) : "—"}<br>` +
        `S/B: ${Number.isFinite(Number(p.SignalToBlankRatio)) ? Number(p.SignalToBlankRatio).toFixed(2) : "—"}<br>` +
        `Confidence: ${Number.isFinite(conf) ? conf.toFixed(2) : "—"}<br>` +
        `Quality: ${p.ReplicateQuality ?? "—"}`
      );
    }

    // Color scheme per group key
    const palette: Record<string, string> = {
      "+ · Real":     "#2563eb",
      "+ · Artifact": "#93c5fd",
      "− · Real":     "#d97706",
      "− · Artifact": "#fcd34d",
    };

    return Object.entries(groups).map(([key, g]) => ({
      type: "scatter3d",
      mode: "markers",
      name: key,
      x: g.x, y: g.y, z: g.z,
      text: g.text,
      hovertemplate: "%{text}<extra></extra>",
      marker: {
        size: g.sizes,
        color: g.colors,
        colorscale: [
          [0,    "#ef4444"],
          [0.4,  "#f59e0b"],
          [0.75, "#22c55e"],
          [1,    "#2563eb"],
        ],
        cmin: 0, cmax: 100,
        showscale: Object.keys(groups).indexOf(key) === 0, // only first trace shows colorbar
        colorbar: {
          title: { text: "Conf", side: "right" },
          thickness: 12,
          len: 0.5,
          y: 0.5,
          tickfont: { size: 9 },
        },
        opacity: key.includes("Artifact") ? 0.45 : 0.82,
        symbol: key.includes("Artifact") ? "cross" : "circle",
        line: { color: "rgba(255,255,255,0.2)", width: 0.5 },
      },
    }));
  }

  function isDark() {
    return document.documentElement.classList.contains("dark");
  }

  function buildLayout(dark: boolean) {
    const bg    = dark ? "#0f172a" : "#ffffff";
    const paper = dark ? "#1e293b" : "#f8fafc";
    const grid  = dark ? "#334155" : "#e2e8f0";
    const label = dark ? "#94a3b8" : "#475569";
    const title = dark ? "#e2e8f0" : "#0f172a";
    const legBg = dark ? "rgba(15,23,42,0.8)" : "rgba(255,255,255,0.85)";

    const axis = {
      gridcolor: grid, backgroundcolor: paper,
      showbackground: true,
      tickfont: { size: 10, color: label },
      titlefont: { size: 12, color: label },
      linecolor: grid, zerolinecolor: grid,
    };

    return {
      scene: {
        xaxis: { ...axis, title: "RT (min)" },
        yaxis: { ...axis, title: "m/z" },
        zaxis: { ...axis, title: "log₁₀(Area)" },
        bgcolor: bg,
        aspectmode: "manual",
        aspectratio: { x: 1.6, y: 1.2, z: 0.8 },
        camera: { eye: { x: 1.5, y: -1.8, z: 0.7 } },
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
    <div bind:this={container} class="h-[640px] min-w-0 flex-1 rounded-xl"></div>
    <aside class="w-48 shrink-0 self-start rounded-xl border border-slate-200 bg-slate-50 p-4 text-xs dark:border-slate-700 dark:bg-slate-900">
      <p class="mb-3 text-slate-600 dark:text-slate-300">
        Аналітичний простір LC-MS: де розташовані компаунди за часом утримання, масою та інтенсивністю. Висота = сила сигналу.
      </p>
      <ul class="space-y-1.5 text-slate-500 dark:text-slate-400">
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">X</span> = RT (хв)</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Y</span> = m/z</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Z</span> = log₁₀(Area)</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Колір</span> = Confidence</li>
        <li><span class="font-semibold text-slate-700 dark:text-slate-200">Розмір</span> = 1/CV% (більший = стабільніший)</li>
      </ul>
      <div class="mt-3 space-y-1 border-t border-slate-200 pt-3 dark:border-slate-700">
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-2.5 rounded-full bg-blue-500"></span><span class="text-slate-500 dark:text-slate-400">Positive · Real</span></div>
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-2.5 rounded-full bg-blue-300"></span><span class="text-slate-500 dark:text-slate-400">Positive · Artifact</span></div>
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-500"></span><span class="text-slate-500 dark:text-slate-400">Negative · Real</span></div>
        <div class="flex items-center gap-1.5"><span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-300"></span><span class="text-slate-500 dark:text-slate-400">Negative · Artifact</span></div>
      </div>
    </aside>
  </div>
</div>
