---
title: "Draku — I built a local AI that reads my files once, and never phones home"
published: true
tags: devchallenge, hf26challenge, gemma, rust
canonical_url: https://github.com/0dvsh0/draku
cover_image: https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/draku-hero-banner.png
---

*This is a submission for the [Hacktoberfest Open-Source AI Challenge Week 4: Ghost in the Machine](https://dev.to/challenges/hacktoberfest-week4-2026-10-26)*

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/draku-hero-banner.png" alt="Draku: Your files, finally understood" width="100%" />
</p>

## What I Built

**[Draku](https://github.com/0dvsh0/draku) is a free, open-source, on-device AI file manager for Windows. One reading per file, four outcomes, zero network.**

### The ghost in my machine wasn't the computer

A deadline haunts me, so let me start with the scene that started this project.

It's 11 pm. The pharmacy closes in twenty minutes. The person at the counter wants to know exactly what the doctor prescribed eight weeks ago, and I *know* I screenshotted that blood-test report — I remember holding my phone over it. I start scrolling. WhatsApp saved it as `IMG-20261011-WA0004.jpg`. My memes are indexed by an app that studies everything I send; my medical history is 3.4 MB of pixels that no search box on any operating system has ever looked inside. Behind me in line, someone coughs.

I found it eventually. But the moment stuck: **the most important document I own is a JPEG, and nothing on my laptop can read it.**

Once I started looking, I saw the machine was full of these. Screenshots are the most popular notebook on the planet, and nobody calls it a notebook. A warranty ending, a bill due, a one-time password, a boarding pass, a router login, an ID photographed "just for sharing." Each one becomes a ghost of a specific kind:

1. **A trapped date.** A warranty that dies next Tuesday, sealed inside an unindexed JPEG. No calendar app can see it. It only introduces itself the day the protected device stops working.
2. **An exposed secret.** An ID number, a card slip, a plaintext password sitting in a gallery that any app with media permissions can read. The screenshot was the convenient part. The trail is the expensive part.
3. **An invisible trace.** A photo that quietly carries home GPS coordinates and device serials inside EXIF metadata — facts the recipient never sees, and every app on the share path does.
4. **Dead weight.** Six OTPs from Tuesday, a boarding pass for a flight that has landed, four near-identical receipts. Still here. Still named `IMG-...`. Still costing attention and storage.

The usual offered fix is: *upload your gallery so our cloud AI can organize it.* But my ghost list is a list of my most intimate documents — prescriptions, tax slips, my family. And "our terms of service won't change" is not a promise that survives being tested by a subpoena. I don't want a better bargain. **I want a machine that doesn't need one.**

So the thesis I wrote down before writing any code:

> Read every file **once, on the device it already lives on.** Write down a structured understanding. Make every product feature a cheap *view* of that single reading. Let nothing leave the laptop — there is no cloud mode, not even a hidden one.

### One reading, four outcomes

Everything Draku does flows from one `reading.json` per file. No second AI pass, no per-feature pipeline — one reading, four views of it:

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

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/one-reading-four-outcomes.png" alt="One Reading, Four Outcomes" width="100%" />
</p>

| Outcome | Before | After | What produces it |
|---|---|---|---|
| **Find** | `IMG-20261011-WA0004.jpg` | `2026-10-11 Dr Patel blood test prescription.jpg`, findable by typing "blood test" | local OCR → rule-based reader → **EmbeddingGemma-300M** → semantic search |
| **Remember** | A warranty screenshot quietly expiring | A real `.ics` calendar reminder, one tap | date extraction with surrounding words → confirm → local `.ics` |
| **Protect** | A photo exposing an ID number and GPS | A safe copy with the blur burned into pixels and location metadata stripped | local OCR bounding boxes + EXIF → confirm → clean copy |
| **Let go** | OTPs, dead tickets, duplicates | Rested in a folder you can restore from | transparent signals → an explainable dead score → "Set aside" |

### The five promises — architecture, not slogans

Each one is enforced by code, not by a policy page:

1. **No upload.** The shipped installer bundles every model — PaddleOCR weights, the EmbeddingGemma Q4 matrix, the DirectML runtime. There is *no cloud mode to toggle*. Pull the ethernet cable mid-scan and the scan doesn't notice.
2. **No delete.** There is no delete command in this codebase. "Set aside" moves files to a Rest folder. Only you can empty it. Restore is one tap.
3. **Suggest → confirm.** Nothing mutates until you press a button, and generated artifacts (renames, `.ics`, safe copies) offer Undo. The app's footer states it plainly: *"Nothing changes on your device until you press a button."*
4. **Free, open source, no account.** MIT at the core; every vendored model and library's terms are collected in `THIRD_PARTY_NOTICES.md`.
5. **No settings screen.** One screen: drop a folder, get three cards and an "Ask anything" bar. If a feature needs a toggle, it isn't finished.

## Demo

### Four real captures — airplane mode on the entire time

Every capture below is the real app on Windows — no mockups, no staged data. The full airplane-mode walkthrough script lives in the repo as [`DEMO.md`](https://github.com/0dvsh0/draku/blob/main/DEMO.md).

**Find — search the pixels, not the filename.** Local PaddleOCR reads the text; a rule-based reader picks a human name, a category and the key words; EmbeddingGemma-300M embeds the reading. I type *"internet password"* and the router login surface appears — meaning-matched, zero keyword overlap. Exact words still work too.

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/feature-1-find.gif" alt="Find: semantic search without keywords" width="100%" />
</p>

**Remember — trapped dates become calendar events.** Dates are extracted *with their surrounding words*, so a reading knows that "Warranty ends 2026-11-04" is a deadline while "Purchase date 2026-10-04" is not. One arm, one confirm, and Windows calendar receives a real `.ics` file your existing calendar app accepts — no sync service in between.

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/feature-2-remember.gif" alt="Remember: one-tap calendar reminder" width="100%" />
</p>

**Protect — redaction you can't un-peel.** PaddleOCR returns exact pixel boxes; "Make a safe copy" burns the blur into *new* pixels and strips EXIF GPS. The original stays untouched, and the first tap only arms the action.

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/feature-3-protect.gif" alt="Protect: burn-in redaction and GPS stripping" width="100%" />
</p>

Before and after, side by side — the ID number and the GPS coordinates readable in the first frame, gone in the second, original preserved:

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/redact-before-after.png" alt="Protect: before and after" width="100%" />
</p>

**Let go — a ghost folder with a door.** An expired date, a near-identical duplicate, a one-time-code screenshot. Each gets a reason you can actually read. "Set aside" rests them in `.draku/rest/` — and there is no delete command anywhere in this codebase, on purpose.

<p align="center">
  <img src="https://raw.githubusercontent.com/0dvsh0/draku/main/site/images/feature-4-letgo.gif" alt="Let go: explainable rest with Undo" width="100%" />
</p>

### Try it

One screen. Drop a folder, three cards appear, ask anything.

- **Download the installer:** [`Draku_0.1.0_x64-setup.exe`](https://github.com/0dvsh0/draku/releases/latest/download/Draku_0.1.0_x64-setup.exe) (182.72 MiB, SHA-256 in the [release notes](https://github.com/0dvsh0/draku/releases)). It bundles all four OCR models and the 768-dimension embedding weights, so the very first scan is already offline.
- **Or read the source the way I'd rather:** 

```pwsh
git clone https://github.com/0dvsh0/draku && cd draku/app
npm install
npm run tauri dev
```

## Code

**Repository:** [github.com/0dvsh0/draku](https://github.com/0dvsh0/draku) — MIT.

```
draku/
├── site/               ← landing page (pure HTML/CSS/JS, WebGL embers, no build step)
└── app/
    ├── src/            ← the One Screen UI — React 19 + TypeScript, mascot design tokens
    ├── scripts/        ← installer bundling + the automated smoke test
    └── src-tauri/      ← the Rust core that touches every byte
        ├── scan.rs     ← folder walker + dHash duplicate detection
        ├── extract/    ← the READER: PaddleOCR, PDF text, EXIF/GPS, filename rules
        ├── embed.rs    ← bundled EmbeddingGemma-300M loader + cosine search
        ├── score.rs    ← the transparent "Let go" judge — weights you can read
        ├── store.rs    ← SQLite (WAL) — one local DB, no daemons
        └── actions.rs  ← safe copies, .ics, set-aside / restore — the ONLY mutations
```

### The embedding model lives beside the exe, and never talks

The part I'm proudest of is that "no network" is **load-bearing**, not a slogan. EmbeddingGemma's 4-bit weights ship as `model_q4.onnx` plus a separate initializer file. fastembed hands both to ONNX Runtime *in memory*, so nothing is ever written beside the installed executable — which may be read-only:

```rust
// app/src-tauri/src/embed.rs — the offline path is the only shipped path
let mut bundled = UserDefinedEmbeddingModel::new(read("model_q4.onnx")?, TokenizerFiles {
    tokenizer_file:          read("tokenizer.json")?,
    config_file:             read("config.json")?,
    special_tokens_map_file: read("special_tokens_map.json")?,
    tokenizer_config_file:   read("tokenizer_config.json")?,
})
.with_pooling(Pooling::Mean)
.with_quantization(QuantizationMode::None)
.with_external_initializer("model_q4.onnx_data".into(), read("model_q4.onnx_data")?);

TextEmbedding::try_new_from_user_defined(bundled, InitOptionsUserDefined::new())
```

Ask *"internet password"* — your phrase becomes a 768-dimension vector, cosine similarity runs against each file's reading, all on top of the local SQLite index. There is no HTTP client anywhere in the crate.

### The judge is arithmetic you can read

Rather than an opaque neural score, `score.rs` exposes a plain struct of signals and one function. Every weight is a named constant with a comment explaining the product story it protects:

```rust
// app/src-tauri/src/score.rs
pub fn judge(s: &Signals) -> Verdict {
    let mut score = 0.10;
    score += (s.age_days / 180.0).min(0.30);
    if s.is_otp      { score += 0.44; }   // one-time codes only live once
    if s.date_passed { score += 0.50; }   // the deadline is gone; the file overstayed
    if s.is_dup      { score += 0.50; }   // a near-identical file exists
    // …the result clamps to [0, 1]; above 0.60 a file earns the Let-go card
    Verdict { dead: score.clamp(0.0, 1.0), reason: choose_reason(&s) }
}
```

### Verified, not vibes

- **42 cargo tests on the full OCR build** — EXIF GPS scrubbing, redaction box math, and an end-to-end semantic query against the bundled weights. **41 more on the light build** (`--no-default-features`) for machines that can't run PaddleOCR.
- **The installer ships with its own lie detector.** `scripts/smoke-test-installer.mjs`, wired into CI, silent-installs the NSIS package on a Windows runner and verifies every critical bundle: `draku.exe`, `DirectML.dll`, all four OCR models, all six embedding files.
- **Checksums published with every release**, so the 182 MiB you download is the 182 MiB that was verified: `SHA256: 19473E9A5AEC88BDE82DF50CD697E7A9450A0B0EA88B49E65B57F89843E52791`.

## How I Built It

### The stack

| Layer | Choice | Why it earns its place |
|---|---|---|
| Shell / core | **Tauri 2 · Rust** | The privacy promise should be *checkable* — you can read the Rust that touches your files and count zero sockets. Also: a 182 MiB installer carries everything, instead of an Electron app that re-downloads its own world. |
| Semantic memory | **Google EmbeddingGemma-300M (4-bit ONNX)** | 768-dimension embeddings from an open-weights model small enough to ship *inside the installer* — the single technical fact that makes "first offline scan" real instead of a roadmap item. |
| Local runtime | **fastembed + ONNX Runtime + DirectML** | Quantized ONNX with external initializers loaded in memory; DirectML accelerates on any modern Windows GPU/NPU with no CUDA ceremony. |
| Vision / OCR | **PaddleOCR PP-OCR (Apache-2.0)** | det + cls + rec give exact pixel boxes — and the *same* boxes feed Protect's burned-in redaction, so what you can read is exactly what gets blurred. |
| Duplicates / metadata | **dHash + kamadak-exif** | 64-bit visual hashes catch rescaled screenshots; EXIF inspection treats GPS and device serials as first-class content, not trivia. |
| Frontend | **React 19 + TypeScript + Vite** | Strict types on every IPC boundary; fonts vendored via `@fontsource`, so the UI never depends on a CDN. |
| Persistence | **SQLite (WAL) via bundled rusqlite** | One local database in the app directory: readings, embeddings, and approved actions in one place. |

### What "open-source AI at its core" means here, concretely

The challenge prompt's load-bearing words are *open weights* and *local inference*. In Draku they reduce to one sentence: **the intelligence that understands your files is Google's open-weights EmbeddingGemma, running on your own GPU through DirectML, shipped inside your own installer.** Making that fit was the project's real engineering exercise — getting the 4-bit quantization through fastembed's user-defined model API, solving the external-initializer loading that ONNX requires for split weight files (and documenting the two-line patch to the vendored PaddleOCR binding that made it compile), then proving with an automated smoke test that "the bytes in the installer" and "the bytes I tested" are the same bytes.

### Honest boundaries

Windows first — that's where DirectML and Webview2 made the packaged experience certain. The light build exists for low-spec hardware: semantic search, dates, duplicates and actions all work without OCR; Protect reports what it can from names and metadata. No claims here about Gemma 4 multimodal or any other roadmap model — this post only claims what the shipped build already does.

## Why Does Open Innovation Matter?

**1. Privacy becomes a property of the architecture, not of a policy.** A closed-API file manager must upload to be smart. An open-weights one has no cloud mode to misconfigure, because the model runs where the files already live. That inverts the trust question from "will they honour their promise" to "does this binary open a socket" — and only one of those can be answered with a grep.

**2. You can audit the exact lines that touch your life.** Your camera roll holds prescriptions, tax documents, your family. Because Draku is open source, you can search the Rust for network APIs and find none — and verify *what you run*, not just what I wrote, through the published checksums and the CI smoke test that unpacks and inspects the installer itself.

**3. The cleanup tool should not be a tollbooth.** Cloud assistants offer to organize your clutter free, then rent you back the storage tier it graduated into. Open weights invert the economics: the intelligence costs zero infrastructure to run, which is the only reason a finished product can be permanently free — no subscription, no account, no upsell on the space you're trying to empty.

**4. Ten-year resilience.** Cloud products die when their servers do. The model files here live beside the program under open licenses, so in ten years this still works on an airplane, in a village, on an old laptop — no server, no key, no consent flow in the way.

## The Spirit of It

Weeks after the pharmacy, someone asked me — the household's unofficial computer person — to find "that electricity bill photo, before the 15th." Same scroll, same rising panic. This time there was a build on my laptop. I dropped the folder, waited two quiet seconds, and typed "electricity." Top hit: the right bill, renamed by no human. One tap → **Add to calendar**. The reminder appeared in the phone's calendar like it had always lived there.

Then the question that made the whole project worth it: *"So it doesn't go anywhere?"*

No. It just sits here.

That silence — two seconds where nothing audibly transmits, where the laptop does the entire job while you can almost hear the network *not* being used — is the moment this project keeps returning to. Draku is the mascot because that kind of companion needed a face: a small black cat with crimson wings who lives quietly in the machine, watches over the ghost files, names what nobody named, and sits with what's dead until you say goodbye. He doesn't gossip to the cloud, doesn't ask for money, doesn't touch a thing until you tell him to.

## Prize Categories

- **Best Use of Gemma ($200)** — Google's open-weights **EmbeddingGemma-300M** is the product's semantic engine, not an accessory: the shipped Windows installer bundles the 4-bit ONNX weights, tokenizer, and external initializer, loaded in memory by fastembed + ONNX Runtime with DirectML acceleration. Every semantic search ("internet password" → the router login, zero keyword overlap) and every human-readable file name originates from a Gemma-family model executing fully offline.
- **Best Use of GitHub Copilot ($100)** — the entire build-and-verify loop is automated with **GitHub Actions** (`ci.yml`): the frontend suite (Vitest + strict `tsc`), the light and full-OCR cargo test suites with a cached model, and a Windows runner job that builds the NSIS installer, **silent-installs it**, and verifies every bundled file — `draku.exe`, `DirectML.dll`, all 4 OCR models, all 6 embedding files — before any release can be tagged.

---

*Built with ♥ for Hacktoberfest 2026 — [Draku on GitHub](https://github.com/0dvsh0/draku) · [Landing page](https://0dvsh0.github.io/draku/) · [Download the installer](https://github.com/0dvsh0/draku/releases/latest/download/Draku_0.1.0_x64-setup.exe)*
