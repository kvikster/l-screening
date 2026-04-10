/**
 * Run the Rust WASM screening algorithm using the same input data as the Python version.
 */
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT = join(__dirname, "..");

// Load the WASM module using dynamic import
const wasmModulePath = join(ROOT, "app/src/lib/wasm-pkg/screening_wasm.js");
const wasmBinaryPath = join(ROOT, "app/src/lib/wasm-pkg/screening_wasm_bg.wasm");
const inputPath = join(ROOT, "tests/_wasm_input.json");
const outputPath = join(ROOT, "tests/_wasm_output.json");

async function main() {
  // Read the WASM binary
  const wasmBinary = readFileSync(wasmBinaryPath);

  // Import the WASM module
  const mod = await import(`file://${wasmModulePath}`);

  // Initialize with the binary buffer
  mod.initSync({ module: wasmBinary });

  // Read the input data (prepared by run_python.py)
  const input = JSON.parse(readFileSync(inputPath, "utf-8"));
  const rowsJson = JSON.stringify(input.rows);
  const configJson = JSON.stringify(input.config);

  console.log(`WASM input: ${input.rows.length} rows`);

  // Run the WASM algorithm
  const resultStr = mod.process_peaks(rowsJson, configJson);
  const result = JSON.parse(resultStr);

  if (result.error) {
    console.error("WASM error:", result.error);
    process.exit(1);
  }

  // Save WASM output
  writeFileSync(outputPath, JSON.stringify(result, null, 2), "utf-8");
  console.log(`WASM output saved to: ${outputPath}`);
  console.log(`WASM results: ${(result.results || []).length} peaks`);
  console.log(`WASM summary: ${(result.summary || []).length} rows`);
}

main().catch((err) => {
  console.error("Failed:", err);
  process.exit(1);
});
