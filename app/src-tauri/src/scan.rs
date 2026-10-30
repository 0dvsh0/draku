//! Reading folders for real: walk, read each file once, dedupe, embed, store,
//! and report quiet progress to the One Screen. Cancellable between files.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::extract::imginfo::hamming_hex;
use crate::extract::read;
use crate::model::ScanProgress;
use crate::state::DrakuState;

pub static CANCEL: AtomicBool = AtomicBool::new(false);

const IMG_EXTS: [&str; 6] = ["jpg", "jpeg", "png", "webp", "gif", "bmp"];
const TEXT_EXTS: [&str; 7] = ["pdf", "txt", "md", "csv", "log", "json", "rtf"];
/// Hard cap so nobody's entire drive becomes one scan.
const MAX_FILES: usize = 20_000;

pub fn is_readable(rel: &Path) -> bool {
    let ext = rel
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    let hidden = rel
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'));
    (IMG_EXTS.contains(&ext.as_str()) || TEXT_EXTS.contains(&ext.as_str()))
        && !rel.starts_with(".draku")
        && !hidden
}

fn walk_all(root: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .strip_prefix(root)
                .map(is_readable)
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .take(MAX_FILES)
        .collect()
}

/// The full pipeline over one or more folders. Runs on its own thread; state
/// lock is taken in short bursts only so the UI stays responsive.
pub fn scan(roots: Vec<String>, app: AppHandle) {
    let state: tauri::State<DrakuState> = app.state();

    // (root, relative path) keeps multi-root scans honest: every reading knows
    // exactly which folder it belongs to.
    let mut jobs: Vec<(Arc<PathBuf>, String)> = Vec::new();
    for raw in &roots {
        let root = Arc::new(PathBuf::from(raw));
        for file in walk_all(&root) {
            if jobs.len() >= MAX_FILES {
                break;
            }
            if let Ok(rel) = file.strip_prefix(root.as_path()) {
                jobs.push((root.clone(), rel.display().to_string()));
            }
        }
        if jobs.len() >= MAX_FILES {
            break;
        }
    }

    let total = jobs.len() as u32;
    if total == 0 {
        let _ = app.emit(
            "app://notice",
            serde_json::json!({
                "level": "info",
                "message": "No readable files there. Draku reads images, PDFs and text files."
            }),
        );
        emit(
            &app,
            ScanProgress {
                done: 0,
                total: 0,
                current: roots.first().cloned().unwrap_or_default(),
                stage: Some("done".into()),
            },
        );
        return;
    }

    let mut readings = Vec::with_capacity(jobs.len());
    for (i, (root, rel)) in jobs.iter().enumerate() {
        if CANCEL.load(Ordering::Relaxed) {
            break;
        }
        let r = read::read_file(root, rel);
        readings.push(r);
        emit(
            &app,
            ScanProgress {
                done: (i + 1) as u32,
                total,
                current: rel.clone(),
                stage: Some("reading".into()),
            },
        );
    }

    // --- duplicates: within-batch phash hamming <= 3 ----------------------
    mark_duplicates(&mut readings);
    for r in readings.iter_mut() {
        read::with_dup(r);
    }

    // --- embeddings: one batch, one model, zero cloud ----------------------
    // The model ships with the app. A dev checkout without the bundled copy
    // fetches it once into the local cache; either way, failure never blocks
    // the readings from being stored.
    let texts: Vec<String> = readings
        .iter()
        .map(|r| {
            // EmbeddingGemma's retrieval prompt for documents. Without it the
            // model still ranks, but the cosine scores understate the match.
            format!("title: {} | text: {}", r.name, r.key_text)
        })
        .collect();
    let embeddings: Option<Vec<Vec<f32>>> = {
        emit(
            &app,
            ScanProgress {
                done: 0,
                total: texts.len() as u32,
                current: String::new(),
                stage: Some("model".into()),
            },
        );
        let mut embedder = state.embedder.lock().unwrap();
        match embedder.embed(&texts) {
            Ok(v) => Some(v),
            Err(e) => {
                let _ = app.emit(
                    "app://notice",
                    serde_json::json!({"level": "warn", "message": format!("Embedding model unavailable, search stays limited: {e}")}),
                );
                None
            }
        }
    };

    // --- persist ------------------------------------------------------------
    {
        let store = state.store.lock().unwrap();
        for (i, r) in readings.into_iter().enumerate() {
            let emb = embeddings
                .as_ref()
                .and_then(|v| v.get(i))
                .map(|e| e.as_slice());
            if let Err(e) = store.upsert_reading(&r, emb) {
                crate::log_warn(&format!("could not store {}: {e}", r.rel));
            }
        }
    }

    emit(
        &app,
        ScanProgress {
            done: total,
            total,
            current: roots.join(" · "),
            stage: Some("done".into()),
        },
    );
}

/// Files whose 64-bit dhash is within two bits are copies. The old threshold
/// of eight was broad enough to call two unrelated white receipts duplicates,
/// which made the Let-go card untrustworthy. Two bits still catches rescans and
/// slightly recompressed copies while leaving genuinely different pages alone.
///
/// When two copies match, the plainer name wins: `otp.png` stays the original
/// and `otp copy.png` becomes the file Draku offers to let go. Otherwise the
/// alphabetically first name would decide, which is arbitrary and often picks
/// the copy.
pub(crate) fn mark_duplicates(readings: &mut [crate::model::Reading]) {
    let mut groups: Vec<(String, Vec<usize>)> = vec![];
    for (i, r) in readings.iter().enumerate() {
        let Some(h) = r.phash.clone() else { continue };
        if let Some((_, members)) = groups
            .iter_mut()
            .find(|(anchor, _)| hamming_hex(anchor, &h) <= 2)
        {
            members.push(i);
        } else {
            groups.push((h, vec![i]));
        }
    }

    for (_, members) in groups {
        if members.len() < 2 {
            continue;
        }
        let canonical = *members
            .iter()
            .min_by_key(|&&i| copy_rank(&readings[i].rel))
            .unwrap();
        let canonical_rel = readings[canonical].rel.clone();
        for i in members {
            if i != canonical {
                readings[i].dup_of = Some(canonical_rel.clone());
            }
        }
    }
}

/// Lower is more original: a plain name beats a copy marker, then fewer
/// characters, then alphabetical order so the result is deterministic.
fn copy_rank(rel: &str) -> (u8, usize, String) {
    let lower = rel.to_ascii_lowercase();
    let copy_marker = lower.contains(" copy")
        || lower.contains("(1)")
        || lower.contains("-copy")
        || lower.contains("duplicate");
    (copy_marker as u8, rel.len(), lower)
}

fn emit(app: &AppHandle, p: ScanProgress) {
    let _ = app.emit("scan://progress", &p);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Reading;

    fn reading(rel: &str, hash: Option<&str>) -> Reading {
        Reading {
            root: "root".into(),
            rel: rel.into(),
            name: rel.into(),
            category: "photo".into(),
            key_text: String::new(),
            dates: vec![],
            secrets: vec![],
            gps: None,
            phash: hash.map(str::to_string),
            dup_of: None,
            dead: 0.0,
            dead_reason: None,
            size: 1,
            mtime: 0,
        }
    }

    #[test]
    fn near_copies_are_flagged_but_unrelated_white_pages_are_not() {
        let mut readings = vec![
            reading("otp.png", Some("8080200000000000")),
            reading("ticket.png", Some("c080800000000000")), // 3 bits away
            reading("otp copy.png", Some("8080200000000000")), // 0 bits
            reading("otp rescan.png", Some("8080200000000001")), // 1 bit
        ];
        mark_duplicates(&mut readings);
        assert_eq!(readings[0].dup_of, None);
        assert_eq!(
            readings[1].dup_of, None,
            "similar-looking but different documents must not be duplicates"
        );
        assert_eq!(readings[2].dup_of.as_deref(), Some("otp.png"));
        assert_eq!(readings[3].dup_of.as_deref(), Some("otp.png"));
    }

    #[test]
    fn the_plainer_name_is_kept_when_the_copy_is_seen_first() {
        let mut readings = vec![
            reading("IMG-1 copy.png", Some("8080200000000000")),
            reading("IMG-1.png", Some("8080200000000000")),
        ];
        mark_duplicates(&mut readings);
        assert_eq!(
            readings[0].dup_of.as_deref(),
            Some("IMG-1.png"),
            "the copy is the one offered to let go"
        );
        assert_eq!(readings[1].dup_of, None);
    }
}
