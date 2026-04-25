<script lang="ts">
  import { onMount } from "svelte";
  import { Download, Loader2 } from "lucide-svelte";
  import PeaksDataTable from "./PeaksDataTable.svelte";
  import QualityCube from "./QualityCube.svelte";
  import FeatureLandscape from "./FeatureLandscape.svelte";
  import CoverageMap from "./CoverageMap.svelte";
  import ScoreSeparation from "./ScoreSeparation.svelte";
  import PrecisionIntensity from "./PrecisionIntensity.svelte";
  import {
    dictionary,
    getNumberLocale,
    getSampleTypeLabel,
    getStatusLabel,
    locale,
    localeLabels,
    setLocale,
    t,
  } from "$lib/i18n";
  import { theme, cycleTheme, themeIcons } from "$lib/theme";

  let allProps = $props<any>();
  let element = $derived(allProps.element || { props: allProps });
  let dict = $derived($dictionary);
  let currentTheme = $derived($theme);
  let currentLocale = $derived($locale);
  let themeLabels = $derived({ auto: dict.themeAuto, light: dict.themeLight, dark: dict.themeDark });
  let title = $derived(element.props?.title || dict.noTitle);
  let summary = $derived(element.props?.summary || []);
  let peaks = $derived(element.props?.peaks || []);
  let parameters = $derived(element.props?.parameters || {});
  let metadata = $derived(element.props?.metadata || {});
  let onlineStatus = $derived(Boolean(allProps.onlineStatus));
  let exporting = $derived(Boolean(allProps.exporting));
  let exportingHtml = $derived(Boolean(allProps.exportingHtml));
  let onExportXlsx = $derived((allProps.onExportXlsx as (() => void) | undefined) ?? (() => {}));
  let onExportHtml = $derived((allProps.onExportHtml as (() => void) | undefined) ?? (() => {}));
  let onUploadNewFile = $derived((allProps.onUploadNewFile as (() => void) | undefined) ?? (() => {}));
  let selectedPeak = $state<any>(null);
  let auditPanelEl = $state<HTMLElement | null>(null);
  let methodParamsOpen = $state(false);
  let summaryMetricsOpen = $state(false);
  let activeTab = $state<"summary" | "data" | "3d">("data");
  let view3d = $state<"quality" | "landscape" | "coverage" | "score" | "precision">("quality");
  let dataFiltersOpen = $state(false);
  let openKpiHint = $state<string | null>(null);
  let activeMetric = $state<"confirmed" | "cv" | "confidence" | null>(null);
  let showAllConfirmedRows = $state(false);
  let auditJsonQuery = $state("");
  let auditDetailTab = $state<"review" | "summary" | "evidence" | "raw">("review");
  let navigablePeaks = $state<any[]>([]);
  let tableNavigationReady = $state(false);
  const confirmedRowsViewStorageKey = "dashboard.confirmedRows.showAll";
  const auditPanelCopy = {
    uk: {
      reviewLabel: "Розбір рішення",
      selectPeak: "Оберіть пік у таблиці",
      selectPeakDesc: "Відкрийте будь-який рядок, щоб побачити verdict, сигнали рішення, контекст і raw details без модалки.",
      likelyArtifact: "Ймовірний артефакт",
      likelyReal: "Ймовірна реальна сполука",
      manualReview: "Потрібна ручна перевірка",
      autoReject: "Можна відхилити без додаткового рев'ю",
      keepDecision: "Можна залишити як реальну сполуку",
      whyFlagged: "Чому система спрацювала",
      evidenceSignals: "Сигнали рішення",
      context: "Контекст",
      recommendation: "Рекомендована дія",
      queue: "Черга",
      prev: "Попередній",
      next: "Наступний",
      closeReview: "Закрити review",
      reviewTab: "Розбір рішення",
      summaryTab: "Summary",
      evidenceTab: "Evidence",
      rawTab: "Raw data",
      blankMatch: "Blank match",
      blankCandidates: "Кандидати blank",
      blankFound: "Знайдено відповідник у blank.",
      blankNotFound: "Збігу з blank не знайдено.",
      cvLabel: "CV%",
      sbLabel: "S/B",
      confidenceLabel: "Confidence",
      actual: "Факт",
      threshold: "Поріг",
      delta: "Delta",
      searchJson: "Пошук по ключу або значенню",
      noMatches: "Нічого не знайдено",
      reasonDefault: "Рішення базується на комбінації replicate quality, blank subtraction і confidence score.",
      blankContext: "Найближчий blank",
      matchingMode: "Режим зіставлення",
      replicateQuality: "Якість реплікатів",
      replicateCount: "К-сть реплікатів",
      sampleType: "Тип зразка",
      polarity: "Полярність",
      highConfidence: "висока впевненість",
      mediumConfidence: "середня впевненість",
      lowConfidence: "низька впевненість",
      aboveThreshold: "вище порогу",
      belowThreshold: "нижче порогу",
      matchingCandidates: "кандидатів",
      noBlankDetail: "Деталі blank не знайдено",
      keyboardHint: "",
    },
    en: {
      reviewLabel: "Decision review",
      selectPeak: "Select a peak from the table",
      selectPeakDesc: "Open any row to inspect the verdict, decision signals, context, and raw details without a modal.",
      likelyArtifact: "Likely artifact",
      likelyReal: "Likely real compound",
      manualReview: "Manual review recommended",
      autoReject: "Can be rejected without extra review",
      keepDecision: "Can remain classified as a real compound",
      whyFlagged: "Why the system flagged this",
      evidenceSignals: "Decision signals",
      context: "Context",
      recommendation: "Recommended action",
      queue: "Queue",
      prev: "Previous",
      next: "Next",
      closeReview: "Close review",
      reviewTab: "Decision review",
      summaryTab: "Summary",
      evidenceTab: "Evidence",
      rawTab: "Raw data",
      blankMatch: "Blank match",
      blankCandidates: "Blank candidates",
      blankFound: "A matching blank peak was found.",
      blankNotFound: "No matching blank peak was found.",
      cvLabel: "CV%",
      sbLabel: "S/B",
      confidenceLabel: "Confidence",
      actual: "Actual",
      threshold: "Threshold",
      delta: "Delta",
      searchJson: "Search by key or value",
      noMatches: "Nothing found",
      reasonDefault: "The decision blends replicate quality, blank subtraction, and confidence score.",
      blankContext: "Closest blank",
      matchingMode: "Matching mode",
      replicateQuality: "Replicate quality",
      replicateCount: "Replicate count",
      sampleType: "Sample type",
      polarity: "Polarity",
      highConfidence: "high confidence",
      mediumConfidence: "medium confidence",
      lowConfidence: "low confidence",
      aboveThreshold: "above threshold",
      belowThreshold: "below threshold",
      matchingCandidates: "candidates",
      noBlankDetail: "No blank details found",
      keyboardHint: "",
    },
    ru: {
      reviewLabel: "Разбор решения",
      selectPeak: "Выберите пик в таблице",
      selectPeakDesc: "Откройте любую строку, чтобы увидеть verdict, сигналы решения, контекст и raw details без модалки.",
      likelyArtifact: "Вероятный артефакт",
      likelyReal: "Вероятное реальное соединение",
      manualReview: "Нужна ручная проверка",
      autoReject: "Можно отклонить без дополнительного ревью",
      keepDecision: "Можно оставить как реальное соединение",
      whyFlagged: "Почему система сработала",
      evidenceSignals: "Сигналы решения",
      context: "Контекст",
      recommendation: "Рекомендуемое действие",
      queue: "Очередь",
      prev: "Предыдущий",
      next: "Следующий",
      closeReview: "Закрыть review",
      reviewTab: "Разбор решения",
      summaryTab: "Summary",
      evidenceTab: "Evidence",
      rawTab: "Raw data",
      blankMatch: "Blank match",
      blankCandidates: "Кандидаты blank",
      blankFound: "Найден соответствующий пик в blank.",
      blankNotFound: "Совпадение с blank не найдено.",
      cvLabel: "CV%",
      sbLabel: "S/B",
      confidenceLabel: "Confidence",
      actual: "Факт",
      threshold: "Порог",
      delta: "Delta",
      searchJson: "Поиск по ключу или значению",
      noMatches: "Ничего не найдено",
      reasonDefault: "Решение основано на комбинации replicate quality, blank subtraction и confidence score.",
      blankContext: "Ближайший blank",
      matchingMode: "Режим сопоставления",
      replicateQuality: "Качество репликатов",
      replicateCount: "Кол-во репликатов",
      sampleType: "Тип образца",
      polarity: "Полярность",
      highConfidence: "высокая уверенность",
      mediumConfidence: "средняя уверенность",
      lowConfidence: "низкая уверенность",
      aboveThreshold: "выше порога",
      belowThreshold: "ниже порога",
      matchingCandidates: "кандидатов",
      noBlankDetail: "Детали blank не найдены",
      keyboardHint: "",
    },
  } as const;

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
  let shownPeaks = $derived(metadata?.shownPeaks ?? metadata?.displayed_peaks ?? peaks.length);
  let totalPeaks = $derived(metadata?.totalPeaks ?? metadata?.total_peaks ?? peaks.length);

  let summaryDetailItems = $derived([
    {
      key: "confirmedPeaks",
      label: dict.confirmedPeaks,
      value: formatNumber(totalConfirmed),
      hintTitle: dict.confirmedPeaks,
      hintBody: "Кількість піків, у яких знайдено узгодження між реплікатами у заданих RT/mz допусках.",
    },
    {
      key: "meanCv",
      label: dict.meanCv,
      value: formatMaybe(meanCv, 1),
      hintTitle: dict.meanCv,
      hintBody: "Середня відносна варіабельність площ між реплікатами. Нижче значення означає стабільніші вимірювання.",
    },
    {
      key: "meanConfidence",
      label: dict.meanConfidence,
      value: formatMaybe(meanConfidence, 1),
      hintTitle: dict.meanConfidence,
      hintBody: "Середній ConfidenceScore (0–100) по показаних піках. Інтегрує реплікатну узгодженість, CV% і signal-to-blank після віднімання blank.",
    },
  ]);

  let statusTotal = $derived(totalCompounds + totalArtifacts);
  let statusSplit = $derived([
    {
      key: "realCompounds",
      label: dict.realCompounds,
      value: totalCompounds,
      pct: statusTotal ? (totalCompounds / statusTotal) * 100 : 0,
      color: "bg-emerald-500 dark:bg-emerald-400",
      chip: "text-emerald-700 bg-emerald-50 dark:bg-emerald-950 dark:text-emerald-300",
      hintTitle: dict.realCompounds,
      hintBody: "Кластери, що пройшли реплікатне підтвердження та не віднесені до артефактів після blank subtraction.",
    },
    {
      key: "artifacts",
      label: dict.artifacts,
      value: totalArtifacts,
      pct: statusTotal ? (totalArtifacts / statusTotal) * 100 : 0,
      color: "bg-rose-500 dark:bg-rose-400",
      chip: "text-rose-700 bg-rose-50 dark:bg-rose-950 dark:text-rose-300",
      hintTitle: dict.artifacts,
      hintBody: "Підтверджені піки, які збігаються з blank і мають недостатній signal-to-blank.",
    },
  ]);

  let displayedPeaksMeta = $derived({
    key: "displayedPeaks",
    label: dict.displayedPeaks,
    value: formatNumber(shownPeaks),
    secondary: totalPeaks > shownPeaks ? t("ofTotal", { total: totalPeaks }) : undefined,
    hintTitle: dict.displayedPeaks,
    hintBody: "Скільки піків зараз показано в таблиці інтерфейсу після фільтрів або обмеження виводу.",
  });

  let realSharePct = $derived(statusTotal ? (totalCompounds / statusTotal) * 100 : 0);
  let statusPieStyle = $derived(
    `background: conic-gradient(rgb(16 185 129) 0 ${realSharePct}%, rgb(244 63 94) ${realSharePct}% 100%)`
  );

  let samplePolarityRows = $derived(
    summary
      .map((row: any) => ({
        label: `${getSampleTypeLabel(row.Sample)} · ${row.Polarity === "positive" ? dict.positive : row.Polarity === "negative" ? dict.negative : row.Polarity}`,
        confirmed: Number(row.Confirmed),
        cv: Number(row.MeanCVPct),
        confidence: Number(row.MeanConfidenceScore),
      }))
      .filter((row: any) => Number.isFinite(row.confirmed) && row.confirmed > 0)
      .sort((a: any, b: any) => b.confirmed - a.confirmed)
  );
  let visibleSamplePolarityRows = $derived(showAllConfirmedRows ? samplePolarityRows : samplePolarityRows.slice(0, 5));

  let selectedPeakKey = $derived(selectedPeak ? getPeakKey(selectedPeak) : null);
  let navigationPeaks = $derived(tableNavigationReady ? navigablePeaks : peaks);
  let selectedPeakIndex = $derived(selectedPeakKey ? navigationPeaks.findIndex((peak: any) => getPeakKey(peak) === selectedPeakKey) : -1);
  let auditCopy = $derived(auditPanelCopy[currentLocale] ?? auditPanelCopy.uk);
  let auditHeaderMetrics = $derived.by(() => {
    if (!selectedPeak) return [];
    return [
      { label: dict.rtMz, value: `${formatMaybe(selectedPeak.RT_mean, 4)} / ${formatMaybe(selectedPeak.MZ_mean, 4)}` },
      { label: dict.areaMean, value: formatNumber(selectedPeak.Area_mean, 2) },
      { label: "CV% / S/B", value: `${formatMaybe(selectedPeak.AreaCVPct, 2)} / ${formatMaybe(selectedPeak.SignalToBlankRatio, 2)}` },
      { label: dict.confidence, value: formatMaybe(selectedPeak.ConfidenceScore, 1) },
    ];
  });
  let auditEvidenceCards = $derived.by(() => {
    if (!selectedPeak) return [];
    const why = selectedPeak.Why || {};
    const sbThreshold = toFiniteNumber(parameters?.signal_to_blank_min, 3);
    const cvThreshold = toFiniteNumber(parameters?.cv_moderate_max, 30);
    const cv = toFiniteNumber(selectedPeak.AreaCVPct);
    const sb = toFiniteNumber(selectedPeak.SignalToBlankRatio);
    const confidence = toFiniteNumber(selectedPeak.ConfidenceScore);
    const blankCandidates = toFiniteNumber(why.BlankCandidateCount, 0);
    const blankMatch = why.BlankMatch;

    return [
      {
        title: auditCopy.blankMatch,
        value: blankMatch === true ? "Yes" : blankMatch === false ? "No" : "—",
        threshold: blankCandidates > 0 ? `${formatMaybe(blankCandidates, 0)} ${auditCopy.matchingCandidates}` : "0",
        delta: blankMatch === true ? auditCopy.aboveThreshold : auditCopy.belowThreshold,
        note: blankMatch === true ? auditCopy.blankFound : auditCopy.blankNotFound,
        tone: blankMatch === true ? "fail" : blankMatch === false ? "pass" : "info",
        meter: blankMatch === true ? 100 : blankMatch === false ? 0 : 40,
        weight: blankMatch === true ? 100 : blankCandidates > 0 ? 70 : 10,
      },
      {
        title: auditCopy.blankCandidates,
        value: formatMaybe(blankCandidates, 0),
        threshold: "0",
        delta: blankCandidates > 0 ? `+${formatMaybe(blankCandidates, 0)}` : "0",
        note: `${formatMaybe(blankCandidates, 0)} ${auditCopy.matchingCandidates}`,
        tone: blankCandidates > 1 ? "warn" : blankCandidates === 1 ? "info" : "pass",
        meter: Math.min(100, blankCandidates * 34),
        weight: blankCandidates > 0 ? 55 : 5,
      },
      {
        title: auditCopy.cvLabel,
        value: Number.isFinite(cv) ? formatMaybe(cv, 2) : "—",
        threshold: `≤ ${formatMaybe(cvThreshold, 1)}`,
        delta: Number.isFinite(cv) ? `${cv >= cvThreshold ? "+" : ""}${formatMaybe(cv - cvThreshold, 2)}` : "—",
        note: Number.isFinite(cv)
          ? `${formatMaybe(cv, 2)} ${cv > cvThreshold ? auditCopy.aboveThreshold : auditCopy.belowThreshold}`
          : auditCopy.reasonDefault,
        tone: Number.isFinite(cv) ? (cv <= cvThreshold ? "pass" : cv <= cvThreshold * 1.35 ? "warn" : "fail") : "info",
        meter: Number.isFinite(cv) ? Math.min(100, (cv / Math.max(cvThreshold, 1)) * 100) : 0,
        weight: Number.isFinite(cv) ? Math.abs(cv - cvThreshold) : 0,
      },
      {
        title: auditCopy.sbLabel,
        value: Number.isFinite(sb) ? formatMaybe(sb, 2) : "—",
        threshold: `≥ ${formatMaybe(sbThreshold, 1)}`,
        delta: Number.isFinite(sb) ? `${sb >= sbThreshold ? "+" : ""}${formatMaybe(sb - sbThreshold, 2)}` : "—",
        note: Number.isFinite(sb)
          ? `${formatMaybe(sb, 2)} ${sb >= sbThreshold ? auditCopy.aboveThreshold : auditCopy.belowThreshold}`
          : auditCopy.reasonDefault,
        tone: Number.isFinite(sb) ? (sb >= sbThreshold ? "pass" : sb >= sbThreshold * 0.8 ? "warn" : "fail") : "info",
        meter: Number.isFinite(sb) ? Math.min(100, (sb / Math.max(sbThreshold, 0.1)) * 100) : 0,
        weight: Number.isFinite(sb) ? Math.abs(sbThreshold - sb) * 10 : 0,
      },
      {
        title: auditCopy.confidenceLabel,
        value: Number.isFinite(confidence) ? formatMaybe(confidence, 1) : "—",
        threshold: "≥ 70",
        delta: Number.isFinite(confidence) ? `${confidence >= 70 ? "+" : ""}${formatMaybe(confidence - 70, 1)}` : "—",
        note: Number.isFinite(confidence)
          ? confidence >= 85
            ? auditCopy.highConfidence
            : confidence >= 70
              ? auditCopy.mediumConfidence
              : auditCopy.lowConfidence
          : auditCopy.reasonDefault,
        tone: Number.isFinite(confidence) ? (confidence >= 85 ? "pass" : confidence >= 70 ? "warn" : "fail") : "info",
        meter: Number.isFinite(confidence) ? Math.min(100, confidence) : 0,
        weight: Number.isFinite(confidence) ? confidence : 0,
      },
    ].sort((a, b) => b.weight - a.weight);
  });
  let auditReasons = $derived.by(() => {
    if (!selectedPeak) return [];
    const why = selectedPeak.Why || {};
    const reasons = auditEvidenceCards
      .filter((card: any) => ["fail", "warn"].includes(card.tone))
      .slice(0, 3)
      .map((card: any) => `${card.title}: ${card.note}`);

    if (!reasons.length) {
      reasons.push(why.BlankMatch === false ? auditCopy.blankNotFound : auditCopy.reasonDefault);
    }
    return reasons;
  });
  let auditOverview = $derived.by(() => {
    if (!selectedPeak) return null;
    const artifact = selectedPeak.Status !== "Real Compound";
    const confidence = toFiniteNumber(selectedPeak.ConfidenceScore);
    const tone = artifact ? (confidence >= 80 ? "fail" : "warn") : confidence >= 85 ? "pass" : "warn";

    return {
      tone,
      title: artifact ? auditCopy.likelyArtifact : auditCopy.likelyReal,
      badge: getStatusLabel(selectedPeak.Status),
      recommendation: artifact
        ? confidence >= 80
          ? auditCopy.autoReject
          : auditCopy.manualReview
        : confidence >= 80
          ? auditCopy.keepDecision
          : auditCopy.manualReview,
      summary: auditReasons.join(" • "),
    };
  });
  let auditContextItems = $derived.by(() => {
    if (!selectedPeak) return [];
    const why = selectedPeak.Why || {};
    return [
      { label: auditCopy.sampleType, value: getSampleTypeLabel(selectedPeak.SampleType) },
      { label: auditCopy.polarity, value: selectedPeak.Polarity === "positive" ? dict.positive : selectedPeak.Polarity === "negative" ? dict.negative : selectedPeak.Polarity },
      { label: auditCopy.replicateQuality, value: selectedPeak.ReplicateQuality || "—" },
      { label: auditCopy.replicateCount, value: formatMaybe(selectedPeak.ReplicateCount ?? 2, 0) },
      { label: auditCopy.matchingMode, value: why.BlankDetail?.matching_mode || "RT+MZ" },
      { label: auditCopy.queue, value: selectedPeakIndex >= 0 ? `${selectedPeakIndex + 1} / ${navigationPeaks.length}` : "—" },
    ];
  });
  let auditBlankDetailItems = $derived.by(() => {
    if (!selectedPeak?.Why?.BlankDetail) return [];
    const detail = selectedPeak.Why.BlankDetail;
    return [
      { label: dict.rtMz, value: `${formatMaybe(detail.RT, 4)} / ${formatMaybe(detail.MZ, 4)}` },
      { label: dict.areaMean, value: formatNumber(detail.Area_mean ?? detail.Area, 2) },
      { label: auditCopy.matchingMode, value: detail.matching_mode || "RT+MZ" },
      { label: "m/z Δ", value: formatMaybe(detail.mz_delta_ppm ?? detail.mz_delta_da, 2) },
    ].filter((item) => item.value !== "— / —");
  });

  let auditJsonEntries = $derived(
    (() => {
      if (!selectedPeak?.Why) return [] as { path: string; value: string; type: string }[];
      const out: { path: string; value: string; type: string }[] = [];

      function walk(value: any, prefix: string) {
        if (Array.isArray(value)) {
          if (!value.length) {
            out.push({ path: prefix, value: "[]", type: "array" });
            return;
          }
          value.forEach((item, idx) => walk(item, `${prefix}[${idx}]`));
          return;
        }
        if (value && typeof value === "object") {
          const entries = Object.entries(value);
          if (!entries.length) {
            out.push({ path: prefix, value: "{}", type: "object" });
            return;
          }
          entries.forEach(([key, nested]) => walk(nested, prefix ? `${prefix}.${key}` : key));
          return;
        }
        const type = value === null ? "null" : typeof value;
        out.push({ path: prefix, value: value === null ? "null" : String(value), type });
      }

      walk(selectedPeak.Why, "");
      return out;
    })()
  );

  let filteredAuditJsonEntries = $derived(
    auditJsonEntries.filter((entry: any) => {
      const q = auditJsonQuery.trim().toLowerCase();
      if (!q) return true;
      return entry.path.toLowerCase().includes(q) || entry.value.toLowerCase().includes(q);
    })
  );
  let visibleAuditJsonEntries = $derived(filteredAuditJsonEntries.slice(0, 14));

  function formatNumber(value: any, digits = 0) {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    return Number(value).toLocaleString(getNumberLocale(), {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });
  }

  function toggleActiveMetric(metric: "confirmed" | "cv" | "confidence") {
    activeMetric = activeMetric === metric ? null : metric;
  }

  function formatMaybe(value: any, digits = 2) {
    if (value === null || value === undefined || Number.isNaN(Number(value))) return "—";
    return Number(value).toLocaleString(getNumberLocale(), {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    });
  }

  function formatMode(mode: string | undefined) {
    return (mode || "da").toUpperCase();
  }

  function toFiniteNumber(value: any, fallback = Number.NaN) {
    const next = Number(value);
    return Number.isFinite(next) ? next : fallback;
  }

  function getPeakKey(peak: any) {
    if (!peak) return "";
    return [
      String(peak.SampleType || ""),
      String(peak.Polarity || ""),
      Number(peak.RT_mean ?? 0).toFixed(6),
      Number(peak.MZ_mean ?? 0).toFixed(6),
      Number(peak.Area_mean ?? 0).toFixed(2),
    ].join("|");
  }

  function scaledWidth(value: number, max: number) {
    if (!Number.isFinite(value) || !Number.isFinite(max) || max <= 0) return "0%";
    return `${Math.max(8, (value / max) * 100)}%`;
  }

  function scaledPercent(value: number, max = 100) {
    if (!Number.isFinite(value) || max <= 0) return "0%";
    const pct = Math.min(100, Math.max(0, (value / max) * 100));
    return `${pct}%`;
  }

  function openAuditTrail(peak: any) {
    selectedPeak = peak;
    auditDetailTab = "review";
    auditJsonQuery = "";
  }

  function closeAuditTrail() {
    selectedPeak = null;
    auditJsonQuery = "";
    auditDetailTab = "review";
  }

  $effect(() => {
    selectedPeak;
    if (auditPanelEl) auditPanelEl.scrollTop = 0;
  });

  function toneClasses(tone: string) {
    if (tone === "pass") return "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-800 dark:bg-emerald-950/50 dark:text-emerald-300";
    if (tone === "warn") return "border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-800 dark:bg-amber-950/50 dark:text-amber-300";
    if (tone === "fail") return "border-rose-200 bg-rose-50 text-rose-800 dark:border-rose-800 dark:bg-rose-950/50 dark:text-rose-300";
    return "border-slate-200 bg-slate-50 text-slate-800 dark:border-slate-700 dark:bg-slate-700/50 dark:text-slate-300";
  }

  function toneBarClasses(tone: string) {
    if (tone === "pass") return "bg-emerald-500 dark:bg-emerald-400";
    if (tone === "warn") return "bg-amber-500 dark:bg-amber-400";
    if (tone === "fail") return "bg-rose-500 dark:bg-rose-400";
    return "bg-slate-400 dark:bg-slate-500";
  }

  function tonePanelClasses(tone: string) {
    if (tone === "pass") return "border-emerald-200 bg-emerald-50/80 dark:border-emerald-800 dark:bg-emerald-950/30";
    if (tone === "warn") return "border-amber-200 bg-amber-50/80 dark:border-amber-800 dark:bg-amber-950/30";
    if (tone === "fail") return "border-rose-200 bg-rose-50/80 dark:border-rose-800 dark:bg-rose-950/30";
    return "border-slate-200 bg-slate-50/80 dark:border-slate-700 dark:bg-slate-800/80";
  }

  function isDecisionKey(path: string) {
    const cleaned = String(path || "").replace(/\[\d+\]/g, "");
    const parts = cleaned.split(".").filter(Boolean);
    const key = parts[parts.length - 1] || "";
    return ["BlankMatch", "BlankCandidateCount", "ColorPaired", "SignalToBlankRatio", "AreaCVPct", "ConfidenceScore"].includes(key);
  }

  function toggleMethodParams() {
    methodParamsOpen = !methodParamsOpen;
  }

  function closeMethodParams() {
    methodParamsOpen = false;
  }

  function toggleSummaryMetrics() {
    summaryMetricsOpen = !summaryMetricsOpen;
  }

  function toggleKpiHint(key: string) {
    openKpiHint = openKpiHint === key ? null : key;
  }

  function setActiveTab(tab: "summary" | "data" | "3d") {
    activeTab = tab;
  }

  function toggleDataFilters() {
    dataFiltersOpen = !dataFiltersOpen;
  }

  function closeKpiHint() {
    openKpiHint = null;
  }

  function toggleConfirmedRowsView() {
    showAllConfirmedRows = !showAllConfirmedRows;
    try {
      localStorage.setItem(confirmedRowsViewStorageKey, showAllConfirmedRows ? "1" : "0");
    } catch {}
  }

  onMount(() => {
    try {
      showAllConfirmedRows = localStorage.getItem(confirmedRowsViewStorageKey) === "1";
    } catch {}
  });

  function moveAuditSelection(offset: number) {
    if (selectedPeakIndex < 0) return;
    const nextIndex = selectedPeakIndex + offset;
    if (nextIndex < 0 || nextIndex >= navigationPeaks.length) return;
    openAuditTrail(navigationPeaks[nextIndex]);
  }

  function cycleAuditTab(offset: number) {
    const tabs: Array<"review" | "summary" | "evidence" | "raw"> = ["review", "summary", "evidence", "raw"];
    const currentIndex = tabs.indexOf(auditDetailTab);
    if (currentIndex < 0) return;
    const nextIndex = currentIndex + offset;
    if (nextIndex < 0 || nextIndex >= tabs.length) return;
    auditDetailTab = tabs[nextIndex];
  }

  function handleVisiblePeaksChange(nextPeaks: any[]) {
    navigablePeaks = nextPeaks;
    tableNavigationReady = true;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    const isTypingTarget =
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target?.isContentEditable;

    if (openKpiHint !== null && event.key === "Escape") {
      event.preventDefault();
      closeKpiHint();
      return;
    }
    if (methodParamsOpen && event.key === "Escape") {
      event.preventDefault();
      closeMethodParams();
      return;
    }
    if (!selectedPeak) return;
    if (event.key === "Escape") {
      event.preventDefault();
      closeAuditTrail();
      return;
    }
    if (isTypingTarget) return;
    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveAuditSelection(-1);
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveAuditSelection(1);
      return;
    }
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      cycleAuditTab(-1);
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      cycleAuditTab(1);
    }
  }

  function handleWindowClick() {
    if (openKpiHint !== null) closeKpiHint();
    if (methodParamsOpen) closeMethodParams();
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} onclick={handleWindowClick} />

<div class="mx-auto max-w-[1400px] flex-col space-y-4 px-4 pb-0">
  <div class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-slate-200 bg-white px-4 py-2.5 shadow-sm dark:border-slate-700 dark:bg-slate-800/80">
    
    <div class="flex items-center gap-3">
      <button
        type="button"
        class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.15em] text-slate-400 transition-colors hover:text-blue-500 dark:text-slate-500 dark:hover:text-blue-400"
        onclick={onUploadNewFile}
        title={dict.uploadNewFile}
      >
        <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        <span class="hidden sm:inline">{dict.appName}</span>
      </button>

      <span class="h-5 w-px bg-slate-200 dark:bg-slate-700" aria-hidden="true"></span>

      <h2 class="text-base font-bold tracking-tight text-slate-900 dark:text-slate-50 md:text-lg">{title}</h2>

      <span
        class={["hidden rounded-full px-2 py-0.5 text-[9px] font-bold uppercase tracking-wider text-emerald-700 sm:inline-block", onlineStatus ? "bg-emerald-50 dark:bg-emerald-950/50 dark:text-emerald-400" : "bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300"].join(" ")}
      >
        {onlineStatus ? dict.online : dict.offline}
      </span>

      <details class="relative">
        <summary
          class="inline-flex h-6 w-6 cursor-pointer list-none items-center justify-center rounded-full bg-blue-50 text-blue-600 transition-colors hover:bg-blue-100 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-blue-900/50 dark:text-blue-400 dark:hover:bg-blue-900"
          aria-label={`${dict.exportXlsx} / ${dict.exportHtml}`}
          title={`${dict.exportXlsx} / ${dict.exportHtml}`}
        >
          {#if exporting || exportingHtml}
            <Loader2 class="h-3.5 w-3.5 animate-spin" />
          {:else}
            <Download class="h-3.5 w-3.5" />
          {/if}
        </summary>
        <div class="absolute left-0 z-20 mt-2 w-32 overflow-hidden rounded-xl border border-slate-200 bg-white shadow-lg dark:border-slate-700 dark:bg-slate-800">
          <button class="block w-full px-3 py-2 text-left text-xs font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50 dark:text-slate-100 dark:hover:bg-slate-700" onclick={onExportXlsx} disabled={exporting || exportingHtml}>XLSX Export</button>
          <button class="block w-full px-3 py-2 text-left text-xs font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50 dark:text-slate-100 dark:hover:bg-slate-700" onclick={onExportHtml} disabled={exporting || exportingHtml}>HTML Export</button>
        </div>
      </details>
    </div>

    <div class="flex items-center gap-3">
      <button
        type="button"
        aria-haspopup="dialog"
        aria-expanded={methodParamsOpen}
        class="hidden items-center gap-1 rounded-lg border border-slate-200 bg-slate-50 px-2 py-1 text-left text-[10px] text-slate-600 transition-colors hover:border-blue-300 dark:border-slate-700 dark:bg-slate-900/50 dark:text-slate-300 dark:hover:border-blue-500 xl:flex"
        onclick={(e) => { e.stopPropagation(); toggleMethodParams(); }}
      >
        <span class="font-semibold text-slate-900 dark:text-slate-100 uppercase tracking-wide">Param:</span>
        <span class="opacity-80">R{formatNumber(parameters.replicate_rt_tol, 2)}/{formatNumber(parameters.replicate_mz_tol, 2)}{formatMode(parameters.replicate_mz_mode)}, B{formatNumber(parameters.blank_rt_tol, 2)}/{formatNumber(parameters.blank_mz_tol, 2)}{formatMode(parameters.blank_mz_mode)}, S/B≥{formatNumber(parameters.signal_to_blank_min, 1)}</span>
        {#if metadata?.cacheHit}
          <span class="ml-1 rounded bg-emerald-100 px-1 py-0.5 text-[8px] font-bold text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300">{dict.cacheHit}</span>
        {/if}
      </button>

      <div class="hidden items-center gap-1 text-[11px] text-slate-500 dark:text-slate-400 sm:flex">
        <svg class="h-3.5 w-3.5 opacity-60" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
        <select class="bg-transparent font-medium outline-none" value={currentLocale} onchange={(e) => setLocale((e.currentTarget as HTMLSelectElement).value)}>
          {#each Object.entries(localeLabels) as [val, label]}<option value={val}>{label}</option>{/each}
        </select>
        <button type="button" onclick={cycleTheme} aria-label="Cycle theme" title="Cycle theme" class="ml-1 flex h-6 w-6 items-center justify-center rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-500 transition-colors">
          <svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d={themeIcons[currentTheme]}/></svg>
        </button>
      </div>

      <div class="inline-flex rounded-lg bg-slate-100 p-0.5 dark:bg-slate-800">
        <button type="button" class={`rounded-md px-3 py-1 text-xs font-semibold transition-all ${activeTab === "summary" ? "bg-white text-blue-600 shadow-sm dark:bg-slate-700 dark:text-blue-400" : "text-slate-500 hover:text-slate-700 dark:text-slate-400"}`} onclick={() => setActiveTab("summary")}>Summary</button>
        <button type="button" class={`rounded-md px-3 py-1 text-xs font-semibold transition-all ${activeTab === "data" ? "bg-white text-blue-600 shadow-sm dark:bg-slate-700 dark:text-blue-400" : "text-slate-500 hover:text-slate-700 dark:text-slate-400"}`} onclick={() => setActiveTab("data")}>Data</button>
        <button type="button" class={`rounded-md px-3 py-1 text-xs font-semibold transition-all ${activeTab === "3d" ? "bg-white text-blue-600 shadow-sm dark:bg-slate-700 dark:text-blue-400" : "text-slate-500 hover:text-slate-700 dark:text-slate-400"}`} onclick={() => setActiveTab("3d")}>3D</button>
      </div>
    </div>
  </div>

  {#if methodParamsOpen}
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/40 p-4 dark:bg-slate-950/70"
      role="button"
      tabindex="0"
      aria-label={dict.close}
      onclick={closeMethodParams}
      onkeydown={(event) => {
        if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          closeMethodParams();
        }
      }}
    >
      <div
        class="w-full max-w-lg overflow-hidden rounded-3xl bg-white shadow-2xl dark:bg-slate-800"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        onclick={(event) => event.stopPropagation()}
        onkeydown={(event) => event.stopPropagation()}
      >
        <div class="flex items-start justify-between border-b border-slate-100 px-5 py-4 dark:border-slate-700">
          <div>
            <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100">{dict.methodParameters}</h3>
            <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">{dict.methodParametersDesc}</p>
          </div>
          <button class="rounded-full border border-slate-200 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-50 dark:border-slate-600 dark:text-slate-400 dark:hover:bg-slate-700" onclick={closeMethodParams}>
            {dict.close}
          </button>
        </div>
        <dl class="grid gap-2 p-4 text-sm">
          <div class="rounded-lg border border-slate-100 bg-slate-50 px-3 py-2 dark:border-slate-700 dark:bg-slate-700/60">
            <dt class="text-xs text-slate-500 dark:text-slate-400">{dict.replicateMatching}</dt>
            <dd class="mt-0.5 font-medium text-slate-900 dark:text-slate-100">{formatNumber(parameters.replicate_rt_tol, 2)} min / {formatNumber(parameters.replicate_mz_tol, 2)} {formatMode(parameters.replicate_mz_mode)}</dd>
          </div>
          <div class="rounded-lg border border-slate-100 bg-slate-50 px-3 py-2 dark:border-slate-700 dark:bg-slate-700/60">
            <dt class="text-xs text-slate-500 dark:text-slate-400">{dict.blankSubtraction}</dt>
            <dd class="mt-0.5 font-medium text-slate-900 dark:text-slate-100">{formatNumber(parameters.blank_rt_tol, 2)} min / {formatNumber(parameters.blank_mz_tol, 2)} {formatMode(parameters.blank_mz_mode)}</dd>
          </div>
          <div class="rounded-lg border border-slate-100 bg-slate-50 px-3 py-2 dark:border-slate-700 dark:bg-slate-700/60">
            <dt class="text-xs text-slate-500 dark:text-slate-400">{dict.signalToBlankThreshold}</dt>
            <dd class="mt-0.5 font-medium text-slate-900 dark:text-slate-100">≥ {formatNumber(parameters.signal_to_blank_min, 1)}</dd>
          </div>
          <div class="rounded-lg border border-slate-100 bg-slate-50 px-3 py-2 dark:border-slate-700 dark:bg-slate-700/60">
            <dt class="text-xs text-slate-500 dark:text-slate-400">{dict.replicateQualityBands}</dt>
            <dd class="mt-0.5 font-medium text-slate-900 dark:text-slate-100">{dict.qualityHigh} ≤ {formatNumber(parameters.cv_high_max, 1)}% CV, {dict.qualityModerate} ≤ {formatNumber(parameters.cv_moderate_max, 1)}% CV</dd>
          </div>
        </dl>
      </div>
    </div>
  {/if}

  {#if metadata?.truncated}
    <div class="rounded-2xl border border-amber-200 bg-amber-50 px-5 py-4 text-sm text-amber-900 dark:border-amber-700 dark:bg-amber-950 dark:text-amber-200">
      {t("truncatedNotice", {
        shown: metadata.shownPeaks ?? metadata.displayed_peaks ?? peaks.length,
        total: metadata.totalPeaks ?? metadata.total_peaks ?? peaks.length,
      })}
    </div>
  {/if}

  {#if activeTab === "summary"}

  <section class="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800 sm:p-5">
    <div class="space-y-3">
      <div class="grid gap-3 sm:grid-cols-2">
        <div class="rounded-xl border border-slate-200 bg-slate-50/70 px-3 py-3 dark:border-slate-700 dark:bg-slate-700/40">
          <div class="mb-2 flex items-center justify-between">
            <p class="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">{dict.realArtifact}</p>
            <span class="text-xs text-slate-500 dark:text-slate-400">{formatNumber(statusTotal)} total</span>
          </div>
          <div class="flex items-center gap-3">
            <div class="relative h-24 w-24 shrink-0 rounded-full" style={statusPieStyle}>
              <div class="absolute inset-4 rounded-full bg-white dark:bg-slate-800"></div>
              <div class="absolute inset-0 flex items-center justify-center text-xs font-semibold text-slate-700 dark:text-slate-200">{formatNumber(statusTotal)}</div>
            </div>
            <div class="space-y-1.5 text-xs">
              {#each statusSplit as segment}
                <div class="relative flex items-center gap-2">
                  <span class={`inline-block h-2.5 w-2.5 rounded-full ${segment.color}`}></span>
                  <button
                    type="button"
                    class="text-left text-slate-700 transition-colors hover:text-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:text-slate-300 dark:hover:text-blue-300"
                    onclick={(event) => {
                      event.stopPropagation();
                      toggleKpiHint(segment.key);
                    }}
                  >{segment.label}: {formatNumber(segment.value)} ({formatMaybe(segment.pct, 1)}%)</button>
                  <div
                    role="tooltip"
                    class={`absolute right-0 top-full z-20 mt-1.5 w-64 rounded-2xl border border-blue-200 bg-white p-3 text-[11px] leading-5 text-slate-700 shadow-2xl dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 ${openKpiHint === segment.key ? "block" : "hidden"}`}
                  >
                    <p class="font-semibold text-slate-900 dark:text-slate-50">{segment.hintTitle}</p>
                    <p class="mt-1 text-slate-600 dark:text-slate-300">{segment.hintBody}</p>
                  </div>
                </div>
              {/each}
              <div class="relative mt-2 border-t border-slate-200 pt-2 dark:border-slate-600">
                <button
                  type="button"
                  class="text-left text-slate-700 transition-colors hover:text-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:text-slate-300 dark:hover:text-blue-300"
                  onclick={(event) => {
                    event.stopPropagation();
                    toggleKpiHint(displayedPeaksMeta.key);
                  }}
                >{displayedPeaksMeta.label}: {displayedPeaksMeta.value}{#if displayedPeaksMeta.secondary} ({displayedPeaksMeta.secondary}){/if}</button>
                <div
                  role="tooltip"
                  class={`absolute right-0 top-full z-20 mt-1.5 w-64 rounded-2xl border border-blue-200 bg-white p-3 text-[11px] leading-5 text-slate-700 shadow-2xl dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 ${openKpiHint === displayedPeaksMeta.key ? "block" : "hidden"}`}
                >
                  <p class="font-semibold text-slate-900 dark:text-slate-50">{displayedPeaksMeta.hintTitle}</p>
                  <p class="mt-1 text-slate-600 dark:text-slate-300">{displayedPeaksMeta.hintBody}</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="rounded-xl border border-slate-200 bg-slate-50/70 px-3 py-3 dark:border-slate-700 dark:bg-slate-700/40">
          <div class="mb-2 flex items-center justify-between">
            <p class="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">{dict.confirmedPeaks} за зразком/полярністю</p>
            {#if samplePolarityRows.length > 5}
              <button
                type="button"
                class="rounded-full border border-slate-300 px-2.5 py-0.5 text-[11px] font-medium text-slate-600 transition-colors hover:border-blue-300 hover:text-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:border-slate-500 dark:text-slate-300 dark:hover:border-blue-500 dark:hover:text-blue-300"
                onclick={toggleConfirmedRowsView}
              >{showAllConfirmedRows ? "Top 5" : "Усі"}</button>
            {:else}
              <span class="text-xs text-slate-500 dark:text-slate-400">Top groups</span>
            {/if}
          </div>
          <div class="mb-2 flex flex-wrap gap-2 text-[11px]">
            <button type="button" class={`rounded-full px-2 py-0.5 ${activeMetric === "confirmed" ? "bg-blue-100 text-blue-700 dark:bg-blue-950 dark:text-blue-300" : "bg-slate-200 text-slate-600 dark:bg-slate-700 dark:text-slate-300"}`} onclick={() => toggleActiveMetric("confirmed")}>Confirmed</button>
            <button type="button" class={`rounded-full px-2 py-0.5 ${activeMetric === "cv" ? "bg-amber-100 text-amber-700 dark:bg-amber-950 dark:text-amber-300" : "bg-slate-200 text-slate-600 dark:bg-slate-700 dark:text-slate-300"}`} onclick={() => toggleActiveMetric("cv")}>CV%</button>
            <button type="button" class={`rounded-full px-2 py-0.5 ${activeMetric === "confidence" ? "bg-cyan-100 text-cyan-700 dark:bg-cyan-950 dark:text-cyan-300" : "bg-slate-200 text-slate-600 dark:bg-slate-700 dark:text-slate-300"}`} onclick={() => toggleActiveMetric("confidence")}>Confidence</button>
          </div>
          <div class="space-y-1.5">
            {#if visibleSamplePolarityRows.length}
              {@const maxConfirmed = Math.max(...samplePolarityRows.map((row: any) => row.confirmed))}
              {#each visibleSamplePolarityRows as row}
                <div class="grid grid-cols-[1fr_auto] items-center gap-2" title={`${row.label}: Confirmed ${formatNumber(row.confirmed)}, CV ${formatMaybe(row.cv, 1)}%, Confidence ${formatMaybe(row.confidence, 1)}`}>
                  <div class="relative overflow-hidden rounded-full bg-slate-200 dark:bg-slate-600">
                    <div class={`h-2 rounded-full transition-opacity ${activeMetric && activeMetric !== "confirmed" ? "opacity-35" : "opacity-100"} bg-blue-500 dark:bg-blue-400`} style={`width:${scaledWidth(row.confirmed, maxConfirmed)}`}></div>
                    {#if Number.isFinite(row.cv)}
                      <span class={`absolute top-1/2 h-2.5 w-2.5 -translate-y-1/2 rounded-full border border-white shadow ${activeMetric && activeMetric !== "cv" ? "opacity-35" : "opacity-100"} bg-amber-500 dark:bg-amber-400`} style={`left:${scaledPercent(row.cv, 100)}`}></span>
                    {/if}
                    {#if Number.isFinite(row.confidence)}
                      <span class={`absolute top-1/2 h-2.5 w-2.5 -translate-y-1/2 rounded-full border border-white shadow ${activeMetric && activeMetric !== "confidence" ? "opacity-35" : "opacity-100"} bg-cyan-500 dark:bg-cyan-400`} style={`left:${scaledPercent(row.confidence, 100)}`}></span>
                    {/if}
                  </div>
                  <div class="min-w-[11rem] truncate text-right text-xs text-slate-600 dark:text-slate-300">{row.label} · {formatNumber(row.confirmed)} / {formatMaybe(row.cv, 1)} / {formatMaybe(row.confidence, 1)}</div>
                </div>
              {/each}
            {:else}
              <div class="text-xs text-slate-500 dark:text-slate-400">—</div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </section>

  <div>
    <section class="rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800">
      <div class="border-b border-slate-100 px-4 py-3.5 dark:border-slate-700">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h3 class="text-base font-semibold text-slate-900 dark:text-slate-100">{dict.qcSummary}</h3>
            <p class="mt-0.5 text-xs text-slate-500 dark:text-slate-400">{dict.qcSummaryDesc}</p>
          </div>
          <button
            type="button"
            class="rounded-full border border-slate-300 px-2.5 py-0.5 text-[11px] font-medium text-slate-600 transition-colors hover:border-blue-300 hover:text-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:border-slate-500 dark:text-slate-300 dark:hover:border-blue-500 dark:hover:text-blue-300"
            onclick={toggleSummaryMetrics}
          >{summaryMetricsOpen ? "Згорнути" : "Деталі"}</button>
        </div>
        <button
          type="button"
          class="mt-3 w-full rounded-xl border border-slate-200 bg-slate-50/80 px-3 py-2.5 text-left text-sm text-slate-700 transition-colors hover:border-blue-300 hover:text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-300 dark:border-slate-700 dark:bg-slate-700/40 dark:text-slate-300"
          aria-expanded={summaryMetricsOpen}
          onclick={toggleSummaryMetrics}
        >
          <span class="font-semibold">{dict.confirmedPeaks} {formatNumber(totalConfirmed)}</span>
          <span class="mx-2 text-slate-400">•</span>
          <span class="font-medium">{dict.meanCv} {formatMaybe(meanCv, 1)}</span>
          <span class="mx-2 text-slate-400">•</span>
          <span class="font-medium">{dict.meanConfidence} {formatMaybe(meanConfidence, 1)}</span>
        </button>
        {#if summaryMetricsOpen}
          <div class="mt-2 grid gap-2 text-sm">
            {#each summaryDetailItems as item}
              <div class="rounded-lg border border-slate-100 bg-slate-50 px-3 py-2 dark:border-slate-700 dark:bg-slate-700/60">
                <div class="flex items-center justify-between gap-3">
                  <dt class="text-xs text-slate-500 dark:text-slate-400">{item.label}</dt>
                  <dd class="font-semibold text-slate-900 dark:text-slate-100">{item.value}</dd>
                </div>
                <p class="mt-1 text-xs text-slate-600 dark:text-slate-300">{item.hintBody}</p>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      {#if summaryMetricsOpen}
        <div class="overflow-auto max-h-[calc(100vh-320px)]">
          <table class="min-w-full text-xs sm:text-sm">
          <thead class="bg-slate-50 text-slate-600 dark:bg-slate-700 dark:text-slate-300">
            <tr>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Тип зразка">{dict.sample}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Режим іонізації">{dict.polarity}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Підтверджені реплікатами піки">{dict.confirmed}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Класифікація підтверджених піків">{dict.realArtifact}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Стабільність між реплікатами">{dict.meanCv}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="High / Moderate / Low">{dict.qualityBands}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Середній confidence score">{dict.meanConfidence}</th>
              <th class="sticky top-0 z-10 bg-inherit px-3 py-2 text-left font-semibold shadow-[0_1px_0_0_#e2e8f0] dark:shadow-[0_1px_0_0_#334155]" title="Signal-to-blank ratio">{dict.meanSb}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100 dark:divide-slate-700">
            {#each summary as row}
              <tr class="hover:bg-slate-50/70 dark:hover:bg-slate-700/50">
                <td class="px-3 py-2 font-semibold capitalize text-slate-900 dark:text-slate-100">{getSampleTypeLabel(row.Sample)}</td>
                <td class="px-3 py-2 uppercase text-slate-500 dark:text-slate-400">{row.Polarity === "positive" ? dict.positive : row.Polarity === "negative" ? dict.negative : row.Polarity}</td>
                <td class="px-3 py-2 dark:text-slate-300">{row.Confirmed}</td>
                <td class="px-3 py-2 dark:text-slate-300">{row.RealCompounds || 0} / {row.Artifacts || 0}</td>
                <td class="px-3 py-2 font-mono dark:text-slate-300">{formatMaybe(row.MeanCVPct, 1)}</td>
                <td class="px-3 py-2 dark:text-slate-300">{row.HighQuality || 0}/{row.ModerateQuality || 0}/{row.LowQuality || 0}</td>
                <td class="px-3 py-2 font-mono dark:text-slate-300">{formatMaybe(row.MeanConfidenceScore, 1)}</td>
                <td class="px-3 py-2 font-mono dark:text-slate-300">{formatMaybe(row.MeanSignalToBlankRatio, 2)}</td>
              </tr>
            {/each}
          </tbody>
          </table>
        </div>
      {/if}
    </section>
  </div>

  {:else if activeTab === "data"}
    <section class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-700 dark:bg-slate-800" style="height: calc(100vh - 120px);">
      <div class="grid h-full min-h-0 xl:grid-cols-[minmax(0,1fr)_26rem]">
        <div class="min-h-0 border-b border-slate-200 p-4 pb-0 pt-0 dark:border-slate-700 xl:border-b-0 xl:border-r">
          <PeaksDataTable
            peaks={peaks}
            parameters={parameters}
            onAuditClick={openAuditTrail}
            onVisiblePeaksChange={handleVisiblePeaksChange}
            selectedPeakKey={selectedPeakKey}
          />
        </div>

        <aside bind:this={auditPanelEl} class="min-h-0 overflow-y-auto bg-[linear-gradient(180deg,#f8fafc_0%,#ffffff_28%)] p-4 dark:bg-[linear-gradient(180deg,#0f172a_0%,#111827_28%)]">
          {#if selectedPeak && auditOverview}
            <div class="space-y-4">
              <section class={`rounded-3xl border p-4 shadow-sm ${tonePanelClasses(auditOverview.tone)}`}>
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <div class="space-y-2">
                    <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-500 dark:text-slate-400">{auditCopy.reviewLabel}</p>
                    <div class="flex flex-wrap items-center gap-2">
                      <h3 class="text-xl font-semibold tracking-tight text-slate-950 dark:text-slate-50">{auditOverview.title}</h3>
                      <span class={`rounded-full px-2.5 py-1 text-[11px] font-semibold ${toneClasses(auditOverview.tone)}`}>{auditOverview.badge}</span>
                    </div>
                    <p class="max-w-[34rem] text-sm leading-6 text-slate-700 dark:text-slate-300">{auditOverview.summary}</p>
                  </div>

                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      class="rounded-full border border-slate-200 px-3 py-1.5 text-xs font-semibold text-slate-600 transition-colors hover:border-blue-300 hover:text-blue-600 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-600 dark:text-slate-300 dark:hover:border-blue-500 dark:hover:text-blue-300"
                      onclick={() => moveAuditSelection(-1)}
                      disabled={selectedPeakIndex <= 0}
                    >← {auditCopy.prev}</button>
                    <button
                      type="button"
                      class="rounded-full border border-slate-200 px-3 py-1.5 text-xs font-semibold text-slate-600 transition-colors hover:border-blue-300 hover:text-blue-600 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-600 dark:text-slate-300 dark:hover:border-blue-500 dark:hover:text-blue-300"
                      onclick={() => moveAuditSelection(1)}
                      disabled={selectedPeakIndex < 0 || selectedPeakIndex >= navigationPeaks.length - 1}
                    >{auditCopy.next} →</button>
                    <button class="rounded-full border border-slate-200 px-3 py-1.5 text-xs font-semibold text-slate-600 transition-colors hover:border-slate-300 hover:bg-white dark:border-slate-600 dark:text-slate-300 dark:hover:bg-slate-700" onclick={closeAuditTrail}>
                      {dict.close}
                    </button>
                  </div>
                </div>

                <div class="mt-4 grid gap-2 sm:grid-cols-2">
                  {#each auditHeaderMetrics as item}
                    <div class="rounded-2xl border border-white/70 bg-white/80 px-3 py-2.5 shadow-sm dark:border-slate-700 dark:bg-slate-900/40">
                      <div class="text-[11px] uppercase tracking-wide text-slate-500 dark:text-slate-400">{item.label}</div>
                      <div class="mt-1 font-mono text-sm font-semibold text-slate-950 dark:text-slate-50">{item.value}</div>
                    </div>
                  {/each}
                </div>

                <div class="mt-4 rounded-2xl border border-white/70 bg-white/80 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                  <div class="flex items-center justify-between gap-3">
                    <p class="text-[11px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">{auditCopy.recommendation}</p>
                    <span class="rounded-full bg-slate-100 px-2 py-0.5 text-[11px] font-semibold text-slate-700 dark:bg-slate-800 dark:text-slate-200">{selectedPeakIndex >= 0 ? `${auditCopy.queue}: ${selectedPeakIndex + 1} / ${navigationPeaks.length}` : "—"}</span>
                  </div>
                  <p class="mt-2 text-sm font-medium text-slate-900 dark:text-slate-100">{auditOverview.recommendation}</p>
                  <p class="mt-2 text-xs text-slate-500 dark:text-slate-400">{auditCopy.keyboardHint}</p>
                </div>
              </section>

              <section class="space-y-3 rounded-3xl border border-slate-200 bg-white p-3 shadow-sm dark:border-slate-700 dark:bg-slate-800/70">
                <div class="inline-flex rounded-xl bg-slate-100 p-1 dark:bg-slate-900/70">
                  <button type="button" class={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-colors ${auditDetailTab === "review" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 dark:text-slate-400"}`} onclick={() => (auditDetailTab = "review")}>{auditCopy.reviewTab}</button>
                  <button type="button" class={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-colors ${auditDetailTab === "summary" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 dark:text-slate-400"}`} onclick={() => (auditDetailTab = "summary")}>{auditCopy.summaryTab}</button>
                  <button type="button" class={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-colors ${auditDetailTab === "evidence" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 dark:text-slate-400"}`} onclick={() => (auditDetailTab = "evidence")}>{auditCopy.evidenceTab}</button>
                  <button type="button" class={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-colors ${auditDetailTab === "raw" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 dark:text-slate-400"}`} onclick={() => (auditDetailTab = "raw")}>{auditCopy.rawTab}</button>
                </div>

                {#if auditDetailTab === "review"}
                  <div class="space-y-3">
                    <section class="rounded-2xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.whyFlagged}</h4>
                      <div class="mt-3 space-y-2">
                        {#each auditReasons as reason}
                          <div class="rounded-2xl border border-slate-200 bg-white px-3 py-2 text-sm leading-6 text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200">{reason}</div>
                        {/each}
                      </div>
                    </section>

                    <section class="rounded-2xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.recommendation}</h4>
                      <div class="mt-3 rounded-2xl border border-slate-200 bg-white px-3 py-3 dark:border-slate-700 dark:bg-slate-800">
                        <p class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditOverview.recommendation}</p>
                        <p class="mt-2 text-xs leading-5 text-slate-500 dark:text-slate-400">{auditOverview.summary}</p>
                      </div>
                    </section>
                  </div>
                {:else if auditDetailTab === "summary"}
                  <div class="space-y-3">
                    <section class="rounded-2xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.summaryTab}</h4>
                      <div class="mt-3 grid gap-2 sm:grid-cols-2">
                        {#each auditHeaderMetrics as item}
                          <div class="rounded-2xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-700 dark:bg-slate-800">
                            <div class="text-[11px] uppercase tracking-wide text-slate-500 dark:text-slate-400">{item.label}</div>
                            <div class="mt-1 font-mono text-sm font-semibold text-slate-900 dark:text-slate-100">{item.value}</div>
                          </div>
                        {/each}
                      </div>
                    </section>

                    <section class="rounded-2xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.context}</h4>
                      <div class="mt-3 grid gap-2 sm:grid-cols-2">
                        {#each auditContextItems as item}
                          <div class="rounded-2xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-700 dark:bg-slate-800">
                            <div class="text-[11px] uppercase tracking-wide text-slate-500 dark:text-slate-400">{item.label}</div>
                            <div class="mt-1 text-sm font-semibold text-slate-900 dark:text-slate-100">{item.value}</div>
                          </div>
                        {/each}
                      </div>
                    </section>
                  </div>
                {:else if auditDetailTab === "evidence"}
                  <div class="space-y-3">
                    <section class="space-y-2">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.evidenceSignals}</h4>
                      {#each auditEvidenceCards as card}
                        <div class={`rounded-2xl border p-3 ${toneClasses(card.tone)}`}>
                          <div class="flex items-start justify-between gap-3">
                            <div>
                              <p class="text-sm font-semibold">{card.title}</p>
                              <p class="mt-1 text-xs leading-5 opacity-90">{card.note}</p>
                            </div>
                            <div class="text-right">
                              <div class="font-mono text-sm font-semibold">{card.value}</div>
                              <div class="text-[11px] opacity-75">{card.threshold}</div>
                            </div>
                          </div>
                          <div class="mt-3 space-y-2">
                            <div class="h-2 overflow-hidden rounded-full bg-white/70 dark:bg-slate-950/40">
                              <div class={`h-full rounded-full ${toneBarClasses(card.tone)}`} style={`width:${Math.min(100, Math.max(4, card.meter))}%`}></div>
                            </div>
                            <div class="grid grid-cols-3 gap-2 text-[11px]">
                              <div>
                                <div class="uppercase tracking-wide opacity-70">{auditCopy.actual}</div>
                                <div class="mt-1 font-mono">{card.value}</div>
                              </div>
                              <div>
                                <div class="uppercase tracking-wide opacity-70">{auditCopy.threshold}</div>
                                <div class="mt-1 font-mono">{card.threshold}</div>
                              </div>
                              <div>
                                <div class="uppercase tracking-wide opacity-70">{auditCopy.delta}</div>
                                <div class="mt-1 font-mono">{card.delta}</div>
                              </div>
                            </div>
                          </div>
                        </div>
                      {/each}
                    </section>

                    <section class="rounded-2xl border border-slate-200 bg-slate-50/70 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.blankContext}</h4>
                      {#if auditBlankDetailItems.length}
                        <div class="mt-3 grid gap-2 sm:grid-cols-2">
                          {#each auditBlankDetailItems as item}
                            <div class="rounded-2xl border border-slate-200 bg-white px-3 py-2 dark:border-slate-700 dark:bg-slate-800">
                              <div class="text-[11px] uppercase tracking-wide text-slate-500 dark:text-slate-400">{item.label}</div>
                              <div class="mt-1 text-sm font-semibold text-slate-900 dark:text-slate-100">{item.value}</div>
                            </div>
                          {/each}
                        </div>
                      {:else}
                        <div class="mt-3 rounded-2xl border border-dashed border-slate-300 px-3 py-6 text-center text-xs text-slate-500 dark:border-slate-600 dark:text-slate-400">{auditCopy.noBlankDetail}</div>
                      {/if}
                    </section>
                  </div>
                {:else}
                  <div class="space-y-3">
                    <div class="flex items-center justify-between gap-3">
                      <h4 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{auditCopy.rawTab}</h4>
                      <span class="text-xs text-slate-500 dark:text-slate-400">{visibleAuditJsonEntries.length} / {filteredAuditJsonEntries.length}</span>
                    </div>
                    <input
                      type="text"
                      bind:value={auditJsonQuery}
                      placeholder={auditCopy.searchJson}
                      class="w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-700 outline-none transition-colors focus:border-blue-300 dark:border-slate-600 dark:bg-slate-900 dark:text-slate-200"
                    />
                    <div class="overflow-hidden rounded-2xl border border-slate-200 dark:border-slate-700">
                      {#if visibleAuditJsonEntries.length}
                        <div class="divide-y divide-slate-100 text-xs dark:divide-slate-700">
                          {#each visibleAuditJsonEntries as entry}
                            <div class={`grid grid-cols-[1fr_auto] gap-3 px-3 py-2 ${isDecisionKey(entry.path) ? "bg-blue-50/70 dark:bg-blue-950/20" : "bg-white dark:bg-slate-800/80"}`}>
                              <span class="truncate font-mono text-slate-600 dark:text-slate-300">{entry.path || "(root)"}</span>
                              <span class="max-w-[12rem] truncate font-mono text-slate-900 dark:text-slate-100">{entry.value}</span>
                            </div>
                          {/each}
                        </div>
                      {:else}
                        <div class="px-3 py-6 text-center text-xs text-slate-500 dark:text-slate-400">{auditCopy.noMatches}</div>
                      {/if}
                    </div>
                    <details class="rounded-2xl border border-slate-200 bg-slate-50 p-3 dark:border-slate-700 dark:bg-slate-900/40">
                      <summary class="cursor-pointer text-xs font-semibold text-slate-700 dark:text-slate-300">Raw JSON</summary>
                      <pre class="mt-3 max-h-72 overflow-auto rounded-2xl bg-slate-950 p-3 text-[11px] leading-5 text-slate-100">{JSON.stringify(selectedPeak.Why, null, 2)}</pre>
                    </details>
                  </div>
                {/if}
              </section>
            </div>
          {:else}
            <div class="flex h-full min-h-[20rem] items-center justify-center">
              <div class="max-w-sm rounded-3xl border border-dashed border-slate-300 bg-slate-50/70 p-6 text-center shadow-sm dark:border-slate-600 dark:bg-slate-900/40">
                <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-300">
                  <svg viewBox="0 0 24 24" class="h-6 w-6" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h16M4 18h10"/><path d="m15 15 3 3 5-5"/></svg>
                </div>
                <h3 class="mt-4 text-lg font-semibold text-slate-900 dark:text-slate-100">{auditCopy.selectPeak}</h3>
                <p class="mt-2 text-sm leading-6 text-slate-600 dark:text-slate-300">{auditCopy.selectPeakDesc}</p>
              </div>
            </div>
          {/if}
        </aside>
      </div>
    </section>
  {/if}

  {#if activeTab === "3d"}
    <section class="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-800 sm:p-5">
      <div class="mb-4 flex items-center justify-between gap-4">
        <div>
          <h3 class="text-base font-semibold text-slate-900 dark:text-slate-100">
            {view3d === "quality" ? "Quality Cube"
              : view3d === "landscape" ? "Feature Landscape"
              : view3d === "coverage" ? "Coverage Map"
              : view3d === "score" ? "Score Separation"
              : "Precision vs Intensity"}
          </h3>
        </div>
        <div class="inline-flex shrink-0 rounded-lg border border-slate-200 bg-slate-100 p-0.5 dark:border-slate-600 dark:bg-slate-700">
          <button
            type="button"
            class={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${view3d === "quality" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"}`}
            onclick={() => (view3d = "quality")}
          >Quality Cube</button>
          <button
            type="button"
            class={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${view3d === "landscape" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"}`}
            onclick={() => (view3d = "landscape")}
          >Feature Landscape</button>
          <button
            type="button"
            class={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${view3d === "coverage" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"}`}
            onclick={() => (view3d = "coverage")}
          >Coverage Map</button>
          <button
            type="button"
            class={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${view3d === "score" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"}`}
            onclick={() => (view3d = "score")}
          >Score Separation</button>
          <button
            type="button"
            class={`rounded-md px-3 py-1 text-xs font-medium transition-colors ${view3d === "precision" ? "bg-white text-slate-900 shadow-sm dark:bg-slate-800 dark:text-slate-100" : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"}`}
            onclick={() => (view3d = "precision")}
          >Precision vs Intensity</button>
        </div>
      </div>
      {#if view3d === "quality"}
        <QualityCube {peaks} {parameters} />
      {:else if view3d === "landscape"}
        <FeatureLandscape {peaks} {parameters} />
      {:else if view3d === "coverage"}
        <CoverageMap {peaks} {parameters} />
      {:else if view3d === "score"}
        <ScoreSeparation {peaks} {parameters} />
      {:else}
        <PrecisionIntensity {peaks} {parameters} />
      {/if}
    </section>
  {/if}
</div>
