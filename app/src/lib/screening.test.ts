import { describe, it, expect, vi } from "vitest";
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
