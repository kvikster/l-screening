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

// ─── Excel parsing ────────────────────────────────────────────────────────────

const REQUIRED_COLS = ["RT", "Polarity", "File", "Area"];

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
 * Select the sheet that has the most required column names.
 * Raises if fewer than 3 required columns are found.
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

  if (bestScore < 3) {
    throw new Error(t("noValidSheet", { columns: REQUIRED_COLS.join(", ") }));
  }
  return bestName;
}

/**
 * Parse an Excel file buffer into an array of row objects.
 * Reads cell fill colors from the first column to assign operator_mark.
 */
function parseExcel(buffer: ArrayBuffer): ParsedRow[] {
  const wb = XLSX.read(buffer, { type: "array", cellStyles: true });
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

  const rows: ParsedRow[] = [];

  for (let r = range.s.r + 1; r <= range.e.r; r++) {
    // Extract cell fill color from the first data column (col = range.s.c).
    const firstCell = ws[XLSX.utils.encode_cell({ r, c: range.s.c })];
    const fillRgb =
      firstCell?.s?.fgColor?.rgb ?? firstCell?.s?.bgColor?.rgb ?? undefined;
    const { operator_color, operator_mark } = resolveOperatorMark(fillRgb);

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

    const polarity = polCell?.v != null ? String(polCell.v).trim() : "";
    const file = fileCell?.v != null ? String(fileCell.v).trim() : "";

    const labelCell =
      labelIdx !== -1
        ? ws[XLSX.utils.encode_cell({ r, c: labelIdx })]
        : undefined;
    const label = labelCell?.v != null ? String(labelCell.v) : null;

    rows.push({
      RT: rt,
      "Base Peak": mz != null && isFinite(mz) ? mz : null,
      Area: area,
      Polarity: polarity,
      File: file,
      Label: label,
      operator_color,
      operator_mark,
    });
  }

  return rows;
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
  const buffer = await file.arrayBuffer();
  const rawRows = parseExcel(buffer);

  if (rawRows.length === 0) {
    throw new Error(t("noValidRows"));
  }

  const wasm = await getWasm();

  const resultJson = wasm.process_peaks(
    JSON.stringify(rawRows),
    JSON.stringify(params),
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
  const displayLimit = 100;
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

// ─── Server-mode helpers ──────────────────────────────────────────────────────

const SERVER_MODE_KEY = "serverMode";

export function isServerMode(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(SERVER_MODE_KEY) === "true";
}

export function setServerMode(enabled: boolean): void {
  localStorage.setItem(SERVER_MODE_KEY, enabled ? "true" : "false");
}
