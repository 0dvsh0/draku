# Draku

**Your files, finally understood.**

Draku is a free, open-source, on-device AI file manager for Windows. It reads
every screenshot, photo and PDF **on your machine** — it finds what you need,
warns you before a date passes, hides what should stay private, and lets go of
what is dead. Nothing leaves your machine.

Built for **Hacktoberfest 2026 · Week 4 — Ghost in the Machine**.

- **Landing page (live):** https://0dvsh0.github.io/draku/
- **Download the installer:** https://github.com/0dvsh0/draku/releases/latest/download/Draku_0.1.0_x64-setup.exe
- **Read the source:** https://github.com/0dvsh0/draku

```
draku/
├── CONTEXT.md     ← project context: plan, stack, tokens, status (read first)
├── site/          ← the landing page (static HTML/CSS/JS + WebGL embers)
├── app/           ← the product: Tauri 2 desktop app (Rust + React/TypeScript)
│   ├── src/           One Screen UI, mascot design tokens (tokens.css)
│   ├── scripts/       bundle prep + one-time model fetch for source builds
│   └── src-tauri/     scanner · reader · OCR · embeddings · actions
└── site/About.md  ← the product write-up all copy comes from
```

## What it does

- **Find** — local OCR reads the pixels, a rule-based reader names the file, and
  **EmbeddingGemma-300M** embeds the reading. Ask in plain language; exact words
  still work too.
- **Remember** — dates are extracted with their surrounding words, so “Warranty
  ends 2026-11-04” becomes a deadline while “Purchase date 2026-10-04” does not.
  A future date becomes a real `.ics` calendar file when you tap.
- **Protect** — ID numbers, passwords, card numbers and GPS are found locally.
  “Make a safe copy” burns the blur into new pixels and strips location metadata;
  the original is untouched.
- **Let go** — expired, duplicate and one-time files get a reason and a score
  you can read. “Set aside” moves them to a Rest folder. There is no delete
  command anywhere in the codebase.

## The five promises

1. **No upload.** The shipped installer bundles PaddleOCR and EmbeddingGemma, so
   the app makes no network request of its own. Pull the network cable and every
   feature still works.
2. **It never deletes your files.** “Set aside” moves a file to a Rest folder;
   every generated artifact has an Undo.
3. **Suggest → confirm.** The first tap arms an action, the second performs it; a
   generated artifact then offers Undo. Nothing on disk changes on its own.
4. **Free and open source.** MIT (see `LICENSE`); model and library terms are in
   `THIRD_PARTY_NOTICES.md`.
5. **No settings screen.** One screen, three cards, one question bar.

## Stack

Tauri 2 · Rust · React 19 + TypeScript · SQLite (WAL, local) · PaddleOCR ONNX
(PP-OCR det/cls/rec) · EmbeddingGemma-300M Q4 ONNX via `fastembed` + ONNX
Runtime · a small transparent scorer for Let go.

## Run the landing page

Open `site/index.html` in a browser, serve it locally (`python -m http.server`),
or just use the deployed copy at **https://0dvsh0.github.io/draku/** (deployed
automatically from `site/` by `.github/workflows/pages.yml`).

## Run the app

Rust 1.85+, Node 20+, Windows 10/11 with WebView2.

```pwsh
cd app
npm install

# One-time: fetch the ~219 MB embedding model into the bundle resources.
# Source builds work without this (they download on first scan), but a release
# build should bundle it so the very first scan is offline.
node scripts/fetch-model.mjs

npm run tauri dev
```

A light build without OCR (`--no-default-features`) is available for machines
that cannot run PaddleOCR. Semantic search, dates, duplicates and actions still
work; Protect reports what it can from filenames and metadata.

## Build and verify the installer

```pwsh
cd app
node scripts/fetch-model.mjs
npm run tauri:release
npm run smoke:installer
```

This produces an NSIS installer in `app/src-tauri/target/release/bundle/nsis/`.
The release build merges `src-tauri/tauri.release.conf.json` so the generated
`resources/DirectML.dll` is bundled beside the executable; `scripts/prepare-bundle.mjs`
copies it into place before packaging, and bundles the EmbeddingGemma model files.
`scripts/smoke-test-installer.mjs` performs a silent test installation verifying
all binaries, OCR models, and embedding weights.

### Release artifacts

Tagged `v0.1.0` on [GitHub Releases](https://github.com/0dvsh0/draku/releases) —
the installer there is what the landing page's download button serves.

- **Windows NSIS Installer**: `Draku_0.1.0_x64-setup.exe` (182.72 MiB)
  - `SHA256: 19473E9A5AEC88BDE82DF50CD697E7A9450A0B0EA88B49E65B57F89843E52791`
- **Release Executable**: `draku.exe` (45.16 MiB)
  - `SHA256: 6EA45C1D852DE604ADF390D220D0F5E4DA5840DD736736AB6DDC50F340CD95C8`

## Tests

```pwsh
cd app
npm test                          # Vitest (human formatting utilities)
npm run check                     # TypeScript (strict checking)
npm run smoke:installer           # Silent installer verification
cd src-tauri
cargo test --features ocr          # 42 tests: full OCR & embedding pipeline
cargo test --no-default-features   # 41 tests: light build without PaddleOCR
```

## Challenge submission & demo

- **DEV Challenge Post**: [`DEV_POST.md`](DEV_POST.md)
- **60-Second Demo Walkthrough**: [`DEMO.md`](DEMO.md)

## Models and licences

- **EmbeddingGemma-300M** — Google DeepMind, open weights under the
  [Gemma Terms of Use](https://ai.google.dev/gemma/terms). ONNX conversion by
  `onnx-community`.
- **PaddleOCR PP-OCR models** — Apache-2.0.
- **PaddleOCR Rust binding** — Apache-2.0, vendored with a two-line ONNX Runtime
  API patch; see `app/src-tauri/vendor/paddle-ocr-rs/README.draku.md`.
- **fastembed / ONNX Runtime / DirectML** — Apache-2.0 / MIT / Microsoft
  redistributable.

Full details: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

See `CONTEXT.md` for architecture, design tokens, and current status.
