import { beforeAll, describe, expect, it } from "vitest";
import { screenRowsWithProcessor, type ParsedRow } from "$lib/screening";
import {
  createLabSpaceRepository,
  createMemoryLabSpaceAdapter,
  defaultAnalyzerPreset,
} from "./repository";
import {
  buildRuntimeParams,
  composeLabAnalysisRun,
  normalizeDatasetRowsForKind,
  retagDatasetRowsForKind,
  resolveSelectedEntities,
  validateDatasetForKind,
} from "./runtime";
import type { AnalyzerPreset, StoredDataset, StoredSurrogateSpec } from "./types";

function makeDataset(
  id: string,
  kind: StoredDataset["kind"],
  rows: ParsedRow[],
  name = id,
): StoredDataset {
  return {
    id,
    kind,
    name,
    sourceName: `${name}.xlsx`,
    rowCount: rows.length,
    rows,
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
  };
}

const analysisRows: ParsedRow[] = [
  {
    RT: 1.99,
    "Base Peak": 150.0,
    Area: 500.0,
    Polarity: "+",
    File: "1_a",
    Label: "s1a",
    operator_color: null,
    operator_mark: null,
  },
  {
    RT: 2.01,
    "Base Peak": 150.1,
    Area: 510.0,
    Polarity: "+",
    File: "1_b",
    Label: "s1b",
    operator_color: null,
    operator_mark: null,
  },
];

const blankRows: ParsedRow[] = [
  {
    RT: 1.94,
    "Base Peak": 150.0,
    Area: 100.0,
    Polarity: "+",
    File: "blank_a",
    Label: "b1",
    operator_color: null,
    operator_mark: null,
  },
  {
    RT: 1.96,
    "Base Peak": 150.1,
    Area: 110.0,
    Polarity: "+",
    File: "blank_b",
    Label: "b2",
    operator_color: null,
    operator_mark: null,
  },
];

const surrogateRows: ParsedRow[] = [
  {
    RT: 5.31,
    "Base Peak": 128.0,
    Area: 143250,
    Polarity: "+",
    File: "surrogate_a",
    Label: "d8-Naphthalene",
    operator_color: null,
    operator_mark: null,
  },
  {
    RT: 5.32,
    "Base Peak": 128.1,
    Area: 146000,
    Polarity: "+",
    File: "surrogate_b",
    Label: "d8-Naphthalene",
    operator_color: null,
    operator_mark: null,
  },
];

describe("LabSpace repository", () => {
  it("creates default meta and analyzer, and persists CRUD entities", async () => {
    const repo = createLabSpaceRepository(createMemoryLabSpaceAdapter());
    const initial = await repo.loadState();

    expect(initial.meta.id).toBe("default");
    expect(initial.analyzers).toHaveLength(1);
    expect(initial.meta.activeAnalyzerId).toBe(initial.analyzers[0].id);

    const dataset = await repo.upsertDataset({
      kind: "analysis",
      name: "Study A",
      sourceName: "study_a.xlsx",
      rows: [
        {
          RT: 1.01,
          "Base Peak": 120.1,
          Area: 1000,
          Polarity: "Positive",
          File: "1_sample_a.d",
          Label: "peak-1",
          operator_color: null,
          operator_mark: null,
        },
      ],
    });

    const analyzer = await repo.upsertAnalyzer({
      name: "Tight QC",
      params: {
        ...initial.analyzers[0].params,
        signal_to_blank_min: 4,
      },
      syncBlankWithReplicate: false,
    });

    const specs: StoredSurrogateSpec[] = [
      {
        id: "spec_1",
        name: "d8-Naphthalene",
        expected_rt: 5.23,
        expected_area: 150000,
        expected_mz: 128.0,
        rt_window: 0.2,
        recovery_min_pct: 70,
        recovery_max_pct: 130,
      },
    ];
    await repo.replaceSurrogateSpecs(specs);

    const report = await repo.createReport({
      title: "Study A • Tight QC",
      datasetRefs: {
        analysisDatasetId: dataset.id,
        blankDatasetId: null,
        surrogateDatasetId: null,
      },
      analyzerId: analyzer.id,
      runtimeParams: {
        ...analyzer.params,
        surrogates: specs.map(({ id: _id, ...spec }) => spec),
      },
      resultSnapshot: {
        dashboardProps: {
          title: "Study A",
          summary: [],
          peaks: [],
        },
        allPeaks: [],
        allSummary: [],
        rawRows: dataset.rows,
      },
    });

    const loaded = await repo.loadState();
    expect(loaded.datasets.map((item) => item.id)).toContain(dataset.id);
    expect(loaded.analyzers.map((item) => item.id)).toContain(analyzer.id);
    expect(loaded.surrogateSpecs).toEqual(specs);
    expect(loaded.reports.map((item) => item.id)).toContain(report.id);

    await repo.deleteReport(report.id);
    await repo.deleteDataset(dataset.id);
    await repo.deleteAnalyzer(analyzer.id);

    const afterDelete = await repo.loadState();
    expect(afterDelete.reports.find((item) => item.id === report.id)).toBeUndefined();
    expect(afterDelete.datasets.find((item) => item.id === dataset.id)).toBeUndefined();
  });
});

describe("LabSpace runtime", () => {
  it("allows mixed analysis datasets and normalizes inferred blank/surrogate rows", () => {
    const mixedRows = [
      ...analysisRows,
      {
        ...blankRows[0],
        operator_mark: "blank_positive",
      },
      {
        ...surrogateRows[0],
        operator_mark: null,
      },
    ];

    expect(validateDatasetForKind("analysis", mixedRows)).toBeNull();

    const normalizedAnalysis = normalizeDatasetRowsForKind(
      makeDataset("analysis_mixed", "analysis", mixedRows),
    );

    const normalizedBlank = normalizeDatasetRowsForKind(makeDataset("blank_1", "blank", blankRows));
    const normalizedSurrogate = normalizeDatasetRowsForKind(
      makeDataset("sur_1", "surrogate_observed", surrogateRows),
    );

    expect(normalizedAnalysis.some((row) => row.operator_mark?.startsWith("blank_"))).toBe(true);
    expect(normalizedAnalysis.some((row) => row.operator_mark === "surrogate")).toBe(true);
    expect(normalizedBlank.every((row) => row.operator_mark?.startsWith("blank_"))).toBe(true);
    expect(normalizedSurrogate.every((row) => row.operator_mark === "surrogate")).toBe(true);
  });

  it("retags whole datasets when switching between normal, blank, and surrogate modes", () => {
    const forcedBlank = retagDatasetRowsForKind(analysisRows, "blank");
    const forcedSurrogate = retagDatasetRowsForKind(analysisRows, "surrogate_observed");
    const resetToNormal = retagDatasetRowsForKind(forcedBlank, "analysis");

    expect(forcedBlank.every((row) => row.operator_mark?.startsWith("blank_"))).toBe(true);
    expect(forcedSurrogate.every((row) => row.operator_mark === "surrogate")).toBe(true);
    expect(resetToNormal.every((row) => row.operator_mark == null)).toBe(true);
  });

  it("composes runtime params without overwriting surrogate specs", () => {
    const analyzer: AnalyzerPreset = {
      ...defaultAnalyzerPreset(),
      id: "analyzer_main",
      params: {
        ...defaultAnalyzerPreset().params,
        signal_to_blank_min: 4.5,
      },
    };
    const specs: StoredSurrogateSpec[] = [
      {
        id: "spec_1",
        name: "d8-Naphthalene",
        expected_rt: 5.23,
        expected_area: 150000,
        expected_mz: 128.0,
      },
    ];

    const runtimeParams = buildRuntimeParams(analyzer, specs);
    expect(runtimeParams.signal_to_blank_min).toBe(4.5);
    expect(runtimeParams.surrogates).toEqual([
      {
        name: "d8-Naphthalene",
        expected_rt: 5.23,
        expected_area: 150000,
        expected_mz: 128.0,
      },
    ]);
  });

  it("resolves selected entities from persisted state", () => {
    const analyzer = { ...defaultAnalyzerPreset(), id: "analyzer_active" };
    const state = {
      meta: {
        id: "default",
        currentLabSpaceId: "default",
        name: "Default LabSpace",
        notes: "",
        activeAnalyzerId: "analyzer_active",
        selectedAnalysisDatasetId: "analysis_1",
        selectedBlankDatasetId: "blank_1",
        selectedSurrogateDatasetId: "sur_1",
        selectedDatasetId: null,
        selectedReportId: null,
        createdAt: "2026-01-01T00:00:00.000Z",
        updatedAt: "2026-01-01T00:00:00.000Z",
      },
      analyzers: [analyzer],
      datasets: [
        makeDataset("analysis_1", "analysis", analysisRows),
        makeDataset("blank_1", "blank", blankRows),
        makeDataset("sur_1", "surrogate_observed", surrogateRows),
      ],
      surrogateSpecs: [],
      reports: [],
    };

    const resolved = resolveSelectedEntities(state);
    expect(resolved.analysisDataset?.id).toBe("analysis_1");
    expect(resolved.blankDataset?.id).toBe("blank_1");
    expect(resolved.surrogateDataset?.id).toBe("sur_1");
    expect(resolved.analyzer?.id).toBe("analyzer_active");
  });
});

describe("LabSpace + WASM integration", () => {
  let processPeaks: (rows: string, config: string) => string;

  beforeAll(async () => {
    const { readFileSync } = await import("fs");
    const { fileURLToPath } = await import("url");
    const { dirname, resolve } = await import("path");
    const { initSync, process_peaks } = await import("../wasm-pkg/screening_wasm.js");
    const __dir = dirname(fileURLToPath(import.meta.url));
    const wasmBuf = readFileSync(resolve(__dir, "../wasm-pkg", "screening_wasm_bg.wasm"));
    initSync({ module: wasmBuf });
    processPeaks = process_peaks;
  });

  const analyzer = defaultAnalyzerPreset();
  const specs: StoredSurrogateSpec[] = [
    {
      id: "spec_1",
      name: "d8-Naphthalene",
      expected_rt: 5.23,
      expected_area: 150000,
      expected_mz: 128.0,
      rt_window: 0.2,
      recovery_min_pct: 70,
      recovery_max_pct: 130,
    },
  ];

  it("runs analysis without blank", () => {
    const composed = composeLabAnalysisRun({
      analysisDataset: makeDataset("analysis_1", "analysis", analysisRows),
      blankDataset: null,
      surrogateDataset: null,
      analyzer,
      surrogateSpecs: [],
    });

    const result = screenRowsWithProcessor(
      composed.rows,
      composed.runtimeParams,
      "analysis-only",
      processPeaks,
    );

    expect(result.allPeaks.length).toBeGreaterThan(0);
    expect(result.allPeaks[0].SampleType).toBe("sample");
  });

  it("classifies blank and surrogate rows embedded in analysis dataset", () => {
    const mixedAnalysisRows = [...analysisRows, ...blankRows, ...surrogateRows];
    const composed = composeLabAnalysisRun({
      analysisDataset: makeDataset("analysis_mixed", "analysis", mixedAnalysisRows),
      blankDataset: null,
      surrogateDataset: null,
      analyzer,
      surrogateSpecs: specs,
    });

    const result = screenRowsWithProcessor(
      composed.rows,
      composed.runtimeParams,
      "analysis-mixed",
      processPeaks,
    );

    expect(result.allPeaks.some((peak) => peak.IsSurrogate === true)).toBe(true);
    expect(result.allPeaks.some((peak) => peak.BlankAreaMean != null)).toBe(true);
  });

  it("runs analysis with blank", () => {
    const composed = composeLabAnalysisRun({
      analysisDataset: makeDataset("analysis_1", "analysis", analysisRows),
      blankDataset: makeDataset("blank_1", "blank", blankRows),
      surrogateDataset: null,
      analyzer,
      surrogateSpecs: [],
    });

    const result = screenRowsWithProcessor(
      composed.rows,
      composed.runtimeParams,
      "analysis-with-blank",
      processPeaks,
    );

    expect(result.allPeaks.some((peak) => peak.BlankAreaMean != null)).toBe(true);
  });

  it("runs analysis with surrogate observed rows and specs", () => {
    const composed = composeLabAnalysisRun({
      analysisDataset: makeDataset("analysis_1", "analysis", analysisRows),
      blankDataset: null,
      surrogateDataset: makeDataset("sur_1", "surrogate_observed", surrogateRows),
      analyzer,
      surrogateSpecs: specs,
    });

    const result = screenRowsWithProcessor(
      composed.rows,
      composed.runtimeParams,
      "analysis-with-surrogates",
      processPeaks,
    );

    expect(result.allPeaks.some((peak) => peak.IsSurrogate === true)).toBe(true);
    expect(result.allPeaks.some((peak) => peak.Status === "Surrogate OK")).toBe(true);
  });

  it("runs analysis with blank and surrogate together", () => {
    const composed = composeLabAnalysisRun({
      analysisDataset: makeDataset("analysis_1", "analysis", analysisRows),
      blankDataset: makeDataset("blank_1", "blank", blankRows),
      surrogateDataset: makeDataset("sur_1", "surrogate_observed", surrogateRows),
      analyzer,
      surrogateSpecs: specs,
    });

    const result = screenRowsWithProcessor(
      composed.rows,
      composed.runtimeParams,
      "analysis-with-all",
      processPeaks,
    );

    expect(result.allPeaks.some((peak) => peak.BlankAreaMean != null)).toBe(true);
    expect(result.allPeaks.some((peak) => peak.IsSurrogate === true)).toBe(true);
  });
});
