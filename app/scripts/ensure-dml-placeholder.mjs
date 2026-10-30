// tauri-build validates every path in `bundle.resources` BEFORE compilation,
// but `resources/DirectML.dll` only exists after the first build (the ort crate
// copies it beside the target executable) and `scripts/prepare-bundle.mjs`
// copies it into `resources/` only after that. Fresh checkouts and CI runners
// would fail resource validation on the very first release build.
//
// This runs before `tauri build` (npm run tauri:release) and guarantees the
// file exists, so compile resource validation passes. prepare-bundle.mjs then
// replaces the placeholder with the real DLL from the build output before NSIS
// packaging, so the installer always carries genuine bytes.

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const tauriRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "src-tauri");
const placeholder = join(tauriRoot, "resources", "DirectML.dll");

if (!existsSync(placeholder)) {
  mkdirSync(dirname(placeholder), { recursive: true });
  writeFileSync(placeholder, Buffer.alloc(0));
  console.log("[draku] placeholder written: src-tauri/resources/DirectML.dll");
} else {
  console.log("[draku] DirectML.dll already present in resources/");
}
