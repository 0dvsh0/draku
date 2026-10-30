// Smoke test for Draku NSIS installer.
// Silently installs the built package into a temporary directory and verifies
// that draku.exe, DirectML.dll, the PaddleOCR models, and the bundled
// EmbeddingGemma model files are present and uncorrupted.

import { execFileSync } from "node:child_process";
import { existsSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const tauriRoot = join(appRoot, "src-tauri");

const installerCandidates = [
  join(tauriRoot, "target", "release", "bundle", "nsis", "Draku_0.1.0_x64-setup.exe"),
  join(tauriRoot, "target", "debug", "bundle", "nsis", "Draku_0.1.0_x64-setup.exe"),
];

const installer = installerCandidates.find((p) => existsSync(p));
if (!installer) {
  console.error(
    "[smoke-test] ERROR: NSIS installer not found. Run `npm run tauri build` first.\n" +
      `Checked:\n  ${installerCandidates.join("\n  ")}`,
  );
  process.exit(1);
}

const testDir = join(tmpdir(), `draku-smoke-${Date.now()}`);
console.log(`[smoke-test] Installer found: ${installer}`);
console.log(`[smoke-test] Installing silently to temporary dir: ${testDir} ...`);

try {
  // NSIS silent install flag: /S /D=<destination>
  // Note: /D must be the last parameter and not enclosed in quotes.
  execFileSync(installer, ["/S", `/D=${testDir}`], { stdio: "inherit" });
} catch (e) {
  console.error(`[smoke-test] Installer failed with exit code ${e.status}: ${e.message}`);
  process.exit(1);
}

const requiredFiles = [
  // Executable and DirectML runtime placed side-by-side
  { path: join(testDir, "draku.exe"), minSize: 1_000_000 },
  { path: join(testDir, "DirectML.dll"), minSize: 10_000_000 },

  // PaddleOCR models in assets/ocr
  { path: join(testDir, "assets", "ocr", "cls.onnx"), minSize: 500_000 },
  { path: join(testDir, "assets", "ocr", "det.onnx"), minSize: 2_000_000 },
  { path: join(testDir, "assets", "ocr", "dict.txt"), minSize: 1_000 },
  { path: join(testDir, "assets", "ocr", "rec.onnx"), minSize: 7_000_000 },

  // Bundled EmbeddingGemma in models/embeddinggemma
  { path: join(testDir, "models", "embeddinggemma", "model_q4.onnx"), minSize: 500_000 },
  { path: join(testDir, "models", "embeddinggemma", "model_q4.onnx_data"), minSize: 190_000_000 },
  { path: join(testDir, "models", "embeddinggemma", "tokenizer.json"), minSize: 19_000_000 },
  { path: join(testDir, "models", "embeddinggemma", "config.json"), minSize: 1_000 },
  { path: join(testDir, "models", "embeddinggemma", "special_tokens_map.json"), minSize: 500 },
  { path: join(testDir, "models", "embeddinggemma", "tokenizer_config.json"), minSize: 1_000_000 },
];

let ok = true;
for (const file of requiredFiles) {
  if (!existsSync(file.path)) {
    console.error(`[smoke-test] MISSING: ${file.path}`);
    ok = false;
    continue;
  }
  const size = statSync(file.path).size;
  if (size < file.minSize) {
    console.error(`[smoke-test] TRUNCATED: ${file.path} (size ${size} < min ${file.minSize})`);
    ok = false;
    continue;
  }
  console.log(`[smoke-test] OK: ${file.path.replace(testDir, ".")} (${(size / 1024 / 1024).toFixed(2)} MB)`);
}

// Clean up test installation
try {
  rmSync(testDir, { recursive: true, force: true });
  console.log(`[smoke-test] Cleaned up temporary test directory.`);
} catch (e) {
  console.warn(`[smoke-test] Note: Could not fully clean ${testDir}: ${e.message}`);
}

if (!ok) {
  console.error("[smoke-test] FAILED: one or more required packaged files are missing or incomplete.");
  process.exit(1);
}

console.log("[smoke-test] SUCCESS: NSIS package verified. DirectML, OCR models, and EmbeddingGemma all present.");
