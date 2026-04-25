/**
 * screening.ts — client-side screening via Rust WASM.
 *
 * Parses an Excel file with SheetJS (including cell fill colors for operator marks),
 * passes the row data to the Rust WASM `process_peaks` function, and returns a
 * DashboardProps object compatible with the Dashboard Svelte component.
 */

import * as XLSX from "xlsx";
import type { DashboardProps } from "$lib/json-render/catalog";
import { t } from "$lib/i18n";

// ─── Operator mark color semantics (mirrors colors.py) ───────────────────────

// SheetJS reports fill colors as ARGB hex strings, e.g. "FFFF00FF" for magenta.
// We compare the last 6 chars (RRGGBB) to identify operator marks.
const COLOR_SEMANTICS: Record<string, string> = {
  FF00FF: "sample_rep1", // Magenta
  FFFF00: "sample_rep2", // Yellow
  "00FFFF": "blank_positive", // Cyan
  "00FF00": "blank_negative", // Green
};

function resolveOperatorMark(argbHex: string | undefined): {
  operator_color: string | null;
  operator_mark: string | null;
} {
  if (!argbHex || argbHex.length < 6) {
    return { operator_color: null, operator_mark: null };
  }
  const rrggbb = argbHex.slice(-6).toUpperCase();
  return {
    operator_color: rrggbb,
    operator_mark: COLOR_SEMANTICS[rrggbb] ?? null,
  };
}

// ─── Tabular parsing ──────────────────────────────────────────────────────────

const REQUIRED_COLS = ["RT", "File", "Area"];

interface ParsedRow {
  RT: number;
  "Base Peak": number | null;
  Area: number;
  Polarity: string;
  File: string;
  Label?: string | null;
  operator_color: string | null;
  operator_mark: string | null;
}

/**
 * Infer ionization polarity from a filename when no `Polarity` column exists.
 * Convention: tokens like `_neg`, `neg_`, `negative` → Negative; otherwise Positive.
 */
function inferPolarityFromFile(filename: string): string {
  return /(?:^|[\W_])neg(?:ative)?(?:[\W_]|$)/i.test(filename)
    ? "Negative"
    : "Positive";
}

/**
 * Select the sheet that has the most required column names.
 * Raises if not all required columns are found.
 */
function selectBestSheet(wb: XLSX.WorkBook): string {
  let bestName = wb.SheetNames[0];
  let bestScore = 0;

  for (const name of wb.SheetNames) {
    const ws = wb.Sheets[name];
    const range = XLSX.utils.decode_range(ws["!ref"] ?? "A1");
    const headers: string[] = [];
    for (let c = range.s.c; c <= range.e.c; c++) {
      const cell = ws[XLSX.utils.encode_cell({ r: range.s.r, c })];
      if (cell?.v != null) {
        headers.push(String(cell.v).trim());
      }
    }
    const score = REQUIRED_COLS.filter((col) => headers.includes(col)).length;
    if (score > bestScore) {
      bestScore = score;
      bestName = name;
    }
  }

  if (bestScore < REQUIRED_COLS.length) {
    throw new Error(t("noValidSheet", { columns: REQUIRED_COLS.join(", ") }));
  }
  return bestName;
}

/**
 * Parse a SheetJS workbook into an array of row objects.
 * Reads cell fill colors from the first column to assign operator_mark
 * (only meaningful for xlsx; csv/txt have no styles).
 */
function parseWorkbook(wb: XLSX.WorkBook): ParsedRow[] {
  const sheetName = selectBestSheet(wb);
  const ws = wb.Sheets[sheetName];

  const range = XLSX.utils.decode_range(ws["!ref"] ?? "A1");

  // Read header row.
  const headers: string[] = [];
  for (let c = range.s.c; c <= range.e.c; c++) {
    const cell = ws[XLSX.utils.encode_cell({ r: range.s.r, c })];
    headers.push(cell?.v != null ? String(cell.v).trim() : "");
  }

  const colIdx: Record<string, number> = {};
  for (const col of REQUIRED_COLS) {
    const idx = headers.indexOf(col);
    if (idx !== -1) colIdx[col] = idx;
  }
  // Optional columns.
  const labelIdx = headers.indexOf("Label");
  const operatorMarkIdx = headers.indexOf("operator_mark");
  const operatorColorIdx = headers.indexOf("operator_color");

  const rows: ParsedRow[] = [];

  for (let r = range.s.r + 1; r <= range.e.r; r++) {
    // Extract cell fill color from the first data column (col = range.s.c).
    const firstCell = ws[XLSX.utils.encode_cell({ r, c: range.s.c })];
    const fillRgb =
      firstCell?.s?.fgColor?.rgb ?? firstCell?.s?.bgColor?.rgb ?? undefined;
    const colorMark = resolveOperatorMark(fillRgb);

    const get = (col: string): XLSX.CellObject | undefined =>
      colIdx[col] !== undefined
        ? ws[XLSX.utils.encode_cell({ r, c: colIdx[col] })]
        : undefined;

    const rtCell = get("RT");
    const mzIdx = headers.indexOf("Base Peak");
    const mzCell =
        mzIdx !== -1 ? ws[XLSX.utils.encode_cell({ r, c: mzIdx })] : undefined;
    const areaCell = get("Area");
    const polCell = get("Polarity");
    const fileCell = get("File");

    // Skip rows missing numeric essentials.
    const rt = rtCell?.v != null ? Number(rtCell.v) : NaN;
    const mz = mzCell?.v != null ? Number(mzCell.v) : null;
    const area = areaCell?.v != null ? Number(areaCell.v) : NaN;
    if (!isFinite(rt) || !isFinite(area)) continue;

    const file = fileCell?.v != null ? String(fileCell.v).trim() : "";
    const polarity =
      polCell?.v != null && String(polCell.v).trim() !== ""
        ? String(polCell.v).trim()
        : inferPolarityFromFile(file);

    const labelCell =
      labelIdx !== -1
        ? ws[XLSX.utils.encode_cell({ r, c: labelIdx })]
        : undefined;
    const operatorMarkCell =
      operatorMarkIdx !== -1
        ? ws[XLSX.utils.encode_cell({ r, c: operatorMarkIdx })]
        : undefined;
    const operatorColorCell =
      operatorColorIdx !== -1
        ? ws[XLSX.utils.encode_cell({ r, c: operatorColorIdx })]
        : undefined;
    const label = labelCell?.v != null ? String(labelCell.v) : null;
    const explicitOperatorMark =
      operatorMarkCell?.v != null ? String(operatorMarkCell.v).trim() : null;
    const explicitOperatorColor =
      operatorColorCell?.v != null ? String(operatorColorCell.v).trim() : null;

    rows.push({
      RT: rt,
      "Base Peak": mz != null && isFinite(mz) ? mz : null,
      Area: area,
      Polarity: polarity,
      File: file,
      Label: label,
      operator_color: explicitOperatorColor || colorMark.operator_color,
      operator_mark: explicitOperatorMark || colorMark.operator_mark,
    });
  }

  return rows;
}

// ─── Format registry ──────────────────────────────────────────────────────────

interface FormatParser {
  name: string;
  match: (file: File) => boolean;
  parse: (file: File) => Promise<ParsedRow[]>;
}

const PARSERS: FormatParser[] = [
  {
    name: "csv",
    match: (f) => /\.(csv|tsv|txt)$/i.test(f.name),
    parse: async (f) =>
      parseWorkbook(XLSX.read(await f.text(), { type: "string" })),
  },
  {
    name: "xlsx",
    match: (f) => /\.(xlsx|xls)$/i.test(f.name),
    parse: async (f) =>
      parseWorkbook(
        XLSX.read(await f.arrayBuffer(), { type: "array", cellStyles: true }),
      ),
  },
];

async function parseFile(file: File): Promise<ParsedRow[]> {
  const parser = PARSERS.find((p) => p.match(file)) ?? PARSERS[PARSERS.length - 1];
  return parser.parse(file);
}

// ─── WASM lazy loader ─────────────────────────────────────────────────────────

let wasmModule: {
  process_peaks: (rows: string, config: string) => string;
} | null = null;

async function getWasm() {
  if (wasmModule) return wasmModule;
  // wasm-pack builds to wasm-pkg/ with --target web.
  // @vite-ignore suppresses the "file not found" error at analysis time —
  // the file is generated by `bun run build:wasm` before dev/build.
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  // eslint-disable-next-line import/no-unresolved
  const mod = await import(
    /* @vite-ignore */ "$lib/wasm-pkg/screening_wasm.js"
  );
  await mod.default(); // initialize WASM (loads the .wasm binary)
  wasmModule = mod;
  return wasmModule;
}

// ─── Public API ───────────────────────────────────────────────────────────────

export interface ScreeningParams {
  replicate_rt_tol: number;
  replicate_mz_tol: number;
  replicate_mz_mode: string;
  blank_rt_tol: number;
  blank_mz_tol: number;
  blank_mz_mode: string;
  signal_to_blank_min: number;
  cv_high_max?: number;
  cv_moderate_max?: number;
  /** Automatically set to false when the dataset has no Base Peak column (RT-only instruments). */
  mz_available?: boolean;
  /**
   * Optional minimum absolute area difference (area_sample − area_blank).
   * When set, a peak is classified as Artifact if area_difference < this value
   * even when the S/B ratio passes the threshold — guards against unstable ratios
   * on very small signals close to the noise floor.
   */
  min_area_difference?: number;
  /**
   * Surrogate standard specifications. When non-empty, rows with
   * operator_mark = "surrogate" are validated against these specs.
   */
  surrogates?: SurrogateSpec[];
}

export interface SurrogateSpec {
  name: string;
  expected_rt: number;
  expected_area: number;
  expected_mz?: number;
  rt_window?: number;
  recovery_min_pct?: number;
  recovery_max_pct?: number;
}

export interface ScreeningResult {
  dashboardProps: DashboardProps;
  /** Full untruncated peak list — use this for export. */
  allPeaks: Record<string, unknown>[];
  allSummary: Record<string, unknown>[];
  /** Original parsed rows from the Excel file. */
  rawRows: ParsedRow[];
}

/**
 * Run screening entirely in the browser (WASM path).
 * Returns both a DashboardProps (display-ready, truncated) and the full results.
 */
export async function screenFile(
  file: File,
  params: ScreeningParams,
): Promise<ScreeningResult> {
  const rawRows = await parseFile(file);

  if (rawRows.length === 0) {
    throw new Error(t("noValidRows"));
  }

  const wasm = await getWasm();

  // Auto-detect RT-only datasets: if no row has a finite Base Peak value,
  // the dataset comes from an instrument without m/z (GC-FID, LC-UV, …).
  // Pass mz_available=false so the confidence scoring skips the MZ penalty.
  const hasMz = rawRows.some(
    (r) => r["Base Peak"] != null && isFinite(r["Base Peak"] as number),
  );
  const effectiveParams: ScreeningParams = hasMz
    ? params
    : { ...params, mz_available: false };

  const resultJson = wasm.process_peaks(
    JSON.stringify(rawRows),
    JSON.stringify(effectiveParams),
  );
  const result = JSON.parse(resultJson) as {
    results?: Record<string, unknown>[];
    summary?: Record<string, unknown>[];
    error?: string;
  };

  if (result.error) {
    throw new Error(result.error);
  }

  const allPeaks = result.results ?? [];
  const allSummary = result.summary ?? [];

  return {
    dashboardProps: buildDashboardProps(
      allPeaks,
      allSummary,
      params,
      file.name,
      rawRows.length,
    ),
    allPeaks,
    allSummary,
    rawRows,
  };
}

// ─── DashboardProps builder ───────────────────────────────────────────────────

function buildDashboardProps(
  peaks: Record<string, unknown>[],
  summary: Record<string, unknown>[],
  params: ScreeningParams,
  filename: string,
  totalRows: number,
): DashboardProps {
  const displayLimit = 1000;
  const truncated = peaks.length > displayLimit;
  const displayedPeaks = truncated ? peaks.slice(0, displayLimit) : peaks;

  return {
    title: `${filename}`,
    summary: summary as DashboardProps["summary"],
    peaks: displayedPeaks as DashboardProps["peaks"],
    parameters: params as unknown as DashboardProps["peaks"],
    metadata: {
      filename,
      total_rows: totalRows,
      total_peaks: peaks.length,
      displayed_peaks: displayedPeaks.length,
      truncated,
      engine: "wasm",
    },
  };
}

// ─── XLSX export ──────────────────────────────────────────────────────────────

/** Columns shown in the "Screened Peaks" sheet, in order. */
const PEAK_COLUMNS = [
  "SampleType",
  "Polarity",
  "Status",
  "RT_mean",
  "MZ_mean",
  "Area_mean",
  "AreaCVPct",
  "ReplicateQuality",
  "ReplicateCount",
  "MatchingMode",
  "ParallelMatch",
  "ParallelSampleCount",
  "ParallelSourceSamples",
  "BlankAreaMean",
  "AreaDifference",
  "ConfidenceScore",
  "SignalToBlankRatio",
  "IsSurrogate",
  "SurrogateRecoveryPct",
  "SurrogateRtShift",
  "SurrogatePass",
  "Rep1_Label",
  "Rep2_Label",
  "Rep1_Mark",
  "Rep2_Mark",
  "Rep1_Color",
  "Rep2_Color",
  "Why",
] as const;

/** Compute column widths from header + data (capped at 60 chars). */
function autoColWidths(
  data: Record<string, unknown>[],
  cols: readonly string[],
): { wch: number }[] {
  return cols.map((col) => {
    const max = data.reduce((w, row) => {
      const v = row[col];
      const len = v == null ? 0 : String(v).length;
      return Math.max(w, len);
    }, col.length);
    return { wch: Math.min(max, 60) };
  });
}

/**
 * Export screening results to an .xlsx file and trigger a browser download.
 * Uses the full (untruncated) peak list — call with the ScreeningResult from screenFile().
 */
export async function exportToXlsx(
  result: ScreeningResult,
  params: ScreeningParams,
  filename: string,
): Promise<void> {
  const XLSX = await import("xlsx");

  const stem = filename.replace(/\.[^.]+$/, "");

  // ── Sheet 1: Screened Peaks ──────────────────────────────────────────────
  const peakRows = result.allPeaks.map((p) => {
    const row: Record<string, unknown> = {};
    for (const col of PEAK_COLUMNS) {
      if (col === "Why") {
        row[col] = p[col] != null ? JSON.stringify(p[col]) : "";
      } else {
        row[col] = p[col] ?? null;
      }
    }
    return row;
  });

  const peaksWs = XLSX.utils.json_to_sheet(peakRows, {
    header: [...PEAK_COLUMNS],
  });
  peaksWs["!cols"] = autoColWidths(peakRows, PEAK_COLUMNS);
  peaksWs["!freeze"] = { xSplit: 0, ySplit: 1 }; // freeze header row

  // ── Sheet 2: Summary ─────────────────────────────────────────────────────
  const summaryWs = XLSX.utils.json_to_sheet(result.allSummary);
  if (result.allSummary.length > 0) {
    const sumCols = Object.keys(result.allSummary[0]) as string[];
    summaryWs["!cols"] = autoColWidths(
      result.allSummary as Record<string, unknown>[],
      sumCols as readonly string[],
    );
  }
  summaryWs["!freeze"] = { xSplit: 0, ySplit: 1 };

  // ── Sheet 3: Parameters ──────────────────────────────────────────────────
  const paramRows = Object.entries(params).map(([key, value]) => ({
    Parameter: key,
    Value: value,
  }));
  const paramsWs = XLSX.utils.json_to_sheet(paramRows);
  paramsWs["!cols"] = [{ wch: 28 }, { wch: 12 }];

  // ── Sheet 4: Raw Data ────────────────────────────────────────────────────
  const rawWs = XLSX.utils.json_to_sheet(result.rawRows);
  rawWs["!freeze"] = { xSplit: 0, ySplit: 1 };

  // ── Assemble workbook ────────────────────────────────────────────────────
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, peaksWs, "Screened Peaks");
  XLSX.utils.book_append_sheet(wb, summaryWs, "Summary");
  XLSX.utils.book_append_sheet(wb, paramsWs, "Parameters");
  XLSX.utils.book_append_sheet(wb, rawWs, "Raw Data");

  XLSX.writeFile(wb, `${stem}_screened.xlsx`);
}

// ─── Sample data generation ───────────────────────────────────────────────────

export async function generateSampleData(format: "xlsx" | "csv" = "xlsx"): Promise<void> {
  const XLSX = await import("xlsx");

  const basePeaks = [120.5, 157.9, 195.2, 238.8, 285.1, 324.9];
  const sampleData: Record<string, unknown>[] = [];
  const surrogateData: Record<string, unknown>[] = [];

  const rtBase = 0.5;
  for (let i = 0; i < 15; i++) {
    const rt = rtBase + i * 0.15 + Math.random() * 0.05;
    const basePeak = basePeaks[i % basePeaks.length] + Math.random() * 2 - 1;
    const area = 500000 + Math.random() * 2000000;

    const polarity = Math.random() > 0.5 ? "Positive" : "Negative";
    const polaritySuffix = polarity === "Negative" ? "_neg" : "";

    for (let rep = 1; rep <= 3; rep++) {
      sampleData.push({
        RT: Number(rt.toFixed(3)),
        "Base Peak": Number(basePeak.toFixed(1)),
        Area: Math.round(area * (0.9 + Math.random() * 0.2)),
        Polarity: polarity,
        File: `${i + 1}_sample${polaritySuffix}_${rep}.d`,
        Label: `Cpd ${i + 1}: ${rt.toFixed(3)} ${basePeak.toFixed(1)}`,
      });
    }
  }

  const blankData: Record<string, unknown>[] = [];
  for (let i = 0; i < 8; i++) {
    const rt = rtBase + i * 0.2 + Math.random() * 0.05;
    const basePeak = basePeaks[i % basePeaks.length] + Math.random() * 2 - 1;
    const area = 100000 + Math.random() * 300000;

    const polarity = Math.random() > 0.5 ? "Positive" : "Negative";
    const polaritySuffix = polarity === "Negative" ? "_neg" : "";

    blankData.push({
      RT: Number(rt.toFixed(3)),
      "Base Peak": Number(basePeak.toFixed(1)),
      Area: Math.round(area),
      Polarity: polarity,
      File: `blank${polaritySuffix}_${i + 1}.d`,
      Label: `Blank ${i + 1}: ${rt.toFixed(3)} ${basePeak.toFixed(1)}`,
    });
  }

  const surrogateSpecs = [
    { name: "d8-Naphthalene", rt: 1.18, mz: 136.1, area: 155000, polarity: "Positive" },
    { name: "d10-Phenanthrene", rt: 2.42, mz: 188.1, area: 182000, polarity: "Positive" },
    { name: "d4-Benzoic acid", rt: 3.36, mz: 125.0, area: 131000, polarity: "Negative" },
  ];
  for (const [index, surrogate] of surrogateSpecs.entries()) {
    const polaritySuffix = surrogate.polarity === "Negative" ? "_neg" : "";
    for (let rep = 1; rep <= 2; rep++) {
      surrogateData.push({
        RT: Number((surrogate.rt + (rep - 1) * 0.012).toFixed(3)),
        "Base Peak": Number((surrogate.mz + (rep - 1) * 0.1).toFixed(1)),
        Area: Math.round(surrogate.area * (0.94 + Math.random() * 0.1)),
        Polarity: surrogate.polarity,
        File: `surrogate_${index + 1}${polaritySuffix}_${rep}.d`,
        Label: `${surrogate.name}: ${surrogate.rt.toFixed(3)} ${surrogate.mz.toFixed(1)}`,
        operator_mark: "surrogate",
        operator_color: null,
      });
    }
  }

  const allData = [...sampleData, ...blankData, ...surrogateData];

  if (format === "csv") {
    const ws = XLSX.utils.json_to_sheet(allData);
    const csv = XLSX.utils.sheet_to_csv(ws);
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "sample_lcms_data.csv";
    link.click();
  } else {
    const ws = XLSX.utils.json_to_sheet(allData);
    const wb = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(wb, ws, "Data");
    XLSX.writeFile(wb, "sample_lcms_data.xlsx");
  }
}

// ─── Server-mode helpers ──────────────────────────────────────────────────────

const SERVER_MODE_KEY = "serverMode";

export function isServerMode(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(SERVER_MODE_KEY) === "true";
}

export function setServerMode(enabled: boolean): void {
  localStorage.setItem(SERVER_MODE_KEY, enabled ? "true" : "false");
}
