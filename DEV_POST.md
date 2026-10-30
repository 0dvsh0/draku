---
title: "Draku: Your files, finally understood. On-device AI file manager."
published: true
tags: hacktoberfest, ai, gemma, rust, opensource
canonical_url: https://github.com/smash/draku
cover_image: https://raw.githubusercontent.com/smash/draku/main/site/images/draku-hero-banner.png
---

*This is a submission for the [Hacktoberfest Open-Source AI Challenge Week 4: Ghost in the Machine](https://dev.to/challenges/hacktoberfest-week4-2026-10-26)*

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/draku-hero-banner.png" alt="Draku: Your files, finally understood" width="100%" />
</p>

> **"Your files, finally understood."**
> A private, 100% on-device AI for your laptop that understands every screenshot, photo, and PDF. It **Finds** what you need in plain English, **Remembers** deadlines before they expire, **Protects** exposed credentials, and **Lets go** of dead weight.
>
> **Nothing ever leaves your machine.**

---

## What I Built

### 1. The Haunting: How We Interpreted "Ghost in the Machine"

Your downloads folder and camera roll are haunted.

Screenshots are the most popular notebook on earth that nobody calls a notebook. Millions of people snap screens every day: a warranty receipt, a prescription, a doctor's referral, a Wi-Fi password in a router portal, a boarding pass, or a utility bill.

Once snapped, those pixels become trapped ghosts:
1. **A Trapped Date**: A laptop warranty expiring next Tuesday, or an electricity bill due next week. Locked inside an unindexed JPEG, no calendar app can see it.
2. **An Exposed Secret**: A government ID number (Aadhaar / SSN), a credit card slip, or a plaintext password sitting in your gallery where any app with media permissions can read it.
3. **An Invisible Trace**: A photo sent across messaging apps that quietly leaks home GPS coordinates and hardware serial numbers in EXIF metadata.
4. **Dead Weight**: Six-month-old one-time passwords (OTPs), expired boarding passes, and duplicate receipts that outlive their utility and clog storage.

Traditional cloud file managers offer a toxic bargain: upload your most intimate personal documents to a remote corporate server so a cloud AI can read them, index them, and sell you storage tiers.

We built **Draku** to prove that **on-device AI is ready today**.

---

### 2. The Core Thesis: One Reading, Four Outcomes

Draku is built around a single foundational rule: **every file is read exactly once on your machine.**

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/one-reading-four-outcomes.png" alt="One Reading, Four Outcomes" width="100%" />
</p>

Everything the product does is simply a native view of that single local reading:

| Outcome | Mascot Part | Before | After | Pipeline |
|---|---|---|---|---|
| **Find** | **Eyes** | `IMG-20261011-WA0004.jpg` | `2026-10-11 Dr Patel blood test prescription.jpg` | Local OCR + rule-based reader → **EmbeddingGemma-300M** → semantic search |
| **Remember** | **Ears** | Trapped warranty screenshot | 1-tap calendar reminder | Reading extracts dates & context → confirmed → native `.ics` |
| **Protect** | **Wings** | ID / password / GPS metadata visible | Pixels burned & EXIF stripped | PaddleOCR bounding boxes → confirmed → clean safe copy |
| **Let go** | **Tail** | Old OTPs, expired passes, duplicates | Rested in a rest folder | Age, category, date passed, dHash distance → transparent dead score |

---

### 3. The Five Promises (Product Law)

We wrote five non-negotiable architectural covenants into the repository before writing a single line of application code:

1. **No upload.** The packaged installer bundles all models and runtimes. There is *no cloud mode to toggle*. Turn off Wi-Fi, pull the ethernet cable, and everything functions identically.
2. **No deleting.** Draku never deletes a file. Files you let go move to a `.draku/rest/` folder that only you can empty. One tap restores any file to its original location.
3. **No guessing out loud.** Suggest → confirm. The app never mutates anything on disk without an explicit tap from the user. Every generated artifact has an Undo button.
4. **No account, subscription, or ads.** 100% free and open-source under the MIT license.
5. **No settings screen.** One screen. Drop a folder → three cards + one "Ask anything" question bar. If a feature needs a settings toggle, it is not finished.

---

## Demo

### 1. Real Screen Captures of All Four Features

#### Feature 1: Find (Semantic Search & Human Naming)
Local OCR reads the pixels, a rule-based reader names the file, and **Google EmbeddingGemma-300M** embeds the reading. You can ask in plain language without knowing the filename or exact words:

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/feature-1-find.gif" alt="Real Screen Capture: Feature 1 - Find" width="100%" />
</p>

*Real app capture: Typing `"internet password"` into the Ask bar semantically matches `Router admin settings` containing credentials with 89% similarity via local EmbeddingGemma — zero keyword overlap required.*

---

#### Feature 2: Remember (Trapped Deadlines to Native Calendar)
Dates are extracted along with their surrounding context words, so *"Warranty ends in 6 days"* is recognized as an urgent deadline while purchase dates are not. With Suggest → Confirm verification, a tap creates a real `.ics` calendar reminder:

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/feature-2-remember.gif" alt="Real Screen Capture: Feature 2 - Remember" width="100%" />
</p>

*Real app capture: The 3 Deadlines card arms on first tap ("Tap again to set it") and generates a native local `.ics` reminder on confirmation.*

---

#### Feature 3: Protect (Redaction & GPS Stripping)
Government ID numbers, passwords, credit card slips, and GPS coordinates are detected locally. Tapping **"Make a safe copy"** permanently burns a pixelation blur over the OCR bounding box and scrubs all EXIF location tags, leaving the original file completely untouched:

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/feature-3-protect.gif" alt="Real Screen Capture: Feature 3 - Protect" width="100%" />
</p>

*Real app capture: Tapping "Make a safe copy" creates an unpeelable redacted copy in `.draku/safe/` with GPS location stripped.*

##### Redaction: Before & After Results

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/redact-before-after.png" alt="Protect Redaction: Before and After Results" width="100%" />
</p>

- **Before**: Sensitive government ID `1248 9821 0042` and home EXIF GPS coordinates (`19.0760, 72.8777`) are exposed to any app with media permissions.
- **After**: PaddleOCR bounding boxes permanently blur the ID region into the pixels, and EXIF metadata is stripped clean into `.draku/safe/` while preserving the untouched original.

---

#### Feature 4: Let Go (Transparent Dead-Weight Scoring & Safe Rest)
Expired flight passes, old OTP screenshots, and near-identical duplicates get an explainable score you can inspect. Tapping **"Set aside"** moves them to `.draku/rest/`. There is no delete command anywhere in the codebase:

<p align="center">
  <img src="https://raw.githubusercontent.com/smash/draku/main/site/images/feature-4-letgo.gif" alt="Real Screen Capture: Feature 4 - Let Go" width="100%" />
</p>

*Real app capture: Tapping "Set aside (137 KB)" arms and moves dead files safely into Rest with instant 1-tap Undo.*

---

### 2. The 60-Second Airplane-Mode Walkthrough

| Timestamp | Visual on Screen | Voiceover / Script |
|---|---|---|
| **0:00 - 0:10** | Start with Windows system tray showing **Airplane Mode: ON**. Show messy demo folder. Drop folder onto Draku. | *"This folder is on every laptop. Airplane mode is on. Everything you are about to see happens right here on this machine."* |
| **0:10 - 0:25** | Scanning completes quietly. Cryptic file names resolve to clear descriptive titles. In the Ask bar, type `"internet password"`. | *"Draku reads each file once. Now every file has a human name, and you can just ask in plain language. Even without matching keywords, local EmbeddingGemma surfaces the router login."* |
| **0:25 - 0:35** | The **3 Deadlines** card is in view. Hover over *Warranty ends in 6 days*. Click **Add to calendar**. | *"That date was trapped inside a picture. Now it's a real calendar reminder in your system with one tap."* |
| **0:35 - 0:48** | Switch to the **4 Exposed Secrets** card. The lab report's ID number and the photo's GPS coordinates are highlighted. Click **Make a safe copy**. | *"Before you share a document, you see what you'd give away. One tap pixelates the private numbers and strips the GPS tag, leaving the original file safe."* |
| **0:48 - 0:55** | Switch to the **3 files ready to let go** card. Click **Set aside**. The files disappear from the view, moving to `.draku/rest/`. | *"Three files nothing depends on — old OTPs, past boarding passes, duplicates. They rest safely in a folder you can restore from. Draku never deletes."* |
| **0:55 - 1:00** | Pan down to the Windows system tray showing **Airplane Mode still active**. | *"One more thing: that was offline the entire time. No uploads, no accounts, no subscriptions."* |

---

## Code

### 1. Repository & Architecture

- **GitHub Repository**: [https://github.com/smash/draku](https://github.com/smash/draku)
- **License**: MIT License (Core) · Apache-2.0 / Gemma Terms of Use (Models)

```
draku/
├── site/               ← Landing page & product manifesto (pure HTML/CSS/JS + WebGL)
│   ├── images/         ← Mascot graphics, before/after illustrations & real feature GIFs
│   └── About.md        ← Product writeup and design philosophy
└── app/                ← The desktop application
    ├── src/            ← One Screen UI (React 19 + TypeScript + Mascot Design Tokens)
    │   ├── components/ ← OneScreen, Scanning, Empty, Eye, Toasts
    │   └── styles/     ← tokens.css (Mascot aesthetic: fur, bone, crimson, ash)
    └── src-tauri/      ← Native Rust Core
        ├── scan.rs     ← Folder walker + perceptual hashing (dHash)
        ├── extract/    ← The Reader (PaddleOCR, PDF parser, EXIF/GPS, Heuristics)
        ├── embed.rs    ← Bundled EmbeddingGemma loader (ONNX Runtime + FastEmbed)
        ├── score.rs    ← Transparent, explainable dead-scoring
        ├── store.rs    ← SQLite database (rusqlite, WAL mode)
        └── actions.rs  ← Safe copy redaction, .ics calendar export, rest & restore
```

---

### 2. Engineering Rigor & Deterministic Verification

No mocks. Every single test operates against real file systems and real local models:

- **42 Unit & Integration Tests (Full OCR Build)**:
  `cargo test --features ocr` covers everything from EXIF GPS scrubbing and pixelation box math to end-to-end semantic queries with the bundled EmbeddingGemma weights.
- **41 Tests (Light Build)**:
  `cargo test --no-default-features` verifies that semantic search, date parsing, and duplicate handling operate even on low-spec hardware without OCR dependencies.
- **Automated Installer Smoke Test**:
  Our CI runs `npm run smoke:installer` on Windows, which performs a silent test installation (`/S /D=...`), verifies byte sizes and integrity of `draku.exe`, `DirectML.dll`, all 4 OCR models, and all 6 EmbeddingGemma files, and verifies clean teardown.
- **Deterministic Checksums**:
  - **Installer**: `Draku_0.1.0_x64-setup.exe` (182.72 MiB)
    `SHA256: 19473E9A5AEC88BDE82DF50CD697E7A9450A0B0EA88B49E65B57F89843E52791`
  - **Release Binary**: `draku.exe` (45.16 MiB)
    `SHA256: 6EA45C1D852DE604ADF390D220D0F5E4DA5840DD736736AB6DDC50F340CD95C8`

---

## How I Built It

### 1. The Open-Source AI Stack

| Component | Technology | Role |
|---|---|---|
| **Semantic Memory** | **Google EmbeddingGemma-300M (4-bit ONNX)** | Generates 768-dimensional dense vector embeddings locally to power semantic search without keywords. |
| **Local Inference Runtime** | **FastEmbed + ONNX Runtime** | Loads quantized ONNX model files and external data initializers in memory with zero temp disk footprint. |
| **Hardware Acceleration** | **DirectML (`DirectML.dll`)** | Delivers hardware-accelerated local tensor execution across any modern Windows GPU/NPU without requiring CUDA. |
| **Vision & OCR** | **PaddleOCR (PP-OCRv4)** | Performs local text detection, angle classification, and character recognition with sub-millimeter bounding boxes. |
| **Perceptual Hashing** | **dHash (64-bit)** | Calculates visual image hashes to detect duplicates and rescaled screenshots within a Hamming distance threshold. |
| **Metadata Extraction** | **kamadak-exif** | Inspects EXIF IFD structures for GPS latitude/longitude, altitude, and device hardware serial numbers. |
| **Local Storage** | **SQLite (WAL Mode via `rusqlite`)** | Local encrypted relational database storing file paths, perceptual hashes, embeddings, and audit logs. |
| **Application Core** | **Tauri 2 + Rust + React 19** | Zero-network native wrapper providing a secure IPC bridge to Windows native WebView2. |

---

### 2. Offline EmbeddingGemma Bundling

To ensure the "no upload" promise is real from the very first scan, we bundled Google's **EmbeddingGemma-300M** directly into the offline installer.

The quantized model relies on external initializer data (`model_q4.onnx` + `model_q4.onnx_data`). In `app/src-tauri/src/embed.rs`, FastEmbed and ONNX Runtime load weights directly relative to the executable's directory:

```rust
// Local offline embedding loader using bundled EmbeddingGemma weights
let model_dir = get_bundled_model_dir()?;
let options = InitOptionsUserDefined::new()
    .with_model_file(model_dir.join("model_q4.onnx"))
    .with_tokenizer_file(model_dir.join("tokenizer.json"))
    .with_config_file(model_dir.join("config.json"))
    .with_special_tokens_map_file(model_dir.join("special_tokens_map.json"))
    .with_tokenizer_config_file(model_dir.join("tokenizer_config.json"));

let model = TextEmbedding::try_new_from_user_defined(options, InitOptions::default())?;
```

When you type `"internet password"`, your query is embedded into a 768-dimension vector in under 12 milliseconds using DirectML. Cosine similarities are computed directly against indexed files in SQLite.

---

### 3. Transparent, Explainable Dead-Weight Scoring

Draku rejects opaque, unexplainable neural networks for destructive file management. Instead, `score.rs` implements a transparent set of inspectable weights:

```rust
pub fn calculate_dead_score(file: &ScannedFile, now: DateTime<Utc>) -> (f32, Vec<String>) {
    let mut score = 0.0f32;
    let mut reasons = Vec::new();

    // 1. Has the date passed?
    if let Some(exp) = file.expiration_date {
        if exp < now {
            score += 0.50;
            reasons.push("Date expired in the past".into());
        }
    }
    // 2. Is it an identical or near-identical duplicate?
    if file.dup_distance.unwrap_or(99) <= 2 {
        score += 0.50;
        reasons.push("Duplicate visual content".into());
    }
    // 3. Is it a single-use OTP or authentication screenshot?
    if file.category == "auth / otp" {
        score += 0.44;
        reasons.push("Single-use one-time password".into());
    }
    // 4. Age decay (180 days)
    let age_days = (now - file.created_at).num_days();
    if age_days > 180 {
        score += 0.30;
        reasons.push("Older than 6 months".into());
    }

    (score.clamp(0.0, 1.0), reasons)
}
```

---

## Why Does Open Innovation Matter?

Open-source AI is not just a nice-to-have for a personal file manager; **it is the only architecture that makes the product ethically possible**:

1. **Intimate Privacy Requires Zero Upload**:
   Your camera roll holds medical prescriptions, tax invoices, government ID cards, and photos of your family. Sending those files to a proprietary cloud API (where terms of service can change overnight to allow model training) is a non-starter. Open-weight models (EmbeddingGemma, PaddleOCR) allow intelligence to run on hardware you already own.
2. **Verifiable Auditability**:
   Closed-source software can promise "we respect your privacy", but users cannot verify what happens behind the binary. Because Draku is 100% open-source under the MIT license, anyone can inspect the Rust source code, monitor network sockets, and verify that **zero network packets are ever transmitted**.
3. **No Artificial Tollbooths or Subscription Ransoms**:
   Cloud file managers use remote AI as a tollbooth: uploading your files creates lock-in, and freeing your storage costs a monthly recurring fee. Open innovation enables a free, permanent tool that solves storage clutter without charging you for storage.
4. **Offline Resilience & True Longevity**:
   Draku will work ten years from now on an offline laptop in the middle of nowhere, completely independent of whether a remote cloud API shuts down, changes pricing, or revokes API keys.

---

## My Agent Session

This project was built from conception to release through an intensive agentic pair-programming workflow with Google Antigravity.

Key milestones accomplished during our agent session:
1. **Architectural Ideation**: Formulating the "One Reading, Four Outcomes" philosophy and codifying the Five Promises directly into the test suite.
2. **Offline Bundling & Native Optimization**: Integrating `fastembed-rs` with ONNX Runtime and bundling `DirectML.dll` to achieve sub-15ms local inference without GPU driver friction.
3. **Rust Core Engineering**: Building the multi-threaded folder scanner, perceptual dHash hasher, PDF/EXIF metadata scrubbers, and SQLite WAL database.
4. **Mascot Design System**: Translating the Draku mascot (black fluffy cat with crimson bat wings and flame eyes) into CSS design tokens (`tokens.css`) and responsive React 19 components.
5. **Deterministic Verification & Smoke Testing**: Writing automated NSIS installer test harnesses (`smoke-test-installer.mjs`) and Playwright headless visual verification scripts to ensure every byte and feature runs cleanly.

---

## The Spirit of It

Everyone has experienced that sudden spike of panic at a pharmacy counter, service center, or airport check-in desk:

> *"I know I took a screenshot of that prescription... was it two months ago or three? What did WhatsApp name it? IMG-20261011-WA0004.jpg?"*

You scroll frantically past thousands of memes, receipts, and blurry photos while the line waits behind you.

And everyone knows the unease of sharing a screenshot of a receipt or ticket, wondering whether you accidentally left your bank account number visible or leaked your home location in the EXIF metadata.

Draku was created to banish those ghosts. The mascot — a small, fluffy black cat with crimson bat wings and round flame eyes — was born from this exact idea: a benevolent gothic familiar who lives quietly inside your machine. He doesn't snitch to the cloud, doesn't ask for a monthly subscription, and doesn't guess out loud.

He simply watches over your files, helps you find what matters, guards what should stay private, and lets go of what is dead.

---

## Prize Categories

We are entering Draku into the following categories for Hacktoberfest 2026:

- **Ghost in the Machine (Primary Track)**: Interpreting the challenge through the lens of digital ghosts — trapped dates, exposed secrets, invisible GPS traces, and dead weight haunted inside everyday files — exorcised via 100% on-device AI.
- **Open-Weight AI & Local Inference**: Powered by **Google EmbeddingGemma-300M (4-bit ONNX)** and **PaddleOCR (PP-OCRv4)** bundled offline with DirectML hardware acceleration.
- **Systems & Native Desktop Engineering**: Built in native **Rust (`src-tauri`)** and **React 19**, featuring a single-screen zero-settings UI, SQLite WAL persistence, perceptual hashing, and verified offline installers.
- **Privacy, Security & Open Innovation**: Zero telemetry, zero cloud calls, client-side PII pixelation, EXIF location scrubbing, and MIT open-source licensing.

---

*Built with ♥ for Hacktoberfest 2026.*
<!-- Team Submissions: Submitted by @smash for the Hacktoberfest Open-Source AI Challenge -->
