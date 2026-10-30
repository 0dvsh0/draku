// DirectML.dll is a static import of the ONNX Runtime build used by
// paddle-ocr-rs and EmbeddingGemma. `ort` copies it next to the freshly built
// executable; the Tauri bundler needs it in src-tauri/resources so it can be
// placed beside the installed executable too.
//
// Runs via tauri.conf.json > build > beforeBundleCommand, after cargo build and
// before NSIS packaging. No network, no user files touched.

import { copyFileSync, existsSync, mkdirSync, renameSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const tauriRoot = join(appRoot, "src-tauri");
const outDir = join(tauriRoot, "resources");
const out = join(outDir, "DirectML.dll");

const candidates = [
  join(tauriRoot, "target", "release", "DirectML.dll"),
  join(tauriRoot, "target", "debug", "DirectML.dll"),
];

const source = candidates.find((p) => existsSync(p));
const staged = existsSync(out) ? statSync(out).size : 0;
if (!source && staged > 10_000_000) {
  // CI stages a pinned DirectML.dll into resources/ before building because
  // ort copies its own only beside targets it has already built.
  console.log("[draku] using the DirectML.dll staged in resources/");
  process.exit(0);
}
if (!source) {
  if (process.platform === "win32") {
    console.error(
      "[draku] DirectML.dll was not found next to the build output. " +
        "Build the Rust app once so ort can copy it, then bundle again.",
    );
    process.exit(1);
  }
  console.log("[draku] DirectML is Windows-only; nothing to copy.");
  process.exit(0);
}

mkdirSync(outDir, { recursive: true });
copyFileSync(source, out);
console.log(`[draku] bundling ${source} -> ${out}`);

// The embedding model is intentionally not committed to git (~219 MB). A
// release build must carry it, or first-run semantic search needs the network
// and the "nothing leaves the machine" promise gets weaker.
const modelDir = join(tauriRoot, "resources", "models", "embeddinggemma");
const modelFiles = [
  "model_q4.onnx",
  "model_q4.onnx_data",
  "tokenizer.json",
  "config.json",
  "special_tokens_map.json",
  "tokenizer_config.json",
];
const missing = modelFiles.filter((f) => !existsSync(join(modelDir, f)));
if (missing.length > 0) {
  console.warn(
    `[draku] WARNING: bundled embedding model is incomplete (${missing.join(", ")}).\n` +
      "        Run `node scripts/fetch-model.mjs` before building a release.\n" +
      "        This build will fall back to a first-run model download.",
  );
} else {
  console.log(`[draku] bundling EmbeddingGemma from ${modelDir}`);
}

