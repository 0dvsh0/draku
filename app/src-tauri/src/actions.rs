//! IPC surface. Read commands answer questions from the store; the action
//! commands at the bottom are the ONLY places in the program permitted to
//! change anything on disk, and every one of them is user-confirmed first.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::Ordering;

use chrono::Local;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::model::{ActionOk, SearchHit};
use crate::state::DrakuState;
use crate::{ics, model};

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_state(state: tauri::State<DrakuState>) -> model::IpcResult<model::AppState> {
    Ok(state.snapshot())
}

#[tauri::command]
pub fn search(query: String, state: tauri::State<DrakuState>) -> model::IpcResult<Vec<SearchHit>> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Ok(vec![]);
    }

    let (embeds, rested) = {
        let store = state.store.lock().unwrap();
        (store.embeds(), store.rested_ids())
    };

    let mut out: Vec<SearchHit> = vec![];

    // Semantic path: load the already-downloaded model lazily after a restart,
    // then cosine against the local index.
    if !embeds.is_empty() {
        let mut embedder = state.embedder.lock().unwrap();
        if !embedder.ready() {
            // The embeddings came from a scan, so the model is normally in the
            // local cache already. If it is gone, initialise once and be honest
            // in the log if that fails.
            if let Err(e) = embedder.init() {
                crate::log_warn(&format!("search model unavailable: {e}"));
            }
        }
        if embedder.ready() {
            if let Ok(qv) = embedder.embed(&[query.clone()]) {
                let q = &qv[0];
                let mut hits: Vec<(String, f32)> = embeds
                    .iter()
                    .filter(|(id, _)| !rested.contains(id))
                    .filter_map(|(id, v)| {
                        let s = crate::embed::cosine(q, v);
                        (s >= 0.20).then(|| (id.clone(), s))
                    })
                    .collect();
                hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                hits.truncate(6);
                let store = state.store.lock().unwrap();
                for (id, score) in hits {
                    if let Some(row) = store.row_by_id(&id) {
                        out.push(SearchHit {
                            file_id: id,
                            path: join_path(&row.root, &row.rel),
                            name: row.name,
                            category: row.category,
                            score,
                        });
                    }
                }
            }
        }
    }

    // Exact words still matter, and they work before any model exists. Fill up
    // to the same limit with real text matches from name / category / body.
    if out.len() < 8 {
        let store = state.store.lock().unwrap();
        for h in store.search_text(&query, 8) {
            if rested.contains(&h.file_id) {
                continue;
            }
            if out.iter().any(|x| x.file_id == h.file_id) {
                continue;
            }
            out.push(h);
            if out.len() >= 8 {
                break;
            }
        }
    }

    Ok(out)
}

fn join_path(root: &str, rel: &str) -> String {
    format!("{root}{MAIN}{rel}", MAIN = std::path::MAIN_SEPARATOR)
}

// ---------------------------------------------------------------------------
// Actions — suggest → confirm → mutate. Nothing here runs on its own.
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn start_scan(paths: Vec<String>, app: AppHandle) -> model::IpcResult<ActionOk> {
    let state: tauri::State<DrakuState> = app.state();
    let roots = resolve_roots(&paths)?;
    if state.scanning.swap(true, Ordering::SeqCst) {
        return Err(model::ipc_err("A scan is already running."));
    }
    crate::scan::CANCEL.store(false, Ordering::Relaxed);

    let root_strings: Vec<String> = roots
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let folders = root_strings.len();
    let app_handle = app.clone();
    std::thread::spawn(move || {
        crate::scan::scan(root_strings, app_handle.clone());
        let st: tauri::State<DrakuState> = app_handle.state();
        st.scanning.store(false, Ordering::SeqCst);
        let _ = app_handle.emit("scan://finished", ());
    });
    Ok(ActionOk {
        message: if folders == 1 {
            "Reading your files. Nothing leaves this laptop.".into()
        } else {
            format!("Reading {folders} folders. Nothing leaves this laptop.")
        },
        undoable: false,
        path: None,
        undo_file_ids: vec![],
        undo_paths: vec![],
    })
}

/// Turn dropped paths into real scan roots. A dropped file means "read the
/// folder it lives in"; nested roots collapse into the parent that contains
/// them so no file is read twice.
fn resolve_roots(paths: &[String]) -> model::IpcResult<Vec<PathBuf>> {
    let mut candidates: Vec<PathBuf> = vec![];
    for raw in paths {
        let p = PathBuf::from(raw);
        let dir = if p.is_dir() {
            p
        } else if p.is_file() {
            p.parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| model::ipc_err(format!("Can't find a folder for {raw}")))?
        } else {
            return Err(model::ipc_err(format!("Not found: {raw}")));
        };
        let abs = std::path::absolute(&dir).map_err(|e| model::ipc_err(e.to_string()))?;
        candidates.push(abs);
    }
    if candidates.is_empty() {
        return Err(model::ipc_err("Nothing to read."));
    }
    candidates.sort();
    candidates.dedup();
    let mut roots: Vec<PathBuf> = vec![];
    for c in candidates {
        if !roots.iter().any(|r| c.starts_with(r)) {
            roots.push(c);
        }
    }
    Ok(roots)
}

#[tauri::command]
pub fn cancel_scan() {
    crate::scan::CANCEL.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn add_to_calendar(file_id: String, app: AppHandle) -> model::IpcResult<ActionOk> {
    let state: tauri::State<DrakuState> = app.state();
    let (row, label, date) = {
        let store = state.store.lock().unwrap();
        let row = store
            .row_by_id(&file_id)
            .ok_or_else(|| model::ipc_err("File not in the index."))?;

        let today = Local::now().date_naive();
        let mut st = match store.conn.prepare(
            "SELECT label, date FROM file_dates
             WHERE file_id=?1 AND kind='deadline' AND date>=?2
             ORDER BY date LIMIT 1",
        ) {
            Ok(s) => s,
            Err(e) => return Err(model::ipc_err(e)),
        };
        let picked = st
            .query_row(rusqlite::params![row.id, today.to_string()], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .ok();
        let (label, date) = picked
            .ok_or_else(|| model::ipc_err("No upcoming date found in this file's reading."))?;
        (row, label, date)
    };

    let today = Local::now().date_naive();
    let ev = ics::Event {
        label: label.clone(),
        date: date.clone(),
        source: format!("From \"{}\", found in {}", row.name, row.rel),
        file_stem: stem_of(&row.rel),
    };
    let write_path = ics::write(Path::new(&row.root), &ev, &today.to_string())
        .map_err(|e| model::ipc_err(format!("couldn't write calendar file: {e}")))?;
    {
        let store = state.store.lock().unwrap();
        store
            .conn
            .execute(
                "INSERT INTO calendar_log (file_id,label,date,ics_path,made_at)
                 VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![
                    file_id,
                    label,
                    date,
                    write_path.display().to_string(),
                    chrono::Utc::now().timestamp()
                ],
            )
            .ok();
    }

    Ok(ActionOk {
        message: format!("{label} — {date} now has a calendar file."),
        undoable: true,
        path: Some(write_path.display().to_string()),
        undo_file_ids: vec![],
        undo_paths: vec![write_path.display().to_string()],
    })
}

fn stem_of(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("event")
        .chars()
        .take(40)
        .collect()
}

#[tauri::command]
pub fn make_safe_copy(file_id: String, app: AppHandle) -> model::IpcResult<ActionOk> {
    let state: tauri::State<DrakuState> = app.state();
    let (row, findings) = {
        let store = state.store.lock().unwrap();
        let row = store
            .row_by_id(&file_id)
            .ok_or_else(|| model::ipc_err("File not in the index."))?;
        let findings = store.secrets_of(row.id);
        (row, findings)
    };

    let src = PathBuf::from(join_path(&row.root, &row.rel));
    if !src.is_file() {
        return Err(model::ipc_err("The file is gone from the disk."));
    }
    let ext = Path::new(&row.rel)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let safe_dir = Path::new(&row.root).join(".draku").join("safe");
    fs::create_dir_all(&safe_dir).map_err(|e| model::ipc_err(e.to_string()))?;
    let out = unique_artifact(&safe_dir, &stem_of(&row.rel), &ext, "safe");

    let report = if is_image(&ext) {
        redact_and_strip(&src, &out, &findings)?
    } else {
        // We do not rewrite PDFs: a byte-for-byte copy is honest, and the UI
        // tells the user exactly what to check before sharing it.
        fs::copy(&src, &out).map_err(|e| model::ipc_err(e.to_string()))?;
        "copied as-is — open it and check the flagged details before sharing".to_string()
    };

    Ok(ActionOk {
        message: format!("Safe copy made — {report}"),
        undoable: true,
        path: Some(out.display().to_string()),
        undo_file_ids: vec![],
        undo_paths: vec![out.display().to_string()],
    })
}

fn is_image(ext: &str) -> bool {
    matches!(ext, "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp")
}

/// `stem-safe.ext`, `stem-safe-2.ext`, … Never overwrites an earlier copy.
fn unique_artifact(dir: &Path, stem: &str, ext: &str, suffix: &str) -> PathBuf {
    let base = {
        let s = stem.trim();
        if s.is_empty() {
            "file"
        } else {
            s
        }
    };
    for n in 1..=999 {
        let tail = if n == 1 {
            suffix.to_string()
        } else {
            format!("{suffix}-{n}")
        };
        let name = if ext.is_empty() {
            format!("{base}-{tail}")
        } else {
            format!("{base}-{tail}.{ext}")
        };
        let candidate = dir.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{base}-{suffix}-{}.{}", std::process::id(), ext))
}

/// Re-encode without EXIF (metadata dies in the resample) and pixelate every
/// region OCR placed a box on. Pixelation is burned into the pixels; it cannot
/// be peeled back off like a metadata tag.
fn redact_and_strip(
    src: &Path,
    out: &Path,
    findings: &[model::SecretRef],
) -> model::IpcResult<String> {
    let img = image::ImageReader::open(src)
        .map_err(|e| model::ipc_err(e.to_string()))?
        .with_guessed_format()
        .map_err(|e| model::ipc_err(e.to_string()))?
        .decode()
        .map_err(|e| model::ipc_err(e.to_string()))?;

    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut redacted = 0;
    let mut boxes: Vec<(u32, u32, u32, u32)> = vec![];

    for [x, y, bw, bh] in findings.iter().filter_map(|f| f.bbox) {
        let pad = 4u32;
        let x0 = ((x * w as f32) as u32).saturating_sub(pad);
        let y0 = ((y * h as f32) as u32).saturating_sub(pad);
        let x1 = (((x + bw) * w as f32) as u32 + pad).min(w);
        let y1 = (((y + bh) * h as f32) as u32 + pad).min(h);
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        if boxes
            .iter()
            .any(|b| b.0.abs_diff(x0) < 2 && b.1.abs_diff(y0) < 2)
        {
            continue;
        }
        boxes.push((x0, y0, x1, y1));
        pixelate(&mut rgba, x0, y0, x1, y1);
        redacted += 1;
    }

    save_image(&rgba, out)?;

    let flagged_without_box =
        findings.iter().any(|f| f.bbox.is_none()) && !findings.iter().any(|f| f.bbox.is_some());
    Ok(match (redacted, flagged_without_box) {
        (0, true) => {
            "metadata stripped — the privacy finding lives in the file itself, not in a text box; check it before sharing"
                .to_string()
        }
        (0, false) => "metadata stripped — no on-image text boxes were found".to_string(),
        (n, _) => format!("metadata stripped, {n} region(s) pixelated"),
    })
}

fn pixelate(rgba: &mut image::RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) {
    let (w, h) = (x1 - x0, y1 - y0);
    if w == 0 || h == 0 {
        return;
    }
    let crop = image::imageops::crop_imm(rgba, x0, y0, w, h).to_image();
    // Down-sample hard, then blow it back up: the letters are gone, not just
    // soft. A small blur takes the last edges with it.
    let tiny_w = (w / 14).max(1);
    let tiny_h = (h / 10).max(1);
    let tiny = image::imageops::resize(&crop, tiny_w, tiny_h, image::imageops::FilterType::Nearest);
    let back = image::imageops::resize(&tiny, w, h, image::imageops::FilterType::Nearest);
    let soft = image::imageops::blur(&back, 3.0);
    image::imageops::overlay(rgba, &soft, x0 as i64, y0 as i64);
}

fn save_image(rgba: &image::RgbaImage, out: &Path) -> model::IpcResult<()> {
    let ext = out
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" => {
            let rgb = image::DynamicImage::ImageRgba8(rgba.clone()).to_rgb8();
            let file = fs::File::create(out).map_err(|e| model::ipc_err(e.to_string()))?;
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(file, 92);
            enc.encode_image(&image::DynamicImage::ImageRgb8(rgb))
                .map_err(|e| model::ipc_err(e.to_string()))
        }
        "png" => image::DynamicImage::ImageRgba8(rgba.clone())
            .save_with_format(out, image::ImageFormat::Png)
            .map_err(|e| model::ipc_err(e.to_string())),
        "webp" => image::DynamicImage::ImageRgba8(rgba.clone())
            .save_with_format(out, image::ImageFormat::WebP)
            .map_err(|e| model::ipc_err(e.to_string())),
        _ => image::DynamicImage::ImageRgba8(rgba.clone())
            .save(out)
            .map_err(|e| model::ipc_err(e.to_string())),
    }
}

#[tauri::command]
pub fn set_aside(file_id: String, app: AppHandle) -> model::IpcResult<ActionOk> {
    Ok(move_to_rest(&file_id, &app, false)?.0)
}

#[tauri::command]
pub fn set_aside_many(file_ids: Vec<String>, app: AppHandle) -> model::IpcResult<ActionOk> {
    if file_ids.is_empty() {
        return Err(model::ipc_err("Nothing selected."));
    }
    let state: tauri::State<DrakuState> = app.state();
    let mut moved: Vec<String> = vec![];
    let mut freed: u64 = 0;
    let mut already_gone = 0usize;
    let mut last_error: Option<String> = None;

    for id in &file_ids {
        let size = state
            .store
            .lock()
            .unwrap()
            .row_by_id(id)
            .map(|r| r.size)
            .unwrap_or(0);
        match move_to_rest(id, &app, false) {
            Ok(_) => {
                moved.push(id.clone());
                freed += size;
            }
            Err(e) => {
                already_gone += 1;
                last_error = Some(e.message);
            }
        }
    }

    if moved.is_empty() {
        return Err(model::ipc_err(
            last_error.unwrap_or_else(|| "Nothing could be set aside.".into()),
        ));
    }

    let mut message = format!(
        "{} file(s) set aside — {} rests in .draku/rest. One tap restores them.",
        moved.len(),
        fmt_bytes(freed)
    );
    if already_gone > 0 {
        message.push_str(&format!(" {already_gone} had already moved."));
    }
    Ok(ActionOk {
        message,
        undoable: true,
        path: None,
        undo_file_ids: moved,
        undo_paths: vec![],
    })
}

#[tauri::command]
pub fn restore(file_id: String, app: AppHandle) -> model::IpcResult<ActionOk> {
    Ok(move_to_rest(&file_id, &app, true)?.0)
}

#[tauri::command]
pub fn restore_many(file_ids: Vec<String>, app: AppHandle) -> model::IpcResult<ActionOk> {
    if file_ids.is_empty() {
        return Err(model::ipc_err("Nothing to restore."));
    }
    let mut restored = 0usize;
    let mut last_error: Option<String> = None;
    for id in &file_ids {
        match move_to_rest(id, &app, true) {
            Ok(_) => restored += 1,
            Err(e) => last_error = Some(e.message),
        }
    }
    if restored == 0 {
        return Err(model::ipc_err(
            last_error.unwrap_or_else(|| "Nothing could be restored.".into()),
        ));
    }
    Ok(ActionOk {
        message: format!("{restored} file(s) back where they were."),
        undoable: false,
        path: None,
        undo_file_ids: vec![],
        undo_paths: vec![],
    })
}

/// Moves a file to the rest folder (or back). Deleting does not exist in this
/// codebase for user files — promise #2 is structural, not procedural.
fn move_to_rest(file_id: &str, app: &AppHandle, back: bool) -> model::IpcResult<(ActionOk, u64)> {
    let state: tauri::State<DrakuState> = app.state();
    let store = state.store.lock().unwrap();
    let row = store
        .row_by_id(file_id)
        .ok_or_else(|| model::ipc_err("File not in the index."))?;

    let root = PathBuf::from(&row.root);
    let now = chrono::Utc::now().timestamp();

    if !back {
        let src = root.join(&row.rel);
        if !src.is_file() {
            return Err(model::ipc_err("The file is already gone from the disk."));
        }
        let wanted = PathBuf::from(".draku").join("rest").join(&row.rel);
        let dst_rel = unique_rest_path(&root, &wanted);
        let dst = root.join(&dst_rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| model::ipc_err(e.to_string()))?;
        }
        move_file(&src, &dst)?;
        if let Err(e) = store.conn.execute(
            "INSERT INTO rest_log (file_id, root, rel, rest_rel, moved_at)
             VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![
                file_id,
                row.root,
                row.rel,
                dst_rel.display().to_string(),
                now
            ],
        ) {
            // Put it back if the log could not be written: a file in the rest
            // folder with no record would be invisible to Undo.
            let _ = move_file(&dst, &src);
            return Err(model::ipc_err(e));
        }
        Ok((
            ActionOk {
                message: format!("{} set aside. Restable from the rest folder.", row.name),
                undoable: true,
                path: Some(dst.display().to_string()),
                undo_file_ids: vec![file_id.to_string()],
                undo_paths: vec![],
            },
            row.size,
        ))
    } else {
        let entry: Option<(String, String, String)> = store
            .conn
            .query_row(
                "SELECT root, rel, rest_rel FROM rest_log
                 WHERE file_id=?1 AND restored_at IS NULL
                 ORDER BY moved_at DESC LIMIT 1",
                [file_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();
        let (root, rel, rest_rel) = entry.ok_or_else(|| model::ipc_err("Nothing to restore."))?;
        let src = PathBuf::from(&root).join(&rest_rel);
        let dst = PathBuf::from(&root).join(&rel);
        if !src.is_file() {
            return Err(model::ipc_err("The rest copy is missing from the disk."));
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| model::ipc_err(e.to_string()))?;
        }
        move_file(&src, &dst)?;
        store
            .conn
            .execute(
                "UPDATE rest_log SET restored_at=?1 WHERE file_id=?2 AND restored_at IS NULL",
                rusqlite::params![now, file_id],
            )
            .map_err(|e| model::ipc_err(e))?;
        Ok((
            ActionOk {
                message: format!("{rel} is back where it was."),
                undoable: false,
                path: Some(dst.display().to_string()),
                undo_file_ids: vec![],
                undo_paths: vec![],
            },
            row.size,
        ))
    }
}

fn move_file(src: &Path, dst: &Path) -> model::IpcResult<()> {
    fs::rename(src, dst)
        .or_else(|_| fs::copy(src, dst).and_then(|_| fs::remove_file(src)))
        .map_err(|e| model::ipc_err(e.to_string()))
}

/// Rest paths keep their relative shape. If an old rest copy somehow still
/// exists, suffix the new one instead of overwriting it.
fn unique_rest_path(root: &Path, wanted: &Path) -> PathBuf {
    let full = root.join(wanted);
    if !full.exists() {
        return wanted.to_path_buf();
    }
    let stem = wanted
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = wanted.extension().and_then(|e| e.to_str()).unwrap_or("");
    let parent = wanted.parent().unwrap_or(Path::new(""));
    for n in 2..=999 {
        let name = if ext.is_empty() {
            format!("{stem}-{n}")
        } else {
            format!("{stem}-{n}.{ext}")
        };
        let candidate = parent.join(name);
        if !root.join(&candidate).exists() {
            return candidate;
        }
    }
    wanted.to_path_buf()
}

/// Remove only artifacts Draku itself generated (safe copies and calendar
/// files). Path validation is deliberate: Undo must never touch user files.
#[tauri::command]
pub fn undo_generated(paths: Vec<String>) -> model::IpcResult<()> {
    for raw in paths {
        let path = PathBuf::from(&raw);
        if is_generated_artifact(&path) {
            let _ = fs::remove_file(&path);
        }
    }
    Ok(())
}

fn is_generated_artifact(path: &Path) -> bool {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return false;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    if !matches!(
        ext.as_str(),
        "ics" | "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"
    ) {
        return false;
    }
    let comps: Vec<_> = path.components().collect();
    comps.windows(2).any(|w| {
        w[0].as_os_str() == ".draku"
            && matches!(w[1].as_os_str().to_str(), Some("safe") | Some("calendar"))
    })
}

fn fmt_bytes(n: u64) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{} KB", n / 1024)
    } else if n < 1024 * 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", n as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Reveal something in the OS file manager.
#[tauri::command]
pub fn reveal_path(path: String, app: AppHandle) -> model::IpcResult<()> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| model::ipc_err(e.to_string()))
}

// Kept for the M7 OCR milestone; the progress event shape is stable.
#[allow(dead_code)]
fn emit_progress(app: &AppHandle, p: &model::ScanProgress) {
    let _ = app.emit("scan://progress", p);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{cleanup, temp_dir};
    use image::GenericImageView;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn pixelation_burns_the_region_and_leaves_the_rest_alone() {
        let mut img = image::RgbaImage::from_fn(120, 60, |x, y| {
            // High-frequency stripes: anything that survives is visible.
            if (x / 2 + y / 2) % 2 == 0 {
                image::Rgba([250, 250, 250, 255])
            } else {
                image::Rgba([10, 10, 10, 255])
            }
        });
        let before = img.clone();
        pixelate(&mut img, 20, 20, 100, 40);

        assert_ne!(
            img.get_pixel(60, 30),
            before.get_pixel(60, 30),
            "the targeted pixels should be replaced"
        );
        assert_eq!(
            img.get_pixel(5, 5),
            before.get_pixel(5, 5),
            "outside the box nothing changes"
        );
        assert_eq!(
            img.get_pixel(119, 59),
            before.get_pixel(119, 59),
            "outside the box nothing changes"
        );
    }

    #[test]
    fn safe_copy_pixelates_the_box_and_strips_gps() {
        let dir = temp_dir("safe-copy");
        let src = fixture("geo.jpg");
        assert!(
            crate::extract::imginfo::probe(&src).gps.is_some(),
            "fixture must carry GPS before redaction"
        );
        let out = dir.join("geo-safe.jpg");
        let report = redact_and_strip(
            &src,
            &out,
            &[model::SecretRef {
                kind: "id_number".into(),
                preview: "••••".into(),
                page: None,
                bbox: Some([0.25, 0.25, 0.5, 0.25]),
            }],
        )
        .expect("safe copy succeeds");

        assert!(out.is_file());
        assert!(report.contains("pixelated"), "report was {report:?}");
        let written = image::ImageReader::open(&out)
            .unwrap()
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        let original = image::ImageReader::open(&src)
            .unwrap()
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert_eq!(written.dimensions(), original.dimensions());
        let written = written.to_rgba8();
        let original = original.to_rgba8();
        let inside_diff: i32 = (0..3)
            .map(|c| {
                (written.get_pixel(160, 90)[c] as i32 - original.get_pixel(160, 90)[c] as i32).abs()
            })
            .sum();
        assert!(
            inside_diff > 30,
            "the boxed area must be pixelated in the output (diff {inside_diff})"
        );
        let outside = written.get_pixel(2, 2);
        let was = original.get_pixel(2, 2);
        for c in 0..3 {
            assert!(
                (outside[c] as i32 - was[c] as i32).abs() <= 12,
                "compression aside, pixels outside the box are untouched"
            );
        }
        assert!(
            crate::extract::imginfo::probe(&out).gps.is_none(),
            "re-encoding must drop GPS metadata"
        );
        assert!(
            crate::extract::imginfo::probe(&src).gps.is_some(),
            "the original file is never modified"
        );

        cleanup(&dir);
    }

    #[test]
    fn unique_artifacts_never_overwrite() {
        let dir = temp_dir("unique-artifacts");
        let first = unique_artifact(&dir, "report", "jpg", "safe");
        assert_eq!(first.file_name().unwrap(), "report-safe.jpg");
        fs::write(&first, b"first").unwrap();
        let second = unique_artifact(&dir, "report", "jpg", "safe");
        assert_eq!(second.file_name().unwrap(), "report-safe-2.jpg");
        fs::write(&second, b"second").unwrap();
        assert_eq!(fs::read(&first).unwrap(), b"first");
        cleanup(&dir);
    }

    #[test]
    fn undo_only_touches_draku_generated_artifacts() {
        let dir = temp_dir("undo-generated");
        let safe_dir = dir.join(".draku").join("safe");
        let cal_dir = dir.join(".draku").join("calendar");
        fs::create_dir_all(&safe_dir).unwrap();
        fs::create_dir_all(&cal_dir).unwrap();

        let safe = safe_dir.join("photo-safe.jpg");
        let ics = cal_dir.join("renew-by.ics");
        let user = dir.join("photo.jpg");
        let user_pdf = safe_dir.join("invoice.pdf");
        fs::write(&safe, b"generated").unwrap();
        fs::write(&ics, b"generated").unwrap();
        fs::write(&user, b"user file").unwrap();
        fs::write(&user_pdf, b"user file").unwrap();

        assert!(is_generated_artifact(&safe));
        assert!(is_generated_artifact(&ics));
        assert!(!is_generated_artifact(&user));
        assert!(
            !is_generated_artifact(&user_pdf),
            "only artifact types Draku writes are removable"
        );
        assert!(!is_generated_artifact(Path::new(
            r"C:\Users\a\Pictures\.draku\safe\..\..\photo.jpg"
        )));

        undo_generated(vec![
            safe.display().to_string(),
            ics.display().to_string(),
            user.display().to_string(),
            user_pdf.display().to_string(),
        ])
        .expect("undo command succeeds");

        assert!(!safe.exists());
        assert!(!ics.exists());
        assert!(user.is_file(), "Undo must never delete a user file");
        assert!(user_pdf.is_file(), "Undo must never delete a user PDF");

        cleanup(&dir);
    }
}
