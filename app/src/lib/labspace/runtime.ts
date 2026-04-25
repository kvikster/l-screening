import type { ParsedRow, ScreeningParams } from "$lib/screening";
import type {
  AnalyzerPreset,
  LabSpaceState,
  StoredDataset,
  StoredSurrogateSpec,
} from "./types";

export interface ComposedLabAnalysisRun {
  rows: ParsedRow[];
  runtimeParams: ScreeningParams;
  analysisDataset: StoredDataset;
  blankDataset: StoredDataset | null;
  surrogateDataset: StoredDataset | null;
  analyzer: AnalyzerPreset;
  reportTitle: string;
}

type RowRole = "sample" | "blank" | "surrogate" | "unknown";

export const DATASET_KIND_LABELS: Record<StoredDataset["kind"], string> = {
  analysis: "Normal",
  blank: "Blank",
  surrogate_observed: "Surrogate",
};

function normalizeText(value: string | null | undefined): string {
  return String(value ?? "").trim();
}

function normalizePolarity(value: string): "positive" | "negative" {
  return /neg/i.test(value) ? "negative" : "positive";
}

export function inferRowRole(row: ParsedRow): RowRole {
  const mark = normalizeText(row.operator_mark).toLowerCase();
  if (mark === "surrogate" || mark === "surrogate_positive" || mark === "surrogate_negative") {
    return "surrogate";
  }
  if (mark === "blank_positive" || mark === "blank_negative") {
    return "blank";
  }
  if (mark === "sample_rep1" || mark === "sample_rep2") {
    return "sample";
  }

  const file = normalizeText(row.File).toLowerCase();
  if (file.includes("blank")) {
    return "blank";
  }
  if (file.includes("surrogate")) {
    return "surrogate";
  }
  if (/^\d+_/.test(file) || /^\d+_neg/.test(file)) {
    return "sample";
  }
  return "unknown";
}

export function retagDatasetRowsForKind(
  rows: ParsedRow[],
  kind: StoredDataset["kind"],
): ParsedRow[] {
  return rows.map((row, index) => {
    const polarity = normalizePolarity(row.Polarity);
    const suffix = polarity === "negative" ? "negative" : "positive";

    if (kind === "analysis") {
      return {
        ...row,
        operator_mark: null,
        operator_color: null,
      };
    }

    if (kind === "blank") {
      return {
        ...row,
        File: normalizeText(row.File) || `blank_${index + 1}.d`,
        operator_mark: `blank_${suffix}`,
        operator_color: null,
      };
    }

    return {
      ...row,
      File: normalizeText(row.File) || `surrogate_${index + 1}.d`,
      operator_mark: "surrogate",
      operator_color: null,
    };
  });
}

export function normalizeDatasetRowsForKind(dataset: StoredDataset): ParsedRow[] {
  return dataset.rows.map((row, index) => {
    if (dataset.kind === "analysis") {
      const role = inferRowRole(row);
      const polarity = normalizePolarity(row.Polarity);
      const suffix = polarity === "negative" ? "negative" : "positive";

      if (role === "blank") {
        return {
          ...row,
          File: normalizeText(row.File) || `blank_${index + 1}.d`,
          operator_mark: row.operator_mark ?? `blank_${suffix}`,
          operator_color: row.operator_color ?? null,
        };
      }

      if (role === "surrogate") {
        return {
          ...row,
          File: normalizeText(row.File) || `surrogate_${index + 1}.d`,
          operator_mark: row.operator_mark ?? "surrogate",
          operator_color: row.operator_color ?? null,
        };
      }

      return { ...row };
    }

    return retagDatasetRowsForKind([row], dataset.kind)[0];
  });
}

export function validateDatasetForKind(kind: StoredDataset["kind"], rows: ParsedRow[]): string | null {
  if (rows.length === 0) {
    return "Dataset is empty.";
  }
  return null;
}

export function buildRuntimeParams(
  analyzer: AnalyzerPreset,
  surrogateSpecs: StoredSurrogateSpec[],
): ScreeningParams {
  const params: ScreeningParams = {
    ...analyzer.params,
    surrogates: surrogateSpecs.map(({ id: _id, ...spec }) => spec),
  };
  return params;
}

export function buildReportTitle(
  analysisDataset: StoredDataset,
  analyzer: AnalyzerPreset,
): string {
  return `${analysisDataset.name} • ${analyzer.name}`;
}

export function composeLabAnalysisRun(input: {
  analysisDataset: StoredDataset | null;
  blankDataset: StoredDataset | null;
  surrogateDataset: StoredDataset | null;
  analyzer: AnalyzerPreset | null;
  surrogateSpecs: StoredSurrogateSpec[];
}): ComposedLabAnalysisRun {
  const { analysisDataset, blankDataset, surrogateDataset, analyzer, surrogateSpecs } = input;

  if (!analysisDataset) {
    throw new Error("Choose an analysis dataset before running.");
  }
  if (!analyzer) {
    throw new Error("Choose an analyzer before running.");
  }

  const analysisError = validateDatasetForKind("analysis", analysisDataset.rows);
  if (analysisError) {
    throw new Error(analysisError);
  }

  const rows = [
    ...normalizeDatasetRowsForKind(analysisDataset),
    ...(blankDataset ? normalizeDatasetRowsForKind(blankDataset) : []),
    ...(surrogateDataset ? normalizeDatasetRowsForKind(surrogateDataset) : []),
  ];

  return {
    rows,
    runtimeParams: buildRuntimeParams(analyzer, surrogateSpecs),
    analysisDataset,
    blankDataset,
    surrogateDataset,
    analyzer,
    reportTitle: buildReportTitle(analysisDataset, analyzer),
  };
}

export function resolveSelectedEntities(state: LabSpaceState) {
  return {
    analysisDataset:
      state.datasets.find((dataset) => dataset.id === state.meta.selectedAnalysisDatasetId) ?? null,
    blankDataset:
      state.datasets.find((dataset) => dataset.id === state.meta.selectedBlankDatasetId) ?? null,
    surrogateDataset:
      state.datasets.find((dataset) => dataset.id === state.meta.selectedSurrogateDatasetId) ?? null,
    analyzer:
      state.analyzers.find((preset) => preset.id === state.meta.activeAnalyzerId) ?? null,
  };
}
