import type { ScreeningParams, ScreeningResult, SurrogateSpec, ParsedRow } from "$lib/screening";

export const LAB_SPACE_ID = "default";

export type DatasetKind = "analysis" | "blank" | "surrogate_observed";

export interface LabSpaceMeta {
  id: string;
  currentLabSpaceId: string;
  name: string;
  notes: string;
  activeAnalyzerId: string | null;
  selectedAnalysisDatasetId: string | null;
  selectedBlankDatasetId: string | null;
  selectedSurrogateDatasetId: string | null;
  selectedDatasetId: string | null;
  selectedReportId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface StoredDataset {
  id: string;
  kind: DatasetKind;
  name: string;
  sourceName: string;
  rowCount: number;
  rows: ParsedRow[];
  createdAt: string;
  updatedAt: string;
}

export interface StoredSurrogateSpec extends SurrogateSpec {
  id: string;
}

export type AnalyzerRuntimeParams = Omit<ScreeningParams, "surrogates" | "mz_available">;

export interface AnalyzerPreset {
  id: string;
  name: string;
  params: AnalyzerRuntimeParams;
  syncBlankWithReplicate: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface StoredReport {
  id: string;
  title: string;
  createdAt: string;
  datasetRefs: {
    analysisDatasetId: string;
    blankDatasetId: string | null;
    surrogateDatasetId: string | null;
  };
  analyzerId: string;
  runtimeParams: ScreeningParams;
  resultSnapshot: ScreeningResult;
}

export interface LabSpaceState {
  meta: LabSpaceMeta;
  datasets: StoredDataset[];
  surrogateSpecs: StoredSurrogateSpec[];
  analyzers: AnalyzerPreset[];
  reports: StoredReport[];
}
