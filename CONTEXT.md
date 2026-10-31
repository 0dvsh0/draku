# Draku — Project Context

> **This file is the single source of truth for building Draku.** It is maintained and updated
> at every milestone. Read it before touching anything.

---

## 1. What Draku is

Draku is a **free, open-source, on-device AI file manager** for laptops.

> **"Your files, finally understood."**

A private AI that understands every screenshot, photo and PDF on your device. It
**Find**s what you need, **Remember**s dates before they pass, **Protect**s what should stay
private, and **Let go**s of what is dead. **Nothing ever leaves the machine.**

The core thesis (from `About.md`, the user-authored product writing — keep its voice, correct only
claims the code does not keep): **one reading, four outcomes.** Each file is read **once**,
producing a structured `reading.json`, and everything the product does is a view of that single
reading:

```json
{
  "name":     "2026-10-11 Dr Patel blood test prescription",
  "category": "medical / prescription",
  "dates":    [{ "refill_by": "2026-11-10" }],
  "private":  ["patient_id", "gps"],
  "dead":     0.04,
  "network":  null
}
```

### The five promises (non-negotiable product law)

1. **No upload.** Every file is read on-device. There is *no cloud mode to switch on*.
2. **No deleting.** Let-go files move to a **rest folder** the user can restore from. Only the user empties it.
3. **No guessing out loud.** Draku suggests, the user confirms. Never mutate anything without a tap.
4. **No account, subscription or ads.** Free and open source (MIT).
5. **No settings screen.** If a feature needs a setting, it is not finished.

One screen. Drop a folder → three cards + one "Ask anything" line. That is the whole product.

### The four outcomes

| Outcome | Before | After | Pipeline |
|---|---|---|---|
| **Find** | `IMG-20261011-WA0004.jpg` | `2026-10-11 Dr Patel blood test prescription.jpg`, found by typing "blood test" | reading → name/category/key text → **embedding** → semantic search |
| **Remember** | warranty screenshot, expiring | calendar reminder, one tap | reading extracts dates + expiry rules → confirmed → `.ics` |
| **Protect** | ID/password visible + GPS metadata | first digits blurred, metadata stripped | local OCR + EXIF → confirmed → safe copy |
| **Let go** | OTPs, dead tickets, dupes | rested in restore-able folder | signals (age, category, date passed, dup distance, blur, size) → **dead score** |

## 2. Tech stack (decided)

| Layer | Choice | Why |
|---|---|---|
| Shell | **Tauri 2** (Rust) | True native desktop, tiny binary, privacy is inspectable in Rust, no Electron runtime shipping to users who are being promised "no upload". |
| Backend | **Rust** (`src-tauri`) | The privacy promise must be *checkable*: anyone can read the exact code that touches files and see it never opens a socket. |
| Vision/LLM | **Local OCR + rule-based reader** (shipped); Gemma 4 vision is roadmap, not a claim | The reader is real today: PaddleOCR text, PDF text, filename heuristics, EXIF. A future Gemma 4 plug-in can enrich names/summaries without changing the promise. |
| Embeddings | **fastembed → `EmbeddingGemma300MQ4`** | The literal "embeddinggemma" from the copy. ONNX 4-bit, **bundled in the installer (~219 MB)** so the first scan is offline; source builds can fetch it once with `node scripts/fetch-model.mjs`. |
| OCR | **PaddleOCR ONNX** (default feature) | PP-OCR det/cls/rec models ship in `assets/ocr` and give exact pixel boxes for Protect. `--no-default-features` keeps a light build. |
| PDF | **pdf-extract** crate | Pure Rust; gives real text (dates, secrets) from PDFs with no model. |
| Images | `image` + `kamadak-exif` | Decode, perceptual hash (duplicates), **EXIF/GPS reading**, and stripped re-encode for safe copies. |
| Store | **SQLite (rusqlite, bundled)** | One local DB in the app dir, no daemons. |
| Frontend | **React 19 + TypeScript + Vite** | Small team velocity, typed IPC, instant HMR. |
| Design system | **tokens.css — the mascot palette from the landing page** | Exact same tokens; see §5. |
| Fonts | **@fontsource packages** (Inter Tight, Instrument Serif, JetBrains Mono) | The app must work with Wi-Fi off — no Google Fonts CDN. |
| Tests | Vitest (frontend) + `cargo check/test` | Sanity for scoring/date/embedding-batch logic. |

## 3. Repository layout

```
draku/
├── CONTEXT.md          ← this file (kept current)
├── README.md           ← repo overview (landing + app)
├── LICENSE             ← MIT
├── site/               ← the landing page (was repo root; moved here, untouched)
│   ├── index.html, css/, js/, images/, About.md
└── app/                ← the Tauri 2 desktop app   ← THE PRODUCT
    ├── package.json / vite.config.ts / tsconfig.json
    ├── index.html
    ├── src/                ← TypeScript frontend ("one screen, no settings")
    │   ├── styles/tokens.css, app.css
    │   ├── App.tsx, main.tsx
    │   └── components/  lib/
    └── src-tauri/          ← Rust core that touches every byte on disk
        ├── Cargo.toml, build.rs, tauri.conf.json, capabilities/
        └── src/
            ├── lib.rs        ← Tauri setup, state, IPC
            ├── model.rs      ← Reading + all IPC types (serde)
            ├── store.rs      ← SQLite schema + queries
            ├── scan.rs       ← folder walker, progress events
            ├── extract/      ← the READER: filename, EXIF, PDF text, OCR → Reading
            ├── score.rs      ← "Let go" dead-scoring (transparent weighted signals)
            ├── embed.rs      ← bundled EmbeddingGemma loader + cosine search
            ├── ics.rs        ← calendar files for Remember
            └── actions.rs    ← safe copy, set aside/restore (the only mutations)
```

## 4. Privacy architecture (must hold forever)

- The **packaged app makes no network request of its own**. EmbeddingGemma, PaddleOCR and the
  `DirectML.dll` runtime are bundled; there is no cloud mode, telemetry or update check.
- A **source checkout** without `resources/models/embeddinggemma` falls back to `fastembed`'s
  one-time download into `~/.cache/fastembed` (or `FASTEMBED_CACHE_DIR`). That is a developer
  convenience, not the shipped behaviour. `scripts/fetch-model.mjs` removes even that exception
  before a release build.
- Desktop build is feature-gated; CI runs the light build
  (`cargo test --no-default-features`) and the full OCR build (`cargo test --features ocr`).
- Everything the AI does is **suggest → confirm**. Raw file actions are only ever executed from
  an explicit button in the UI (promise #3). Deleting is impossible; there is no such command
  (promise #2).

## 5. Design tokens — "the mascot" (must match `site/css/style.css` exactly)

Draku, the mascot, is a **black fluffy cat with crimson bat wings and round white-ringed eyes
full of black flames**. Every token is lifted from him. Both `site/` and `app/` consume the
same values; `app/src/styles/tokens.css` is the app's canonical copy:

| Token | Value | Source on the cat |
|---|---|---|
| `--fur-0` | `#050405` | deepest creeping fur / death-black |
| `--fur-1` | `#080708` | fur + outlines (**pure base**) |
| `--fur-2` | `#0f0d0e` | raised surface |
| `--fur-3` | `#171415` | raised surface -2 |
| `--fur-4` | `#221e1f` | border surfaces |
| `--bone` | `#EAE6E6` | white eye rings (**primary text**) |
| `--bone-2` | `#c9c3c2` | dimmed bone |
| `--ash` | `#9D928F` | original grey backdrop (**muted text**) |
| `--ash-2` | `#6f6765` | deeper muted |
| `--ash-3` | `#3a3435` | faintest text |
| `--wing` | `#841E28` | the crimson wings (**primary action**) |
| `--wing-hi` | `#A42A36` | wing hover / raised crimson |
| `--wing-glow` | `#C2414E` | accent + glow (found / highlight) |
| `--wing-lo` | `#6C1620` | pressed crimson |
| `--iris` | `#7A1D25` | the iris |
| `--line` | `rgba(234,230,230,.08)` | hairline |
| `--line-2` | `rgba(234,230,230,.14)` | stronger hairline |
| `--f-sans` | `"Inter Tight"` | UI |
| `--f-serif` | `"Instrument Serif"` | emphasis, always *italic* |
| `--f-mono` | `"JetBrains Mono"` | labels, dots, code, statuses |
| `--r-s 10px` / `--r-m 18px` / `--r-l 28px` | radii | soft like fur |
| `--ease: cubic-bezier(.2,.8,.2,1)` | easing | calm, no bounce |

Voice of the UI: dark-only, calm, generous letter-spaced mono labels, serif-italic emphasis.
Motion respects `prefers-reduced-motion`.

## 6. Product surfacing — the One Screen (UX law)

Layout, exactly as the `About.md` "One screen, no settings" section:

1. **Empty state** = a drop zone + "point me at a folder" (native folder picker). Nothing else.
2. **Scanning** = quiet progress, per file, cancellable. "Reading your files. Nothing leaves this laptop."
3. **One screen** =
   - `Ask anything` search line (⌘K / Ctrl+K focuses it). Hint chips after first empty scan.
   - **Three cards**: ① *n* deadlines (→ *Add to calendar*), ② *n* exposed secrets
     (→ *Make a safe copy*), ③ *n* files ready to let go (→ *Set aside*).
   - Every action shows **one confirm step** in place, then a toast with **Undo** (rest is one tap).
   - Footer truth, always visible: *"Nothing changes on your device until you press a button."*
4. **Never**: no tabs, no settings, no sidebar, no accounts. If a view needs chrome, it is the wrong design.

## 7. Architecture & Philosophy

- **Theme & Motivation:** Files that outlive their usefulness linger as unindexed digital residue; Draku reads them, names them, protects secrets, and lets expired items rest rather than deleting them.
- **Engine:** **Gemma** through EmbeddingGemma-300M (open weights, local). PaddleOCR (Apache-2.0) is the local vision pipeline.
- **Rule:** Every claim in the documentation, README, landing page, and About.md must match what the shipped build actually does. Gemma 4 vision and TabPFN are roadmap, not shipped features.
- **Distribution:** Public repo is `0dvsh0/draku`; landing site deploys to `https://0dvsh0.github.io/draku/` via `.github/workflows/pages.yml`; release `v0.1.0` carries `Draku_0.1.0_x64-setup.exe` + `draku.exe`; all download/source links point there.

## 8. Roadmap / status (do not start a new item before closing the open one)

- [x] **M0 — Landing page** (shipped, lives in `site/`)
- [x] **M1 — Plan + context file + stack decision** (`CONTEXT.md`, this milestone)
- [x] **M2 — App scaffold** — Tauri 2 + React/Vite, tokens.css, folder/mono structure
- [x] **M3 — Rust core** — scan (walker + progress), model.rs (Reading), store (SQLite),
      extract (classify / dates / secrets / dhash duplicates / EXIF-GPS), score (Let-go),
      embed (EmbeddingGemma + cosine), ics, actions (safe-copy, set-aside/restore)
- [x] **M4 — One Screen UI** — drop zone, scanning progress, ask bar + hint chips,
      results view, three cards, confirmations, toasts + undo, keyboard, reduced-motion, a11y
- [x] **M5 — Wiring + verification** — IPC contracts, `cargo check`, `tsc`, `vite build`,
      scan state paths on the real filesystem
- [x] **M6 — Native first run** — `tauri dev` window launches and renders; empty and ready
      states verified on a real 8-file fixture folder
- [x] **M7 — OCR feature + reader tests** — PaddleOCR det/cls/rec end-to-end, dictionary and
      landscape fixes, real OCR/GPS fixtures, **35 cargo tests** on the OCR build
- [x] **M8 — Bundled model + honest scoring** — `embed.rs` loads the shipped
      `model_q4.onnx` + `model_q4.onnx_data` through fastembed's external initializer;
      semantic search E2E proof verified with the bundled files (42 cargo tests on OCR build,
      41 on light build). Landing/About/README verified: no Gemma 4 or TabPFN claims.
- [x] **M9 — Desktop packaging + CI** — NSIS installer (`182.72 MiB` release setup),
      `DirectML.dll` beside the exe, `assets/ocr` and `models/embeddinggemma` bundled;
      automated install smoke test (`scripts/smoke-test-installer.mjs`) verified;
      CI workflow updated with frontend, cached model tests (light + OCR), and installer build.
- [x] **M10 — Ship** — 60-second airplane-mode demo guide (`DEMO.md`), release assets verified
      (NSIS `182.72 MiB`, SHA-256 documented), final claim audit completed across all product materials.
- [x] **M11 — Publish** — public repo `0dvsh0/draku` pushed; GitHub Pages deploy
      workflow added (site live); release `v0.1.0` published with installer + exe;
      landing-page download buttons wired to the real release asset; repo URLs
      unified everywhere (site, main.js, README).

## 9. Conventions

- Code must be production-grade Rust and TypeScript. **No mocks**: shipping features must
  operate on the real filesystem/store, never on fake data.
- Suggest→confirm must remain suggest→confirm forever.
- The words "Find / Remember / Protect / Let go" are capitalized **outcomes**, not features.
- Every file is keyed by `root + separator + rel`; rescans keep history, and the separator
  prevents path-prefix collisions.
- All user-visible text stays true to the landing-page voice (short, concrete, no hype).

— Last updated: M11 (Oct 30, 2026)
