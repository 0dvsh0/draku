# Third-party notices

Draku itself is MIT-licensed (see `LICENSE`). It bundles or links the following
components. This file is a summary, not a replacement for the full licence texts
shipped by each project.

## Models

### EmbeddingGemma-300M

- **What:** text embedding model used by Find.
- **Source:** Google DeepMind, converted to ONNX by
  [`onnx-community/embeddinggemma-300m-ONNX`](https://huggingface.co/onnx-community/embeddinggemma-300m-ONNX).
- **Terms:** [Gemma Terms of Use](https://ai.google.dev/gemma/terms). The weights
  are open weights, not an OSI-approved open-source licence. The model runs
  locally; Draku does not train or redistribute a modified model.
- **Files bundled:** `model_q4.onnx`, `model_q4.onnx_data`, `tokenizer.json`,
  `config.json`, `special_tokens_map.json`, `tokenizer_config.json`.

### PaddleOCR PP-OCR models

- **What:** text detection, angle classification and recognition used by Protect.
- **Source:** PaddleOCR / PaddlePaddle.
- **Licence:** Apache-2.0.
- **Files bundled:** `assets/ocr/det.onnx`, `cls.onnx`, `rec.onnx`, `dict.txt`.

## Libraries and runtimes

| Component | Licence | Notes |
| --- | --- | --- |
| Tauri 2 | Apache-2.0 / MIT | Desktop shell |
| React, React DOM | MIT | UI |
| Rust crates (`serde`, `rusqlite`, `image`, `pdf-extract`, `regex`, `walkdir`, `chrono`, …) | MIT / Apache-2.0 | See `app/src-tauri/Cargo.lock` |
| `fastembed` | Apache-2.0 | EmbeddingGemma loader |
| ONNX Runtime | MIT | Statically linked inference runtime |
| DirectML | Microsoft redistributable | GPU acceleration on Windows; `DirectML.dll` is placed beside the executable |
| `paddle-ocr-rs` | Apache-2.0 | Vendored with an ONNX Runtime API patch; see `app/src-tauri/vendor/paddle-ocr-rs/README.draku.md` |
| Inter Tight, Instrument Serif, JetBrains Mono | SIL OFL 1.1 | Bundled through `@fontsource` |
| Vite, TypeScript, Vitest | MIT | Build and test tooling |

## Fonts and images

The landing page loads Inter Tight, Instrument Serif and JetBrains Mono from
Google Fonts. The desktop app ships the fonts locally through `@fontsource` and
does not make a font request at runtime.

The sample files used in screenshots and the demo video are synthetic. No real
user file is included in this repository.
