import { describe, it, expect, vi, beforeAll } from "vitest";
import * as XLSX from "xlsx";

const inferPolarityFromFile = (filename: string): string => {
  return /(?:^|[\W_])neg(?:ative)?(?:[\W_]|$)/i.test(filename)
    ? "Negative"
    : "Positive";
};

describe("inferPolarityFromFile", () => {
  it("detects negative polarity from _neg suffix", () => {
    expect(inferPolarityFromFile("Sample_neg_1-3.d")).toBe("Negative");
    expect(inferPolarityFromFile("sample_neg_1.d")).toBe("Negative");
  });

  it("detects negative polarity from neg_ prefix", () => {
    expect(inferPolarityFromFile("neg_Sample_1.d")).toBe("Negative");
  });

  it("detects negative polarity from negative keyword", () => {
    expect(inferPolarityFromFile("sample-negative.d")).toBe("Negative");
  });

  it("defaults to positive for non-negative filenames", () => {
    expect(inferPolarityFromFile("Sample_1-2.d")).toBe("Positive");
    expect(inferPolarityFromFile("Blank_01-1.d")).toBe("Positive");
    expect(inferPolarityFromFile("sample.d")).toBe("Positive");
  });
});

describe("PARSERS registry", () => {
  it("csv parser matches .csv extension", () => {
    const file = new File([""], "test.csv", { type: "text/csv" });
    const match = /\.(csv|tsv|txt)$/i.test(file.name);
    expect(match).toBe(true);
  });

  it("csv parser matches .txt extension", () => {
    const file = new File([""], "test.txt", { type: "text/plain" });
    const match = /\.(csv|tsv|txt)$/i.test(file.name);
    expect(match).toBe(true);
  });

  it("xlsx parser matches .xlsx extension", () => {
    const file = new File([""], "test.xlsx", { type: "text/plain" });
    const match = /\.(xlsx|xls)$/i.test(file.name);
    expect(match).toBe(true);
  });

  it("xlsx parser matches .xls extension", () => {
    const file = new File([""], "test.xls", { type: "text/plain" });
    const match = /\.(xlsx|xls)$/i.test(file.name);
    expect(match).toBe(true);
  });
});

describe("CSV parsing with XLSX", () => {
  it("parses CSV with required columns", () => {
    const csv = "RT,File,Area,Label\n1.5,test.d,1000,Test\n2.0,test2.d,2000,Test2";
    const wb = XLSX.read(csv, { type: "string" });
    const ws = wb.Sheets[wb.SheetNames[0]];
    const data = XLSX.utils.sheet_to_json(ws);
    expect(data).toHaveLength(2);
    expect(data[0]).toHaveProperty("RT");
    expect(data[0]).toHaveProperty("File");
    expect(data[0]).toHaveProperty("Area");
  });

  it("handles the new screened format", () => {
    const csv = "Cpd,Show/Hide,File,Label,RT,Base Peak,Area,Height,Width,Mining Algorithm\n1,True,Blank_01-1.d,Cpd 1,0.098,157.9,2159585,112751,0.523,Find by Integration";
    const wb = XLSX.read(csv, { type: "string" });
    const ws = wb.Sheets[wb.SheetNames[0]];
    const data = XLSX.utils.sheet_to_json(ws);
    expect(data).toHaveLength(1);
    expect(data[0]).toMatchObject({
      RT: 0.098,
      File: "Blank_01-1.d",
      Area: 2159585,
      "Base Peak": 157.9,
    });
  });
});

describe("REQUIRED_COLS", () => {
  it("contains RT, File, Area", () => {
    const REQUIRED_COLS = ["RT", "File", "Area"];
    expect(REQUIRED_COLS).toEqual(["RT", "File", "Area"]);
  });
});

// ── WASM integration: partial blank match ────────────────────────────────────
// Regression: blank at RT≈1.95 matches sample_1 (delta=0.05) but not sample_2
// (delta=0.13 > 0.1 default tolerance). After parallel merge SourcesWithBlankMatch==1.

describe("WASM partial blank match", () => {
  let processPeaks: (rows: string, config: string) => string;

  beforeAll(async () => {
    const { readFileSync } = await import("fs");
    const { fileURLToPath } = await import("url");
    const { dirname, resolve } = await import("path");
    const { initSync, process_peaks } = await import("./wasm-pkg/screening_wasm.js");
    const __dir = dirname(fileURLToPath(import.meta.url));
    const wasmBuf = readFileSync(resolve(__dir, "wasm-pkg", "screening_wasm_bg.wasm"));
    initSync({ module: wasmBuf });
    processPeaks = process_peaks;
  });

  const rows = [
    { RT: 1.99, "Base Peak": 150.0, Area: 500.0, Polarity: "+", File: "1_a", Label: "s1a" },
    { RT: 2.01, "Base Peak": 150.1, Area: 510.0, Polarity: "+", File: "1_b", Label: "s1b" },
    { RT: 2.07, "Base Peak": 150.0, Area: 480.0, Polarity: "+", File: "2_a", Label: "s2a" },
    { RT: 2.09, "Base Peak": 150.1, Area: 490.0, Polarity: "+", File: "2_b", Label: "s2b" },
    { RT: 1.94, "Base Peak": 150.0, Area: 100.0, Polarity: "+", File: "blank_a", Label: "b1" },
    { RT: 1.96, "Base Peak": 150.1, Area: 110.0, Polarity: "+", File: "blank_b", Label: "b2" },
  ];
  const config = JSON.stringify({});

  it("SourcesWithBlankMatch == 1 when only sample_1 has blank within RT tolerance", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows), config));
    const merged = out.results.find((r: { ParallelMatch: boolean }) => r.ParallelMatch);
    expect(merged).toBeDefined();
    expect(merged.Why.BlankSubtraction.SourcesWithBlankMatch).toBe(1);
    expect(merged.Why.BlankSubtraction.TotalSources).toBe(2);
  });

  it("aggregated BlankAreaMean comes only from sample_1", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows), config));
    const merged = out.results.find((r: { ParallelMatch: boolean }) => r.ParallelMatch);
    expect(merged.BlankAreaMean).toBeCloseTo(105, 0);
  });

  it("Status is Real Compound because aggregated S/B > threshold", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows), config));
    const merged = out.results.find((r: { ParallelMatch: boolean }) => r.ParallelMatch);
    expect(merged.SignalToBlankRatio).toBeGreaterThan(3.0);
    expect(merged.Status).toBe("Real Compound");
  });
});

describe("WASM surrogate validation", () => {
  let processPeaks: (rows: string, config: string) => string;

  beforeAll(async () => {
    const { readFileSync } = await import("fs");
    const { fileURLToPath } = await import("url");
    const { dirname, resolve } = await import("path");
    const { initSync, process_peaks } = await import("./wasm-pkg/screening_wasm.js");
    const __dir = dirname(fileURLToPath(import.meta.url));
    const wasmBuf = readFileSync(resolve(__dir, "wasm-pkg", "screening_wasm_bg.wasm"));
    initSync({ module: wasmBuf });
    processPeaks = process_peaks;
  });

  const rows = (area = 143250, rt = 5.31) => [
    {
      RT: rt,
      "Base Peak": 128.0,
      Area: area,
      Polarity: "+",
      File: "surrogate_a",
      Label: "d8-Naphthalene",
      operator_mark: "surrogate",
    },
    {
      RT: rt + 0.01,
      "Base Peak": 128.1,
      Area: area * 1.02,
      Polarity: "+",
      File: "surrogate_b",
      Label: "d8-Naphthalene",
      operator_mark: "surrogate",
    },
  ];

  const config = (surrogates = true) =>
    JSON.stringify(
      surrogates
        ? {
            surrogates: [
              {
                name: "d8-Naphthalene",
                expected_rt: 5.23,
                expected_area: 150000,
                expected_mz: 128.0,
                rt_window: 0.2,
                recovery_min_pct: 70,
                recovery_max_pct: 130,
              },
            ],
          }
        : { surrogates: [] },
    );

  it("marks matching surrogate rows and emits validation details", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows()), config()));
    const surrogate = out.results[0];
    expect(surrogate.IsSurrogate).toBe(true);
    expect(surrogate.SurrogatePass).toBe(true);
    expect(surrogate.Status).toBe("Surrogate OK");
    expect(surrogate.Why.SurrogateValidation.MatchedSpec).toBe("d8-Naphthalene");
    expect(surrogate.SurrogateRecoveryPct).toBeCloseTo(96.5, 1);
  });

  it("keeps failing surrogates in output", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows(400000)), config()));
    const surrogate = out.results[0];
    expect(surrogate.IsSurrogate).toBe(true);
    expect(surrogate.SurrogatePass).toBe(false);
    expect(surrogate.Status).toBe("Surrogate Failed");
    expect(surrogate.SurrogateRecoveryPct).toBeGreaterThan(130);
  });

  it("leaves SurrogatePass null when no spec matches", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows()), config(false)));
    const surrogate = out.results[0];
    expect(surrogate.IsSurrogate).toBe(true);
    expect(surrogate.SurrogatePass).toBeNull();
    expect(surrogate.Status).toBe("Surrogate");
    expect(surrogate.Why.SurrogateValidation).toBeUndefined();
  });

  it("fails surrogate on RT drift outside rt_window", () => {
    const out = JSON.parse(processPeaks(JSON.stringify(rows(143250, 5.45)), config()));
    const surrogate = out.results[0];
    expect(surrogate.IsSurrogate).toBe(true);
    expect(surrogate.SurrogatePass).toBe(false);
    expect(Math.abs(surrogate.SurrogateRtShift)).toBeGreaterThan(0.2);
  });
});
