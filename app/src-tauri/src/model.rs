//! Every type that crosses the Rust ⇄ TypeScript boundary, plus the core
//! product concept: one `Reading` per file, four outcomes from it.

use serde::{Deserialize, Serialize};

/// One chronological fact found in a file ("when it matters").
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DateRef {
    /// e.g. "Warranty ends", "Refill by", "Exam form closes"
    pub label: String,
    /// ISO-8601 date: 2026-11-10
    pub date: String,
    /// "deadline" | "expiry" | "event" | "taken"
    pub kind: String,
}

/// One privacy finding ("what is private in it").
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SecretRef {
    /// "password" | "id_number" | "card_number" | "location"
    pub kind: String,
    /// Masked preview safe to render in the UI.
    pub preview: String,
    /// 1-based page for PDFs.
    pub page: Option<u32>,
    /// Normalized [x, y, w, h] in the image (for burned-in redaction once OCR is on).
    pub bbox: Option<[f32; 4]>,
}

/// The single reading. Every outcome is a view of this.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub root: String,
    pub rel: String,
    /// Proposed human name, e.g. "2026-10-11 Dr Patel blood test prescription".
    pub name: String,
    pub category: String,
    /// Short text body used for the embedding ("key text").
    pub key_text: String,
    pub dates: Vec<DateRef>,
    pub secrets: Vec<SecretRef>,
    /// GPS degrees as (lat, lon) when EXIF carries them.
    pub gps: Option<(f64, f64)>,
    /// 64-bit perceptual hash, hex string, for duplicate detection.
    pub phash: Option<String>,
    /// Relative path of the file this duplicates (when detectable).
    pub dup_of: Option<String>,
    /// 0.0 = alive … 1.0 = dead.
    pub dead: f32,
    /// One-line reason the reader thinks it is dead, if any.
    pub dead_reason: Option<String>,
    pub size: u64,
    /// Epoch seconds.
    pub mtime: i64,
}

impl Reading {
    pub fn file_id(&self) -> String {
        // Stable composite key across rescans. The separator matters: without
        // it, root `.../photos` + rel `hop.jpg` would collide with root
        // `.../photo` + rel `shop.jpg`.
        std::path::Path::new(&self.root)
            .join(&self.rel)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

// ---------------------------------------------------------------------------
// IPC payloads (what the One Screen consumes)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeadlineItem {
    pub file_id: String,
    pub path: String,
    pub name: String,
    pub label: String,
    /// ISO date
    pub date: String,
    /// Whole days from today; negative = passed.
    pub days_left: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SecretItem {
    pub file_id: String,
    pub path: String,
    pub name: String,
    pub kind: String,
    pub preview: String,
    pub has_gps: bool,
    /// True when OCR gave a pixel box, so "Make a safe copy" can burn the
    /// finding out of the image instead of only stripping metadata.
    pub redactable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestItem {
    pub file_id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    /// Whole days since the file was last touched.
    pub age_days: u32,
    pub dead: f32,
    pub dead_reason: Option<String>,
}

/// The whole One Screen in one payload.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub deadlines: Vec<DeadlineItem>,
    pub secrets: Vec<SecretItem>,
    pub rest: Vec<RestItem>,
    pub files_total: usize,
    /// Bytes that Set aside would free right now.
    pub rest_freed: u64,
    pub roots: Vec<String>,
}

impl AppState {
    pub fn empty() -> Self {
        Self {
            deadlines: vec![],
            secrets: vec![],
            rest: vec![],
            files_total: 0,
            rest_freed: 0,
            roots: vec![],
        }
    }
}

/// Progress events emitted while a folder is being read (event `scan://progress`).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub done: u32,
    pub total: u32,
    pub current: String,
    /// Human line about the embedding pass ("model", "index", or null).
    pub stage: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub file_id: String,
    pub path: String,
    pub name: String,
    pub category: String,
    /// Cosine similarity 0..1
    pub score: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionOk {
    pub message: String,
    pub undoable: bool,
    pub path: Option<String>,
    /// File ids a single tap can restore. Empty when nothing can be undone.
    #[serde(default)]
    pub undo_file_ids: Vec<String>,
    /// Generated artifacts (safe copy, calendar file) a tap can remove.
    #[serde(default)]
    pub undo_paths: Vec<String>,
}

/// Generic JSON error for IPC.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub message: String,
}

pub type IpcResult<T> = Result<T, IpcError>;

pub fn ipc_err(things: impl std::fmt::Display) -> IpcError {
    IpcError {
        message: things.to_string(),
    }
}
