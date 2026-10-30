// Fetch the exact EmbeddingGemma-300M Q4 files used by Draku into
// src-tauri/resources/models/embeddinggemma, so `tauri build` can bundle a
// fully offline first run.
//
// The model is ~219 MB and is not committed to git. Run this once in a source
// checkout:
//
//   cd app
//   node scripts/fetch-model.mjs
//
// Downloads are resumable: an interrupted file continues from its `.part`.
// Nothing here runs in the shipped app — the finished files are simply bundled.

import { createWriteStream, existsSync, mkdirSync, renameSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

const REPO = "onnx-community/embeddinggemma-300m-ONNX";
const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(appRoot, "src-tauri", "resources", "models", "embeddinggemma");

// ONNX weights live in the repo's onnx/ subfolder; tokenizer/config stay at
// the root. Sizes assert the exact bytes we bundle.
const files = [
  { name: "model_q4.onnx", path: "onnx/model_q4.onnx", size: 519_322 },
  { name: "model_q4.onnx_data", path: "onnx/model_q4.onnx_data", size: 196_725_760 },
  { name: "tokenizer.json", size: 20_323_312 },
  { name: "config.json", size: 1_765 },
  { name: "special_tokens_map.json", size: 662 },
  { name: "tokenizer_config.json", size: 1_156_830 },
];

const fmt = (n) => `${(n / 1024 / 1024).toFixed(1)} MB`;

async function fetchFile({ name, path, size }) {
  const dest = join(outDir, name);
  if (existsSync(dest) && statSync(dest).size === size) {
    console.log(`[draku] ${name} already present (${fmt(size)})`);
    return;
  }

  // Root paths broke in 2026 when the upstream repo moved ONNX files into onnx/.
  const url = `https://huggingface.co/${REPO}/resolve/main/${path ?? name}`;
  const part = `${dest}.part`;

  for (let attempt = 1; attempt <= 8; attempt++) {
    let start = existsSync(part) ? statSync(part).size : 0;
    if (start >= size) {
      renameSync(part, dest);
      return;
    }
    const headers = start > 0 ? { Range: `bytes=${start}-` } : {};
    try {
      const res = await fetch(url, { headers, redirect: "follow" });
      if (res.status !== 200 && res.status !== 206) {
        throw new Error(`HTTP ${res.status}`);
      }
      // A server that ignores Range sends 200; restart rather than append.
      const append = start > 0 && res.status === 206;
      if (!append) start = 0;
      console.log(
        `[draku] ${name}: ${append ? `resuming at ${fmt(start)}` : "downloading"} → ${fmt(size)}`,
      );
      await pipeline(
        Readable.fromWeb(res.body),
        createWriteStream(part, { flags: append ? "a" : "w" }),
      );
      if (statSync(part).size !== size) {
        throw new Error(`size ${statSync(part).size}, expected ${size}`);
      }
      renameSync(part, dest);
      console.log(`[draku] ${name}: done`);
      return;
    } catch (e) {
      const have = existsSync(part) ? statSync(part).size : 0;
      console.error(`[draku] ${name}: attempt ${attempt} failed (${e.message}); kept ${fmt(have)}`);
      await new Promise((r) => setTimeout(r, Math.min(30_000, 2_000 * attempt)));
    }
  }
  throw new Error(`${name}: download failed after 8 attempts`);
}

mkdirSync(outDir, { recursive: true });
for (const file of files) {
  await fetchFile(file);
}
console.log(`[draku] model ready in ${outDir}`);
